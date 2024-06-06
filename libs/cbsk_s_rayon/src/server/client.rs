use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::Arc;
use cbsk_base::{anyhow, log, parking_lot};
use cbsk_mut_data::mut_data_obj::MutDataObj;
use cbsk_mut_data::mut_data_vec::MutDataVec;
use cbsk_socket::tcp::common::server::config::TcpServerConfig;
use cbsk_socket::tcp::common::sync::tcp_write_trait::TcpWriteTrait;
use cbsk_socket::tcp::common::tcp_time_trait::TcpTimeTrait;
use crate::server::callback::TcpServerCallBack;
use crate::server::TcpServer;

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
    /// time see [fastdate::DateTime::unix_timestamp_millis]
    pub recv_time: Arc<MutDataObj<i64>>,
    /// the tcp last read timeout<br />
    /// time see [fastdate::DateTime::unix_timestamp_millis]
    pub timeout_time: Arc<MutDataObj<i64>>,
    /// tcp client write
    pub(crate) tcp_client: Arc<MutDataObj<TcpStream>>,
    /// is wait callback
    wait_callback: Arc<MutDataObj<bool>>,
    /// thre tcp client is reading
    pub(crate) reading: Arc<MutDataObj<bool>>,
    /// read data buf len
    buf_len: usize,
    /// read data buf
    buf: Arc<MutDataVec<u8>>,
    /// next buf data
    pub(crate) next_buf: Arc<MutDataVec<u8>>,
    /// the tcp client is keep connecting
    pub(crate) connecting: Arc<MutDataObj<bool>>,
    /// write data lock
    lock: parking_lot::Mutex<()>,
}

/// support tcp time trait
impl TcpTimeTrait for TcpServerClient {
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
impl TcpWriteTrait for TcpServerClient {
    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
    }

    /// try send bytes to TCP<br />
    /// note that this operation will block the thread until the data is sent out
    fn try_send_bytes(&self, bytes: &[u8]) -> anyhow::Result<()> {
        let lock = self.lock.lock();
        let result = self.try_send_bytes_no_lock(bytes);
        drop(lock);
        result
    }
}

/// custom method
impl TcpServerClient {
    /// try send bytes to TCP and not lock
    pub fn try_send_bytes_no_lock(&self, bytes: &[u8]) -> anyhow::Result<()> {
        let mut ts = self.tcp_client.as_mut();

        ts.write_all(bytes)?;
        ts.flush()?;
        Ok(())
    }

    /// create tcp server client
    pub fn new(addr: SocketAddr, ts: &TcpServer, tcp_client: Arc<MutDataObj<TcpStream>>) -> Self {
        let log_head = format!("{} tcp client[{}]", ts.conf.name, addr);
        Self {
            addr,
            log_head,
            cb: ts.cb.clone(),
            conf: ts.conf.clone(),
            recv_time: MutDataObj::new(Self::now()).into(),
            timeout_time: MutDataObj::new(Self::now()).into(),
            tcp_client,
            wait_callback: Arc::new(MutDataObj::default()),
            reading: Arc::new(MutDataObj::default()),
            buf_len: ts.buf_len,
            buf: Arc::new(vec![0; ts.buf_len].into()),
            next_buf: MutDataVec::with_capacity(ts.buf_len).into(),
            connecting: MutDataObj::new(true).into(),
            lock: parking_lot::Mutex::new(()),
        }
    }

    /// notify tcp client shutdown connection
    pub fn shutdown(&self) {
        if let Err(e) = self.tcp_client.shutdown(Shutdown::Both) {
            if self.conf.log {
                log::error!("shutdown tcp error: {e:?}");
            }
        }
        #[cfg(feature = "debug_mode")]
        log::warn!("client [{}] is shutdown",self.addr);
        self.connecting.set_false();
    }

    /// read data from tcp client
    pub(crate) fn read(&self, tc: Arc<Self>) {
        self.reading.set_true();
        if let Err(e) = self.try_read(tc.clone()) {
            if self.conf.log {
                log::error!("{} tcp client read data error: {e:?}",self.log_head);
            }
            #[cfg(feature = "debug_mode")]
            log::warn!("{} read err", self.log_head);
            self.shutdown();
            self.cb.dis_conn(tc);
        }
        self.reading.set_false();
    }

    /// try read data from tcp client
    fn try_read(&self, tc: Arc<Self>) -> anyhow::Result<()> {
        #[cfg(feature = "debug_mode")]
        log::info!("{} try read", self.log_head);
        self.set_now();
        let mut ts = self.tcp_client.as_mut();
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
        if len == 0 { return Err(anyhow::anyhow!("read data length is 0, indicating that tcp client is disconnected")); }

        // set recv time
        self.set_recv_time_now();
        let buf = self.buf.get(0..len).unwrap_or_default();
        self.next_buf.append(&mut buf.to_vec());
        self.wait_callback();
        let mut temp = self.cb.recv(self.next_buf.as_mut().drain(0..).collect(), tc);
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
