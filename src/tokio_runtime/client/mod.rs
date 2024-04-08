use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use cbsk_socket::cbsk_base::tokio::task::JoinHandle;
use cbsk_socket::config::re_conn::SocketReConn;
use cbsk_socket::tcp::client::config::TcpClientConfig;
use cbsk_socket::tcp::client::TcpClient;
use cbsk_socket::tcp::common::r#async::tcp_write_trait::TcpWriteTrait;
use crate::business::cbsk_write_trait::CbskWriteTrait;
use crate::client::business::CbskClientBusiness;
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
        Self::new_with_tcp_config(cb, Self::default_tcp_config(addr).into())
    }

    /// use tcp client config create cbsk client
    pub fn new_with_tcp_config(cb: Arc<C>, conf: Arc<TcpClientConfig>) -> Self {
        let cbsk_cb = business::CbskClientBusiness::new(cb);
        Self::new_with_business(cbsk_cb, conf)
    }

    /// custom header create cbsk client
    pub fn new_with_header(cb: Arc<C>, addr: SocketAddr, header: Vec<u8>) -> Self {
        let cbsk_cb = business::CbskClientBusiness::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, Self::default_tcp_config(addr).into())
    }

    /// htc is an abbreviation for header_tcp_config
    pub fn new_with_htc(cb: Arc<C>, header: Vec<u8>, conf: Arc<TcpClientConfig>) -> Self {
        let cbsk_cb = business::CbskClientBusiness::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, conf)
    }

    /// use business create cbsk client
    fn new_with_business(cb: CbskClientBusiness<C>, conf: Arc<TcpClientConfig>) -> Self {
        let tcp_client = TcpClient::new(conf, cb.into()).into();
        Self { tcp_client }
    }

    /// get default tcp config
    pub fn default_tcp_config(addr: SocketAddr) -> TcpClientConfig {
        TcpClientConfig::new("cbsk".into(), addr, SocketReConn::enable(Duration::from_secs(3)))
    }

    /// start cbsk client
    /// N: TCP read data bytes size at once, usually 1024, If you need to accept big data, please increase this value
    pub fn start<const N: usize>(&self) -> JoinHandle<()> {
        self.tcp_client.start::<N>()
    }

    /// get has the cbsk server connection been success
    pub fn is_connected(&self) -> bool {
        self.tcp_client.is_connected()
    }

    /// stop cbsk server connect<br />
    /// will shutdown tcp connection and will not new connection
    pub async fn stop(&self) {
        #[cfg(feature = "tokio_tcp")]
        self.tcp_client.stop().await;
        #[cfg(feature = "system_tcp")]
        self.tcp_client.stop();
    }

    /// notify tcp to re connect<br />
    /// will shutdown tcp connection, if [`TcpClientConfig`] reconn is disable<br />
    /// will shutdown and create new tcp connection,if [`TcpClientConfig`] reconn is enable
    pub async fn re_conn(&self) {
        #[cfg(feature = "tokio_tcp")]
        self.tcp_client.re_conn().await;
        #[cfg(feature = "system_tcp")]
        self.tcp_client.re_conn();
    }

    /// the last time the data was received
    pub fn get_recv_time(&self) -> i64 {
        **self.tcp_client.recv_time
    }

    /// get tcp config
    pub fn get_config(&self) -> Arc<TcpClientConfig> {
        self.tcp_client.conf.clone()
    }

    /// get callback
    pub fn get_callback(&self) -> Arc<CbskClientBusiness<C>> {
        self.tcp_client.cb.clone()
    }
}

/// support write data to cbsk
impl<C: CbskClientCallBack> CbskWriteTrait for CbskClient<C> {
    fn get_log_head(&self) -> &str {
        self.tcp_client.get_log_head()
    }

    async fn try_send_bytes(&self, bytes: Vec<u8>) -> cbsk_base::anyhow::Result<()> {
        let frame = crate::business::frame(bytes, self.tcp_client.cb.header.as_ref());
        self.tcp_client.try_send_bytes(frame.as_slice()).await
    }
}