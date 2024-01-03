use std::sync::Arc;
use cbsk_base::{anyhow, log, tokio};
use cbsk_base::async_recursion::async_recursion;
use cbsk_base::tokio::io::{AsyncReadExt, AsyncWriteExt};
use cbsk_base::tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use cbsk_base::tokio::net::TcpStream;
use cbsk_base::tokio::task::JoinHandle;
use cbsk_mut_data::mut_data_obj::MutDataObj;
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
    /// tcp client writer
    pub(crate) write: Arc<MutDataObj<Option<MutDataObj<OwnedWriteHalf>>>>,
}

/// support clone
impl<C: TcpClientCallBack> Clone for TcpClient<C> {
    fn clone(&self) -> Self {
        Self { conf: self.conf.clone(), cb: self.cb.clone(), write: self.write.clone() }
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
        Self { conf, cb, write: Arc::new(MutDataObj::default()) }
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

        // 只要调用过shutdown，都直接将写置空
        self.write.set(None);
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
        self.write.set(Some(MutDataObj::new(write).into()));

        log::info!("{} started tcp server read data async success",self.conf.log_head);
        self.cb.conn().await;

        if let Err(e) = self.try_read_spawn::<N>(read).await {
            log::error!("{} tcp server read data error: {e:?}",self.conf.log_head);
        }

        // TCP读取关闭了，直接认为TCP已经关闭了，同时关闭读取
        self.shutdown().await;
        log::info!("{} tcp server read data async is shutdown",self.conf.log_head);
    }

    /// try read data from tcp server
    async fn try_read_spawn<const N: usize>(&self, mut read: OwnedReadHalf) -> anyhow::Result<()> {
        let mut buf = [0; N];
        let mut buf_tmp = Vec::new();

        loop {
            let read = read.read(&mut buf);
            let len =
                match tokio::time::timeout(self.conf.read_time_out, read).await {
                    Ok(read) => { read? }
                    Err(_) => {
                        // if just timeout, continue
                        continue;
                    }
                };

            // 读取到了长度为0，直接认为已经断开了连接
            if len == 0 { return Err(anyhow::anyhow!("read data length is 0, indicating that tcp server is disconnected")); }

            // 长度非0，执行逻辑等
            // 获取长度打印日志
            let buf = &buf[0..len];
            log::debug!("{} tcp read data[{buf:?}] of length {len}",self.conf.log_head);

            // 合并数据并转到回调中
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
