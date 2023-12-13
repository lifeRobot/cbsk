use cbsk_base::{anyhow, log};
use cbsk_base::async_trait::async_trait;
use cbsk_base::json::to_json::ToJson;
use cbsk_base::serde::Serialize;
use cbsk_base::tokio::io::AsyncWriteExt;
use cbsk_base::tokio::net::tcp::OwnedWriteHalf;
use cbsk_mut_data::mut_data_obj::MutDataObj;

/// send data and print log
macro_rules! send_log {
    ($result:expr,$log_head:expr,$name:expr,$data:expr) => {
        if let Err(e) = $result.await {
            log::error!("{} try send {} data [{:?}] to TCP error : {e:?}",$log_head,$name,$data);
            return;
        }
        log::debug!("{} send {} data [{:?}] to TCP success",$log_head,$name,$data);
    };
}

/// tcp write trait
#[async_trait]
pub trait WriteTrait {
    /// try get tcp client write
    fn try_get_write(&self) -> anyhow::Result<&MutDataObj<OwnedWriteHalf>>;

    /// get internal log name
    fn get_log_head(&self) -> &str;

    /// send text to tcp
    async fn send_text(&self, text: &str) {
        send_log!(self.try_send_text(text),self.get_log_head(),"text",text);
    }

    /// try send text to TCP
    async fn try_send_text(&self, text: &str) -> anyhow::Result<()> {
        self.try_send_bytes(text.as_bytes()).await
    }

    /// send json to TCP
    async fn send_json(&self, json: &(impl Serialize + Sync)) {
        send_log!(self.try_send_json(json),self.get_log_head(),"json",json.to_json());
    }

    /// try send json to TCP
    async fn try_send_json(&self, json: &(impl Serialize + Sync)) -> anyhow::Result<()> {
        let text = json.to_json()?.to_string();
        self.try_send_bytes(text.as_bytes()).await
    }

    /// send bytes to TCP
    async fn send_bytes(&self, bytes: &[u8]) {
        send_log!(self.try_send_bytes(bytes),self.get_log_head(),"bytes",bytes);
    }

    /// try send bytes to TCP
    async fn try_send_bytes(&self, bytes: &[u8]) -> anyhow::Result<()> {
        let mut write = self.try_get_write()?.as_mut();
        write.write_all(bytes).await?;
        write.flush().await?;
        Ok(())
    }
}