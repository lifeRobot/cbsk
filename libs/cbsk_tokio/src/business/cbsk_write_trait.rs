use std::io;
use cbsk_base::{anyhow, log};
use cbsk_base::json::to_json::ToJson;
use cbsk_base::serde::Serialize;

macro_rules! send_cbsk_log {
    ($result:expr,$log_head:expr,$name:expr,$data:expr) => {
        cbsk_socket_tokio::send_log!($result,$log_head,$name,$data,"cbsk")
    };
}

/// cbsk write data trait
pub trait CbskWriteTrait {
    /// get internal log name
    fn get_log_head(&self) -> &str;

    /// send text to cbsk
    async fn send_text(&self, text: &str) {
        send_cbsk_log!(self.try_send_text(text),self.get_log_head(),"text",text);
    }

    /// try send text to cbsk
    async fn try_send_text(&self, text: impl Into<String>) -> io::Result<()> {
        self.try_send_bytes(text.into().into_bytes()).await
    }

    /// send json to cbsk
    async fn send_json(&self, json: &(impl Serialize + Sync)) {
        send_cbsk_log!(self.try_send_json(json),self.get_log_head(),"json",json.to_json());
    }

    /// try send json to cbsk
    async fn try_send_json(&self, json: &(impl Serialize + Sync)) -> anyhow::Result<()> {
        let text = json.to_json()?.to_string();
        self.try_send_bytes(text.into_bytes()).await?;
        Ok(())
    }

    /// send bytes to cbsk
    async fn send_bytes(&self, bytes: Vec<u8>) {
        send_cbsk_log!(self.try_send_bytes(bytes),self.get_log_head(),"bytes",bytes);
    }

    /// try send bytes to cbsk
    async fn try_send_bytes(&self, bytes: Vec<u8>) -> io::Result<()>;
}
