use std::net::SocketAddr;
use std::time::Duration;

/// tcp server config
pub struct TcpServerConfig {
    /// name, used for log printing
    pub name: String,
    /// tcp bind addr
    pub addr: SocketAddr,
    /// internal log name, used for log printing
    pub(crate) log_head: String,
    /// TCP read time out
    pub read_time_out: Duration,
    /// is enable log printing
    pub log: bool,
}

/// custom method
impl TcpServerConfig {
    /// create a new config<br />
    /// name: business name, used for log printing<br />
    /// addr: TCP bind addr<br />
    /// log: is enable log printing
    pub fn new(name: String, addr: SocketAddr, log: bool) -> Self {
        let log_head = format!("{}[{}]", name, addr);
        Self { name, addr, log_head, read_time_out: Duration::from_secs(1), log }
    }

    /// set name
    pub fn set_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    /// set addr
    pub fn set_addr(mut self, addr: SocketAddr) -> Self {
        self.addr = addr;
        self
    }

    /// set read time out
    pub fn set_read_time_out(mut self, read_time_out: Duration) -> Self {
        self.read_time_out = read_time_out;
        self
    }

    /// set enable log printing
    pub fn set_log(mut self, log: bool) -> Self {
        self.log = log;
        self
    }
}
