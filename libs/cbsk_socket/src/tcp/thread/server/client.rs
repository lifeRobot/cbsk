use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::tcp::common::server::config::TcpServerConfig;
use crate::tcp::common::sync::sync_tcp_time_trait::SyncTcpTimeTrait;
use crate::tcp::common::sync::tcp_write_trait::TcpWriteTrait;
use crate::tcp::common::tcp_time_trait::TcpTimeTrait;
use crate::tcp::thread::thread_tcp_time_trait::ThreadTcpTimeTrait;

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
    pub(crate) write: Arc<MutDataObj<TcpStream>>,
}

/// custom method
impl TcpServerClient {
    /// create tcp server client
    pub fn new(addr: SocketAddr, conf: &TcpServerConfig, write: Arc<MutDataObj<TcpStream>>) -> Self {
        let log_head = format!("{} tcp client[{}]", conf.name, addr);
        Self {
            addr,
            log_head,
            recv_time: MutDataObj::new(Self::now()).into(),
            timeout_time: MutDataObj::new(Self::now()).into(),
            write,
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
}

/// support tcp time trait
impl SyncTcpTimeTrait for TcpServerClient {
    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
    }
}

/// support tcp time trait
impl ThreadTcpTimeTrait for TcpServerClient {}

/// support tcp write trait
impl TcpWriteTrait for TcpServerClient {
    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
    }

    fn try_send_bytes(&self, bytes: &[u8]) -> cbsk_base::anyhow::Result<()> {
        let mut tcp_client = self.write.as_mut();

        tcp_client.write_all(bytes)?;
        tcp_client.flush()?;
        Ok(())
    }
}
