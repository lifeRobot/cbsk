use std::io;
use cbsk_base::{anyhow, log, tokio};
use cbsk_base::tokio::task::JoinHandle;

/// monitor CTRL+C signals <br />
/// more see [`tokio::signal`]
pub async fn ctrl_c() -> io::Result<()> {
    tokio::signal::ctrl_c().await
}

/// monitor CTRL+C signals, if  and exit process
pub async fn ctrl_c_stop_handles<T>(handles: &[JoinHandle<T>]) {
    if let Err(e) = ctrl_c().await {
        log::error!("monitor ctrl + c error : {e:?}");
        return;
    }

    // monitor ctrl success
    log::warn!("the program has received an end command and is about to exit the program");
    handles.iter().for_each(|handle| { handle.abort(); });
    log::logger().flush();// wait log flush
    std::process::exit(0);// exit program
}

/// monitor CTRL+C signals and run handles
pub async fn run(runable: anyhow::Result<Vec<JoinHandle<()>>>) {
    super::run(runable, |handles| async {
        ctrl_c_stop_handles(&handles).await;
        handles
    }).await;
}
