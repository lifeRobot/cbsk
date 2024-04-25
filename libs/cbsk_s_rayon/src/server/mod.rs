use std::net::TcpListener;
use std::sync::Arc;
use cbsk_base::{anyhow, log};
use cbsk_mut_data::mut_data_obj::MutDataObj;
use cbsk_socket::tcp::common::server::config::TcpServerConfig;
use crate::runtime::runtime;
use crate::server::callback::TcpServerCallBack;
use crate::server::client::TcpServerClient;

pub mod callback;
pub mod client;

/// tcp server
#[derive(Clone)]
pub struct TcpServer {
    /// tcp config
    pub conf: Arc<TcpServerConfig>,
    /// tcp server business callback
    pub cb: Arc<Box<dyn TcpServerCallBack>>,
    /// tcp server listener
    pub(crate) listener: Arc<MutDataObj<Option<TcpListener>>>,
    /// is tcp server listening
    pub(crate) listening: Arc<MutDataObj<bool>>,
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
            listener: Arc::new(MutDataObj::default()),
            listening: Arc::new(MutDataObj::default()),
            buf_len,
        }
    }

    /// start tcp server
    pub fn start(&self) {
        runtime.tcp_server.push(self.clone());
        runtime.start();
    }

    /// listener server
    pub(crate) fn listener(&self) {
        if let Err(e) = self.try_listener() {
            log::error!("listener server[{}] fail: {e:?}",self.conf.addr);
        }
        self.listening.set_false();
    }

    /// try listener server
    fn try_listener(&self) -> anyhow::Result<()> {
        let tl = cbsk_base::match_some_exec!(self.listener.as_ref().as_ref(),{
            let tl = TcpListener::bind(self.conf.addr)?;
            self.listener.set(Some(tl));
            log::info!("{} listener [{}] success",self.conf.log_head,self.conf.addr);
            self.listener.as_ref().as_ref().as_ref().ok_or_else(||{anyhow::anyhow!("get listener fail")})?
        });

        self.listening.set_true();
        let (ts, addr) = tl.accept()?;
        if let Err(e) = ts.set_read_timeout(Some(self.conf.read_time_out)) {
            log::error!("set read time out fail: {e:?}");
        }
        let tc = Arc::new(TcpServerClient::new(addr, self, MutDataObj::new(ts).into()));
        runtime.tcp_server_client.push(tc.clone());
        self.cb.conn(tc);
        Ok(())
    }
}
