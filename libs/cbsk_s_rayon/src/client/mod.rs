use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use cbsk_base::{anyhow, log};
use cbsk_mut_data::mut_data_obj::MutDataObj;
use cbsk_mut_data::mut_data_vec::MutDataVec;
use cbsk_socket::tcp::common::client::config::TcpClientConfig;
use cbsk_socket::tcp::common::sync::tcp_write_trait::TcpWriteTrait;
use cbsk_socket::tcp::common::tcp_time_trait::TcpTimeTrait;
use crate::client::callback::TcpClientCallBack;
use crate::client::state::TcpState;
use crate::runtime::runtime;

pub mod callback;
pub mod state;

/// tcp client
#[derive(Clone)]
pub struct TcpClient {
    /// tcp client config
    pub conf: Arc<TcpClientConfig>,
    /// tcp client business callback
    pub cb: Arc<Box<dyn TcpClientCallBack>>,
    /// the last time the data was received<br />
    /// Because some Linux systems have network disconnections,
    /// but the system still considers TCP connections to be normal,
    /// a new last data reception time has been added,
    /// allowing users to determine whether they need to reconnect to TCP on their own<br />
    /// time see [fastdate::DateTime::unix_timestamp_millis]
    pub recv_time: Arc<MutDataObj<i64>>,
    /// the tcp last read timeout
    /// time see [fastdate::DateTime::unix_timestamp_millis]
    pub timeout_time: Arc<MutDataObj<i64>>,
    /// tcp client
    tcp_client: Arc<MutDataObj<Option<Arc<MutDataObj<TcpStream>>>>>,
    /// is wait callback
    wait_callback: Arc<MutDataObj<bool>>,
    /// read data buf len
    buf_len: usize,
    /// read data buf
    buf: Arc<MutDataVec<u8>>,
    /// next buf data
    next_buf: Arc<MutDataVec<u8>>,
    /// tcp client state
    pub(crate) state: Arc<MutDataObj<TcpState>>,
}

/// support tcp time trait
impl TcpTimeTrait for TcpClient {
    fn set_recv_time(&self, time: i64) {
        self.recv_time.set(time)
    }
    fn get_recv_time(&self) -> i64 {
        **self.recv_time
    }
    fn set_timeout_time(&self, time: i64) {
        self.timeout_time.set(time)
    }
    fn get_timeout_time(&self) -> i64 {
        **self.timeout_time
    }
    fn set_wait_callback(&self, is_wait: bool) {
        self.wait_callback.set(is_wait)
    }
    fn get_wait_callback(&self) -> bool {
        **self.wait_callback
    }
}

/// support tcp write trait
impl TcpWriteTrait for TcpClient {
    fn get_log_head(&self) -> &str {
        self.conf.log_head.as_str()
    }

    fn try_send_bytes(&self, bytes: &[u8]) -> anyhow::Result<()> {
        let mut tcp_client = self.tcp_client.as_ref().as_ref().as_ref().ok_or_else(|| {
            anyhow::anyhow!("try send data to server, but connect to tcp server not yet")
        })?.as_mut();

        tcp_client.write_all(bytes)?;
        tcp_client.flush()?;
        Ok(())
    }
}

/// custom method
impl TcpClient {
    /// stop tcp server connect<br />
    /// will shutdown tcp connection and will not new connection
    pub fn stop(&self) {
        self.conf.reconn.as_mut().enable = false;
        self.shutdown();
    }

    /// notify tcp to re connect<br />
    /// will shutdown tcp connection, if [`TcpClientConfig`] reconn is disable<br />
    /// will shutdown and create new tcp connection,if [`TcpClientConfig`] reconn is enable
    pub fn re_conn(&self) {
        self.shutdown();
    }

    /// shutdown tcp read and write
    fn shutdown(&self) {
        if let Some(tcp_client) = self.tcp_client.as_ref().as_ref() {
            if let Err(e) = tcp_client.shutdown(Shutdown::Both) {
                log::error!("shutdown tcp error: {e:?}");
            }
        }

        // as long as shutdown is called, tcp_client will be left blank directly
        self.tcp_client.set_none();
    }
}

/// custom method
impl TcpClient {
    /// create tcp client<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn new<C: TcpClientCallBack>(conf: Arc<TcpClientConfig>, cb: C) -> Self {
        Self::new_with_buf_len(conf, 1024, cb)
    }

    /// use custom read data buf len create tcp client
    pub fn new_with_buf_len<C: TcpClientCallBack>(conf: Arc<TcpClientConfig>, buf_len: usize, cb: C) -> Self {
        Self {
            conf,
            cb: Arc::new(Box::new(cb)),
            recv_time: MutDataObj::new(Self::now()).into(),
            timeout_time: MutDataObj::new(Self::now()).into(),
            tcp_client: Arc::new(MutDataObj::default()),
            wait_callback: Arc::new(MutDataObj::default()),
            buf_len,
            buf: MutDataVec::default().into(),
            next_buf: MutDataVec::with_capacity(buf_len).into(),
            state: Arc::new(MutDataObj::default()),
        }
    }

    /// start tcp client
    pub fn start(&self) {
        self.buf.set(vec![0; self.buf_len]);
        runtime.tcp_client.push(self.clone());
        runtime.start();
    }

    /// get has the tcp server connection been success
    pub fn is_connected(&self) -> bool {
        self.tcp_client.is_some()
    }

    /// conn tcp server
    pub(crate) fn conn(&self) {
        if self.state.first {
            self.state.as_mut().first = false;
            self.conn_exec();
            return;
        }

        // not first conn, check is re conn
        if !self.conf.reconn.enable {
            return;
        }

        // re conn
        let diff = u128::try_from(Self::now() - self.state.last_re_time).unwrap_or_default();
        if diff < self.conf.reconn.time.as_millis() { return; }
        self.state.as_mut().re_num = self.state.re_num.saturating_add(1);
        self.cb.re_conn(self.state.re_num);
        self.conn_exec();
    }

    /// exec connection to tcp server
    fn conn_exec(&self) {
        self.state.as_mut().last_re_time = Self::now();
        let ts =
            match self.try_conn() {
                Ok(tcp_stream) => { tcp_stream }
                Err(e) => {
                    log::error!("{} tcp server connect error: {e:?}",self.conf.log_head);
                    if self.conf.reconn.enable {
                        log::info!("{} tcp service will reconnect in {:?}",self.conf.log_head,self.conf.reconn.time);
                    }
                    return;
                }
            };

        let ts = Arc::new(MutDataObj::new(ts));
        self.tcp_client.set_some(ts);
        self.cb.conn();
    }

    /// try conn to tcp server
    fn try_conn(&self) -> io::Result<TcpStream> {
        log::info!("{} try connect to tcp server",self.conf.log_head);
        let tcp_stream = TcpStream::connect_timeout(&self.conf.addr, self.conf.conn_time_out)?;
        if let Err(e) = tcp_stream.set_read_timeout(Some(self.conf.read_time_out)) {
            log::error!("{}set tcp read timeout fail: {e:?}",self.conf.log_head);
        }

        self.state.as_mut().re_num = 0;
        log::info!("{} tcp server connect success",self.conf.log_head);
        Ok(tcp_stream)
    }

    /// read data from tcp server
    pub(crate) fn read(&self) {
        self.state.as_mut().reading = true;
        if let Err(e) = self.try_read() {
            if self.is_connected() {
                log::error!("{} tcp server read data error: {e:?}",self.conf.log_head);
            }
            // read error, directly assume that the tcp client has been closed
            self.shutdown();
            self.cb.dis_conn();
            log::info!("{} tcp server shutdown",self.conf.log_head);
        }
        self.state.as_mut().reading = false;
    }

    /// try read data from tcp server
    fn try_read(&self) -> anyhow::Result<()> {
        self.set_now();

        let mut ts = self.tcp_client.as_ref().as_ref().as_ref().ok_or_else(|| { anyhow::anyhow!("tcp server not connection") })?.as_mut();
        let len =
            match ts.read(self.buf.as_mut().as_mut_slice()) {
                Ok(len) => { len }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                            // timeout just return
                            self.set_timeout_time_now();
                            return Ok(());
                        }
                        _ => {
                            return Err(e.into());
                        }
                    }
                }
            };

        // reading a length of 0, it is assumed that the connection has been disconnected
        if len == 0 { return Err(anyhow::anyhow!("read data length is 0, indicating that tcp server is disconnected")); }

        // set recv time
        self.set_recv_time_now();
        let buf = self.buf.get(0..len).unwrap_or_default();
        self.next_buf.append(&mut buf.to_vec());
        self.wait_callback();
        let mut temp = self.cb.recv(self.next_buf.as_mut().drain(0..).collect());
        self.finish_callback();

        if temp.capacity() < self.buf_len {
            let mut new_tmp = Vec::with_capacity(self.buf_len);
            new_tmp.append(&mut temp);
            temp = new_tmp;
        }
        self.next_buf.set(temp);

        Ok(())
    }

    /// check tcp read is finished
    pub(crate) fn check_read_finished(&self) {
        let check_time_out = i64::try_from(self.conf.read_time_out.as_millis()).unwrap_or(1_000) + 1_000;
        let now = Self::now();
        let timeout_diff = now - self.get_timeout_time();
        let recv_diff = now - self.get_recv_time();

        if !self.get_wait_callback() && timeout_diff > check_time_out && recv_diff > check_time_out {
            // tcp read timeout, directly assuming that tcp has been disconnected
            self.shutdown();
        }
    }
}
