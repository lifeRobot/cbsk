cbsk is a TCP data callback tool that allows you to focus on your business processing without having to worry about TCP
data read, write, and split

### internal protocol

cbsk has a custom TCP data verification protocol internally, and the protocol logic is as follows:

1. Verify if the data uses ['c ',' b ','s',' k '] as the header frame. If not, the data will be discarded. Of course,
   you can customize the data frame header

2. Obtain the first byte after the header frame, which represents the length description of the data length

3. Obtain the true data length based on the length description of the data length

4. Read the real data, if there is packet occupancy, split it and call a callback. If the length is insufficient, wait
   for the next TCP data to be obtained until the length is consistent

5. Repeat the above steps and start from the first one again

### example

Cargo.toml:

```toml
cbsk = { version = "0.1.0", features = ["server"] }
```

main.rs:

```rust
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use cbsk::business::cbsk_write_trait::CbskWriteTrait;
use cbsk::client::callback::CbskClientCallBack;
use cbsk::client::CbskClient;
use cbsk::server::callback::CbskServerCallBack;
use cbsk::server::CbskServer;
use cbsk::server::client::CbskServerClient;
use cbsk::cbsk_socket::cbsk_base::once_cell::sync::Lazy;
use cbsk::cbsk_socket::cbsk_base::tokio;

#[allow(non_upper_case_globals)]
static addr: Lazy<SocketAddr> = Lazy::new(|| { SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8080) });

#[allow(non_upper_case_globals)]
static cbsk_client: Lazy<CbskClient<CbskClientBusiness>> = Lazy::new(|| { CbskClient::new(CbskClientBusiness {}.into(), *addr) });

#[tokio::main]
async fn main() {
    let cbsk_server = CbskServer::new(CbskServerBusiness {}.into(), *addr);

    // start cbsk and wait stop
    for handle in [cbsk_server.start::<1024>(), cbsk_client.start::<1024>()] {
        handle.await.unwrap()
    }
}

// ------------------------- you server business ---------------------
struct CbskServerBusiness {}

impl CbskServerCallBack for CbskServerBusiness {
    async fn recv(&self, bytes: Vec<u8>, client: Arc<CbskServerClient>) {
        println!("recv client data is {:?}", String::from_utf8(bytes));

        // wait for 1 second before sending data to the client
        // simulate business logic processing time
        tokio::time::sleep(Duration::from_secs(1)).await;
        client.send_text("this is cbsk server, hello cbsk client").await;
    }
}

// ------------------------- you client business ---------------------

struct CbskClientBusiness {}

impl CbskClientCallBack for CbskClientBusiness {
    async fn conn(&self) {
        cbsk_client.send_text("hello cbsk server").await;
    }

    async fn recv(&self, bytes: Vec<u8>) {
        println!("recv server data is {:?}", String::from_utf8(bytes));
        cbsk_client.send_text("this is cbsk client, hello cbsk server").await;
    }
}
```