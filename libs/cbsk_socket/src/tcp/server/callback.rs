use std::sync::Arc;
use cbsk_base::async_trait::async_trait;
use crate::tcp::server::client::TcpServerClient;

/// tcp read data call back
#[async_trait]
pub trait TcpServerCallBack: Send + Sync + 'static {
    /// a new tcp client come in
    async fn conn(&self, client: Arc<TcpServerClient>);

    /// the tcp client disconnected
    async fn dis_conn(&self, client: Arc<TcpServerClient>);

    /// tcp server recv tcp client data will call this method<br />
    /// bytes: tcp client data<br />
    /// client: tcp client, you can use this send data to tcp client<br />
    /// return Vec<u8>: If you think the data length is insufficient, you can return the data for data merging,
    /// if data normal, you should be return Vec::new() or vec![]
    async fn recv(&self, bytes: Vec<u8>, client: Arc<TcpServerClient>) -> Vec<u8>;
}