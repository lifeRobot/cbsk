cbsk is a TCP data callback tool that allows you to focus on your business processing without having to worry about TCP
data read, write, and split

### support the minimum version of Rust

1.80.0

### internal protocol

cbsk has a custom TCP data verification protocol internally, and the protocol logic is as follows:

1. Verify if the data uses ['c ',' b ','s',' k '] as the header frame. If not, the data will be discarded. Of course,
   you can customize the data frame header

2. Obtain the first byte after the header frame, which represents the length description of the data length

3. Obtain the true data length based on the length description of the data length

4. Read the real data, if there is packet occupancy, split it and call a callback. If the length is insufficient, wait
   for the next TCP data to be obtained until the length is consistent

5. Repeat the above steps and start from the first one again

### cbsk client example

Cargo.toml:

```toml
cbsk_base = "2.1.2"
cbsk_tokio = { version = "2.1.2", default-features = false, features = ["client"] }
fast_log = "1.7.6"
```

main.rs:

```rust
use std::net::{IpAddr, SocketAddr};
use std::sync::LazyLock;
use cbsk_base::{log, tokio};
use cbsk_base::log::LevelFilter;
use cbsk_tokio::business::cbsk_write_trait::CbskWriteTrait;
use cbsk_tokio::client::callback::CbskClientCallBack;
use cbsk_tokio::client::CbskClient;

#[allow(non_upper_case_globals)]
static addr: LazyLock<SocketAddr> = LazyLock::new(|| { SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8080) });

#[allow(non_upper_case_globals)]
static cbsk_client: LazyLock<CbskClient> = LazyLock::new(|| {
    CbskClient::new(Cb {}.into(), *addr, 1024)
});

#[tokio::main]
pub async fn main() {
    fast_log::init(fast_log::config::Config::default().level(LevelFilter::Info).console()).unwrap();
    cbsk_client.start().await;
}

struct Cb {}

impl CbskClientCallBack for Cb {
    async fn conn(&self) {
        cbsk_client.send_bytes(b"hello server".to_vec()).await;
    }

    async fn recv(&self, bytes: Vec<u8>) {
        log::info!("bytes is {bytes:?}");
        cbsk_client.send_bytes(b"hello server".to_vec()).await;
    }
}
```

### cbsk server example

Cargo.toml:

```toml
cbsk_base = "2.1.2"
cbsk_tokio = "2.1.2"
fast_log = "1.7.6"
```

main.rs:

```rust
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use cbsk_base::log::LevelFilter;
use cbsk_base::{log, tokio};
use cbsk_tokio::business::cbsk_write_trait::CbskWriteTrait;
use cbsk_tokio::server::callback::CbskServerCallBack;
use cbsk_tokio::server::CbskServer;
use cbsk_tokio::server::client::CbskServerClient;

#[tokio::main]
pub async fn main() {
    fast_log::init(fast_log::config::Config::default().level(LevelFilter::Info).console()).unwrap();
    let addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8080);
    let cbsk_server = CbskServer::new(Cb {}.into(), addr, 1024);
    cbsk_server.start().await;
}

struct Cb {}

impl CbskServerCallBack for Cb {
    async fn recv(&self, bytes: Vec<u8>, client: Arc<CbskServerClient>) {
        log::info!("recv is {bytes:?}");
        client.send_bytes(b"hello client".to_vec()).await;
    }
}
```
