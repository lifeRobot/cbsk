use std::future::Future;
use std::time::Duration;
use cbsk_base::{anyhow, log, tokio};
use cbsk_base::tokio::io::AsyncReadExt;
use cbsk_base::tokio::net::tcp::OwnedReadHalf;
use cbsk_base::tokio::task::JoinHandle;
use fastdate::DateTime;

/// TCP time control related trait<br />
/// partial TCP data reading logic, also implemented through this trait
pub(crate) trait TcpTimeTrait {
    /// set the last time the data was received for tcp
    fn set_recv_time(&self, time: i64);

    /// get the last time the data was received for tcp
    fn get_recv_time(&self) -> i64;

    /// set tcp last read timeout
    fn set_timeout_time(&self, time: i64);

    /// get tcp last read timeout
    fn get_timeout_time(&self) -> i64;

    /// get internal log name
    fn get_log_head(&self) -> &str;

    /// set recv time and timeout time to now
    fn set_now(&self) {
        self.set_recv_time_now();
        self.set_timeout_time_now();
    }

    /// set recv time to now
    fn set_recv_time_now(&self) {
        self.set_recv_time(Self::now());
    }

    fn set_timeout_time_now(&self) {
        self.set_timeout_time(Self::now())
    }

    /// get now unix_timestamp_millis
    fn now() -> i64 {
        DateTime::now().unix_timestamp_millis()
    }

    /// wait read data finished
    async fn wait_read_handle_finished<F, R>(&self, read_handle: JoinHandle<()>, read_time_out: Duration, abort_fn: F)
        where F: Fn() -> R, R: Future<Output=()> {
        let check_time_out = i64::try_from(read_time_out.as_millis()).unwrap_or(1000) + 1000;
        loop {
            let now = Self::now();
            let timeout_diff = now - self.get_timeout_time();
            let recv_diff = now - self.get_recv_time();

            // if read handle is finished, directly assume that tcp has been closed
            if read_handle.is_finished() {
                break;
            }

            // it is possible that tokio::time::timeout has failed, notify read_handle abort, and break loop
            // at this point, it is directly assumed that TCP has been closed
            if timeout_diff > check_time_out && recv_diff > check_time_out {
                read_handle.abort();
                abort_fn().await;
                break;
            }

            tokio::time::sleep(Duration::from_secs(1)).await
        }
    }

    /// try read data from tcp
    async fn try_read_data<const N: usize, TO, R, O>
    (&self, mut read: OwnedReadHalf, read_time_out: Duration, msg: &'static str, timeout_fn: TO, recv_callback: R)
     -> anyhow::Result<()>
        where TO: Fn() -> bool, R: Fn(Vec<u8>) -> O, O: Future<Output=Vec<u8>> {
        // start read data success, set recv_time and timeout_time once
        self.set_now();

        let mut buf = [0; N];
        let mut buf_tmp = Vec::new();

        loop {
            let read = read.read(&mut buf);
            let len =
                // the timeout of tokio may be an issue, which may cause the CPU to idle. It needs to be fixed here
                match tokio::time::timeout(read_time_out, read).await {
                    Ok(read) => { read? }
                    Err(_) => {
                        // set timeout time
                        self.set_timeout_time_now();
                        log::info!("{} time out",self.get_log_head());
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
            buf_tmp = recv_callback(buf_tmp).await;
        }
    }
}
