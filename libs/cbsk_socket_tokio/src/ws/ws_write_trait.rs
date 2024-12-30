use crate::ws::client::callback::Utf8Bytes;
use cbsk_base::log;
use std::fmt::Debug;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::protocol::frame::Frame;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::{Bytes, Message};

/// send data and print log
macro_rules! send_ws_log {
    ($result:expr,$log_head:expr,$name:expr,$data:expr) => {
        $crate::send_log!($result, $log_head, $name, $data, "WebSocket")
    };
}

/// websocket write trait
pub trait WsWriteTrait {
    /// get internal log name
    fn get_log_head(&self) -> &str;

    /// send text to websocket
    async fn send_text(&self, text: impl Into<Utf8Bytes> + Debug) {
        send_ws_log!(self.try_send_text(text), self.get_log_head(), "text", text);
    }

    /// try send text to websocket
    async fn try_send_text(&self, text: impl Into<Utf8Bytes>) -> tungstenite::Result<()> {
        self.try_send(Message::Text(text.into())).await
    }

    /// send binary to websocket
    async fn send_binary(&self, binary: impl Into<Bytes> + Debug) {
        send_ws_log!(
            self.try_send_binary(binary),
            self.get_log_head(),
            "binary",
            binary
        );
    }

    /// try send binary to websocket
    async fn try_send_binary(&self, binary: impl Into<Bytes>) -> tungstenite::Result<()> {
        self.try_send(Message::Binary(binary.into())).await
    }

    /// send ping to websocket
    async fn send_ping(&self, ping: impl Into<Bytes> + Debug) {
        send_ws_log!(self.try_send_ping(ping), self.get_log_head(), "ping", ping);
    }

    /// try sned ping to websocket
    async fn try_send_ping(&self, ping: impl Into<Bytes>) -> tungstenite::Result<()> {
        self.try_send(Message::Ping(ping.into())).await
    }

    /// send pong to websocket
    async fn send_pong(&self, pong: impl Into<Bytes> + Debug) {
        send_ws_log!(self.try_send_pong(pong), self.get_log_head(), "pong", pong);
    }

    /// try sned pong to websocket
    async fn try_send_pong(&self, pong: impl Into<Bytes>) -> tungstenite::Result<()> {
        self.try_send(Message::Pong(pong.into())).await
    }

    /// send close to websocket
    async fn send_close(&self, close: Option<CloseFrame>) {
        send_ws_log!(
            self.try_send_close(close),
            self.get_log_head(),
            "close",
            close
        );
    }

    /// try send close to websocket
    async fn try_send_close(&self, close: Option<CloseFrame>) -> tungstenite::Result<()> {
        self.try_send(Message::Close(close)).await
    }

    /// send frame to websocket
    async fn send_frame(&self, frame: Frame) {
        send_ws_log!(
            self.try_send_frame(frame),
            self.get_log_head(),
            "frame",
            frame
        );
    }

    /// try sned frame to websocket
    async fn try_send_frame(&self, frame: Frame) -> tungstenite::Result<()> {
        self.try_send(Message::Frame(frame)).await
    }

    /// try send message to websocket
    async fn try_send(&self, msg: Message) -> tungstenite::Result<()>;
}
