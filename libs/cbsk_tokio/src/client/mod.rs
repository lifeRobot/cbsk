use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use cbsk_base::tokio::task::JoinHandle;
use cbsk_socket_tokio::cbsk_socket::config::re_conn::SocketReConn;
use cbsk_socket_tokio::cbsk_socket::tcp::client::config::TcpClientConfig;
use cbsk_socket_tokio::cbsk_socket::tcp::common::time_trait::TimeTrait;
use cbsk_socket_tokio::tcp::client::TcpClient;
use cbsk_socket_tokio::tcp::common::tcp_write_trait::TcpWriteTrait;
use crate::business::cbsk_write_trait::CbskWriteTrait;
use crate::client::business::CbskClientBusiness;
use crate::client::callback::CbskClientCallBack;

pub mod callback;
mod business;

/// cbsk client
pub struct CbskClient {
    /// tcp client
    tcp_client: Arc<TcpClient>,
    /// cbsk header
    pub header: Arc<Vec<u8>>,
}

/// custom method
impl CbskClient {
    /// new cbsk client<br />
    /// if the tcp connection is disconnected, it will reconnect after 3 seconds
    pub fn new<C: CbskClientCallBack>(cb: Arc<C>, addr: SocketAddr, buf_len: usize) -> Self {
        Self::new_with_tcp_config(cb, Self::default_tcp_config(addr).into(), buf_len)
    }

    /// use tcp client config create cbsk client
    pub fn new_with_tcp_config<C: CbskClientCallBack>(cb: Arc<C>, conf: Arc<TcpClientConfig>, buf_len: usize) -> Self {
        let cbsk_cb = business::CbskClientBusiness::new(cb);
        Self::new_with_business(cbsk_cb, conf, buf_len)
    }

    /// custom header create cbsk client
    pub fn new_with_header<C: CbskClientCallBack>(cb: Arc<C>, addr: SocketAddr, header: Vec<u8>, buf_len: usize) -> Self {
        let cbsk_cb = business::CbskClientBusiness::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, Self::default_tcp_config(addr).into(), buf_len)
    }

    /// htc is an abbreviation for header_tcp_config
    pub fn new_with_htc<C: CbskClientCallBack>(cb: Arc<C>, header: Vec<u8>, conf: Arc<TcpClientConfig>, buf_len: usize) -> Self {
        let cbsk_cb = business::CbskClientBusiness::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, conf, buf_len)
    }

    /// use business create cbsk client
    fn new_with_business<C: CbskClientCallBack>(cb: CbskClientBusiness<C>, conf: Arc<TcpClientConfig>, buf_len: usize) -> Self {
        let header = cb.header.clone();
        let tcp_client = TcpClient::new_with_buf_len(conf, buf_len, cb).into();
        Self { tcp_client, header }
    }

    /// get default tcp config
    pub fn default_tcp_config(addr: SocketAddr) -> TcpClientConfig {
        TcpClientConfig::new("cbsk".into(), addr, SocketReConn::enable(Duration::from_secs(3)))
    }

    /// start cbsk client
    pub async fn start(&self) {
        self.tcp_client.start().await
    }

    /// start cbsk client in join handle
    pub fn start_in_handle(&self) -> JoinHandle<()> {
        self.tcp_client.start_in_handle()
    }

    /// get has the cbsk server connection been success
    pub async fn is_connected(&self) -> bool {
        self.tcp_client.is_connected().await
    }

    /// stop cbsk server connect<br />
    /// will shutdown tcp connection and will not new connection
    pub async fn stop(&self) {
        self.tcp_client.stop().await;
    }

    /// notify tcp to re connect<br />
    /// will shutdown tcp connection, if [`TcpClientConfig`] reconn is disable<br />
    /// will shutdown and create new tcp connection,if [`TcpClientConfig`] reconn is enable
    pub async fn re_conn(&self) {
        self.tcp_client.re_conn().await;
    }

    /// the last time the data was received
    pub fn get_recv_time(&self) -> i64 {
        self.tcp_client.get_recv_time()
    }

    /// get tcp config
    pub fn get_config(&self) -> Arc<TcpClientConfig> {
        self.tcp_client.conf.clone()
    }
}

/// support write data to cbsk
impl CbskWriteTrait for CbskClient {
    fn get_log_head(&self) -> &str {
        self.tcp_client.get_log_head()
    }

    async fn try_send_bytes(&self, bytes: Vec<u8>) -> std::io::Result<()> {
        let frame = cbsk::business::frame(bytes, self.header.as_ref());
        self.tcp_client.try_send_bytes(frame.as_slice()).await
    }
}

/// support time trait
impl TimeTrait for CbskClient {
    fn set_recv_time(&self, time: i64) {
        self.tcp_client.set_recv_time(time)
    }
    fn get_recv_time(&self) -> i64 {
        self.tcp_client.get_recv_time()
    }
    fn set_timeout_time(&self, time: i64) {
        self.tcp_client.set_timeout_time(time)
    }
    fn get_timeout_time(&self) -> i64 {
        self.tcp_client.get_timeout_time()
    }
    fn set_wait_callback(&self, is_wait: bool) {
        self.tcp_client.set_wait_callback(is_wait)
    }
    fn get_wait_callback(&self) -> bool {
        self.tcp_client.get_wait_callback()
    }
    fn set_ignore_once(&self, is_ignore: bool) {
        self.tcp_client.set_ignore_once(is_ignore)
    }
    fn get_ignore(&self) -> bool {
        self.tcp_client.get_ignore()
    }
}
