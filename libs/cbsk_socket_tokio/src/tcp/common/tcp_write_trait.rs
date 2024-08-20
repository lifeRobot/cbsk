use std::io;
use cbsk_base::{anyhow, log};
use cbsk_base::json::to_json::ToJson;
use cbsk_base::serde::Serialize;

/// send data and print log
macro_rules! send_tcp_log {
    ($result:expr,$log_head:expr,$name:expr,$data:expr) => {
        $crate::send_log!($result,$log_head,$name,$data,"TCP")
    };
}

/// tcp write trait
pub trait TcpWriteTrait {
    /// get internal log name
    fn get_log_head(&self) -> &str;

    /// send text to tcp
    async fn send_text(&self, text: &str) {
        send_tcp_log!(self.try_send_text(text),self.get_log_head(),"text",text);
    }

    /// try send text to TCP
    async fn try_send_text(&self, text: &str) -> io::Result<()> {
        self.try_send_bytes(text.as_bytes()).await
    }

    /// send json to TCP
    async fn send_json(&self, json: &(impl Serialize + Sync)) {
        send_tcp_log!(self.try_send_json(json),self.get_log_head(),"json",json.to_json());
    }

    /// try send json to TCP
    async fn try_send_json(&self, json: &(impl Serialize + Sync)) -> anyhow::Result<()> {
        let text = json.to_json()?.to_string();
        self.try_send_bytes(text.as_bytes()).await?;
        Ok(())
    }

    /// send bytes to TCP
    async fn send_bytes(&self, bytes: &[u8]) {
        send_tcp_log!(self.try_send_bytes(bytes),self.get_log_head(),"bytes",bytes);
    }

    /// try send bytes to TCP
    async fn try_send_bytes(&self, bytes: &[u8]) -> io::Result<()>;
}
