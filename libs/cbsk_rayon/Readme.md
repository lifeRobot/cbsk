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
cbsk_base = "2.0.0"
cbsk_rayon = { version = "2.0.0", default-features = false, features = ["client"] }
fast_log = "1.7.3"
```

main.rs:

```rust
use std::net::{IpAddr, SocketAddr};
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;
use cbsk_base::log;
use cbsk_base::log::LevelFilter;
use cbsk_rayon::business::cbsk_write_trait::CbskWriteTrait;
use cbsk_rayon::client::callback::CbskClientCallBack;
use cbsk_rayon::client::CbskClient;

#[allow(non_upper_case_globals)]
static addr: LazyLock<SocketAddr> = LazyLock::new(|| { SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8080) });

#[allow(non_upper_case_globals)]
pub static cbsk_client: LazyLock<CbskClient> = LazyLock::new(|| {
   CbskClient::new(Cb {}.into(), *addr, 1024)
});

pub fn main() {
   fast_log::init(fast_log::Config::new().level(LevelFilter::Info).console()).unwrap();
   cbsk_client.start();
   loop {
      thread::sleep(Duration::from_secs(10));
   }
}

struct Cb {}

impl CbskClientCallBack for Cb {
   fn conn(&self) {
      cbsk_client.send_bytes(b"hello server".to_vec());
   }

   fn recv(&self, bytes: Vec<u8>) {
      log::info!("bytes is {:?}",String::from_utf8(bytes));
      cbsk_client.send_bytes(b"hello server".to_vec());
   }
}
```

### cbsk server example

Cargo.toml:

```toml
cbsk_base = "2.0.0"
cbsk_rayon = "2.0.0"
fast_log = "1.7.3"
```

main.rs:

```rust
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use cbsk_base::log;
use cbsk_base::log::LevelFilter;
use cbsk_rayon::business::cbsk_write_trait::CbskWriteTrait;
use cbsk_rayon::server::callback::CbskServerCallBack;
use cbsk_rayon::server::CbskServer;
use cbsk_rayon::server::client::CbskServerClient;

pub fn main() {
    fast_log::init(fast_log::config::Config::default().level(LevelFilter::Info).console()).unwrap();
    let addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8080);
    let cbsk_server = CbskServer::new(Cb {}.into(), addr, 1024);
    cbsk_server.start();

    loop {
        thread::sleep(Duration::from_secs(10));
    }
}

struct Cb {}

impl CbskServerCallBack for Cb {
    fn recv(&self, bytes: Vec<u8>, client: Arc<CbskServerClient>) {
        log::info!("recv client data is {:?}", String::from_utf8(bytes));
        // thread::sleep(Duration::from_secs(3));
        client.send_bytes(b"hello client".to_vec());
    }
}
```
