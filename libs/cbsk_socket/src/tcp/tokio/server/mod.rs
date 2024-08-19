use std::io;
use std::sync::Arc;
use std::time::Duration;
use cbsk_base::{log, tokio};
use cbsk_base::tokio::net::tcp::OwnedReadHalf;
use cbsk_base::tokio::net::TcpListener;
use cbsk_base::tokio::task::JoinHandle;
use crate::tcp::common::r#async::async_tcp_time_trait::AsyncTcpTimeTrait;
// pub use crate::tcp::common::server::r#async::callback;
use crate::tcp::common::server::r#async::callback::TcpServerCallBack;
pub use crate::tcp::common::server::r#async::client;
use crate::tcp::common::server::r#async::client::TcpServerClient;
pub use crate::tcp::common::server::config;
use crate::tcp::common::server::config::TcpServerConfig;
use crate::tcp::tokio::tokio_tcp_read_trait::TokioTcpReadTrait;

/// tcp server
#[derive(Clone)]
pub struct TcpServer {
    /// tcp config
    pub conf: Arc<TcpServerConfig>,
    /// tcp server business callback
    pub cb: Arc<Box<dyn TcpServerCallBack>>,
    /// tcp read data len
    buf_len: usize,
}

/// data init etc
impl TcpServer {
    /// create a tcp server<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn new<C: TcpServerCallBack>(conf: Arc<TcpServerConfig>, cb: C) -> Self {
        Self::new_with_buf_len(conf, cb, 1024)
    }

    pub fn new_with_buf_len<C: TcpServerCallBack>(conf: Arc<TcpServerConfig>, cb: C, buf_len: usize) -> Self {
        Self { conf, cb: Arc::new(Box::new(cb)), buf_len }
    }
}

/// tcp read logic
impl TcpServer {
    /// start tcp server<br />
    /// N: TCP read data bytes size at once, usually 1024, If you need to accept big data, please increase this value
    pub fn start(&self) -> JoinHandle<()> {
        let tcp_server = self.clone();
        tokio::spawn(async move {
            if let Err(e) = tcp_server.try_start().await {
                log::error!("{} tcp bind [{}] error: {e:?}",tcp_server.conf.log_head,tcp_server.conf.addr);
            }
        })
    }

    /// try start tcp server
    async fn try_start(&self) -> io::Result<()> {
        let listener = TcpListener::bind(self.conf.addr).await?;
        let conf = self.conf.as_ref();

        log::info!("{} listener TCP[{}] success",conf.log_head,conf.addr);
        // loop waiting for client to connect
        loop {
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
            let read_headle = tcp_server.try_read_spawn(client.clone(), read);

            client.wait_read_handle_finished(read_headle, tcp_server.conf.read_time_out, || async {
                // it is possible that TCP has not been closed here, so notify to close it once
                // ignoring notification results
                let _ = client.write.as_mut().shutdown().await;
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
                client.try_read_data_tokio(read, tcp_server.buf_len, tcp_server.conf.read_time_out, "client", || {
                    false
                }, |data| async {
                    tcp_server.cb.recv(data, client.clone()).await
                }).await;

            if let Err(e) = result {
                if tcp_server.conf.log { log::error!("{} read tcp client data error: {e:?}",client.log_head); }
            }
        })
    }
}
