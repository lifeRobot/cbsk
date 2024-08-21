use std::net::SocketAddr;
use std::sync::Arc;
use cbsk_base::tokio::task::JoinHandle;
use cbsk_socket_tokio::cbsk_socket::tcp::server::config::TcpServerConfig;
use cbsk_socket_tokio::tcp::server::TcpServer;
use crate::server::business::CbskServerBusines;
use crate::server::callback::CbskServerCallBack;

pub mod client;
pub mod callback;
mod business;

/// cbsk server
pub struct CbskServer {
    /// tcp server
    tcp_server: Arc<TcpServer>,
}

/// custom method
impl CbskServer {
    /// new cbsk server<br />
    /// default log is false
    pub fn new<C: CbskServerCallBack>(cb: Arc<C>, addr: SocketAddr, buf_len: usize) -> Self {
        Self::new_with_tcp_config(cb, Self::default_tcp_config(addr).into(), buf_len)
    }

    /// use tcp server config create cbsk server
    pub fn new_with_tcp_config<C: CbskServerCallBack>(cb: Arc<C>, conf: Arc<TcpServerConfig>, buf_len: usize) -> Self {
        let cbsk_cb = business::CbskServerBusines::new(cb);
        Self::new_with_business(cbsk_cb.into(), conf, buf_len)
    }

    /// custom header create cbsk server
    pub fn new_with_header<C: CbskServerCallBack>(cb: Arc<C>, addr: SocketAddr, header: Vec<u8>, buf_len: usize) -> Self {
        let cbsk_cb = business::CbskServerBusines::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, Self::default_tcp_config(addr).into(), buf_len)
    }

    /// htc is an abbreviation for header_tcp_config
    pub fn new_with_htc<C: CbskServerCallBack>(cb: Arc<C>, header: Vec<u8>, conf: Arc<TcpServerConfig>, buf_len: usize) -> Self {
        let cbsk_cb = business::CbskServerBusines::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, conf, buf_len)
    }

    /// use business create cbsk server
    fn new_with_business<C: CbskServerCallBack>(cb: CbskServerBusines<C>, conf: Arc<TcpServerConfig>, buf_len: usize) -> Self {
        let tcp_server = TcpServer::new_with_buf_len(conf, cb, buf_len).into();
        Self { tcp_server }
    }

    /// get default tcp config
    pub fn default_tcp_config(addr: SocketAddr) -> TcpServerConfig {
        TcpServerConfig::new("cbsk".into(), addr, false)
    }

    /// start cbsk server
    pub async fn start(&self) {
        self.tcp_server.start().await
    }

    /// start cbsk server in join handle
    pub fn start_in_handle(&self) -> JoinHandle<()> {
        self.tcp_server.start_in_handle()
    }

    /// get tcp config
    pub fn get_config(&self) -> Arc<TcpServerConfig> {
        self.tcp_server.conf.clone()
    }
}
