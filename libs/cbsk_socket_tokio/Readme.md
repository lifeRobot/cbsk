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
fast_log = "1.7.6"
cbsk_base = "2.1.1"
cbsk_socket_tokio = { version = "2.1.1", default-features = false, features = ["tcp_server"] }
```

main.rs file:

```rust
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use cbsk_base::{log, tokio};
use cbsk_base::async_trait::async_trait;
use cbsk_base::log::LevelFilter;
use cbsk_socket_tokio::cbsk_socket::tcp::server::config::TcpServerConfig;
use cbsk_socket_tokio::tcp::common::tcp_write_trait::TcpWriteTrait;
use cbsk_socket_tokio::tcp::server::callback::TcpServerCallBack;
use cbsk_socket_tokio::tcp::server::client::TcpServerClient;
use cbsk_socket_tokio::tcp::server::TcpServer;

#[tokio::main]
pub async fn main() {
    fast_log::init(fast_log::config::Config::default().level(LevelFilter::Info).console()).unwrap();
    let addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8080);
    let conf = TcpServerConfig::new("".into(), addr, false);
    let tcp_server = TcpServer::new(conf.into(), Cb {});
    tcp_server.start().await;
}

struct Cb {}

#[async_trait]
impl TcpServerCallBack for Cb {
    async fn recv(&self, bytes: Vec<u8>, client: Arc<TcpServerClient>) -> Vec<u8> {
        log::info!("recv is {bytes:?}");
        client.send_bytes(b"hello client").await;
        Vec::with_capacity(1)
    }
}
```

</details>

### tcp client example

<details open>
<summary>tcp client example</summary>

Cargo.toml file:

```toml
fast_log = "1.7.6"
cbsk_base = "2.1.1"
cbsk_socket_tokio = "2.1.1" 
```

main.rs file:

```rust
use std::net::{IpAddr, SocketAddr};
use std::sync::LazyLock;
use std::time::Duration;
use cbsk_base::{log, tokio};
use cbsk_base::async_trait::async_trait;
use cbsk_base::log::LevelFilter;
use cbsk_socket_tokio::cbsk_socket::config::re_conn::SocketReConn;
use cbsk_socket_tokio::cbsk_socket::tcp::client::config::TcpClientConfig;
use cbsk_socket_tokio::tcp::client::callback::TcpClientCallBack;
use cbsk_socket_tokio::tcp::client::TcpClient;
use cbsk_socket_tokio::tcp::common::tcp_write_trait::TcpWriteTrait;

#[allow(non_upper_case_globals)]
static addr: LazyLock<SocketAddr> = LazyLock::new(|| { SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8080) });

#[allow(non_upper_case_globals)]
static tcp_client: LazyLock<TcpClient> = LazyLock::new(|| {
    let conf = TcpClientConfig::new("tcp client".into(), *addr, SocketReConn::enable(Duration::from_secs(3)));
    TcpClient::new(conf.into(), Cb {})
});

#[tokio::main]
pub async fn main() {
    fast_log::init(fast_log::config::Config::default().level(LevelFilter::Info).console()).unwrap();
    tcp_client.start().await;
}

struct Cb {}

#[async_trait]
impl TcpClientCallBack for Cb {
    async fn conn(&self) {
        tcp_client.send_bytes(b"hello server").await;
    }

    async fn recv(&self, bytes: Vec<u8>) -> Vec<u8> {
        log::info!("bytes is {bytes:?}");
        tcp_client.send_bytes(b"hello server").await;
        Vec::with_capacity(1)
    }
}
```

</details>
