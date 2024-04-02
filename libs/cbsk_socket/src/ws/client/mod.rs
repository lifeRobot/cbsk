use std::sync::Arc;
use std::time::Duration;
use cbsk_base::{anyhow, log, tokio};
use cbsk_base::tokio::net::TcpStream;
use cbsk_base::tokio::task::JoinHandle;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
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
    /// websocket client writer
    pub(crate) write: Arc<MutDataObj<Option<MutDataObj<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>>,
}

/// support clone
impl<C: WsClientCallBack> Clone for WsClient<C> {
    fn clone(&self) -> Self {
        Self { conf: self.conf.clone(), cb: self.cb.clone(), write: self.write.clone() }
    }
}

/// data init etc
impl<C: WsClientCallBack> WsClient<C> {
    /// create websocket client<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn new(conf: Arc<WsClientConfig>, cb: Arc<C>) -> Self {
        Self { conf, cb, write: Arc::new(MutDataObj::default()) }
    }

    /// stop websocket server connect<br />
    /// will shutdown tcp connection and will not new connection
    pub async fn stop(&self) {
        self.conf.reconn.as_mut().enable = false;
        self.shutdown().await;
    }

    /// notify websocket to re connect<br />
    /// will shutdown websocket connection, if [`WsClientConfig`] reconn is disable<br />
    /// will shutdown and create new websocket connection,if [`WsClientConfig`] reconn is enable
    pub async fn re_conn(&self) {
        self.shutdown().await;
    }

    /// shutdown websocket server connect
    async fn shutdown(&self) {
        if let Some(write) = self.write.as_ref().as_ref() {
            if let Err(e) = write.as_mut().close().await {
                log::error!("shutdown websocket error: {e:?}");
            }
        }

        // as long as shutdown is called, write will be left blank directly
        self.write.set(None);
    }

    /// get has the websocket server connection been success
    pub fn is_connected(&self) -> bool {
        self.write.is_some()
    }
}

/// tcp read logic
impl<C: WsClientCallBack> WsClient<C> {
    /// start websocket client
    pub fn start(&self) -> JoinHandle<()> {
        let ws_client = self.clone();
        tokio::spawn(async move {
            // there are two loops here, so it should be possible to optimize them
            loop {
                ws_client.conn().await;

                if !ws_client.conf.reconn.enable { break; }
                log::error!("{} websocket server disconnected, preparing for reconnection",ws_client.conf.log_head);
            }

            log::info!("{} websocket client async has ended",ws_client.conf.log_head);
        })
    }

    /// connect websocket server and start read data
    async fn conn(&self) {
        let mut re_num = 0;
        loop {
            re_num += 1;
            let err =
                match self.try_conn().await {
                    Ok(ws_stream) => {
                        self.read_spawn(ws_stream).await;
                        return;
                    }
                    Err(e) => { e }
                };

            log::error!("{} websocket server connect error: {err:?}",self.conf.log_head);
            if !self.conf.reconn.enable { return; }

            // re conn
            self.cb.re_conn(re_num).await;
            log::info!("{} websocket service will reconnect in {:?}",self.conf.log_head,self.conf.reconn.time);
            tokio::time::sleep(self.conf.reconn.time).await;
        }
    }

    /// read websocket server data
    async fn read_spawn(&self, ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>) {
        let (write, read) = ws_stream.split();
        self.write.set(Some(MutDataObj::new(write)).into());

        log::info!("{} started websocket server read data async success",self.conf.log_head);
        self.cb.conn().await;

        if let Err(e) = self.try_read_spawn(read).await {
            // if the write is not closed, print the log.
            // otherwise, it is considered as actively closing the connection and there is no need to print the log
            if self.write.is_some() {
                log::error!("{} websocket server read data error: {e:?}",self.conf.log_head);
            }
        }

        // websocket read disabled, directly assume that websocket has been closed, simultaneously close read
        self.shutdown().await;
        self.cb.dis_conn().await;
        log::info!("{} websocket server read data async is shutdown",self.conf.log_head);
    }

    /// try read data from websocket server
    async fn try_read_spawn(&self, mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) -> anyhow::Result<()> {
        loop {
            let read = read.next();
            let msg =
                match tokio::time::timeout(self.conf.read_time_out, read).await {
                    Ok(msg) => {
                        cbsk_base::match_some_exec!(msg,{
                            // read empty data, it is possible that ws has already been shut down
                            if self.write.is_none(){
                                return Ok(());
                            }

                            // if read empty data, and ws connection is normal, sleep 1 millis, and continue to next loop
                            // read empty data, which may indicate a bug in tokio_tungstenite, so sleep for 1 millimeter and repeat the loop
                            tokio::time::sleep(Duration::from_millis(1)).await;
                            continue;
                        })?
                    }
                    Err(_e) => {
                        // if ws has already been shut down, exit loop
                        if self.write.is_none() {
                            return Ok(());
                        }

                        // if just timeout, continue
                        continue;
                    }
                };

            match msg {
                Message::Text(text) => { self.cb.recv_text(text).await }
                Message::Binary(binary) => { self.cb.recv_binary(binary).await }
                Message::Ping(ping) => { self.cb.recv_ping(ping).await }
                Message::Pong(pong) => { self.cb.recv_pong(pong).await }
                Message::Close(close) => { self.cb.recv_close(close).await }
                Message::Frame(frame) => { self.cb.recv_frame(frame).await }
            }
        }
    }

    /// try connect websocket server
    async fn try_conn(&self) -> anyhow::Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        log::info!("{} try connect to websocket server",self.conf.log_head);
        let ws_stream = tokio_tungstenite::connect_async(self.conf.ws_url.as_str());
        let (ws_stream, _) = tokio::time::timeout(self.conf.conn_time_out, ws_stream).await??;

        Ok(ws_stream)
    }
}
