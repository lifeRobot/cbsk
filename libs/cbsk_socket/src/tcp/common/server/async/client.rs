use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::tcp::common::r#async::async_tcp_time_trait::AsyncTcpTimeTrait;
use crate::tcp::common::r#async::tcp_write_trait::TcpWriteTrait;
use crate::tcp::common::server::r#async::client_write::ClientWrite;
use crate::tcp::common::server::config::TcpServerConfig;
use crate::tcp::common::tcp_time_trait::TcpTimeTrait;
#[cfg(feature = "system_tcp")]
use crate::tcp::system::system_tcp_read_trait::SystemTcpReadTrait;
#[cfg(feature = "tokio_tcp")]
use crate::tcp::tokio::tokio_tcp_read_trait::TokioTcpReadTrait;

/// tcp client
pub struct TcpServerClient {
    /// tcp client addr
    pub addr: SocketAddr,
    /// internal log name
    pub log_head: String,
    /// the last time the data was received<br />
    /// time see [fastdate::DateTime::unix_timestamp_millis]
    pub recv_time: Arc<MutDataObj<i64>>,
    /// the tcp last read timeout<br />
    /// time see [fastdate::DateTime::unix_timestamp_millis]
    pub timeout_time: Arc<MutDataObj<i64>>,
    /// tcp client write
    pub write: Arc<MutDataObj<ClientWrite>>,
    /// is wait callback
    wait_callback: Arc<MutDataObj<bool>>,
}

/// custom method
impl TcpServerClient {
    /// create tcp server client
    pub fn new(addr: SocketAddr, conf: &TcpServerConfig, write: ClientWrite) -> Self {
        let log_head = format!("{} tcp client[{}]", conf.name, addr);
        Self {
            addr,
            log_head,
            recv_time: MutDataObj::new(Self::now()).into(),
            timeout_time: MutDataObj::new(Self::now()).into(),
            write: MutDataObj::new(write).into(),
            wait_callback: Arc::new(Default::default()),
        }
    }
}

/// support tcp time trait
impl TcpTimeTrait for TcpServerClient {
    fn set_recv_time(&self, time: i64) {
        self.recv_time.set(time)
    }
    fn get_recv_time(&self) -> i64 {
        **self.recv_time
    }
    fn set_timeout_time(&self, time: i64) {
        self.timeout_time.set(time)
    }
    fn get_timeout_time(&self) -> i64 {
        **self.timeout_time
    }
    fn set_wait_callback(&self, is_wait: bool) {
        self.wait_callback.set(is_wait)
    }
    fn get_wait_callback(&self) -> bool {
        **self.wait_callback
    }
}

/// support tcp time trait
impl AsyncTcpTimeTrait for TcpServerClient {
    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
    }
}

/// support tcp write trait
impl TcpWriteTrait for TcpServerClient {
    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
    }

    async fn try_send_bytes(&self, bytes: &[u8]) -> io::Result<()> {
        self.write.as_mut().try_send_bytes(bytes).await
    }
}

/// support tcp read trait
#[cfg(feature = "tokio_tcp")]
impl TokioTcpReadTrait for TcpServerClient {}

/// support system tcp read trait
#[cfg(feature = "system_tcp")]
impl SystemTcpReadTrait for TcpServerClient {}
