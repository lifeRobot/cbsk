use std::future::Future;
use std::time::Duration;
use cbsk_base::{log, tokio};
use cbsk_base::tokio::task::JoinHandle;
use crate::tcp::common::tcp_time_trait::TcpTimeTrait;

/// TCP time control related trait<br />
/// partial TCP data reading logic, also implemented through this trait
pub trait AsyncTcpTimeTrait: TcpTimeTrait {
    /// get internal log name
    fn get_log_head(&self) -> &str;

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

            // it is possible that tokio_runtime::time::timeout has failed, notify read_handle abort, and break loop
            // at this point, it is directly assumed that TCP has been closed
            if !self.get_wait_callback() && timeout_diff > check_time_out && recv_diff > check_time_out {
                log::info!("neet abort");
                read_handle.abort();
                abort_fn().await;
                break;
            }

            tokio::time::sleep(Duration::from_secs(1)).await
        }
    }
}