use std::net::SocketAddr;
use std::sync::Arc;
use cbsk_base::tokio::net::tcp::OwnedWriteHalf;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use fastdate::DateTime;
use crate::tcp::server::config::TcpServerConfig;
use crate::tcp::tcp_time_trait::TcpTimeTrait;
use crate::tcp::tcp_write_trait::TcpWriteTrait;

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
    write: Arc<MutDataObj<OwnedWriteHalf>>,
}

/// custom method
impl TcpServerClient {
    /// create tcp server client
    pub(crate) fn new(addr: SocketAddr, conf: &TcpServerConfig, writer: OwnedWriteHalf) -> Self {
        let log_head = format!("{} tcp client[{}]", conf.name, addr);
        Self {
            addr,
            log_head,
            recv_time: MutDataObj::new(DateTime::now().unix_timestamp_millis()).into(),
            timeout_time: MutDataObj::new(DateTime::now().unix_timestamp_millis()).into(),
            write: MutDataObj::new(writer).into(),
        }
    }
}

/// support writer trait
impl TcpWriteTrait for TcpServerClient {
    fn try_get_write(&self) -> cbsk_base::anyhow::Result<&MutDataObj<OwnedWriteHalf>> {
        Ok(self.write.as_ref())
    }

    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
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

    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
    }
}
