use std::io::Write;
use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use cbsk_base::{anyhow, log};
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::tcp::common::client::config::TcpClientConfig;
use crate::tcp::thread::client::callback::TcpClientCallBack;
use crate::tcp::thread::tcp_time_trait::TcpTimeTrait;
use crate::tcp::thread::tcp_write_trait::TcpWriteTrait;

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
    /// the tcp last read timeout
    /// time see [fastdate::DateTime::unix_timestamp_millis]
    pub timeout_time: Arc<MutDataObj<i64>>,
    /// tcp client
    tcp_client: Arc<MutDataObj<Option<Arc<MutDataObj<TcpStream>>>>>,
}

/// support clone
impl<C: TcpClientCallBack> Clone for TcpClient<C> {
    fn clone(&self) -> Self {
        Self {
            conf: self.conf.clone(),
            cb: self.cb.clone(),
            recv_time: self.recv_time.clone(),
            timeout_time: self.timeout_time.clone(),
            tcp_client: self.tcp_client.clone(),
        }
    }
}

/// support tcp time trait
impl<C: TcpClientCallBack> TcpTimeTrait for TcpClient<C> {
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
    fn get_log_head(&self) -> &str {
        self.conf.log_head.as_str()
    }
}

/// support tcp write trait
impl<C: TcpClientCallBack> TcpWriteTrait for TcpClient<C> {
    fn get_log_head(&self) -> &str {
        self.conf.log_head.as_str()
    }
    fn try_send_bytes(&self, bytes: &[u8]) -> anyhow::Result<()> {
        let mut tcp_client = cbsk_base::match_some_return!(self.tcp_client.as_ref().as_ref(),
            Err(anyhow::anyhow!("try send data to server, but connect to tcp server not yet"))).as_mut();

        tcp_client.write_all(bytes)?;
        tcp_client.flush()?;
        Ok(())
    }
}

/// data init etc
impl<C: TcpClientCallBack> TcpClient<C> {
    /// create tcp client<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn new(conf: Arc<TcpClientConfig>, cb: Arc<C>) -> Self {
        Self {
            conf,
            cb,
            recv_time: MutDataObj::new(Self::now()).into(),
            timeout_time: MutDataObj::new(Self::now()).into(),
            tcp_client: Arc::new(MutDataObj::default()),
        }
    }

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

    fn shutdown(&self) {
        if let Some(tcp_client) = self.tcp_client.as_ref().as_ref() {
            if let Err(e) = tcp_client.shutdown(Shutdown::Both) {
                log::error!("shutdown tcp error: {e:?}");
            }
        }

        // as long as shutdown is called, tcp_client will be left blank directly
        self.tcp_client.set_none();
    }

    /// get has the tcp server connection been success
    pub fn is_connected(&self) -> bool {
        self.tcp_client.is_some()
    }
}

/// tcp read logic
impl<C: TcpClientCallBack> TcpClient<C> {
    /// start tcp client<br />
    /// N: TCP read data bytes size at once, usually 1024, If you need to accept big data, please increase this value
    pub fn start<const N: usize>(&self) -> JoinHandle<()> {
        let tcp_client = self.clone();
        thread::spawn(move || {
            // there are two loops here, so it should be possible to optimize them
            loop {
                tcp_client.conn::<N>();

                if !tcp_client.conf.reconn.enable { break; }
                log::error!("{} tcp server disconnected, preparing for reconnection",tcp_client.conf.log_head);
            }
        })
    }

    /// connect tcp server and start read data
    fn conn<const N: usize>(&self) {
        let mut re_num = 0_i32;
        loop {
            re_num = re_num.saturating_add(1);
            let err =
                match self.try_conn() {
                    Ok(tcp_stream) => {
                        self.read_spawn::<N>(tcp_stream);
                        return;
                    }
                    Err(e) => e,
                };

            log::error!("{} tcp server connect error: {err:?}",self.conf.log_head);
            if !self.conf.reconn.enable { return; }

            // reconn
            self.cb.re_conn(re_num);
            log::info!("{} tcp service will reconnect in {:?}",self.conf.log_head,self.conf.reconn.time);
            thread::sleep(self.conf.reconn.time);
        }
    }

    /// read tcp server data
    fn read_spawn<const N: usize>(&self, tcp_stream: TcpStream) {
        let tcp_stream = Arc::new(MutDataObj::new(tcp_stream));
        self.tcp_client.set(Some(tcp_stream.clone()));

        log::info!("{} started tcp server read data async success",self.conf.log_head);
        self.cb.conn();

        // read data logic
        let handle = self.try_read_spawn::<N>(tcp_stream);
        self.wait_read_handle_finished(handle, self.conf.read_time_out, || {});

        // tcp read disabled, directly assume that tcp has been closed, simultaneously close read
        self.shutdown();
        self.cb.dis_conn();
        log::info!("{} tcp server read data async is shutdown",self.conf.log_head);
    }

    /// read data handle
    fn try_read_spawn<const N: usize>(&self, tcp_stream: Arc<MutDataObj<TcpStream>>) -> JoinHandle<()> {
        self.set_now();

        let tcp_client = self.clone();
        thread::spawn(move || {
            let result = tcp_client.try_read_data::<N, _, _>(tcp_stream, tcp_client.conf.read_time_out, "server", || {
                tcp_client.tcp_client.is_none()
            }, |bytes| {
                tcp_client.cb.recv(bytes)
            });

            if let Err(e) = result {
                if tcp_client.tcp_client.is_some() {
                    log::error!("{} tcp server read data error: {e:?}",tcp_client.conf.log_head);
                }
            }
        })
    }

    /// try connect tcp server
    fn try_conn(&self) -> anyhow::Result<TcpStream> {
        log::info!("{} try connect to tcp server",self.conf.log_head);
        let tcp_stream = TcpStream::connect_timeout(&self.conf.addr, self.conf.conn_time_out)?;

        log::info!("{} tcp server connect success",self.conf.log_head);
        Ok(tcp_stream)
    }
}
