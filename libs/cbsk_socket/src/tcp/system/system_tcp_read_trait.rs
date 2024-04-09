use std::future::Future;
use std::io::{ErrorKind, Read};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;
use cbsk_base::{anyhow, log};
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::tcp::common::r#async::async_tcp_time_trait::AsyncTcpTimeTrait;

/// cbsk socket tcp read trait
pub trait SystemTcpReadTrait: AsyncTcpTimeTrait {
    async fn try_read_data_system<const N: usize, TO, R, O>(
        &self, tcp_stream: Arc<MutDataObj<TcpStream>>, read_time_out: Duration, msg: &'static str, timeout_fn: TO, recv_callback: R)
        -> anyhow::Result<()>
        where TO: Fn() -> bool, R: Fn(Vec<u8>) -> O, O: Future<Output=Vec<u8>> {
        // start read data success, set recv_time and timeout_time once
        self.set_now();

        let mut buf = [0; N];
        let mut buf_tmp = Vec::with_capacity(N);
        let mut tcp_stream = tcp_stream.as_mut();
        if let Err(e) = tcp_stream.set_read_timeout(Some(read_time_out)) {
            log::error!("{}set tcp read timeout fail: {e:?}",self.get_log_head());
        }

        loop {
            let len =
                match tcp_stream.read(&mut buf) {
                    Ok(len) => { len }
                    Err(e) => {
                        match e.kind() {
                            // temporarily assume that timeout equals woldblock
                            // fix the bug of WoodBlock in some Linux causing TCP to continuously reconnect
                            ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                                self.set_timeout_time_now();
                                // read time out call timeout_fn
                                if timeout_fn() {
                                    return Ok(());
                                }

                                // But if just timeout, continue
                                continue;
                            }
                            _ => {
                                // order error, only break
                                return Err(e.into());
                            }
                        }
                    }
                };

            // reading a length of 0, it is assumed that the connection has been disconnected
            if len == 0 { return Err(anyhow::anyhow!("read data length is 0, indicating that tcp {msg} is disconnected")); }

            // set recv time
            self.set_recv_time_now();
            // non zero length, execution logic, etc
            // obtain length and print logs
            let buf = &buf[0..len];
            log::trace!("{} tcp read data[{buf:?}] of length {len}",self.get_log_head());
            // merge data and transfer to callback
            buf_tmp.append(&mut buf.to_vec());
            self.wait_callback();
            buf_tmp = recv_callback(buf_tmp).await;

            // check capacity, to reduce memory fragmentation
            if buf_tmp.capacity() < N {
                let mut new_buf_tmp = Vec::with_capacity(N);
                new_buf_tmp.append(&mut buf_tmp);
                buf_tmp = new_buf_tmp;
            }
            self.finish_callback();
        }
    }
}