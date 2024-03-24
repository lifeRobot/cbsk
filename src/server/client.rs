use std::net::SocketAddr;
use std::sync::Arc;
use cbsk_socket::cbsk_base::anyhow;
use cbsk_socket::cbsk_base::tokio::net::tcp::OwnedWriteHalf;
use cbsk_socket::cbsk_mut_data::mut_data_obj::MutDataObj;
use cbsk_socket::tcp::server::client::TcpServerClient;
use cbsk_socket::tcp::tcp_write_trait::TcpWriteTrait;
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

    /// get internal log name
    pub fn get_log_head(&self) -> &str {
        self.tcp_server_client.get_log_head()
    }

    /// get client addr
    pub fn get_addr(&self) -> SocketAddr {
        self.tcp_server_client.addr
    }
}

impl CbskWriteTrait for CbskServerClient {
    fn try_get_write(&self) -> anyhow::Result<&MutDataObj<OwnedWriteHalf>> {
        self.tcp_server_client.try_get_write()
    }

    fn get_log_head(&self) -> &str {
        self.tcp_server_client.get_log_head()
    }

    fn get_header(&self) -> &[u8] {
        self.header.as_slice()
    }
}
