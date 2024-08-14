cbsk_socket is a socket callback tool  
you can use cbsk_socket create TCP/WebSocket server or client, you don't need to focus on TCP/WebSocket read and write,
just focus on business processing

### minimum supported Rust version

Rust 1.80.0

### now supported sockets

* tcp client √
* tcp server √
* ws client √
* ws server √

### tcp server example

<details>
<summary>tcp server example</summary>

Cargo.toml file:

```toml
fast_log = "1.7.3"
cbsk_base = { version = "1.3.10" }
cbsk_socket = { version = "1.3.10", features = ["tcp_server"] }
```

main.rs file:

```rust
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use cbsk_base::{log, tokio};
use cbsk_socket::tcp::server::callback::TcpServerCallBack;
use cbsk_socket::tcp::server::client::TcpServerClient;
use cbsk_socket::tcp::server::config::TcpServerConfig;
use cbsk_socket::tcp::server::TcpServer;
use cbsk_socket::tcp::write_trait::WriteTrait;

#[tokio::main]
async fn main() {
    // print log to console
    let fast_config = fast_log::config::Config::default().console();
    fast_log::init(fast_config).unwrap();

    // start tcp server
    let addr = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 8080);
    let tcp_config = TcpServerConfig::new("test".into(), addr, false);
    let tcp_server = TcpServer::new(tcp_config.into(), TcpServerBusiness {}.into());
    let handle = tcp_server.start::<1024>();

    // wait handle
    handle.await.unwrap();

    // wait log flush
    log::logger().flush();
}

/// you tcp server business
pub struct TcpServerBusiness {}

/// business callback
impl TcpServerCallBack for TcpServerBusiness {
    async fn recv(&self, bytes: Vec<u8>, client: Arc<TcpServerClient>) -> Vec<u8> {
        println!("{} read bytes [{bytes:?}]", client.log_head);

        // send bytes to tcp client
        client.send_bytes(b"hello world").await;
        client.send_text("hello world").await;
        client.send_json(&"hello world".to_string()).await;

        Vec::new()
    }
}
```

</details>

### tcp client example

<details>
<summary>tcp client example</summary>

Cargo.toml file:

```toml
fast_log = "1.7.3"
cbsk_base = { version = "1.3.10", features = ["async-trait"] }
cbsk_socket = { version = "1.3.10", features = ["tcp_client"] }
```

main.rs file:

```rust
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use cbsk_base::async_trait::async_trait;
use cbsk_base::{log, tokio};
use cbsk_socket::config::re_conn::SocketReConn;
use cbsk_socket::tcp::client::callback::TcpClientCallBack;
use cbsk_socket::tcp::client::config::TcpClientConfig;
use cbsk_socket::tcp::client::TcpClient;
use cbsk_socket::tcp::write_trait::WriteTrait;

#[tokio::main]
async fn main() {
    // print log to console
    let fast_config = fast_log::config::Config::default().console();
    fast_log::init(fast_config).unwrap();

    // start tcp client
    let addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8080);
    let tcp_config = TcpClientConfig::new("test".into(), addr, SocketReConn::enable(Duration::from_secs(3)));
    let tcp_client = TcpClient::new(tcp_config.into(), TcpClientBusiness {}.into());
    let read_handle = tcp_client.start::<1024>();

    // if tcp server connect success, send bytes to tcp server
    let write_handle = tokio::spawn(async move {
        loop {
            if tcp_client.is_connected() {
                tcp_client.send_bytes(b"hello world").await;
                tcp_client.send_text("hello world").await;
                tcp_client.send_json(&"hello world".to_string()).await;
            }

            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });

    // wait handle
    read_handle.await.unwrap();
    write_handle.await.unwrap();

    // wait log flush
    log::logger().flush();
}

/// you tcp client business
pub struct TcpClientBusiness {}

/// business callback
#[async_trait]
impl TcpClientCallBack for TcpClientBusiness {
    async fn recv(&self, bytes: Vec<u8>) -> Vec<u8> {
        println!("read bytes [{bytes:?}]");

        Vec::new()
    }
}
```

</details>

### features explain

the following features are only valid for `tcp_server` or `tcp_client`

1. default is `tokio_tcp`, use tokio runtime and tokio tcp
2. `tokio_tcp`, use tokio runtime and tokio tcp
3. `system_tcp`, use tokio runtime and system tcp

### other issues

1. The reason for adding system tcp and thread runtime is that during some Linux testing, there was a deadlock issue
   with `tokio::net::TcpStream` + `tokio runtime`, causing tokio to not run. As this Linux is customized, we are
   currently
   unable to provide testable issues to Tokio. If you are using Windows, Windows Server 2012, macos, ubuntu, etc., this
   cargo crate is normal and you can use the default `tokio_tcp`

2. websocket tls coming soon  
   if y want to use tls, y can
   use [tokio-tungstenite](https://crates.io/crates/tokio-tungstenite)([github](https://github.com/snapview/tokio-tungstenite))
