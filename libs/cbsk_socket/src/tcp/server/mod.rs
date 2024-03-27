use std::io;
use std::sync::Arc;
use std::time::Duration;
use cbsk_base::{log, tokio};
use cbsk_base::tokio::io::AsyncWriteExt;
use cbsk_base::tokio::net::tcp::OwnedReadHalf;
use cbsk_base::tokio::net::TcpListener;
use cbsk_base::tokio::task::JoinHandle;
use crate::tcp::server::callback::TcpServerCallBack;
use crate::tcp::server::client::TcpServerClient;
use crate::tcp::server::config::TcpServerConfig;
use crate::tcp::tcp_time_trait::TcpTimeTrait;
use crate::tcp::tcp_write_trait::TcpWriteTrait;

pub mod config;
pub mod client;
pub mod callback;

/// tcp server
pub struct TcpServer<C: TcpServerCallBack> {
    /// tcp config
    pub conf: Arc<TcpServerConfig>,
    /// tcp server business callback
    pub cb: Arc<C>,
}

/// support clone
impl<C: TcpServerCallBack> Clone for TcpServer<C> {
    fn clone(&self) -> Self {
        Self { conf: self.conf.clone(), cb: self.cb.clone() }
    }
}

/// data init etc
impl<C: TcpServerCallBack> TcpServer<C> {
    /// create a tcp server<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn new(conf: Arc<TcpServerConfig>, cb: Arc<C>) -> Self {
        Self { conf, cb }
    }
}

/// tcp read logic
impl<C: TcpServerCallBack> TcpServer<C> {
    /// start tcp server<br />
    /// N: TCP read data bytes size at once, usually 1024, If you need to accept big data, please increase this value
    pub fn start<const N: usize>(&self) -> JoinHandle<()> {
        let tcp_server = self.clone();
        tokio::spawn(async move {
            let conf = tcp_server.conf.clone();
            let mut read_handles = Vec::new();

            if let Err(e) = tcp_server.try_start::<N>(&mut read_handles).await {
                log::error!("{} tcp bind [{}] error: {e:?}",conf.log_head,conf.addr);
            }

            // wait read async
            for handle in read_handles {
                if let Err(e) = handle.await {
                    log::error!("{} read async error: {e:?}",conf.log_head);
                }
            }
        })
    }

    /// try start tcp server
    async fn try_start<const N: usize>(&self, read_handles: &mut Vec<JoinHandle<()>>) -> io::Result<()> {
        let listener = TcpListener::bind(self.conf.addr).await?;
        let conf = self.conf.as_ref();

        log::info!("{} listener TCP[{}] success",conf.log_head,conf.addr);
        // Loop waiting for client to connect
        loop {
            if let Err(e) = self.try_accept::<N>(&listener, read_handles).await {
                log::error!("{} wait tcp accept error. wait for the next accept in three seconds. error: {:?}",conf.log_head,e);
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }

    /// try accept TCP client and read tcp client data
    async fn try_accept<const N: usize>(&self, listener: &TcpListener, read_handles: &mut Vec<JoinHandle<()>>) -> io::Result<()> {
        // tcp client come in, stream split to read and write
        let (tcp_stream, addr) = listener.accept().await?;
        let (read, write) = tcp_stream.into_split();

        // start read data
        let client = Arc::new(TcpServerClient::new(addr, self.conf.as_ref(), write));
        read_handles.push(self.read_spawn::<N>(client.clone(), read));
        self.cb.conn(client).await;

        Ok(())
    }

    /// start read async
    fn read_spawn<const N: usize>(&self, client: Arc<TcpServerClient>, read: OwnedReadHalf) -> JoinHandle<()> {
        let tcp_server = self.clone();
        tokio::spawn(async move {
            let read_headle = tcp_server.try_read_spawn::<N>(client.clone(), read);

            client.wait_read_handle_finished(read_headle, tcp_server.conf.read_time_out, || async {
                // it is possible that TCP has not been closed here, so notify to close it once
                if let Ok(tcp_client) = client.try_get_write() {
                    // ignoring notification results
                    let _ = tcp_client.as_mut().shutdown().await;
                }
            }).await;

            // if TCP read is closed, it is considered that TCP has been closed
            tcp_server.cb.dis_conn(client.clone()).await;
            if tcp_server.conf.log { log::info!("{} tcp client read async closed",client.log_head); }
        })
    }

    /// try read tcp client data
    fn try_read_spawn<const N: usize>(&self, client: Arc<TcpServerClient>, read: OwnedReadHalf) -> JoinHandle<()> {
        if self.conf.log { log::info!("{} start tcp client read async success",client.log_head); }
        let tcp_server = self.clone();

        tokio::spawn(async move {
            let result =
                client.try_read_data::<N, _, _, _>(read, tcp_server.conf.read_time_out, "server", || {
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
