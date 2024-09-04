use std::io;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use cbsk_base::{anyhow, log, tokio};
use cbsk_base::tokio::net::TcpStream;
use cbsk_base::tokio::sync::RwLock;
use cbsk_base::tokio::task::JoinHandle;
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::SplitStream;
use tokio_tungstenite::{MaybeTlsStream, tungstenite, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use crate::ws::client::callback::WsClientCallBack;
use crate::ws::client::config::WsClientConfig;
use crate::ws::client::ws_write::WsWrite;
use crate::ws::ws_write_trait::WsWriteTrait;

pub mod config;
pub mod callback;
mod ws_write;

/// websocket client
pub struct WsClient<C: WsClientCallBack> {
    /// websocket config
    pub conf: Arc<WsClientConfig>,
    /// websocket client business callback
    pub cb: Arc<C>,
    /// websocket client writer
    pub(crate) write: Arc<RwLock<WsWrite>>,
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
        Self { conf, cb, write: Arc::new(RwLock::new(WsWrite::default())) }
    }

    /// stop websocket server connect<br />
    /// will shutdown tcp connection and will not new connection
    pub async fn stop(&self) {
        self.conf.reconn.enable.store(false, Ordering::Release);
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
        let mut write = self.write.write().await;
        if let Some(write) = write.write.as_mut() {
            if let Err(e) = write.close().await {
                log::error!("shutdown websocket error: {e:?}");
            }
        }

        // as long as shutdown is called, write will be left blank directly
        write.set_none();
    }

    /// get has the websocket server connection been success
    pub async fn is_connected(&self) -> bool {
        self.write.read().await.write.is_some()
    }
}

/// tcp read logic
impl<C: WsClientCallBack> WsClient<C> {
    /// start websocket client
    pub async fn start(&self) {
        // there are two loops here, so it should be possible to optimize them
        loop {
            self.conn().await;

            if !self.conf.reconn.enable.load(Ordering::Acquire) { break; }
            log::error!("{} websocket server disconnected, preparing for reconnection",self.conf.log_head);
        }

        log::info!("{} websocket client async has ended",self.conf.log_head);
    }

    pub fn start_in_handle(&self) -> JoinHandle<()> {
        let ws_server = self.clone();
        tokio::spawn(async move { ws_server.start().await; })
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
            if !self.conf.reconn.enable.load(Ordering::Acquire) { return; }

            // re conn
            self.cb.re_conn(re_num).await;
            log::info!("{} websocket service will reconnect in {:?}",self.conf.log_head,self.conf.reconn.time);
            tokio::time::sleep(self.conf.reconn.time).await;
        }
    }

    /// read websocket server data
    async fn read_spawn(&self, ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>) {
        let (write, read) = ws_stream.split();
        self.write.write().await.set_write(write);

        log::info!("{} started websocket server read data async success",self.conf.log_head);
        self.cb.conn().await;

        if let Err(e) = self.try_read_spawn(read).await {
            // if the write is not closed, print the log.
            // otherwise, it is considered as actively closing the connection and there is no need to print the log
            if self.write.read().await.write.is_some() {
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
                            if self.write.read().await.write.is_none(){
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
                        if self.write.read().await.write.is_none() {
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

/// support ws write trait
impl<C: WsClientCallBack> WsWriteTrait for WsClient<C> {
    fn get_log_head(&self) -> &str {
        self.conf.log_head.as_str()
    }

    async fn try_send(&self, msg: Message) -> tungstenite::Result<()> {
        let mut write = self.write.write().await;
        let write = write.write.as_mut().ok_or_else(|| {
            io::Error::from(io::ErrorKind::NotConnected)
        })?;
        write.send(msg).await?;
        write.flush().await
    }
}
