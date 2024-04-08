use std::sync::Arc;
use std::{io, thread};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread::JoinHandle;
use std::time::Duration;
use cbsk_base::log;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::tcp::common::server::config::TcpServerConfig;
use crate::tcp::common::sync::sync_tcp_time_trait::SyncTcpTimeTrait;
use crate::tcp::thread::server::callback::TcpServerCallBack;
use crate::tcp::thread::server::client::TcpServerClient;
use crate::tcp::thread::thread_tcp_time_trait::ThreadTcpTimeTrait;

pub mod callback;
pub mod client;

/// tcp server
pub struct TcpServer<C: TcpServerCallBack> {
    /// tcp config
    pub conf: Arc<TcpServerConfig>,
    /// tcp server business callback
    pub cb: Arc<C>,
}

/// support clone
impl<C: TcpServerCallBack> Clone for TcpServer<C> {
    fn clone(&self) -> Self {
        Self { conf: self.conf.clone(), cb: self.cb.clone() }
    }
}

/// data init etc
impl<C: TcpServerCallBack> TcpServer<C> {
    /// create a tcp server<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn new(conf: Arc<TcpServerConfig>, cb: Arc<C>) -> Self {
        Self { conf, cb }
    }
}

/// tcp read logic
impl<C: TcpServerCallBack> TcpServer<C> {
    /// start tcp server<br />
    /// N: TCP read data bytes size at once, usually 1024, If you need to accept big data, please increase this value
    pub fn start<const N: usize>(&self) -> JoinHandle<()> {
        let tcp_server = self.clone();
        thread::spawn(move || {
            let mut read_handles = Vec::new();

            if let Err(e) = tcp_server.try_start::<N>(&mut read_handles) {
                log::error!("{} tcp bind [{}] error: {e:?}",tcp_server.conf.log_head,tcp_server.conf.addr);
            }

            // wait read async
            for handle in read_handles {
                if let Err(e) = handle.join() {
                    log::error!("{} read async error: {e:?}",tcp_server.conf.log_head);
                }
            }
        })
    }

    /// try start tcp server
    fn try_start<const N: usize>(&self, read_handles: &mut Vec<JoinHandle<()>>) -> io::Result<()> {
        let listener = Arc::new(TcpListener::bind(self.conf.addr)?);
        let conf = self.conf.as_ref();

        log::info!("{} listener TCP[{}] success",conf.log_head,conf.addr);

        loop {
            if let Err(e) = self.try_accept::<N>(listener.clone(), read_handles) {
                log::error!("{} wait tcp accept error. wait for the next accept in three seconds. error: {:?}",conf.log_head,e);
                thread::sleep(Duration::from_secs(3));
            }
        }
    }

    /// try accept TCP client and read tcp client data
    fn try_accept<const N: usize>(&self, listener: Arc<TcpListener>, read_handles: &mut Vec<JoinHandle<()>>) -> io::Result<()> {
        // this will cause blocking asynchronous, so use tokio_runtime::spawn and wait next accept
        let (tcp_stream, addr) = listener.accept()?;

        let tcp_stream = Arc::new(MutDataObj::new(tcp_stream));
        let tcp_client = Arc::new(TcpServerClient::new(addr, self.conf.as_ref(), tcp_stream.clone()));
        read_handles.push(self.read_spawn::<N>(tcp_client.clone(), tcp_stream));
        self.cb.conn(tcp_client);

        Ok(())
    }

    /// start read async
    fn read_spawn<const N: usize>(&self, client: Arc<TcpServerClient>, read: Arc<MutDataObj<TcpStream>>) -> JoinHandle<()> {
        let tcp_server = self.clone();
        thread::spawn(move || {
            let read_headle = tcp_server.try_read_spawn::<N>(client.clone(), read);
            client.wait_read_handle_finished(read_headle, tcp_server.conf.read_time_out, || {
                // it is possible that TCP has not been closed here, so notify to close it once
                // ignoring notification results
                let _ = client.write.as_mut().shutdown(Shutdown::Both);
            });

            // if TCP read is closed, it is considered that TCP has been closed
            tcp_server.cb.dis_conn(client.clone());
            if tcp_server.conf.log { log::info!("{} tcp client read async closed",client.log_head); }
        })
    }

    /// try read tcp client data
    fn try_read_spawn<const N: usize>(&self, client: Arc<TcpServerClient>, read: Arc<MutDataObj<TcpStream>>) -> JoinHandle<()> {
        if self.conf.log { log::info!("{} start tcp client read async success",client.log_head); }
        let tcp_server = self.clone();

        thread::spawn(move || {
            let result = client.try_read_data::<N, _, _>(read, tcp_server.conf.read_time_out, "client", || {
                false
            }, |data| {
                tcp_server.cb.recv(data, client.clone())
            });

            if let Err(e) = result {
                if tcp_server.conf.log { log::error!("{} read tcp client data error: {e:?}",client.log_head); }
            }
        })
    }
}
