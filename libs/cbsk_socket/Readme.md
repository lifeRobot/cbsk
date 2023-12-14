cbsk_socket is a socket callback tool  
you can use cbsk_socket create tcp server or tcp client, you don't need to focus on TCP read and write, just focus on
business processing

### now supported sockets

* tcp client √
* tcp server √
* ws client coming soon
* ws server coming soon

### tcp server example

Cargo.toml file:

```toml
fast_log = "1.6.10"
cbsk_base = { version = "0.1.0" }
cbsk_run = { version = "0.1.1", default-features = false, features = ["async_pool"] }
cbsk_socket = { version = "0.1.0", features = ["tcp_server"] }
```

main.rs file:

```rust
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use cbsk_base::async_trait::async_trait;
use cbsk_base::{log, tokio};
use cbsk_base::tokio::task::JoinHandle;
use cbsk_socket::tcp::server::callback::TcpServerCallBack;
use cbsk_socket::tcp::server::client::TcpServerClient;
use cbsk_socket::tcp::server::config::TcpServerConfig;
use cbsk_socket::tcp::server::TcpServer;
use cbsk_socket::tcp::write_trait::WriteTrait;

/// static tcp server client<br />
/// you can you this client send bytes to tcp client
pub static TCP_CLIENT: OnceLock<Arc<TcpServerClient>> = OnceLock::new();

#[tokio::main]
async fn main() {
    // print log to console
    let fast_config = fast_log::config::Config::default().console();
    fast_log::init(fast_config).unwrap();

    // start tcp server
    cbsk_run::async_pool::push(async {
        TcpServerBusiness::start().await.unwrap()
    });

    // if tcp client connect success,send bytes to tcp client
    cbsk_run::async_pool::push(async {
        loop {
            if let Some(tcp_client) = TCP_CLIENT.get() {
                tcp_client.send_bytes(b"hello world").await;
                tcp_client.send_json(&"hello world".to_string()).await;
                tcp_client.send_text("hello world").await;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    // listener async pool
    cbsk_run::async_pool::listener().await.unwrap();

    // wait log flush
    log::logger().flush();
}

/// you tcp server business
pub struct TcpServerBusiness {}

/// custom method
impl TcpServerBusiness {
    /// start tcp server
    pub fn start() -> JoinHandle<()> {
        // bind 0.0.0.0:8080
        let addr = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 8080);
        let tcp_config = TcpServerConfig::new("test".into(), addr, false);
        let tcp_server = TcpServer::new(tcp_config.into(), Self {}.into());
        tcp_server.start::<1024>()
    }
}

/// business callback
#[async_trait]
impl TcpServerCallBack for TcpServerBusiness {
    async fn conn(&self, client: Arc<TcpServerClient>, handle: JoinHandle<()>) {
        // you can write the business after the tcp client connect success in here
        println!("a new connect come in: {}", client.log_head);
        if TCP_CLIENT.set(client.clone()).is_err() {
            println!("set global tcp client fail");
        }

        // add client read async to global async pool
        cbsk_run::async_pool::push(async move {
            if let Err(e) = handle.await {
                eprintln!("{} run error: {e:?}", client.log_head);
            }
        })
    }

    async fn dis_conn(&self, client: Arc<TcpServerClient>) {
        // you can write the business after the tcp client disconnecting in here
        println!("{} tcp client disconnect", client.log_head)
    }

    async fn recv(&self, mut bytes: Vec<u8>, client: Arc<TcpServerClient>) -> Vec<u8> {
        // you can process the bytes read from the tcp client here
        println!("{} read bytes [{bytes:?}]", client.log_head);

        // if you think the data length is to short, you can return bytes wait next recv
        // the next recv will be append data to bytes
        if bytes.len() < 10 {
            return bytes;
        }

        // if you think the data length is to long
        // you should handle the appropriate data and return the data that does not meet the requirements in the end
        if bytes.len() > 13 {
            let next_bytes = bytes.split_off(10);
            println!("{} valid data: [{bytes:?}]", client.log_head);
            // return the next recv should be append data
            return next_bytes;
        }

        // business processing completed, returning empty vec
        Vec::new()
    }
}
```

### tcp client example

Cargo.toml file:

```toml
fast_log = "1.6.10"
cbsk_base = { version = "0.1.0" }
cbsk_run = { version = "0.1.1", default-features = false, features = ["async_pool"] }
cbsk_socket = { version = "0.1.0", features = ["tcp_client"] }
```

main.rs file:

```rust
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use cbsk_base::{log, tokio};
use cbsk_base::async_trait::async_trait;
use cbsk_base::tokio::task::JoinHandle;
use cbsk_socket::config::re_conn::SocketReConn;
use cbsk_socket::tcp::client::callback::TcpClientCallBack;
use cbsk_socket::tcp::client::config::TcpClientConfig;
use cbsk_socket::tcp::client::TcpClient;
use cbsk_socket::tcp::write_trait::WriteTrait;

/// static tcp client<br />
/// you can you this client send bytes to tcp server
pub static TCP_CLIENT: OnceLock<Arc<TcpClient<TcpClientBusiness>>> = OnceLock::new();

#[tokio::main]
async fn main() {
    // print log to console
    let fast_config = fast_log::config::Config::default().console();
    fast_log::init(fast_config).unwrap();

    // start tcp client
    cbsk_run::async_pool::push(async {
        TcpClientBusiness::start().await.unwrap();
    });

    // if tcp client connect success,send bytes to tcp server
    cbsk_run::async_pool::push(async {
        loop {
            if let Some(tcp_client) = TCP_CLIENT.get() {
                tcp_client.send_bytes(b"hello world").await;
                tcp_client.send_json(&"hello world".to_string()).await;
                tcp_client.send_text("hello world").await;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    // listener async pool
    cbsk_run::async_pool::listener().await.unwrap();

    // wait log flush
    log::logger().flush();
}

/// you tcp client business
pub struct TcpClientBusiness {}

/// custom method
impl TcpClientBusiness {
    /// start tcp client
    pub fn start() -> JoinHandle<()> {
        let addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8080);
        let tcp_config = TcpClientConfig::new("test".into(), addr, SocketReConn::enable(Duration::from_secs(3)));
        let tcp_client = TcpClient::new(tcp_config.into(), Self {}.into());
        let handle = tcp_client.start::<1024>();

        if TCP_CLIENT.set(tcp_client.into()).is_err() {
            println!("set global tcp client fail");
        }

        handle
    }
}

/// business callback
#[async_trait]
impl TcpClientCallBack for TcpClientBusiness {
    async fn conn(&self) {
        println!("connect tcp server success");
    }

    async fn dis_conn(&self) {
        println!("disconnect tcp server");
    }

    async fn re_conn(&self, num: i32) {
        println!("re connect to tcp server, re num is {num}");
    }

    async fn recv(&self, mut bytes: Vec<u8>) -> Vec<u8> {
        // you can process the bytes read from the tcp client here
        println!("read bytes [{bytes:?}]");

        // if you think the data length is to short, you can return bytes wait next recv
        // the next recv will be append data to bytes
        if bytes.len() < 10 {
            return bytes;
        }

        // if you think the data length is to long
        // you should handle the appropriate data and return the data that does not meet the requirements in the end
        if bytes.len() > 13 {
            let next_bytes = bytes.split_off(10);
            println!("valid data: [{bytes:?}]");
            // return the next recv should be append data
            return next_bytes;
        }

        // business processing completed, returning empty vec
        Vec::new()
    }
}
```