use std::net::SocketAddr;
use std::sync::Arc;
use cbsk_socket::tcp::common::server::config::TcpServerConfig;
use cbsk_socket::tcp::rayon::server::TcpServer;
use crate::rayon_runtime::server::business::CbskServerBusines;
use crate::rayon_runtime::server::callback::CbskServerCallBack;

pub mod business;
pub mod callback;
pub mod client;

/// cbsk server
pub struct CbskServer<C: CbskServerCallBack> {
    /// tcp server
    tcp_server: Arc<TcpServer<CbskServerBusines<C>>>,
}


/// custom method
impl<C: CbskServerCallBack> CbskServer<C> {
    /// new cbsk server<br />
    /// default log is false
    pub fn new(cb: Arc<C>, addr: SocketAddr) -> Self {
        Self::new_with_tcp_config(cb, Self::default_tcp_config(addr).into())
    }

    /// use tcp server config create cbsk server
    pub fn new_with_tcp_config(cb: Arc<C>, conf: Arc<TcpServerConfig>) -> Self {
        let cbsk_cb = business::CbskServerBusines::new(cb);
        Self::new_with_business(cbsk_cb.into(), conf)
    }

    /// custom header create cbsk server
    pub fn new_with_header(cb: Arc<C>, addr: SocketAddr, header: Vec<u8>) -> Self {
        let cbsk_cb = business::CbskServerBusines::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, Self::default_tcp_config(addr).into())
    }

    /// htc is an abbreviation for header_tcp_config
    pub fn new_with_htc(cb: Arc<C>, header: Vec<u8>, conf: Arc<TcpServerConfig>) -> Self {
        let cbsk_cb = business::CbskServerBusines::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, conf)
    }

    /// use business create cbsk server
    fn new_with_business(cb: CbskServerBusines<C>, conf: Arc<TcpServerConfig>) -> Self {
        let tcp_server = TcpServer::new(conf, cb.into()).into();
        Self { tcp_server }
    }

    /// get default tcp config
    pub fn default_tcp_config(addr: SocketAddr) -> TcpServerConfig {
        TcpServerConfig::new("cbsk".into(), addr, false)
    }

    /// start cbsk server
    /// N: TCP read data bytes size at once, usually 1024, If you need to accept big data, please increase this value<br />
    /// please ensure that the main thread does not end, otherwise this TCP will automatically end, more see [TcpServer::start]
    pub fn start<const N: usize>(&self) {
        self.tcp_server.start::<N>()
    }

    /// get tcp config
    pub fn get_config(&self) -> Arc<TcpServerConfig> {
        self.tcp_server.conf.clone()
    }

    /// get callback
    pub fn get_callback(&self) -> Arc<CbskServerBusines<C>> {
        self.tcp_server.cb.clone()
    }
}