use std::time::Duration;
use cbsk_socket::config::re_conn::SocketReConn;

/// websocket client config
pub struct WsClientConfig {
    /// name, used for log printing
    pub name: String,
    /// ws url<br />
    /// example: ws://127.0.0.1:8080
    pub ws_url: String,
    /// internal log name, used for log printing
    pub log_head: String,
    /// websocket connect timeout
    pub conn_time_out: Duration,
    /// websocket read data timeout
    pub read_time_out: Duration,
    /// websocket sockets need to be reconnect
    pub(crate) reconn: SocketReConn,
}

/// custom method
impl WsClientConfig {
    /// create a websocket client config<br />
    /// conn_time_out default 10 secs<br />
    /// read_time_out default 1 secs
    pub fn new(name: String, ws_url: String, reconn: SocketReConn) -> Self {
        let log_head = format!("{}[{}]", name, ws_url);
        Self {
            name,
            ws_url,
            log_head,
            conn_time_out: Duration::from_secs(10),
            read_time_out: Duration::from_secs(1),
            reconn: reconn,
        }
    }

    /// set websocket connect timeout
    pub fn set_conn_time_out(mut self, time_out: Duration) -> Self {
        self.conn_time_out = time_out;
        self
    }

    /// set websocket read data timeout
    pub fn set_read_time_out(mut self, time_out: Duration) -> Self {
        self.read_time_out = time_out;
        self
    }
}
