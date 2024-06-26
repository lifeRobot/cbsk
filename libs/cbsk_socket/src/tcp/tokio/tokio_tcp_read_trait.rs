use std::future::Future;
use std::io::ErrorKind;
use std::time::Duration;
use cbsk_base::{anyhow, log, tokio};
use cbsk_base::tokio::io::AsyncReadExt;
use cbsk_base::tokio::net::tcp::OwnedReadHalf;
use crate::tcp::common::r#async::async_tcp_time_trait::AsyncTcpTimeTrait;

/// cbsk socket tcp read trait
pub trait TokioTcpReadTrait: AsyncTcpTimeTrait {
    /// read data
    async fn try_read_data_tokio<TO, R, O>
    (&self, mut read: OwnedReadHalf, buf_len: usize, read_time_out: Duration, msg: &'static str, timeout_fn: TO, recv_callback: R)
     -> anyhow::Result<()>
        where TO: Fn() -> bool, R: Fn(Vec<u8>) -> O, O: Future<Output=Vec<u8>> {
        // start read data success, set recv_time and timeout_time once
        self.set_now();

        let mut buf = vec![0; buf_len];
        let mut buf_tmp = Vec::with_capacity(buf_len);

        loop {
            let read = read.read(buf.as_mut_slice());
            let len =
                // the timeout of tokio_runtime may be an issue, which may cause the CPU to idle. It needs to be fixed here
                match tokio::time::timeout(read_time_out, read).await {
                    Ok(read) => {
                        match read {
                            Ok(len) => { len }
                            Err(e) => {
                                match e.kind() {
                                    ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                                        // log::info!("{} time out",self.get_log_head());
                                        // set timeout time
                                        self.set_timeout_time_now();
                                        // if just timeout, check write is conn
                                        if timeout_fn() {
                                            // if timeout_fn return true, exit the loop directly
                                            return Ok(());
                                        }
                                        // But if just timeout, continue
                                        continue;
                                    }
                                    _ => {
                                        return Err(e.into());
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // log::info!("{} time out",self.get_log_head());
                        // set timeout time
                        self.set_timeout_time_now();
                        // if just timeout, check write is conn
                        if timeout_fn() {
                            // if timeout_fn return true, exit the loop directly
                            return Ok(());
                        }
                        // But if just timeout, continue
                        continue;
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
            if buf_tmp.capacity() < buf_len {
                let mut new_buf_tmp = Vec::with_capacity(buf_len);
                new_buf_tmp.append(&mut buf_tmp);
                buf_tmp = new_buf_tmp;
            }
            self.finish_callback();
        }
    }
}