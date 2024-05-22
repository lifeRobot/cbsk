use std::net::SocketAddr;
use std::sync::Arc;
use cbsk_s_rayon::server::TcpServer;
use cbsk_socket::tcp::common::server::config::TcpServerConfig;
use crate::rayon_runtime::server::business::CbskServerBusines;
use crate::rayon_runtime::server::callback::CbskServerCallBack;

pub mod business;
pub mod callback;
pub mod client;

/// cbsk server
pub struct CbskServer {
    /// tcp server
    tcp_server: Arc<TcpServer>,
    /// cbsk header
    pub header: Arc<Vec<u8>>,
}


/// custom method
impl CbskServer {
    /// new cbsk server<br />
    /// default log is false<br />
    /// buf_len is tcp read data once lengle
    pub fn new<C: CbskServerCallBack>(cb: Arc<C>, addr: SocketAddr, buf_len: usize) -> Self {
        Self::new_with_tcp_config(cb, Self::default_tcp_config(addr).into(), buf_len)
    }

    /// use tcp server config create cbsk server<br />
    /// buf_len is tcp read data once lengle
    pub fn new_with_tcp_config<C: CbskServerCallBack>(cb: Arc<C>, conf: Arc<TcpServerConfig>, buf_len: usize) -> Self {
        let cbsk_cb = business::CbskServerBusines::new(cb);
        Self::new_with_business(cbsk_cb.into(), conf, buf_len)
    }

    /// custom header create cbsk server<br />
    /// buf_len is tcp read data once lengle
    pub fn new_with_header<C: CbskServerCallBack>(cb: Arc<C>, addr: SocketAddr, header: Vec<u8>, buf_len: usize) -> Self {
        let cbsk_cb = business::CbskServerBusines::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, Self::default_tcp_config(addr).into(), buf_len)
    }

    /// htc is an abbreviation for header_tcp_config<br />
    /// buf_len is tcp read data once lengle
    pub fn new_with_htc<C: CbskServerCallBack>(cb: Arc<C>, header: Vec<u8>, conf: Arc<TcpServerConfig>, buf_len: usize) -> Self {
        let cbsk_cb = business::CbskServerBusines::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, conf, buf_len)
    }

    /// use business create cbsk server<br />
    /// buf_len is tcp read data once lengle
    fn new_with_business<C: CbskServerCallBack>(mut cb: CbskServerBusines<C>, conf: Arc<TcpServerConfig>, buf_len: usize) -> Self {
        let header = cb.header.clone();
        cb.log_head = conf.log_head.clone();
        let tcp_server = TcpServer::new_with_buf_len(conf, cb, buf_len).into();
        Self { tcp_server, header }
    }

    /// get default tcp config
    pub fn default_tcp_config(addr: SocketAddr) -> TcpServerConfig {
        TcpServerConfig::new("cbsk".into(), addr, false)
    }

    /// start cbsk server
    /// N: TCP read data bytes size at once, usually 1024, If you need to accept big data, please increase this value<br />
    /// please ensure that the main thread does not end, otherwise this TCP will automatically end, more see [TcpServer::start]
    pub fn start(&self) {
        self.tcp_server.start()
    }

    /// get tcp config
    pub fn get_config(&self) -> Arc<TcpServerConfig> {
        self.tcp_server.conf.clone()
    }
}