/// send data and print log
#[macro_export]
macro_rules! send_log {
    ($result:expr,$log_head:expr,$name:expr,$data:expr,$protocol:expr) => {
        let msg = format!("send {} data [{:?}] to {}", $name,$data,$protocol);
        if let Err(e) = $result.await {
            log::error!("{} try {msg} error : {e:?}",$log_head);
            return;
        }
        log::debug!("{} {msg} success",$log_head);
    };
}