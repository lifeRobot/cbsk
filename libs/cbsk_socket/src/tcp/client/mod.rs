use std::sync::Arc;
use cbsk_base::{anyhow, log, tokio};
use cbsk_base::async_recursion::async_recursion;
use cbsk_base::tokio::io::{AsyncReadExt, AsyncWriteExt};
use cbsk_base::tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use cbsk_base::tokio::net::TcpStream;
use cbsk_base::tokio::task::JoinHandle;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use fastdate::DateTime;
use crate::tcp::client::callback::TcpClientCallBack;
use crate::tcp::client::config::TcpClientConfig;
use crate::tcp::tcp_write_trait::TcpWriteTrait;

pub mod config;
pub mod callback;

/// tcp client
pub struct TcpClient<C: TcpClientCallBack> {
    /// tcp client config
    pub conf: Arc<TcpClientConfig>,
    /// tcp client business callback
    pub cb: Arc<C>,
    /// the last time the data was received<br />
    /// Because some Linux systems have network disconnections,
    /// but the system still considers TCP connections to be normal,
    /// a new last data reception time has been added,
    /// allowing users to determine whether they need to reconnect to TCP on their own<br />
    /// time see [fastdate::DateTime::unix_timestamp_millis]
    pub recv_time: Arc<MutDataObj<i64>>,
    /// tcp client writer
    pub(crate) write: Arc<MutDataObj<Option<MutDataObj<OwnedWriteHalf>>>>,
}

/// support clone
impl<C: TcpClientCallBack> Clone for TcpClient<C> {
    fn clone(&self) -> Self {
        Self { conf: self.conf.clone(), cb: self.cb.clone(), recv_time: self.recv_time.clone(), write: self.write.clone() }
    }
}

/// support writer trait
impl<C: TcpClientCallBack> TcpWriteTrait for TcpClient<C> {
    fn try_get_write(&self) -> anyhow::Result<&MutDataObj<OwnedWriteHalf>> {
        self.write.as_ref().as_ref().as_ref().ok_or_else(|| { anyhow::anyhow!("try send data to server, but connect to tcp server not yet") })
    }

    fn get_log_head(&self) -> &str {
        self.conf.log_head.as_str()
    }
}

/// data init etc
impl<C: TcpClientCallBack> TcpClient<C> {
    /// create tcp client<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn new(conf: Arc<TcpClientConfig>, cb: Arc<C>) -> Self {
        Self { conf, cb, recv_time: MutDataObj::new(DateTime::now().unix_timestamp_millis()).into(), write: Arc::new(MutDataObj::default()) }
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
impl<C: TcpClientCallBack> TcpClient<C> {
    /// start tcp client<br />
    /// N: TCP read data bytes size at once, usually 1024, If you need to accept big data, please increase this value
    pub fn start<const N: usize>(&self) -> JoinHandle<()> {
        let tcp_client = self.clone();
        tokio::spawn(async move {
            loop {
                tcp_client.conn::<N>(1).await;

                tcp_client.cb.dis_conn().await;
                if !tcp_client.conf.reconn.enable { break; }
                log::error!("{} tcp server disconnected, preparing for reconnection",tcp_client.conf.log_head);
            }

            log::info!("{} tcp client async has ended",tcp_client.conf.log_head);
        })
    }

    /// connect tcp server and start read data<br />
    /// re_num: re conn number, default is 1
    #[async_recursion]
    async fn conn<const N: usize>(&self, re_num: i32) {
        let err =
            match self.try_conn().await {
                Ok(tcp_stream) => {
                    self.read_spawn::<N>(tcp_stream).await;
                    return;
                }
                Err(e) => { e }
            };

        log::error!("{} tcp server connect error: {err:?}",self.conf.log_head);
        if !self.conf.reconn.enable { return; }

        // re conn
        self.cb.re_conn(re_num).await;
        log::info!("{} tcp service will reconnect in {:?}",self.conf.log_head,self.conf.reconn.time);
        tokio::time::sleep(self.conf.reconn.time).await;
        self.conn::<N>(re_num + 1).await;
    }

    /// read tcp server data
    async fn read_spawn<const N: usize>(&self, tcp_stream: TcpStream) {
        let (read, write) = tcp_stream.into_split();
        self.write.set_some(MutDataObj::new(write).into());

        log::info!("{} started tcp server read data async success",self.conf.log_head);
        self.cb.conn().await;

        if let Err(e) = self.try_read_spawn::<N>(read).await {
            // if the write is not closed, print the log.
            // otherwise, it is considered as actively closing the connection and there is no need to print the log
            if self.write.is_some() {
                log::error!("{} tcp server read data error: {e:?}",self.conf.log_head);
            }
        }

        // tcp read disabled, directly assume that tcp has been closed, simultaneously close read
        self.shutdown().await;
        log::info!("{} tcp server read data async is shutdown",self.conf.log_head);
    }

    /// try read data from tcp server
    async fn try_read_spawn<const N: usize>(&self, mut read: OwnedReadHalf) -> anyhow::Result<()> {
        // start read data success, set recv_time once
        self.recv_time.set(DateTime::now().unix_timestamp_millis());
        let mut buf = [0; N];
        let mut buf_tmp = Vec::new();

        loop {
            let read = read.read(&mut buf);
            let len =
                match tokio::time::timeout(self.conf.read_time_out, read).await {
                    Ok(read) => { read? }
                    Err(_) => {
                        // if just timeout, check write is conn
                        if self.write.is_none() {
                            // if write is disconnection，Believing that the connection has been manually closed
                            // exit the loop directly
                            return Ok(());
                        }
                        // But if just timeout, continue
                        continue;
                    }
                };

            // reading a length of 0, it is assumed that the connection has been disconnected
            if len == 0 { return Err(anyhow::anyhow!("read data length is 0, indicating that tcp server is disconnected")); }

            // set recv time
            self.recv_time.set(DateTime::now().unix_timestamp_millis());
            // non zero length, execution logic, etc
            // obtain length and print logs
            let buf = &buf[0..len];
            log::trace!("{} tcp read data[{buf:?}] of length {len}",self.conf.log_head);

            // merge data and transfer to callback
            buf_tmp.append(&mut buf.to_vec());
            buf_tmp = self.cb.recv(buf_tmp).await;
        }
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
