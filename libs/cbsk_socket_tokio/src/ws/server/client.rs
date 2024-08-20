use std::net::SocketAddr;
use std::sync::Arc;
use cbsk_base::tokio::net::TcpStream;
use cbsk_base::tokio::sync::RwLock;
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::ws::server::config::WsServerConfig;
use crate::ws::ws_write_trait::WsWriteTrait;

/// websocket client
pub struct WsServerClient {
    /// websocket client addr
    pub addr: SocketAddr,
    /// internal log name
    pub log_head: String,
    /// websocket client write
    write: Arc<RwLock<SplitSink<WebSocketStream<TcpStream>, Message>>>,
}

/// custom method
impl WsServerClient {
    /// create tcp server client
    pub(crate) fn new(addr: SocketAddr, conf: &WsServerConfig, writer: SplitSink<WebSocketStream<TcpStream>, Message>) -> Self {
        let log_head = format!("{} tcp client[{}]", conf.name, addr);
        Self { addr, log_head, write: RwLock::new(writer).into() }
    }
}

impl WsWriteTrait for WsServerClient {
    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
    }

    async fn try_send(&self, msg: Message) -> tokio_tungstenite::tungstenite::Result<()> {
        let mut write = self.write.write().await;
        write.send(msg).await?;
        write.flush().await
    }
}
