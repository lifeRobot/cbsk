use std::{io, thread};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;
use cbsk_base::log;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use rayon::ThreadPool;
use crate::tcp::common::server::config::TcpServerConfig;
use crate::tcp::common::sync::sync_tcp_time_trait::SyncTcpTimeTrait;
use crate::tcp::rayon::rayon_tcp_time_trait::RayonTcpTimeTrait;
use crate::tcp::rayon::server::callback::TcpServerCallBack;
use crate::tcp::rayon::server::client::TcpServerClient;

pub mod client;
pub mod callback;

/// tcp server
pub struct TcpServer<C: TcpServerCallBack> {
    /// tcp config
    pub conf: Arc<TcpServerConfig>,
    /// tcp server business callback
    pub cb: Arc<C>,
    /// rayon thread pool, default 1 threads
    thread_pool: Arc<ThreadPool>,
}

/// support clone
impl<C: TcpServerCallBack> Clone for TcpServer<C> {
    fn clone(&self) -> Self {
        Self { conf: self.conf.clone(), cb: self.cb.clone(), thread_pool: self.thread_pool.clone() }
    }
}

/// data init etc
impl<C: TcpServerCallBack> TcpServer<C> {
    /// create a tcp server<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    /// please use [Self::try_new]
    pub fn new(conf: Arc<TcpServerConfig>, cb: Arc<C>) -> Self {
        Self::try_new(conf, cb).expect("create thread fail")
    }

    /// create a tcp server<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn try_new(conf: Arc<TcpServerConfig>, cb: Arc<C>) -> Result<Self, rayon::ThreadPoolBuildError> {
        Ok(Self { conf, cb, thread_pool: Arc::new(rayon::ThreadPoolBuilder::new().num_threads(1).build()?) })
    }
}

/// tcp read logic
impl<C: TcpServerCallBack> TcpServer<C> {
    /// start tcp server<br />
    /// N: TCP read data bytes size at once, usually 1024, If you need to accept big data, please increase this value<br />
    /// please ensure that the main thread does not end, otherwise this TCP will automatically end, more see [ThreadPool::spawn]
    pub fn start<const N: usize>(&self) {
        let tcp_server = self.clone();
        self.thread_pool.spawn(move || {
            if let Err(e) = tcp_server.try_start::<N>() {
                log::error!("{} tcp bind [{}] error: {e:?}",tcp_server.conf.log_head,tcp_server.conf.addr);
            }
        });
    }

    /// try start tcp server
    fn try_start<const N: usize>(&self) -> io::Result<()> {
        let listener = Arc::new(TcpListener::bind(self.conf.addr)?);
        let conf = self.conf.as_ref();

        log::info!("{} listener TCP[{}] success",conf.log_head,conf.addr);
        loop {
            if let Err(e) = self.try_accept::<N>(listener.clone()) {
                log::error!("{} wait tcp accept error. wait for the next accept in three seconds. error: {:?}",conf.log_head,e);
                thread::sleep(Duration::from_secs(3));
            }
        }
    }

    /// try accept TCP client and read tcp client data
    fn try_accept<const N: usize>(&self, listener: Arc<TcpListener>) -> io::Result<()> {
        // this will cause blocking asynchronous, so use tokio_runtime::spawn and wait next accept
        let (tcp_stream, addr) = listener.accept()?;

        let tcp_stream = Arc::new(MutDataObj::new(tcp_stream));
        let tcp_client =
            match TcpServerClient::try_new(addr, self.conf.as_ref(), tcp_stream.clone()) {
                Ok(tcp_client) => { Arc::new(tcp_client) }
                Err(e) => {
                    log::error!("create thread fial: {e:?}");
                    // just shutdown, ignoring errors
                    let _ = tcp_stream.shutdown(Shutdown::Both);
                    return Ok(());
                }
            };
        self.read_spawn::<N>(tcp_client.clone(), tcp_stream);
        self.cb.conn(tcp_client);
        Ok(())
    }

    /// start read thread
    fn read_spawn<const N: usize>(&self, client: Arc<TcpServerClient>, read: Arc<MutDataObj<TcpStream>>) {
        let tcp_server = self.clone();
        client.clone().thread_pool.spawn(move || {
            tcp_server.try_read_spawn::<N>(client.clone(), read);
            client.wait_read_finished(tcp_server.conf.read_time_out, || {
                // it is possible that TCP has not been closed here, so notify to close it once
                // ignoring notification results
                let _ = client.write.as_mut().shutdown(Shutdown::Both);
            });
            // if TCP read is closed, it is considered that TCP has been closed
            tcp_server.cb.dis_conn(client.clone());
            if tcp_server.conf.log { log::info!("{} tcp client read async closed",client.log_head); }
        });
    }

    /// try read tcp client data
    fn try_read_spawn<const N: usize>(&self, client: Arc<TcpServerClient>, read: Arc<MutDataObj<TcpStream>>) {
        if self.conf.log { log::info!("{} start tcp client read async success",client.log_head); }
        let tcp_server = self.clone();
        client.read_end.set_false();

        client.clone().thread_pool.spawn(move || {
            let result = client.try_read_data::<N, _, _>(read, tcp_server.conf.read_time_out, "client", || {
                false
            }, |data| {
                tcp_server.cb.recv(data, client.clone())
            });

            if let Err(e) = result {
                if tcp_server.conf.log { log::error!("{} read tcp client data error: {e:?}",client.log_head); }
            }
            client.read_end.set_true();
        })
    }
}
