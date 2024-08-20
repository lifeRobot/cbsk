use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use cbsk_base::{anyhow, log};
use cbsk_base::parking_lot::RwLock;
use cbsk_socket::tcp::client::config::TcpClientConfig;
use cbsk_socket::tcp::common::time_trait::TimeTrait;
use cbsk_timer::timer::Timer;
use crate::tcp::client::callback::TcpClientCallBack;
use crate::tcp::client::state::TcpState;
use crate::tcp::client::tcp_read_write::TcpReadWrite;
use crate::tcp::common::tcp_write_trait::TcpWriteTrait;

pub mod callback;
pub mod state;
mod tcp_read_write;
mod timer;
pub mod timer_state;

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
    /// time see [cbsk_base::fastdate::DateTime::unix_timestamp_millis]
    pub recv_time: Arc<AtomicI64>,
    /// the tcp last read timeout
    /// time see [cbsk_base::fastdate::DateTime::unix_timestamp_millis]
    pub timeout_time: Arc<AtomicI64>,
    /// tcp client
    tcp_client: Arc<RwLock<TcpReadWrite>>,
    /// is wait callback
    wait_callback: Arc<AtomicBool>,
    /// read data buf len
    buf_len: usize,
    /// read data buf
    buf: Arc<RwLock<Vec<u8>>>,
    /// next buf data
    pub(crate) next_buf: Arc<RwLock<Vec<u8>>>,
    /// tcp client state
    pub(crate) state: Arc<RwLock<TcpState>>,
}

/// support tcp time trait
impl TimeTrait for TcpClient {
    fn set_recv_time(&self, time: i64) {
        self.recv_time.store(time, Ordering::Relaxed)
    }
    fn get_recv_time(&self) -> i64 {
        self.recv_time.load(Ordering::Relaxed)
    }
    fn set_timeout_time(&self, time: i64) {
        self.timeout_time.store(time, Ordering::Relaxed)
    }
    fn get_timeout_time(&self) -> i64 {
        self.timeout_time.load(Ordering::Relaxed)
    }
    fn set_wait_callback(&self, is_wait: bool) {
        self.wait_callback.store(is_wait, Ordering::Relaxed)
    }
    fn get_wait_callback(&self) -> bool {
        self.wait_callback.load(Ordering::Relaxed)
    }
}

/// support tcp write trait
impl TcpWriteTrait for TcpClient {
    fn get_log_head(&self) -> &str {
        self.conf.log_head.as_str()
    }

    fn try_send_bytes(&self, bytes: &[u8]) -> std::io::Result<()> {
        let mut tcp_client = self.tcp_client.write();
        let write = tcp_client.tcp_stream.as_mut().ok_or_else(|| {
            io::Error::from(io::ErrorKind::NotConnected)
        })?;

        write.write_all(bytes)?;
        write.flush()
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
        let mut write = self.tcp_client.write();
        if let Some(tcp_client) = write.tcp_stream.as_mut() {
            if let Err(e) = tcp_client.shutdown(Shutdown::Both) {
                log::error!("shutdown tcp error: {e:?}");
            }
        }

        // as long as shutdown is called, tcp_client will be left blank directly
        if write.tcp_stream.is_some() {
            self.cb.dis_conn();
        }
        write.set_none();
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
            recv_time: AtomicI64::new(Self::now()).into(),
            timeout_time: AtomicI64::new(Self::now()).into(),
            tcp_client: Arc::new(RwLock::default()),
            wait_callback: Arc::new(AtomicBool::default()),
            buf_len,
            buf: RwLock::new(Vec::with_capacity(1)).into(),
            next_buf: RwLock::new(Vec::with_capacity(buf_len)).into(),
            state: Arc::new(RwLock::default()),
        }
    }

    /// start tcp client
    pub fn start(&self) {
        *self.buf.write() = vec![0; self.buf_len];
        timer::TcpClientTimer::new(self.clone()).start();
    }

    /// get has the tcp server connection been success
    pub fn is_connected(&self) -> bool {
        self.tcp_client.read().tcp_stream.is_some()
    }

    /// conn tcp server
    pub(crate) fn conn(&self) {
        #[cfg(feature = "debug_mode")]
        log::info!("{} conn", self.get_log_head());
        let mut state = self.state.write();
        state.connecting = true;
        if state.first {
            state.first = false;
            drop(state);
            self.conn_exec();
            return;
        }

        // not first conn, check is re conn
        if !self.conf.reconn.enable { return; }

        // re conn
        let diff = u128::try_from(Self::now() - state.last_re_time).unwrap_or_default();
        if diff < self.conf.reconn.time.as_millis() { return; }
        state.re_num = state.re_num.saturating_add(1);
        self.cb.re_conn(state.re_num);
        drop(state);
        self.conn_exec();
    }

    /// exec connection to tcp server
    fn conn_exec(&self) {
        #[cfg(feature = "debug_mode")]
        log::info!("{} conn exec", self.get_log_head());
        self.state.write().last_re_time = Self::now();
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

        self.tcp_client.write().set_stream(ts);
        self.cb.conn();
    }

    /// try conn to tcp server
    fn try_conn(&self) -> io::Result<TcpStream> {
        log::info!("{} try connect to tcp server",self.conf.log_head);
        let tcp_stream = TcpStream::connect_timeout(&self.conf.addr, self.conf.conn_time_out)?;
        if let Err(e) = tcp_stream.set_read_timeout(Some(self.conf.read_time_out)) {
            log::error!("{}set tcp read timeout fail: {e:?}",self.conf.log_head);
        }
        if let Err(e) = tcp_stream.set_nonblocking(true) {
            log::error!("{}set nonblocking fail: {e:?}",self.conf.log_head);
        }

        self.state.write().re_num = 0;
        log::info!("{} tcp server connect success",self.conf.log_head);
        Ok(tcp_stream)
    }

    /// read data from tcp server
    pub(crate) fn read(&self) {
        let mut state = self.state.write();
        state.reading = true;
        if let Err(e) = self.try_read() {
            log::info!("read err");
            if self.is_connected() {
                log::error!("{} tcp server read data error: {e:?}",self.conf.log_head);
            }
            // read error, directly assume that the tcp client has been closed
            self.shutdown();
            log::info!("{} tcp server shutdown",self.conf.log_head);
        }
        #[cfg(feature = "debug_mode")]
        log::info!("{} try read release",self.conf.log_head);
        state.reading = false;
    }

    /// try read data from tcp server
    fn try_read(&self) -> anyhow::Result<()> {
        #[cfg(feature = "debug_mode")]
        log::info!("{} try read",self.conf.log_head);
        self.set_now();

        let mut write = self.tcp_client.write();
        let ts = write.tcp_stream.as_mut().ok_or_else(|| { anyhow::anyhow!("tcp server not connection") })?;
        let mut buf = self.buf.write();
        let len =
            match ts.read(buf.as_mut_slice()) {
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
        drop(write);

        #[cfg(feature = "debug_mode")]
        log::info!("{} read data len is {len}",self.conf.log_head);
        // set recv time
        let mut next_buf = self.next_buf.write();
        self.set_recv_time_now();
        let buf = buf.get(0..len).unwrap_or_default();
        next_buf.append(&mut buf.to_vec());
        self.wait_callback();
        let mut temp = self.cb.recv(next_buf.drain(0..).collect());
        self.finish_callback();

        if temp.capacity() < self.buf_len {
            let mut new_tmp = Vec::with_capacity(self.buf_len);
            new_tmp.append(&mut temp);
            temp = new_tmp;
        }
        *next_buf = temp;

        Ok(())
    }

    /// check tcp read is finished
    pub(crate) fn check_read_finished(&self) {
        let check_time_out = i64::try_from(self.conf.read_time_out.as_millis()).unwrap_or(1_000) + 1_000;
        let now = Self::now();
        let timeout_diff = now - self.get_timeout_time();
        let recv_diff = now - self.get_recv_time();

        if !self.get_wait_callback() && timeout_diff > check_time_out && recv_diff > check_time_out {
            #[cfg(feature = "debug_mode")] {
                log::info!("{} timeout_diff is {timeout_diff}", self.get_log_head());
                log::info!("{} recv_diff is {recv_diff}", self.get_log_head());
                log::info!("{} check_time_out is {check_time_out}", self.get_log_head());
                log::warn!("{} neet abort", self.get_log_head());
            }
            // tcp read timeout, directly assuming that tcp has been disconnected
            self.shutdown();
        }
    }
}
