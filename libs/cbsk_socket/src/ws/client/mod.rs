use std::sync::Arc;
use cbsk_base::tokio;
use cbsk_base::tokio::task::JoinHandle;
use futures_util::StreamExt;
use crate::ws::client::callback::WsClientCallBack;
use crate::ws::client::config::WsClientConfig;

pub mod config;
pub mod callback;

/// websocket client
pub struct WsClient<C: WsClientCallBack> {
    /// websocket config
    pub conf: Arc<WsClientConfig>,
    /// websocket client business callback
    pub cb: Arc<C>,
}

/// support clone
impl<C: WsClientCallBack> Clone for WsClient<C> {
    fn clone(&self) -> Self {
        Self { conf: self.conf.clone(), cb: self.cb.clone() }
    }
}

/// tcp read logic
impl<C: WsClientCallBack> WsClient<C> {
    pub fn start(&self) -> JoinHandle<()> {
        let ws_client = self.clone();
        tokio::spawn(async move {
            // TODO under development
            ws_client.conf.reconn.as_mut().enable = false;
            let (stream, _) = tokio_tungstenite::connect_async(ws_client.conf.ws_url.as_str()).await.unwrap();
            let (_write, _read) = stream.split();
        })
    }
}
