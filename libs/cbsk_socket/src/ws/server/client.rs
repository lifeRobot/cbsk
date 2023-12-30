use std::net::SocketAddr;
use std::sync::Arc;
use cbsk_base::tokio::net::TcpStream;
use cbsk_mut_data::mut_data_obj::MutDataObj;
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
    write: Arc<MutDataObj<SplitSink<WebSocketStream<TcpStream>, Message>>>,
}

/// custom method
impl WsServerClient {
    /// create tcp server client
    pub(crate) fn new(addr: SocketAddr, conf: &WsServerConfig, writer: SplitSink<WebSocketStream<TcpStream>, Message>) -> Self {
        let log_head = format!("{} tcp client[{}]", conf.name, addr);
        Self { addr, log_head, write: MutDataObj::new(writer).into() }
    }

    pub fn send() {}
}

/// support ws write trait
impl WsWriteTrait for WsServerClient {
    fn try_get_write(&self) -> cbsk_base::anyhow::Result<&MutDataObj<SplitSink<WebSocketStream<TcpStream>, Message>>> {
        Ok(self.write.as_ref())
    }

    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
    }
}
