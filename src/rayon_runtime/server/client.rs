use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use cbsk_base::parking_lot;
use cbsk_s_rayon::server::client::TcpServerClient;
use cbsk_socket::tcp::common::sync::tcp_write_trait::TcpWriteTrait;
use crate::business;
use crate::business::cbsk_write_trait_thread::CbskWriteTrait;

/// cbsk server client
pub struct CbskServerClient {
    /// the cbsk first frame<br />
    /// Used to determine if it is cbsk data
    pub header: Arc<Vec<u8>>,
    /// tcp server client
    tcp_server_client: Arc<TcpServerClient>,
    /// cbsk write lock
    lock: parking_lot::Mutex<()>,
}


/// custom method
impl CbskServerClient {
    /// create cbsk server client
    pub(crate) fn new(header: Arc<Vec<u8>>, tcp_server_client: Arc<TcpServerClient>) -> Self {
        Self { header, tcp_server_client, lock: parking_lot::Mutex::new(()) }
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
    fn get_log_head(&self) -> &str {
        self.tcp_server_client.get_log_head()
    }

    fn try_send_bytes(&self, bytes: Vec<u8>) -> io::Result<()> {
        let lock = self.lock.lock();
        let frame = business::frame(bytes, self.header.as_slice());
        let result = self.tcp_server_client.try_send_bytes_no_lock(frame.as_slice());
        drop(lock);
        result
    }
}

