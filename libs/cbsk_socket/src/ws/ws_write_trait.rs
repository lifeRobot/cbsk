use std::fmt::Debug;
use cbsk_base::{anyhow, log};
use cbsk_base::tokio::net::TcpStream;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
pub use tokio_tungstenite::tungstenite::Message;
pub use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::Frame;
use tokio_tungstenite::WebSocketStream;

/// send data and print log
macro_rules! send_ws_log {
    ($result:expr,$log_head:expr,$name:expr,$data:expr) => {
        $crate::send_log!($result,$log_head,$name,$data,"WebSocket")
    };
}

/// websocket write trait
pub trait WsWriteTrait {
    /// try get tcp client write
    fn try_get_write(&self) -> anyhow::Result<&MutDataObj<SplitSink<WebSocketStream<TcpStream>, Message>>>;

    /// get internal log name
    fn get_log_head(&self) -> &str;

    /// send text to websocket
    async fn send_text(&self, text: impl Into<String> + Debug) {
        send_ws_log!(self.try_send_text(text),self.get_log_head(),"text",text);
    }

    /// try send text to websocket
    async fn try_send_text(&self, text: impl Into<String>) -> anyhow::Result<()> {
        self.try_send(Message::Text(text.into())).await
    }

    /// send binary to websocket
    async fn send_binary(&self, binary: Vec<u8>) {
        send_ws_log!(self.try_send_binary(binary),self.get_log_head(),"binary",binary);
    }

    /// try send binary to websocket
    async fn try_send_binary(&self, binary: Vec<u8>) -> anyhow::Result<()> {
        self.try_send(Message::Binary(binary)).await
    }

    /// send ping to websocket
    async fn send_ping(&self, ping: Vec<u8>) {
        send_ws_log!(self.try_send_ping(ping),self.get_log_head(),"ping",ping);
    }

    /// try sned ping to websocket
    async fn try_send_ping(&self, ping: Vec<u8>) -> anyhow::Result<()> {
        self.try_send(Message::Ping(ping)).await
    }

    /// send pong to websocket
    async fn send_pong(&self, pong: Vec<u8>) {
        send_ws_log!(self.try_send_pong(pong),self.get_log_head(),"pong",pong);
    }

    /// try sned pong to websocket
    async fn try_send_pong(&self, pong: Vec<u8>) -> anyhow::Result<()> {
        self.try_send(Message::Pong(pong)).await
    }

    /// send close to websocket
    async fn send_close(&self, close: Option<CloseFrame<'static>>) {
        send_ws_log!(self.try_send_colse(close),self.get_log_head(),"close",close);
    }

    /// try send close to websocket
    async fn try_send_colse(&self, close: Option<CloseFrame<'static>>) -> anyhow::Result<()> {
        self.try_send(Message::Close(close)).await
    }

    /// send frame to websocket
    async fn send_frame(&self, frame: Frame) {
        send_ws_log!(self.try_send_frame(frame),self.get_log_head(),"frame",frame);
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