use std::io;
use std::sync::Arc;
use cbsk_base::{anyhow, log, tokio};
use cbsk_base::tokio::io::AsyncWriteExt;
use cbsk_base::tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use cbsk_base::tokio::net::TcpStream;
use cbsk_base::tokio::task::JoinHandle;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::tcp::common::r#async::async_tcp_time_trait::AsyncTcpTimeTrait;
use crate::tcp::common::r#async::tcp_write_trait::TcpWriteTrait;
pub use crate::tcp::common::client::r#async::callback;
use crate::tcp::common::client::r#async::callback::TcpClientCallBack;
pub use crate::tcp::common::client::config;
use crate::tcp::common::client::config::TcpClientConfig;
use crate::tcp::common::tcp_time_trait::TcpTimeTrait;
use crate::tcp::tokio::tokio_tcp_read_trait::TokioTcpReadTrait;

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
    /// the tcp last read timeout<br />
    /// because sometimes tokio_runtime:: time:: timeout will fail, causing the CPU to run continuously, a timeout logic has been added<br />
    /// time see [fastdate::DateTime::unix_timestamp_millis]
    pub timeout_time: Arc<MutDataObj<i64>>,
    /// tcp client writer
    write: Arc<MutDataObj<Option<MutDataObj<OwnedWriteHalf>>>>,
    /// is wait callback
    wait_callback: Arc<MutDataObj<bool>>,
    /// tcp read data len
    buf_len: usize,
}

/// support writer trait
impl TcpWriteTrait for TcpClient {
    fn get_log_head(&self) -> &str {
        self.conf.log_head.as_str()
    }

    async fn try_send_bytes(&self, bytes: &[u8]) -> io::Result<()> {
        let mut write = self.write.as_ref().as_ref().as_ref().ok_or_else(|| {
            io::Error::from(io::ErrorKind::NotConnected)
        })?.as_mut();
        /*let mut write = cbsk_base::match_some_return!(self.write.as_ref().as_ref(),
            Err(anyhow::anyhow!("try send data to server, but connect to tcp server not yet"))).as_mut();*/

        write.write_all(bytes).await?;
        write.flush().await?;
        Ok(())
    }
}

/// support tcp client read trait
impl TokioTcpReadTrait for TcpClient {}

///  support tcp time trait
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

/// support tcp time trait
impl AsyncTcpTimeTrait for TcpClient {
    fn get_log_head(&self) -> &str {
        self.conf.log_head.as_str()
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
            recv_time: MutDataObj::new(Self::now()).into(),
            timeout_time: MutDataObj::new(Self::now()).into(),
            write: Arc::new(MutDataObj::default()),
            wait_callback: Arc::new(Default::default()),
            buf_len,
        }
    }

    /// stop tcp server connect<br />
    /// will shutdown tcp connection and will not new connection
    pub async fn stop(&self) {
        self.conf.reconn.as_mut().enable = false;
        self.shutdown().await;
    }

    /// notify tcp to re connect<br />
    /// will shutdown tcp connection, if [`TcpClientConfig`] reconn is disable<br />
    /// will shutdown and create new tcp connection,if [`TcpClientConfig`] reconn is enable
    pub async fn re_conn(&self) {
        self.shutdown().await;
    }

    /// shutdown tcp server connect
    async fn shutdown(&self) {
        if let Some(write) = self.write.as_ref().as_ref() {
            if let Err(e) = write.as_mut().shutdown().await {
                log::error!("shutdown tcp error: {e:?}");
            }
        }

        // as long as shutdown is called, write will be left blank directly
        self.write.set_none();
    }

    /// get has the tcp server connection been success
    pub fn is_connected(&self) -> bool {
        self.write.is_some()
    }
}

/// tcp read logic
impl TcpClient {
    /// start tcp client<br />
    /// N: TCP read data bytes size at once, usually 1024, If you need to accept big data, please increase this value
    pub fn start(&self) -> JoinHandle<()> {
        let tcp_client = self.clone();
        tokio::spawn(async move {
            // there are two loops here, so it should be possible to optimize them
            loop {
                tcp_client.conn().await;

                if !tcp_client.conf.reconn.enable { break; }
                log::error!("{} tcp server disconnected, preparing for reconnection",tcp_client.conf.log_head);
            }

            log::info!("{} tcp client async has ended",tcp_client.conf.log_head);
        })
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
            if !self.conf.reconn.enable { return; }

            // reconn
            self.cb.re_conn(re_num).await;
            log::info!("{} tcp service will reconnect in {:?}",self.conf.log_head,self.conf.reconn.time);
            tokio::time::sleep(self.conf.reconn.time).await;
        }
    }

    /// read tcp server data
    async fn read_spawn(&self, tcp_stream: TcpStream) {
        let (read, write) = tcp_stream.into_split();
        self.write.set_some(MutDataObj::new(write).into());

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
                tcp_client.try_read_data_tokio(read, tcp_client.buf_len, tcp_client.conf.read_time_out, "server", || {
                    tcp_client.write.is_none()
                }, |data| async {
                    tcp_client.cb.recv(data).await
                }).await;

            if let Err(e) = result {
                if tcp_client.write.is_some() {
                    log::error!("{} tcp server read data error: {e:?}",tcp_client.conf.log_head);
                }
            }
        })
    }

    /// try connect tcp server
    async fn try_conn(&self) -> anyhow::Result<TcpStream> {
        log::info!("{} try connect to tcp server",self.conf.log_head);
        let tcp_stream = TcpStream::connect(self.conf.addr);
        let tcp_stream = tokio::time::timeout(self.conf.conn_time_out, tcp_stream).await??;

        log::info!("{} tcp server connect success",self.conf.log_head);
        Ok(tcp_stream)
    }
}
