use std::sync::Arc;
use cbsk_base::async_trait::async_trait;
use cbsk_base::{log, tokio};
use cbsk_base::tokio::task::JoinHandle;
use crate::tcp::server::client::TcpServerClient;

/// tcp connect and read data callback
#[async_trait]
pub trait TcpServerCallBack: Send + Sync + 'static {
    /// a new tcp client come in<br />
    /// handle: tcp client read async<br />
    /// don't call handle.await in this method, because handle.await will be block, the next tcp client will be can't connect
    async fn conn(&self, client: Arc<TcpServerClient>, handle: JoinHandle<()>) {
        tokio::spawn(async move {
            if let Err(e) = handle.await {
                log::error!("{} tcp client async error: {e:?}",client.log_head);
            }
        });
    }

    /// the tcp client disconnected
    async fn dis_conn(&self, client: Arc<TcpServerClient>) {
        log::info!("{} tcp client disconnect", client.log_head)
    }

    /// tcp server recv tcp client data will call this method<br />
    /// bytes: tcp client data<br />
    /// client: tcp client, you can use this send data to tcp client<br />
    /// return Vec<u8>: If you think the data length is insufficient, you can return the data for data merging,
    /// if data normal, you should be return Vec::new() or vec![]
    async fn recv(&self, bytes: Vec<u8>, client: Arc<TcpServerClient>) -> Vec<u8>;
}