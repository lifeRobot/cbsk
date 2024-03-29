use std::net::SocketAddr;
use std::time::Duration;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::config::re_conn::SocketReConn;

/// tcp client config
pub struct TcpClientConfig {
    /// name, used for log printing
    pub name: String,
    /// tcp server addr
    pub addr: SocketAddr,
    /// internal log name, used for log printing
    pub log_head: String,
    /// tcp connect timeout
    pub conn_time_out: Duration,
    /// tcp read data timeout
    pub read_time_out: Duration,
    /// tcp sockets need to be reconnect
    pub reconn: MutDataObj<SocketReConn>,
}

/// custom method
impl TcpClientConfig {
    /// create a tcp client config<br />
    /// conn_time_out default 10 secs<br />
    /// read_time_out default 1 secs
    pub fn new(name: String, addr: SocketAddr, reconn: SocketReConn) -> Self {
        let log_head = format!("{}[{}]", name, addr);
        Self {
            name,
            addr,
            log_head,
            conn_time_out: Duration::from_secs(10),
            read_time_out: Duration::from_secs(1),
            reconn: MutDataObj::new(reconn),
        }
    }

    /// set tcp connect timeout
    pub fn set_conn_time_out(mut self, time_out: Duration) -> Self {
        self.conn_time_out = time_out;
        self
    }

    /// set tcp read data timeout
    pub fn set_read_time_out(mut self, time_out: Duration) -> Self {
        self.read_time_out = time_out;
        self
    }
}
