use cbsk_base::anyhow;
use cbsk_base::tokio::net::TcpStream;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
pub use tokio_tungstenite::tungstenite::Message;
pub use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::Frame;
use tokio_tungstenite::WebSocketStream;

/// websocket write trait
pub trait WsWriteTrait {
    /// try get tcp client write
    fn try_get_write(&self) -> anyhow::Result<&MutDataObj<SplitSink<WebSocketStream<TcpStream>, Message>>>;

    /// get internal log name
    fn get_log_head(&self) -> &str;

    /// try send text to websocket
    async fn try_send_text(&self, text: impl Into<String>) -> anyhow::Result<()> {
        self.try_send(Message::Text(text.into())).await
    }

    /// try send binary to websocket
    async fn try_send_binary(&self, binary: Vec<u8>) -> anyhow::Result<()> {
        self.try_send(Message::Binary(binary)).await
    }

    /// try sned ping to websocket
    async fn try_send_ping(&self, ping: Vec<u8>) -> anyhow::Result<()> {
        self.try_send(Message::Ping(ping)).await
    }

    /// try sned pong to websocket
    async fn try_send_pong(&self, pong: Vec<u8>) -> anyhow::Result<()> {
        self.try_send(Message::Pong(pong)).await
    }

    /// try send close to websocket
    async fn try_send_colse(&self, close: Option<CloseFrame<'static>>) -> anyhow::Result<()> {
        self.try_send(Message::Close(close)).await
    }

    /// try sned frame to websocket
    async fn try_send_frame(&self, frame: Frame) -> anyhow::Result<()> {
        self.try_send(Message::Frame(frame)).await
    }

    /// try send message to websocket
    async fn try_send(&self, msg: Message) -> anyhow::Result<()> {
        let mut write = self.try_get_write()?.as_mut();
        write.send(msg).await?;
        write.flush().await?;
        Ok(())
    }
}