use cbsk_socket::cbsk_base::{anyhow, log};
use cbsk_socket::cbsk_base::json::to_json::ToJson;
use cbsk_socket::cbsk_base::serde::Serialize;
use cbsk_socket::cbsk_base::tokio::io::AsyncWriteExt;
use cbsk_socket::cbsk_base::tokio::net::tcp::OwnedWriteHalf;
use cbsk_socket::cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::business;

macro_rules! send_cbsk_log {
    ($result:expr,$log_head:expr,$name:expr,$data:expr) => {
        cbsk_socket::send_log!($result,$log_head,$name,$data,"cbsk")
    };
}

/// cbsk write data trait
pub trait CbskWriteTrait {
    /// try get tcp client write
    fn try_get_write(&self) -> anyhow::Result<&MutDataObj<OwnedWriteHalf>>;

    /// get internal log name
    fn get_log_head(&self) -> &str;

    fn get_header(&self) -> &[u8];

    /// send text to cbsk
    async fn send_text(&self, text: &str) {
        send_cbsk_log!(self.try_send_text(text),self.get_log_head(),"text",text);
    }

    /// try send text to cbsk
    async fn try_send_text(&self, text: impl Into<String>) -> anyhow::Result<()> {
        self.try_send_bytes(text.into().into_bytes()).await
    }

    /// send json to cbsk
    async fn send_json(&self, json: &(impl Serialize + Sync)) {
        send_cbsk_log!(self.try_send_json(json),self.get_log_head(),"json",json.to_json());
    }

    /// try send json to cbsk
    async fn try_send_json(&self, json: &(impl Serialize + Sync)) -> anyhow::Result<()> {
        let text = json.to_json()?.to_string();
        self.try_send_bytes(text.into_bytes()).await
    }

    /// send bytes to cbsk
    async fn send_bytes(&self, bytes: Vec<u8>) {
        send_cbsk_log!(self.try_send_bytes(bytes),self.get_log_head(),"bytes",bytes);
    }

    /// try send bytes to cbsk
    async fn try_send_bytes(&self, bytes: Vec<u8>) -> anyhow::Result<()> {
        let frame = business::frame(bytes, self.get_header());
        let mut write = self.try_get_write()?.as_mut();
        write.write_all(frame.as_slice()).await?;
        write.flush().await?;
        Ok(())
    }
}