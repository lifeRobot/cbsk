use std::sync::Arc;
use cbsk_base::log;
use crate::tcp::rayon::server::client::TcpServerClient;

/// tcp connect and read data callback
pub trait TcpServerCallBack: Send + Sync + 'static {
    /// a new tcp client come in
    fn conn(&self, client: Arc<TcpServerClient>) {
        log::info!("{} tcp client connected",client.log_head);
    }

    /// the tcp client disconnected
    fn dis_conn(&self, client: Arc<TcpServerClient>) {
        log::info!("{} tcp client disconnect", client.log_head);
    }

    /// tcp server recv tcp client data will call this method<br />
    /// bytes: tcp client data<br />
    /// client: tcp client, you can use this send data to tcp client<br />
    /// return Vec<u8>: If you think the data length is insufficient, you can return the data for data merging,
    /// if data normal, you should be return Vec::new() or vec![]
    fn recv(&self, bytes: Vec<u8>, client: Arc<TcpServerClient>) -> Vec<u8>;
}