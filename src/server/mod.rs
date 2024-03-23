use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use cbsk_socket::tcp::server::config::TcpServerConfig;
use cbsk_socket::tcp::server::TcpServer;
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
        let tcp_config = TcpServerConfig::new("cbsk".into(), addr, false);
        Self::new_with_tcp_config(cb, tcp_config.into())
    }

    /// use tcp server config create cbsk server
    pub fn new_with_tcp_config(cb: Arc<C>, conf: Arc<TcpServerConfig>) -> Self {
        let cbsk_cb = business::CbskServerBusiness::new(cb);
        let tcp_server = TcpServer::new(conf, cbsk_cb.into()).into();
        Self { tcp_server }
    }
}

/// support use tcp server method etc
impl<C: CbskServerCallBack> Deref for CbskServer<C> {
    type Target = TcpServer<business::CbskServerBusiness<C>>;

    fn deref(&self) -> &Self::Target {
        self.tcp_server.as_ref()
    }
}
