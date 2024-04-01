use cbsk_base::{anyhow, log};
use cbsk_base::json::to_json::ToJson;
use cbsk_base::serde::Serialize;
/// send data and print log
macro_rules! send_tcp_log {
    ($result:expr,$log_head:expr,$name:expr,$data:expr) => {
        let msg = format!("send {} data [{:?}] to TCP", $name,$data);
        if let Err(e) = $result {
            log::error!("{} try {msg} error : {e:?}",$log_head);
            return;
        }
        log::debug!("{} {msg} success",$log_head);
    };
}

/// tcp write trait
pub trait TcpWriteTrait {
    /// get internal log name
    fn get_log_head(&self) -> &str;

    /// send text to tcp
    fn send_text(&self, text: &str) {
        send_tcp_log!(self.try_send_text(text),self.get_log_head(),"text",text);
    }

    /// try send text to TCP
    fn try_send_text(&self, text: &str) -> anyhow::Result<()> {
        self.try_send_bytes(text.as_bytes())
    }

    /// send json to TCP
    fn send_json(&self, json: &(impl Serialize + Sync)) {
        send_tcp_log!(self.try_send_json(json),self.get_log_head(),"json",json.to_json());
    }

    /// try send json to TCP
    fn try_send_json(&self, json: &(impl Serialize + Sync)) -> anyhow::Result<()> {
        let text = json.to_json()?.to_string();
        self.try_send_bytes(text.as_bytes())
    }

    /// send bytes to TCP
    fn send_bytes(&self, bytes: &[u8]) {
        send_tcp_log!(self.try_send_bytes(bytes),self.get_log_head(),"bytes",bytes);
    }

    /// try send bytes to TCP
    fn try_send_bytes(&self, bytes: &[u8]) -> anyhow::Result<()>;
}