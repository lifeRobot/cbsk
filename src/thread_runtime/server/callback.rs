use std::sync::Arc;
use cbsk_base::log;
use crate::thread_runtime::server::client::CbskServerClient;

/// cbsk connect and read data callback
pub trait CbskServerCallBack: Send + Sync + 'static {
    /// a new tcp client come in
    fn conn(&self, client: Arc<CbskServerClient>) {
        log::info!("{} tcp client connected",client.get_log_head());
    }

    /// the tcp client disconnected
    fn dis_conn(&self, client: Arc<CbskServerClient>) {
        log::info!("{} tcp client disconnect", client.get_log_head());
    }

    /// error frame
    fn error_frame(&self, error_frame: Vec<u8>, client: Arc<CbskServerClient>) {
        log::warn!("{} received non cbsk frame, will be discarded, error frame is: {error_frame:?}",client.get_log_head());
    }

    /// data frame first byte is too long
    fn too_long_frame(&self, byte: u8, client: Arc<CbskServerClient>) {
        log::warn!("{} received cbsk frame, but first byte[{byte}] is too long",client.get_log_head());
    }

    /// tcp server recv tcp client data will call this method<br />
    /// bytes: tcp client data<br />
    /// client: tcp client, you can use this send data to tcp client<br />
    /// return Vec<u8>: If you think the data length is insufficient, you can return the data for data merging,
    /// if data normal, you should be return Vec::new() or vec![]
    fn recv(&self, bytes: Vec<u8>, client: Arc<CbskServerClient>);
}