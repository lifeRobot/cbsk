use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use cbsk_base::{anyhow, log};
use cbsk_base::parking_lot::RwLock;
use cbsk_socket::tcp::common::time_trait::TimeTrait;
use cbsk_socket::tcp::server::config::TcpServerConfig;
use crate::tcp::common::tcp_write_trait::TcpWriteTrait;
use crate::tcp::server::callback::TcpServerCallBack;
use crate::tcp::server::TcpServer;

/// tcp client
pub struct TcpServerClient {
    /// tcp client addr
    pub addr: SocketAddr,
    /// internal log name
    pub log_head: String,
    /// tcp server business callback
    pub cb: Arc<Box<dyn TcpServerCallBack>>,
    /// tcp server config
    pub conf: Arc<TcpServerConfig>,
    /// the last time the data was received<br />
    /// time see [cbsk_base::fastdate::DateTime::unix_timestamp_millis]
    pub recv_time: Arc<AtomicI64>,
    /// the tcp last read timeout<br />
    /// time see [cbsk_base::fastdate::DateTime::unix_timestamp_millis]
    pub timeout_time: Arc<AtomicI64>,
    /// is ignore once time check
    pub ignore_once: Arc<AtomicBool>,
    /// tcp client write
    pub(crate) tcp_client: Arc<RwLock<TcpStream>>,
    /// is wait callback
    wait_callback: Arc<AtomicBool>,
    /// thre tcp client is reading
    pub(crate) reading: Arc<AtomicBool>,
    /// read data buf len
    buf_len: usize,
    /// read data buf
    buf: Arc<RwLock<Vec<u8>>>,
    /// next buf data
    pub(crate) next_buf: Arc<RwLock<Vec<u8>>>,
    /// the tcp client is keep connecting
    pub(crate) connecting: Arc<AtomicBool>,
}

/// support tcp time trait
impl TimeTrait for TcpServerClient {
    fn set_recv_time(&self, time: i64) {
        self.recv_time.store(time, Ordering::Release)
    }
    fn get_recv_time(&self) -> i64 {
        self.recv_time.load(Ordering::Acquire)
    }
    fn set_timeout_time(&self, time: i64) {
        self.timeout_time.store(time, Ordering::Release)
    }
    fn get_timeout_time(&self) -> i64 {
        self.timeout_time.load(Ordering::Acquire)
    }
    fn set_wait_callback(&self, is_wait: bool) {
        self.wait_callback.store(is_wait, Ordering::Release)
    }
    fn get_wait_callback(&self) -> bool {
        self.wait_callback.load(Ordering::Acquire)
    }
    fn set_ignore_once(&self, is_ignore: bool) {
        self.ignore_once.store(is_ignore, Ordering::Release)
    }
    fn get_ignore(&self) -> bool {
        self.ignore_once.load(Ordering::Acquire)
    }
}

/// support tcp write trait
impl TcpWriteTrait for TcpServerClient {
    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
    }

    fn try_send_bytes(&self, bytes: &[u8]) -> std::io::Result<()> {
        let mut write = self.tcp_client.write();
        write.write_all(bytes)?;
        write.flush()
    }
}

/// custom method
impl TcpServerClient {
    /// create tcp server client
    pub fn new(addr: SocketAddr, ts: &TcpServer, tcp_client: TcpStream) -> Self {
        let log_head = format!("{} tcp client[{}]", ts.conf.name, addr);
        Self {
            addr,
            log_head,
            cb: ts.cb.clone(),
            conf: ts.conf.clone(),
            recv_time: AtomicI64::new(Self::now()).into(),
            timeout_time: AtomicI64::new(Self::now()).into(),
            tcp_client: Arc::new(RwLock::new(tcp_client)),
            wait_callback: Arc::new(AtomicBool::default()),
            ignore_once: Arc::new(AtomicBool::default()),
            reading: Arc::new(AtomicBool::default()),
            buf_len: ts.buf_len,
            buf: Arc::new(vec![0; ts.buf_len].into()),
            next_buf: RwLock::new(Vec::with_capacity(ts.buf_len)).into(),
            connecting: AtomicBool::new(true).into(),
        }
    }

    /// notify tcp client shutdown connection
    pub fn shutdown(&self) {
        if let Err(e) = self.tcp_client.write().shutdown(Shutdown::Both) {
            if self.conf.log {
                log::error!("shutdown tcp error: {e:?}");
            }
        }
        #[cfg(feature = "debug_mode")]
        log::warn!("client [{}] is shutdown",self.addr);
        self.connecting.store(false, Ordering::Relaxed);
    }

    /// read data from tcp client
    pub(crate) fn read(&self, tc: Arc<Self>) {
        self.reading.store(true, Ordering::Relaxed);
        if let Err(e) = self.try_read(tc.clone()) {
            if self.conf.log {
                log::error!("{} tcp client read data error: {e:?}",self.log_head);
            }
            #[cfg(feature = "debug_mode")]
            log::warn!("{} read err", self.log_head);
            self.shutdown();
            self.cb.dis_conn(tc);
        }
        self.reading.store(false, Ordering::Release);
    }

    /// try read data from tcp client
    fn try_read(&self, tc: Arc<Self>) -> anyhow::Result<()> {
        #[cfg(feature = "debug_mode")]
        log::info!("{} try read", self.log_head);
        self.set_now();
        let mut ts = self.tcp_client.write();
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
        if len == 0 { return Err(anyhow::anyhow!("read data length is 0, indicating that tcp client is disconnected")); }
        drop(ts);

        // set recv time
        self.set_recv_time_now();
        let mut next_buf = self.next_buf.write();
        let buf = buf.get(0..len).unwrap_or_default();
        next_buf.append(&mut buf.to_vec());
        self.wait_callback();
        let mut temp = self.cb.recv(next_buf.drain(0..).collect(), tc);
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
    pub(crate) fn check_read_finished(&self, tc: Arc<Self>) {
        let check_time_out = i64::try_from(self.conf.read_time_out.as_millis()).unwrap_or(1_000) + 1_000;
        let now = Self::now();
        let timeout_diff = now - self.get_timeout_time();
        let recv_diff = now - self.get_recv_time();

        if !self.get_wait_callback() && timeout_diff > check_time_out && recv_diff > check_time_out {
            #[cfg(feature = "debug_mode")] {
                log::info!("{} timeout_diff is {timeout_diff}", self.log_head);
                log::info!("{} recv_diff is {recv_diff}", self.log_head);
                log::info!("{} check_time_out is {check_time_out}", self.log_head);
                log::warn!("{} neet abort", self.log_head);
            }

            // tcp read timeout, directly assuming that tcp has been disconnected
            self.shutdown();
            self.cb.dis_conn(tc);
        }
    }
}
