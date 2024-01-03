use std::io;
use std::sync::Arc;
use std::time::Duration;
use cbsk_base::{anyhow, log, tokio};
use cbsk_base::tokio::net::{TcpListener, TcpStream};
use cbsk_base::tokio::task::JoinHandle;
use futures_util::StreamExt;
use futures_util::stream::SplitStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::ws::server::callback::WsServerCallBack;
use crate::ws::server::client::WsServerClient;
use crate::ws::server::config::WsServerConfig;

pub mod config;
pub mod client;
pub mod callback;

/// websocket server
pub struct WsServer<C: WsServerCallBack> {
    /// websocket config
    pub conf: Arc<WsServerConfig>,
    /// websocket server business callback
    pub cb: Arc<C>,
}

/// support clone
impl<C: WsServerCallBack> Clone for WsServer<C> {
    fn clone(&self) -> Self {
        Self { conf: self.conf.clone(), cb: self.cb.clone() }
    }
}

/// data init etc
impl<C: WsServerCallBack> WsServer<C> {
    /// create a websocket server<br />
    /// just create data, if you want to read data to recv method, you should be call start method
    pub fn new(conf: Arc<WsServerConfig>, cb: Arc<C>) -> Self {
        Self { conf, cb }
    }
}

/// websocket read logic
impl<C: WsServerCallBack> WsServer<C> {
    /// start websocket server
    pub async fn start(&self) -> JoinHandle<()> {
        let ws_server = self.clone();
        tokio::spawn(async move {
            let mut read_handles = Vec::new();

            if let Err(e) = ws_server.try_start(&mut read_handles).await {
                log::error!("{} tcp bind [{}] error: {e:?}",ws_server.conf.log_head,ws_server.conf.addr);
            }

            // wait read async
            for handle in read_handles {
                if let Err(e) = handle.await {
                    log::error!("{} read async error: {e:?}",ws_server.conf.log_head);
                }
            }
        })
    }

    /// try start websocket server
    async fn try_start(&self, read_handles: &mut Vec<JoinHandle<()>>) -> io::Result<()> {
        let listener = TcpListener::bind(self.conf.addr).await?;
        log::info!("{} listener WebSocket[{}] success",self.conf.log_head,self.conf.addr);

        // loop waiting for client to connect
        loop {
            if let Err(e) = self.try_accept(&listener, read_handles).await {
                log::error!("{} wait websocket accept error. wait for the next accept in three seconds. error: {e:?}",self.conf.log_head);
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }

    /// try accept websocket client and read websocket client data
    async fn try_accept(&self, listener: &TcpListener, read_handles: &mut Vec<JoinHandle<()>>) -> anyhow::Result<()> {
        // accept client and split write and read
        let (tcp_stream, addr) = listener.accept().await?;
        let (write, read) = tokio_tungstenite::accept_async(tcp_stream).await?.split();

        // start read data
        let client = Arc::new(WsServerClient::new(addr, self.conf.as_ref(), write));
        read_handles.push(self.read_spawn(client.clone(), read));
        self.cb.conn(client).await;

        Ok(())
    }

    /// start read async
    fn read_spawn(&self, client: Arc<WsServerClient>, read: SplitStream<WebSocketStream<TcpStream>>) -> JoinHandle<()> {
        let ws_server = self.clone();
        tokio::spawn(async move {
            if let Err(e) = ws_server.try_read_spawn(client.clone(), read).await {
                if ws_server.conf.log { log::error!("{} read websocket client data error: {e:?}",client.log_head); }
            }

            // if websocket read is closed, it is considered that websocket has been closed
            ws_server.cb.dis_conn(client.clone()).await;
        })
    }

    /// try read websocket client data
    async fn try_read_spawn(&self, client: Arc<WsServerClient>, mut read: SplitStream<WebSocketStream<TcpStream>>) -> anyhow::Result<()> {
        if self.conf.log { log::info!("{} start websocket client read async success",client.log_head); }

        loop {
            let read = read.next();
            let msg = match tokio::time::timeout(self.conf.read_time_out, read).await {
                Ok(msg) => {
                    // if read empty data, continue to next loop
                    cbsk_base::match_some_exec!(msg,{continue;})?
                }
                Err(_e) => {
                    // if just timeout, continue to next loop
                    continue;
                }
            };

            match msg {
                Message::Text(text) => { self.cb.recv_text(text, client.clone()).await }
                Message::Binary(binary) => { self.cb.recv_binary(binary, client.clone()).await }
                Message::Ping(ping) => { self.cb.recv_ping(ping, client.clone()).await }
                Message::Pong(pong) => { self.cb.recv_pong(pong, client.clone()).await }
                Message::Close(close) => { self.cb.recv_close(close, client.clone()).await }
                Message::Frame(frame) => { self.cb.recv_frame(frame, client.clone()).await }
            }
        }
    }
}
