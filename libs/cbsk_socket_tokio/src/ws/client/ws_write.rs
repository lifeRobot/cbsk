use cbsk_base::tokio::net::TcpStream;
use futures_util::stream::SplitSink;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;

/// websocket write
#[derive(Default)]
pub struct WsWrite {
    /// websocket write
    pub write: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
}

/// custom method
impl WsWrite {
    /// set write
    pub fn set_write(&mut self, write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) {
        self.write = Some(write);
    }

    /// set none
    pub fn set_none(&mut self) {
        self.write = None;
    }
}
