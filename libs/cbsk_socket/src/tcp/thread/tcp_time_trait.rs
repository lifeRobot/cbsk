use std::io::{ErrorKind, Read};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use cbsk_base::{anyhow, log};
use cbsk_mut_data::mut_data_obj::MutDataObj;
use fastdate::DateTime;

/// TCP time control related trait<br />
/// partial TCP data reading logic, also implemented through this trait
pub trait TcpTimeTrait {
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

    /// set timeout time to now
    fn set_timeout_time_now(&self) {
        self.set_timeout_time(Self::now())
    }

    /// get now unix_timestamp_millis
    fn now() -> i64 {
        DateTime::now().unix_timestamp_millis()
    }

    /// try read tcp data
    fn try_read_data<const N: usize, TO, R>(
        &self, tcp_stream: Arc<MutDataObj<TcpStream>>, read_time_out: Duration, msg: &'static str, timeout_fn: TO, recv_callback: R,
    ) -> anyhow::Result<()>
        where TO: Fn() -> bool, R: Fn(Vec<u8>) -> Vec<u8> {
        // start read data success, set recv_time and timeout_time once
        self.set_now();

        let mut buf = [0; N];
        let mut buf_tmp = Vec::new();
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
            buf_tmp = recv_callback(buf_tmp);
        }
    }

    /// wait read data finished
    fn wait_read_handle_finished(&self, read_handle: JoinHandle<()>, read_time_out: Duration, abort_fn: impl Fn()) {
        let check_time_out = i64::try_from(read_time_out.as_millis()).unwrap_or(1000) + 1000;
        loop {
            let now = Self::now();
            let timeout_diff = now - self.get_timeout_time();
            let recv_diff = now - self.get_recv_time();

            // if read handle is finished, directly assume that tcp has been closed
            if read_handle.is_finished() {
                break;
            }

            // it is possible that tokio_runtime::time::timeout has failed, notify read_handle abort, and break loop
            // at this point, it is directly assumed that TCP has been closed
            if timeout_diff > check_time_out && recv_diff > check_time_out {
                // TODO read_handle neet abort, but thread_runtime not abort function
                abort_fn();
                break;
            }

            thread::sleep(Duration::from_secs(1));
        }
    }
}