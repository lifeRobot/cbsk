use std::thread;
use std::time::Duration;
use crate::tcp::common::sync::sync_tcp_time_trait::SyncTcpTimeTrait;

/// TCP time control related trait<br />
/// partial TCP data reading logic, also implemented through this trait
pub trait RayonTcpTimeTrait: SyncTcpTimeTrait {
    /// get tcp read is ended
    fn get_read_end(&self) -> bool;

    /// wait read data finished
    fn wait_read_finished(&self, read_time_out: Duration, abort_fn: impl Fn()) {
        let check_time_out = i64::try_from(read_time_out.as_millis()).unwrap_or(1000) + 1000;
        loop {
            let now = Self::now();
            let timeout_diff = now - self.get_timeout_time();
            let recv_diff = now - self.get_recv_time();

            // if read handle is finished, directly assume that tcp has been closed
            if self.get_read_end() {
                break;
            }

            // it is possible that tokio_runtime::time::timeout has failed, notify read_handle abort, and break loop
            // at this point, it is directly assumed that TCP has been closed
            if timeout_diff > check_time_out && recv_diff > check_time_out {
                // TODO rayon neet abort, but rayon not abort function
                abort_fn();
                break;
            }

            thread::sleep(Duration::from_secs(1));
        }
    }
}