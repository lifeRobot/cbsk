use std::sync::Arc;
use cbsk_socket::tcp::server::client::TcpServerClient;
use cbsk_socket::tcp::tcp_write_trait::TcpWriteTrait;

/// cbsk server client
pub struct CbskServerClient {
    tcp_server_client: Arc<TcpServerClient>,
}

/// custom method
impl CbskServerClient {
    /// create cbsk server client
    pub(crate) fn new(tcp_server_client: Arc<TcpServerClient>) -> Self {
        Self { tcp_server_client }
    }

    /// get internal log name
    pub fn get_log_head(&self) -> &str {
        self.tcp_server_client.get_log_head()
    }
}

/// support tcp server client into cbsk server client
impl From<Arc<TcpServerClient>> for CbskServerClient {
    fn from(value: Arc<TcpServerClient>) -> Self {
        Self::new(value)
    }
}
