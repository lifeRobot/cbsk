use std::net::SocketAddr;
use std::sync::Arc;
use cbsk::business;
use cbsk_socket_tokio::cbsk_socket::tcp::common::time_trait::TimeTrait;
use cbsk_socket_tokio::tcp::common::tcp_write_trait::TcpWriteTrait;
use cbsk_socket_tokio::tcp::server::client::TcpServerClient;
use crate::business::cbsk_write_trait::CbskWriteTrait;

/// cbsk server client
pub struct CbskServerClient {
    /// the cbsk first frame<br />
    /// Used to determine if it is cbsk data
    pub header: Arc<Vec<u8>>,
    /// tcp server client
    tcp_server_client: Arc<TcpServerClient>,
}

/// custom method
impl CbskServerClient {
    /// create cbsk server client
    pub(crate) fn new(header: Arc<Vec<u8>>, tcp_server_client: Arc<TcpServerClient>) -> Self {
        Self { header, tcp_server_client }
    }

    /// get client addr
    pub fn get_addr(&self) -> SocketAddr {
        self.tcp_server_client.addr
    }
}

/// support cbsk write trait
impl CbskWriteTrait for CbskServerClient {
    fn get_log_head(&self) -> &str {
        self.tcp_server_client.get_log_head()
    }

    async fn try_send_bytes(&self, bytes: Vec<u8>) -> std::io::Result<()> {
        let frame = business::frame(bytes, self.header.as_slice());
        self.tcp_server_client.try_send_bytes(frame.as_slice()).await
    }
}

/// support time trait
impl TimeTrait for CbskServerClient {
    fn set_recv_time(&self, time: i64) {
        self.tcp_server_client.set_recv_time(time);
    }
    fn get_recv_time(&self) -> i64 {
        self.tcp_server_client.get_recv_time()
    }
    fn set_timeout_time(&self, time: i64) {
        self.tcp_server_client.set_timeout_time(time)
    }
    fn get_timeout_time(&self) -> i64 {
        self.tcp_server_client.get_timeout_time()
    }
    fn set_wait_callback(&self, is_wait: bool) {
        self.tcp_server_client.set_wait_callback(is_wait)
    }
    fn get_wait_callback(&self) -> bool {
        self.tcp_server_client.get_wait_callback()
    }
    fn set_ignore_once(&self, is_ignore: bool) {
        self.tcp_server_client.set_ignore_once(is_ignore)
    }
    fn get_ignore(&self) -> bool {
        self.tcp_server_client.get_ignore()
    }
}
