use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use crate::tcp::common::sync::sync_tcp_time_trait::SyncTcpTimeTrait;

/// TCP time control related trait<br />
/// partial TCP data reading logic, also implemented through this trait
pub trait ThreadTcpTimeTrait: SyncTcpTimeTrait {
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
            if !self.get_wait_callback() && timeout_diff > check_time_out && recv_diff > check_time_out {
                // TODO read_handle neet abort, but thread_runtime not abort function
                abort_fn();
                break;
            }

            thread::sleep(Duration::from_secs(1));
        }
    }
}