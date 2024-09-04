use std::future::Future;
use std::io::ErrorKind;
use std::time::Duration;
use cbsk_base::{anyhow, log, tokio};
use cbsk_base::tokio::io::AsyncReadExt;
use cbsk_base::tokio::net::tcp::OwnedReadHalf;
use cbsk_base::tokio::task::JoinHandle;
use cbsk_socket::tcp::common::time_trait::TimeTrait;

/// cbsk socket tcp read trait
pub trait ReadTrait: TimeTrait {
    /// get internal log name
    fn get_log_head(&self) -> &str;

    /// wait read data finished
    async fn wait_read_handle_finished<F, R>(&self, read_handle: JoinHandle<()>, read_time_out: Duration, abort_fn: F)
    where
        F: Fn() -> R,
        R: Future<Output=()>,
    {
        let check_time_out = i64::try_from(read_time_out.as_millis()).unwrap_or(1000) + 1000;
        let mut ignore_check_now = false;
        loop {
            // if read handle is finished, directly assume that tcp has been closed
            if read_handle.is_finished() {
                log::warn!("[{}]read handle is finished",self.get_log_head());
                break;
            }

            // if wait time callback, sleep 1 secs and continue
            if self.get_wait_callback() {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }

            // check time
            let now = Self::now();
            let timeout_diff = now - self.get_timeout_time();
            let recv_diff = now - self.get_recv_time();
            let is_need_abort = timeout_diff > check_time_out && recv_diff > check_time_out;
            // no need abort, sleep 1 secs and continue
            if !is_need_abort {
                if ignore_check_now {
                    ignore_check_now = false;
                    self.set_ignore_once(false);
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }

            // need abort, check is ignore once
            if !self.get_ignore() {
                log::warn!("[{}]read timeout,timeout_diff is {timeout_diff},recv_diff is {recv_diff}, check_time_out is {check_time_out}",self.get_log_head());
                // it is possible that tokio_runtime::time::timeout has failed, notify read_handle abort, and break loop
                // at this point, it is directly assumed that TCP has been closed
                read_handle.abort();
                abort_fn().await;
                break;
            }

            // ignore once
            ignore_check_now = true;
            tokio::time::sleep(Duration::from_secs(1)).await
        }
    }

    /// read data
    async fn try_read_data_tokio<TO, TOO, R, O>
    (&self, mut read: OwnedReadHalf, buf_len: usize, read_time_out: Duration, msg: &'static str, timeout_fn: TO, recv_callback: R)
     -> anyhow::Result<()>
    where
        TO: Fn() -> TOO,
        TOO: Future<Output=bool>,
        R: Fn(Vec<u8>) -> O,
        O: Future<Output=Vec<u8>>,
    {
        // start read data success, set recv_time and timeout_time once
        self.set_now();

        let mut buf = vec![0; buf_len];
        let mut buf_tmp = Vec::with_capacity(buf_len);

        loop {
            let read = read.read(buf.as_mut_slice());
            #[cfg(feature = "debug_mode")]
            log::info!("{} start read",self.get_log_head());
            let len =
                // the timeout of tokio_runtime may be an issue, which may cause the CPU to idle. It needs to be fixed here
                match tokio::time::timeout(read_time_out, read).await {
                    Ok(read) => {
                        match read {
                            Ok(len) => { len }
                            Err(e) => {
                                match e.kind() {
                                    ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                                        // set timeout time
                                        self.set_timeout_time_now();
                                        #[cfg(feature = "debug_mode")]
                                        log::info!("{} time out",self.get_log_head());
                                        // if just timeout, check write is conn
                                        if timeout_fn().await {
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
                        // set timeout time
                        self.set_timeout_time_now();
                        #[cfg(feature = "debug_mode")]
                        log::info!("{} time out",self.get_log_head());
                        // if just timeout, check write is conn
                        if timeout_fn().await {
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
            #[cfg(feature = "debug_mode")]
            log::info!("{} recv time",self.get_log_head());
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