use std::future::Future;
use std::time::Duration;
use cbsk_base::tokio;
use cbsk_base::tokio::task::JoinHandle;
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
}