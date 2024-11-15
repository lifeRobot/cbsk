cbsk_socket is a socket callback tool  
you can use cbsk_socket create TCP/WebSocket server or client, you don't need to focus on TCP/WebSocket read and write,
just focus on business processing

### minimum supported Rust version

Rust 1.80.0

### now supported sockets

* tcp client √
* tcp server √

### tcp server example

<details>
<summary>tcp server example</summary>

Cargo.toml file:

```toml
fast_log = "1.7.5"
cbsk_base = "2.0.6"
cbsk_socket_rayon = { version = "2.0.6", default-features = false, features = ["tcp_server"] }
```

main.rs file:

```rust
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use cbsk_base::log;
use cbsk_base::log::LevelFilter;
use cbsk_socket_rayon::cbsk_socket::tcp::server::config::TcpServerConfig;
use cbsk_socket_rayon::tcp::common::tcp_write_trait::TcpWriteTrait;
use cbsk_socket_rayon::tcp::server::callback::TcpServerCallBack;
use cbsk_socket_rayon::tcp::server::client::TcpServerClient;
use cbsk_socket_rayon::tcp::server::TcpServer;

pub fn main() {
    fast_log::init(fast_log::config::Config::default().level(LevelFilter::Info).console()).unwrap();
    let addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8080);
    let conf = TcpServerConfig::new("".into(), addr, false);
    let tcp_server = TcpServer::new(conf.into(), Cb {});
    tcp_server.start();

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

struct Cb {}

impl TcpServerCallBack for Cb {
    fn recv(&self, bytes: Vec<u8>, client: Arc<TcpServerClient>) -> Vec<u8> {
        log::info!("recv is {bytes:?}");
        client.send_bytes(b"hello client");
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
fast_log = "1.7.5"
cbsk_base = "2.0.6"
cbsk_socket_rayon = "2.0.6" 
```

main.rs file:

```rust
use std::net::{IpAddr, SocketAddr};
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;
use cbsk_base::log;
use cbsk_base::log::LevelFilter;
use cbsk_socket_rayon::cbsk_socket::config::re_conn::SocketReConn;
use cbsk_socket_rayon::cbsk_socket::tcp::client::config::TcpClientConfig;
use cbsk_socket_rayon::tcp::client::callback::TcpClientCallBack;
use cbsk_socket_rayon::tcp::client::TcpClient;
use cbsk_socket_rayon::tcp::common::tcp_write_trait::TcpWriteTrait;

#[allow(non_upper_case_globals)]
static addr: LazyLock<SocketAddr> = LazyLock::new(|| { SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8080) });

#[allow(non_upper_case_globals)]
static tcp_client: LazyLock<TcpClient> = LazyLock::new(|| {
    let conf = TcpClientConfig::new("tcp client".into(), *addr, SocketReConn::enable(Duration::from_secs(3)));
    TcpClient::new(conf.into(), Cb {})
});

pub fn main() {
    fast_log::init(fast_log::config::Config::default().level(LevelFilter::Info).console()).unwrap();
    tcp_client.start();

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

struct Cb {}

impl TcpClientCallBack for Cb {
    fn conn(&self) {
        tcp_client.send_bytes(b"hello server");
    }

    fn recv(&self, bytes: Vec<u8>) -> Vec<u8> {
        log::info!("bytes is {bytes:?}");
        tcp_client.send_bytes(b"hello server");
        Vec::with_capacity(1)
    }
}
```

</details>
