use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use cbsk_base::{anyhow, log};
use cbsk_base::parking_lot::RwLock;
use cbsk_socket::tcp::server::config::TcpServerConfig;
use cbsk_timer::timer::Timer;
use crate::tcp::server::callback::TcpServerCallBack;
use crate::tcp::server::client::TcpServerClient;
use crate::tcp::server::tcp_server_listener::TcpServerListener;

pub mod callback;
pub mod client;
pub mod tcp_server_listener;
mod client_timer;
mod timer;

/// tcp server
#[derive(Clone)]
pub struct TcpServer {
    /// tcp config
    pub conf: Arc<TcpServerConfig>,
    /// tcp server business callback
    pub cb: Arc<Box<dyn TcpServerCallBack>>,
    /// tcp server listener
    pub(crate) listener: Arc<RwLock<TcpServerListener>>,
    /// is tcp server listening
    pub(crate) listening: Arc<AtomicBool>,
    /// read data buf len
    buf_len: usize,
}

/// custom method
impl TcpServer {
    /// create a tcp server<br />
    /// default buf_len is 1024, more see [Self::new_with_buf_len]<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn new<C: TcpServerCallBack>(conf: Arc<TcpServerConfig>, cb: C) -> Self {
        Self::new_with_buf_len(conf, cb, 1024)
    }

    /// create a tcp server<br />
    /// custom buf_len, buf_len is tcp read data once lengle<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn new_with_buf_len<C: TcpServerCallBack>(conf: Arc<TcpServerConfig>, cb: C, buf_len: usize) -> Self {
        Self {
            conf,
            cb: Arc::new(Box::new(cb)),
            listener: Arc::new(RwLock::default()),
            listening: Arc::new(AtomicBool::default()),
            buf_len,
        }
    }

    /// start tcp server
    pub fn start(&self) {
        timer::TcpServerTimer::new(self.clone()).start();
    }

    /// listener server
    pub(crate) fn listener(&self) {
        if let Err(e) = self.try_listener() {
            log::error!("listener server[{}] fail: {e:?}",self.conf.addr);
        }
        self.listening.store(false, Ordering::Release)
    }

    /// try listener server
    fn try_listener(&self) -> anyhow::Result<()> {
        let mut listener = self.listener.write();
        let tl = cbsk_base::match_some_exec!(listener.listener.as_ref(),{
            let tl = TcpListener::bind(self.conf.addr)?;
            listener.set_listener(tl);
            log::info!("{} listener [{}] success",self.conf.log_head,self.conf.addr);
            listener.listener.as_ref().ok_or_else(||{anyhow::anyhow!("get listener fail")})?
        });

        self.listening.store(true, Ordering::Release);
        let (ts, addr) = tl.accept()?;
        if let Err(e) = ts.set_read_timeout(Some(self.conf.read_time_out)) {
            log::error!("set read time out fail: {e:?}");
        }
        let tc = Arc::new(TcpServerClient::new(addr, self, ts));
        client_timer::TcpServerClientTimer::new(tc.clone()).start();
        #[cfg(feature = "debug_mode")]
        log::info!("{} add to tcp server client",tc.log_head);
        self.cb.conn(tc);
        Ok(())
    }
}
