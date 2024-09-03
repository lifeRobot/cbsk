use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use cbsk_base::tokio::io::AsyncWriteExt;
use cbsk_base::tokio::net::tcp::OwnedWriteHalf;
use cbsk_base::tokio::sync::RwLock;
use cbsk_socket::tcp::common::time_trait::TimeTrait;
use cbsk_socket::tcp::server::config::TcpServerConfig;
use crate::tcp::common::read_trait::ReadTrait;
use crate::tcp::common::tcp_write_trait::TcpWriteTrait;

/// tcp client
pub struct TcpServerClient {
    /// tcp client addr
    pub addr: SocketAddr,
    /// internal log name
    pub log_head: String,
    /// the last time the data was received<br />
    /// time see [cbsk_base::fastdate::DateTime::unix_timestamp_millis]
    pub recv_time: Arc<AtomicI64>,
    /// the tcp last read timeout<br />
    /// time see [cbsk_base::fastdate::DateTime::unix_timestamp_millis]
    pub timeout_time: Arc<AtomicI64>,
    /// is ignore once time check
    pub ignore_once: Arc<AtomicBool>,
    /// tcp client write
    pub write: Arc<RwLock<OwnedWriteHalf>>,
    /// is wait callback
    wait_callback: Arc<AtomicBool>,
}

/// custom method
impl TcpServerClient {
    /// create tcp server client
    pub fn new(addr: SocketAddr, conf: &TcpServerConfig, write: OwnedWriteHalf) -> Self {
        let log_head = format!("{} tcp client[{}]", conf.name, addr);
        Self {
            addr,
            log_head,
            recv_time: AtomicI64::new(Self::now()).into(),
            timeout_time: AtomicI64::new(Self::now()).into(),
            ignore_once: AtomicBool::default().into(),
            write: Arc::new(RwLock::new(write)),
            wait_callback: Arc::new(Default::default()),
        }
    }
}

/// support tcp time trait
impl TimeTrait for TcpServerClient {
    fn set_recv_time(&self, time: i64) {
        self.recv_time.store(time, Ordering::Release)
    }
    fn get_recv_time(&self) -> i64 {
        self.recv_time.load(Ordering::Acquire)
    }
    fn set_timeout_time(&self, time: i64) {
        self.timeout_time.store(time, Ordering::Release)
    }
    fn get_timeout_time(&self) -> i64 {
        self.timeout_time.load(Ordering::Acquire)
    }
    fn set_wait_callback(&self, is_wait: bool) {
        self.wait_callback.store(is_wait, Ordering::Release)
    }
    fn get_wait_callback(&self) -> bool {
        self.wait_callback.load(Ordering::Acquire)
    }
    fn set_ignore_once(&self, is_ignore: bool) {
        self.ignore_once.store(is_ignore, Ordering::Release)
    }
    fn get_ignore(&self) -> bool {
        self.ignore_once.load(Ordering::Acquire)
    }
}

/// support tcp write trait
impl TcpWriteTrait for TcpServerClient {
    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
    }

    async fn try_send_bytes(&self, bytes: &[u8]) -> std::io::Result<()> {
        let mut write = self.write.write().await;
        write.write_all(bytes).await?;
        write.flush().await
    }
}

/// support tcp read trait
impl ReadTrait for TcpServerClient {
    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
    }
}
