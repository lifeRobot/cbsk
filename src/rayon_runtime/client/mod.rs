use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use cbsk_s_rayon::client::TcpClient;
use cbsk_socket::config::re_conn::SocketReConn;
use cbsk_socket::tcp::common::client::config::TcpClientConfig;
use cbsk_socket::tcp::common::sync::tcp_write_trait::TcpWriteTrait;
use crate::business::cbsk_write_trait_thread::CbskWriteTrait;
use crate::business::client_callback_thread::CbskClientCallBack;
use crate::rayon_runtime::client::business::CbskClientBusines;

pub mod business;


/// cbsk client
pub struct CbskClient {
    /// tcp client
    tcp_client: Arc<TcpClient>,
    /// cbsk header
    pub header: Arc<Vec<u8>>,
}

/// custom method
impl CbskClient {
    /// new cbsk client<br /><br />
    /// buf_len is tcp read data once lengle
    /// if the tcp connection is disconnected, it will reconnect after 3 seconds
    pub fn new<C: CbskClientCallBack>(cb: Arc<C>, addr: SocketAddr, buf_len: usize) -> Self {
        Self::new_with_tcp_config(cb, Self::default_tcp_config(addr).into(), buf_len)
    }

    /// use tcp client config create cbsk client<br />
    /// buf_len is tcp read data once lengle
    pub fn new_with_tcp_config<C: CbskClientCallBack>(cb: Arc<C>, conf: Arc<TcpClientConfig>, buf_len: usize) -> Self {
        let cbsk_cb = business::CbskClientBusines::new(cb);
        Self::new_with_business(cbsk_cb, conf, buf_len)
    }

    /// custom header create cbsk client<br />
    /// buf_len is tcp read data once lengle
    pub fn new_with_header<C: CbskClientCallBack>(cb: Arc<C>, addr: SocketAddr, header: Vec<u8>, buf_len: usize) -> Self {
        let cbsk_cb = business::CbskClientBusines::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, Self::default_tcp_config(addr).into(), buf_len)
    }

    /// htc is an abbreviation for header_tcp_config<br />
    /// buf_len is tcp read data once lengle
    pub fn new_with_htc<C: CbskClientCallBack>(cb: Arc<C>, header: Vec<u8>, conf: Arc<TcpClientConfig>, buf_len: usize) -> Self {
        let cbsk_cb = business::CbskClientBusines::new_with_head(cb, header);
        Self::new_with_business(cbsk_cb, conf, buf_len)
    }

    /// use business create cbsk client<br />
    /// buf_len is tcp read data once lengle
    fn new_with_business<C: CbskClientCallBack>(mut cb: CbskClientBusines<C>, conf: Arc<TcpClientConfig>, buf_len: usize) -> Self {
        let header = cb.header.clone();
        cb.log_head = conf.log_head.clone();
        let tcp_client = TcpClient::new_with_buf_len(conf.clone(), buf_len, cb).into();
        Self { tcp_client, header }
    }

    /// get default tcp config
    pub fn default_tcp_config(addr: SocketAddr) -> TcpClientConfig {
        TcpClientConfig::new("cbsk".into(), addr, SocketReConn::enable(Duration::from_secs(3)))
    }

    /// start cbsk client
    pub fn start(&self) {
        self.tcp_client.start()
    }

    /// get has the cbsk server connection been success
    pub fn is_connected(&self) -> bool {
        self.tcp_client.is_connected()
    }

    /// stop cbsk server connect<br />
    /// will shutdown tcp connection and will not new connection
    pub fn stop(&self) {
        self.tcp_client.stop();
    }

    /// notify tcp to re connect<br />
    /// will shutdown tcp connection, if [`TcpClientConfig`] reconn is disable<br />
    /// will shutdown and create new tcp connection,if [`TcpClientConfig`] reconn is enable
    pub fn re_conn(&self) {
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
}

/// support write data to cbsk
impl CbskWriteTrait for CbskClient {
    fn get_log_head(&self) -> &str {
        self.tcp_client.get_log_head()
    }

    fn try_send_bytes(&self, bytes: Vec<u8>) -> cbsk_base::anyhow::Result<()> {
        let frame = crate::business::frame(bytes, self.header.as_ref());
        self.tcp_client.try_send_bytes(frame.as_slice())
    }
}
