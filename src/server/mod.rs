use std::net::SocketAddr;
use std::sync::Arc;
use cbsk_socket::cbsk_base::tokio::task::JoinHandle;
use cbsk_socket::tcp::server::config::TcpServerConfig;
use cbsk_socket::tcp::server::TcpServer;
use crate::server::business::CbskServerBusiness;
use crate::server::callback::CbskServerCallBack;

pub mod client;
pub mod callback;
mod business;

/// cbsk server
pub struct CbskServer<C: CbskServerCallBack> {
    /// tcp server
    tcp_server: Arc<TcpServer<business::CbskServerBusiness<C>>>,
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
        let cbsk_cb = business::CbskServerBusiness::new(cb);
        Self::new_with_business(cbsk_cb.into(), conf)
    }

    /// custom header create cbsk server
    pub fn new_with_header(cb: Arc<C>, addr: SocketAddr, header: Vec<u8>) -> Self {
        let cbsk_cb = business::CbskServerBusiness::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, Self::default_tcp_config(addr).into())
    }

    /// htc is an abbreviation for header_tcp_config
    pub fn new_with_htc(cb: Arc<C>, header: Vec<u8>, conf: Arc<TcpServerConfig>) -> Self {
        let cbsk_cb = business::CbskServerBusiness::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, conf)
    }

    /// use business create cbsk server
    fn new_with_business(cb: CbskServerBusiness<C>, conf: Arc<TcpServerConfig>) -> Self {
        let tcp_server = TcpServer::new(conf, cb.into()).into();
        Self { tcp_server }
    }

    /// get default tcp config
    pub fn default_tcp_config(addr: SocketAddr) -> TcpServerConfig {
        TcpServerConfig::new("cbsk".into(), addr, false)
    }

    /// start cbsk server
    /// N: TCP read data bytes size at once, usually 1024, If you need to accept big data, please increase this value
    pub fn start<const N: usize>(&self) -> JoinHandle<()> {
        self.tcp_server.start::<N>()
    }

    /// get tcp config
    pub fn get_config(&self) -> Arc<TcpServerConfig> {
        self.tcp_server.conf.clone()
    }

    /// get callback
    pub fn get_callback(&self) -> Arc<CbskServerBusiness<C>> {
        self.tcp_server.cb.clone()
    }
}
