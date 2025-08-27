use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use cbsk_base::{anyhow, log, tokio};
use cbsk_base::tokio::io::AsyncWriteExt;
use cbsk_base::tokio::net::tcp::OwnedReadHalf;
use cbsk_base::tokio::net::TcpStream;
use cbsk_base::tokio::sync::RwLock;
use cbsk_base::tokio::task::JoinHandle;
use cbsk_socket::tcp::client::config::TcpClientConfig;
use cbsk_socket::tcp::common::time_trait::TimeTrait;
use crate::tcp::client::callback::TcpClientCallBack;
use crate::tcp::client::tcp_write::TcpWrite;
use crate::tcp::common::read_trait::ReadTrait;
use crate::tcp::common::tcp_write_trait::TcpWriteTrait;

pub mod callback;
mod tcp_write;

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
    /// the tcp last read timeout<br />
    /// because sometimes tokio_runtime:: time:: timeout will fail, causing the CPU to run continuously, a timeout logic has been added<br />
    /// time see [cbsk_base::fastdate::DateTime::unix_timestamp_millis]
    pub timeout_time: Arc<AtomicI64>,
    /// is ignore once time check
    pub ignore_once: Arc<AtomicBool>,
    /// is waiting stop
    wait_stop: Arc<AtomicBool>,
    /// tcp client writer
    write: Arc<RwLock<TcpWrite>>,
    /// is wait callback
    wait_callback: Arc<AtomicBool>,
    /// tcp read data len
    buf_len: usize,
}

/// support writer trait
impl TcpWriteTrait for TcpClient {
    fn get_log_head(&self) -> &str {
        self.conf.log_head.as_str()
    }

    async fn try_send_bytes(&self, bytes: &[u8]) -> io::Result<()> {
        let mut write = self.write.write().await;
        let write = write.write.as_mut().ok_or_else(|| {
            io::Error::from(io::ErrorKind::NotConnected)
        })?;

        write.write_all(bytes).await?;
        write.flush().await
    }
}

/// support tcp client read trait
impl ReadTrait for TcpClient {
    fn get_log_head(&self) -> &str {
        self.conf.log_head.as_str()
    }
}

///  support tcp time trait
impl TimeTrait for TcpClient {
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

/// data init etc
impl TcpClient {
    /// create tcp client<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn new<C: TcpClientCallBack>(conf: Arc<TcpClientConfig>, cb: C) -> Self {
        Self::new_with_buf_len(conf, 1024, cb)
    }

    pub fn new_with_buf_len<C: TcpClientCallBack>(conf: Arc<TcpClientConfig>, buf_len: usize, cb: C) -> Self {
        Self {
            conf,
            cb: Arc::new(Box::new(cb)),
            recv_time: AtomicI64::new(Self::now()).into(),
            timeout_time: AtomicI64::new(Self::now()).into(),
            ignore_once: AtomicBool::default().into(),
            wait_stop: AtomicBool::default().into(),
            write: Arc::new(RwLock::new(TcpWrite::default())),
            wait_callback: Arc::new(Default::default()),
            buf_len,
        }
    }

    /// stop tcp server connect<br />
    /// will shutdown tcp connection and will not new connection
    pub async fn stop(&self) {
        self.conf.reconn.enable.store(false, Ordering::Release);
        self.shutdown().await;
    }

    /// notify tcp to re connect<br />
    /// will shutdown tcp connection, if [`TcpClientConfig`] reconn is disable<br />
    /// will shutdown and create new tcp connection,if [`TcpClientConfig`] reconn is enable
    #[inline]
    pub async fn re_conn(&self) {
        self.shutdown().await;
    }

    /// shutdown tcp server connect
    async fn shutdown(&self) {
        let mut write = self.write.write().await;
        let owner_write = cbsk_base::match_some_return!(write.write.as_mut(),self.wait_stop.store(true, Ordering::Release));
        if let Err(e) = owner_write.shutdown().await {
            log::error!("shutdown tcp error: {e:?}");
        }

        // as long as shutdown is called, write will be left blank directly
        write.set_none();
    }

    /// get has the tcp server connection been success
    pub async fn is_connected(&self) -> bool {
        self.write.read().await.write.is_some()
    }
}

/// tcp read logic
impl TcpClient {
    /// start tcp client in join handle
    pub fn start_in_handle(&self) -> JoinHandle<()> {
        let tcp_client = self.clone();
        tokio::spawn(async move { tcp_client.start().await })
    }

    /// start tcp client
    pub async fn start(&self) {
        // there are two loops here, so it should be possible to optimize them
        loop {
            self.conn().await;

            if !self.conf.reconn.enable.load(Ordering::Acquire) { break; }
            log::error!("{} tcp server disconnected, preparing for reconnection",self.conf.log_head);
        }

        log::info!("{} tcp client async has ended",self.conf.log_head);
    }

    /// connect tcp server and start read data
    async fn conn(&self) {
        let mut re_num = 0;
        loop {
            re_num += 1;
            let err =
                match self.try_conn().await {
                    Ok(tcp_stream) => {
                        self.read_spawn(tcp_stream).await;
                        return;
                    }
                    Err(e) => e,
                };

            log::error!("{} tcp server connect error: {err:?}",self.conf.log_head);
            if !self.conf.reconn.enable.load(Ordering::Acquire) { return; }

            // reconn
            self.cb.re_conn(re_num).await;
            log::info!("{} tcp service will reconnect in {:?}",self.conf.log_head,self.conf.reconn.time);
            tokio::time::sleep(self.conf.reconn.time).await;
        }
    }

    /// read tcp server data
    async fn read_spawn(&self, tcp_stream: TcpStream) {
        let (read, write) = tcp_stream.into_split();
        self.write.write().await.set_write(write);

        log::info!("{} started tcp server read data async success",self.conf.log_head);
        self.cb.conn().await;

        let read_handle = self.try_read_spawn(read);
        self.wait_read_handle_finished(read_handle, self.conf.read_time_out, || async {}).await;

        // tcp read disabled, directly assume that tcp has been closed, simultaneously close read
        self.shutdown().await;
        self.cb.dis_conn().await;
        log::info!("{} tcp server read data async is shutdown",self.conf.log_head);
    }

    /// read data handle
    fn try_read_spawn(&self, read: OwnedReadHalf) -> JoinHandle<()> {
        // start read headle, set recvtime and timeouttime is now
        self.set_now();

        let tcp_client = self.clone();
        tokio::spawn(async move {
            let result =
                tcp_client.try_read_data_tokio(read, tcp_client.buf_len, tcp_client.conf.read_time_out, "server", || async {
                    tcp_client.write.read().await.write.is_none()
                }, |data| async {
                    tcp_client.cb.recv(data).await
                }).await;

            if let Err(e) = result {
                if tcp_client.write.read().await.write.is_some() {
                    log::error!("{} tcp server read data error: {e:?}",tcp_client.conf.log_head);
                }
            }
        })
    }

    /// try connect tcp server
    async fn try_conn(&self) -> anyhow::Result<TcpStream> {
        log::info!("{} try connect to tcp server",self.conf.log_head);
        let tcp_stream = TcpStream::connect(self.conf.addr);
        let mut tcp_stream = tokio::time::timeout(self.conf.conn_time_out, tcp_stream).await??;

        if self.wait_stop.load(Ordering::Acquire) {
            tcp_stream.shutdown().await?;
            return Err(anyhow::anyhow!("need shutdown"));
        }

        log::info!("{} tcp server connect success",self.conf.log_head);
        Ok(tcp_stream)
    }
}
