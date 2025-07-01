use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use cbsk_base::{log, tokio};
use cbsk_base::tokio::io::AsyncWriteExt;
use cbsk_base::tokio::net::tcp::OwnedReadHalf;
use cbsk_base::tokio::net::TcpListener;
use cbsk_base::tokio::task::JoinHandle;
use cbsk_socket::tcp::server::config::TcpServerConfig;
use crate::tcp::common::read_trait::ReadTrait;
use crate::tcp::server::callback::TcpServerCallBack;
use crate::tcp::server::client::TcpServerClient;

pub mod callback;
pub mod client;

/// tcp server
#[derive(Clone)]
pub struct TcpServer {
    /// tcp config
    pub conf: Arc<TcpServerConfig>,
    /// tcp server business callback
    pub cb: Arc<Box<dyn TcpServerCallBack>>,
    /// tcp read data len
    buf_len: usize,
    /// stop tcp server
    stopped: Arc<AtomicBool>,
}

/// data init etc
impl TcpServer {
    /// create a tcp server<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn new<C: TcpServerCallBack>(conf: Arc<TcpServerConfig>, cb: C) -> Self {
        Self::new_with_buf_len(conf, cb, 1024)
    }

    pub fn new_with_buf_len<C: TcpServerCallBack>(conf: Arc<TcpServerConfig>, cb: C, buf_len: usize) -> Self {
        Self { conf, cb: Arc::new(Box::new(cb)), buf_len, stopped: Arc::new(AtomicBool::new(false)) }
    }
}

/// tcp read logic
impl TcpServer {
    /// start tcp server
    pub async fn start(&self) {
        if let Err(e) = self.try_start().await {
            log::error!("{} tcp bind [{}] error: {e:?}",self.conf.log_head,self.conf.addr);
        }
    }

    /// start tcp server in join handle
    pub fn start_in_handle(&self) -> JoinHandle<()> {
        let tcp_server = self.clone();
        tokio::spawn(async move { tcp_server.start().await; })
    }

    /// stop tcp server
    #[inline]
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Release);
    }

    /// try start tcp server
    async fn try_start(&self) -> io::Result<()> {
        let listener = TcpListener::bind(self.conf.addr).await?;
        let conf = self.conf.as_ref();

        log::info!("{} listener TCP[{}] success",conf.log_head,conf.addr);
        // loop waiting for client to connect
        loop {
            // if stop the server, return function
            if self.stopped.load(Ordering::Acquire) { return Ok(()); }
            if let Err(e) = self.try_accept(&listener).await {
                log::error!("{} wait tcp accept error. wait for the next accept in three seconds. error: {:?}",conf.log_head,e);
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }

    /// try accept TCP client and read tcp client data
    async fn try_accept(&self, listener: &TcpListener) -> io::Result<()> {
        // tcp client come in, stream split to read and write
        let (tcp_stream, addr) = listener.accept().await?;
        let (read, write) = tcp_stream.into_split();

        // start read data
        let client = Arc::new(client::TcpServerClient::new(addr, self.conf.as_ref(), write.into()));
        self.read_spawn(client.clone(), read);
        self.cb.conn(client).await;

        Ok(())
    }

    /// start read async
    fn read_spawn(&self, client: Arc<TcpServerClient>, read: OwnedReadHalf) {
        let tcp_server = self.clone();
        tokio::spawn(async move {
            let read_handle = tcp_server.try_read_spawn(client.clone(), read);

            client.wait_read_handle_finished(read_handle, tcp_server.conf.read_time_out, || async {
                // it is possible that TCP has not been closed here, so notify to close it once
                // ignoring notification results
                let _ = client.write.write().await.shutdown().await;
            }).await;

            // if TCP read is closed, it is considered that TCP has been closed
            tcp_server.cb.dis_conn(client.clone()).await;
            if tcp_server.conf.log { log::info!("{} tcp client read async closed",client.log_head); }
        });
    }

    /// try read tcp client data
    fn try_read_spawn(&self, client: Arc<TcpServerClient>, read: OwnedReadHalf) -> JoinHandle<()> {
        if self.conf.log { log::info!("{} start tcp client read async success",client.log_head); }
        let tcp_server = self.clone();

        tokio::spawn(async move {
            let result =
                client.try_read_data_tokio(read, tcp_server.buf_len, tcp_server.conf.read_time_out, "client", || async {
                    tcp_server.stopped.load(Ordering::Acquire)
                }, |data| async {
                    tcp_server.cb.recv(data, client.clone()).await
                }).await;

            if let Err(e) = result {
                if tcp_server.conf.log { log::error!("{} read tcp client data error: {e:?}",client.log_head); }
            }
        })
    }
}
