use crate::ws::server::client::WsServerClient;
use cbsk_base::log;
use std::future::Future;
use std::sync::Arc;
pub use tokio_tungstenite::tungstenite::protocol::frame::Frame;
pub use tokio_tungstenite::tungstenite::protocol::CloseFrame;
pub use tokio_tungstenite::tungstenite::{Bytes, Utf8Bytes};

/// websocket connect and read data callback
pub trait WsServerCallBack: Send + Sync + 'static {
    /// a new websocket client come in
    fn conn(&self, client: Arc<WsServerClient>) -> impl Future<Output = ()> + Send {
        log::info!("{} websocket client connected", client.log_head);
        async {}
    }

    /// the websocket client disconnected
    fn dis_conn(&self, client: Arc<WsServerClient>) -> impl Future<Output = ()> + Send {
        log::info!("{} websocket client disconnect", client.log_head);
        async {}
    }

    /// websocket server recv websocket client text data will call this method<br />
    /// text: websocket client text data<br />
    /// client: websocket client, you can use this send data to websocket client
    fn recv_text(
        &self,
        text: Utf8Bytes,
        client: Arc<WsServerClient>,
    ) -> impl Future<Output = ()> + Send;

    /// websocket server recv websocket client binary data will call this method<br />
    /// in general, you can ignore this data, if client not send binary
    fn recv_binary(
        &self,
        binary: Bytes,
        client: Arc<WsServerClient>,
    ) -> impl Future<Output = ()> + Send {
        log::warn!(
            "{} recv websocket client binary data: [{binary:?}]",
            client.log_head
        );
        async {}
    }

    /// websocket server recv websocket client ping data will call this method<br />
    /// in general, you can ignore this data, if client not send ping
    fn recv_ping(
        &self,
        ping: Bytes,
        client: Arc<WsServerClient>,
    ) -> impl Future<Output = ()> + Send {
        log::warn!(
            "{} recv websocket client ping data: [{ping:?}]",
            client.log_head
        );
        async {}
    }

    /// websocket server recv websocket client pong data will call this method<br />
    /// in general, you can ignore this data, if client not send pong
    fn recv_pong(
        &self,
        pong: Bytes,
        client: Arc<WsServerClient>,
    ) -> impl Future<Output = ()> + Send {
        log::warn!(
            "{} recv websocket client pong data: [{pong:?}]",
            client.log_head
        );
        async {}
    }

    /// websocket server recv websocket client close data will call this method<br />
    /// in general, you can ignore this data, if client not send close
    fn recv_close(
        &self,
        close: Option<CloseFrame>,
        client: Arc<WsServerClient>,
    ) -> impl Future<Output = ()> + Send {
        log::warn!(
            "{} recv websocket client close data: [{close:?}]",
            client.log_head
        );
        async {}
    }

    /// websocket server recv websocket client frame data will call this method<br />
    /// in general, you can ignore this data, if client not send frame
    fn recv_frame(
        &self,
        frame: Frame,
        client: Arc<WsServerClient>,
    ) -> impl Future<Output = ()> + Send {
        log::warn!(
            "{} recv websocket client frame data: [{frame:?}]",
            client.log_head
        );
        async {}
    }
}
