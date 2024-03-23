use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use cbsk_socket::config::re_conn::SocketReConn;
use cbsk_socket::tcp::client::config::TcpClientConfig;
use cbsk_socket::tcp::client::TcpClient;
use crate::client::callback::CbskClientCallBack;

pub mod callback;
mod business;

/// cbsk client
pub struct CbskClient<C: CbskClientCallBack> {
    /// tcp client
    tcp_client: Arc<TcpClient<business::CbskClientBusiness<C>>>,
}

/// custom method
impl<C: CbskClientCallBack> CbskClient<C> {
    /// new cbsk client<br />
    /// if the tcp connection is disconnected, it will reconnect after 3 seconds
    pub fn new(cb: Arc<C>, addr: SocketAddr) -> Self {
        let tcp_config = TcpClientConfig::new("cbsk".into(), addr, SocketReConn::enable(Duration::from_secs(3)));
        Self::new_with_tcp_config(cb, tcp_config.into())
    }

    /// use tcp client config create cbsk client
    pub fn new_with_tcp_config(cb: Arc<C>, conf: Arc<TcpClientConfig>) -> Self {
        let cbsk_cb = business::CbskClientBusiness::new(cb);
        let tcp_client = TcpClient::new(conf, cbsk_cb.into()).into();
        Self { tcp_client }
    }
}

/// support use tcp client method etc
impl<C: CbskClientCallBack> Deref for CbskClient<C> {
    type Target = TcpClient<business::CbskClientBusiness<C>>;

    fn deref(&self) -> &Self::Target {
        self.tcp_client.as_ref()
    }
}
