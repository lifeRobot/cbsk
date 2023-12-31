use std::time::Duration;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::config::re_conn::SocketReConn;

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
    pub(crate) reconn: MutDataObj<SocketReConn>,

}
