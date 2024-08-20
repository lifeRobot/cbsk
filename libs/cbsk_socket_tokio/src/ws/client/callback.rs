use std::future::Future;
use cbsk_base::log;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::Frame;

/// websocket connect and read data callback
pub trait WsClientCallBack: Send + Sync + 'static {
    /// connect websocket server success will call this method
    fn conn(&self) -> impl Future<Output=()> + Send {
        log::info!("connect websocket server success");
        async {}
    }

    /// this method will be called when the websocket service is disconnected
    fn dis_conn(&self) -> impl Future<Output=()> + Send {
        log::info!("disconnect websocket server");
        async {}
    }

    /// connect tcp server fail and try connect server will call this method<br />
    /// num: number of try connect
    fn re_conn(&self, num: i32) -> impl Future<Output=()> + Send {
        log::info!("re connect to websocket server, re num is {num}");
        async {}
    }

    /// websocket client recv websocket server text data will call this method<br />
    /// text: websocket client text data
    fn recv_text(&self, text: String) -> impl Future<Output=()> + Send;

    /// websocket client recv websocket server binary data will call this method<br />
    /// in general, you can ignore this data, if server not send binary
    fn recv_binary(&self, binary: Vec<u8>) -> impl Future<Output=()> + Send {
        log::warn!("recv websocket server binary data: [{binary:?}]");
        async {}
    }

    /// websocket client recv websocket server ping data will call this method<br />
    /// in general, you can ignore this data, if server not send ping
    fn recv_ping(&self, ping: Vec<u8>) -> impl Future<Output=()> + Send {
        log::warn!("recv websocket server ping data: [{ping:?}]");
        async {}
    }

    /// websocket client recv websocket server pong data will call this method<br />
    /// in general, you can ignore this data, if server not send pong
    fn recv_pong(&self, pong: Vec<u8>) -> impl Future<Output=()> + Send {
        log::warn!("recv websocket server pong data: [{pong:?}]");
        async {}
    }

    /// websocket client recv websocket server close data will call this method<br />
    /// in general, you can ignore this data, if server not send close
    fn recv_close(&self, close: Option<CloseFrame>) -> impl Future<Output=()> + Send {
        log::warn!("recv websocket server close data: [{close:?}]");
        async {}
    }

    /// websocket client recv websocket server frame data will call this method<br />
    /// in general, you can ignore this data, if server not send frame
    fn recv_frame(&self, frame: Frame) -> impl Future<Output=()> + Send {
        log::warn!("recv websocket server frame data: [{frame:?}]");
        async {}
    }
}