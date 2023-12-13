use std::future::Future;
use cbsk_base::{anyhow, log};
use cbsk_base::tokio::task::JoinHandle;

pub mod signal;

/// run handle<br />
/// handles_exec：the operation on the handle before it ends, usually by notifying the end of the handle
pub async fn run<F>(runable: anyhow::Result<Vec<JoinHandle<()>>>, handles_exec: impl FnOnce(Vec<JoinHandle<()>>) -> F)
    where F: Future<Output=Vec<JoinHandle<()>>> {
    let handles = match runable {
        Ok(handles) => { handles }
        Err(e) => {
            eprintln!("run error: {e:?}");
            log::error!("run error: {e:?}");
            return;
        }
    };

    // wait handle exec and over
    let handles = handles_exec(handles).await;
    for handle in handles {
        if let Err(e) = handle.await {
            log::error!("handle error: {e:?}");
        }
    }

    // wait log flush
    log::logger().flush();
}
