cbsk_log is a log tool, the inspiration for this log writing library comes
from [fast_log](https://crates.io/crates/fast_log)

### support the minimum version of Rust

1.80.0

### file split example

Cargo.toml:

```toml
cbsk_base = "2.1.0"
cbsk_log_tokio = "2.1.0"
```

main.rs:

```rust
use cbsk_base::{log, tokio};
use cbsk_log_tokio::cbsk_log::config::Config;
use cbsk_log_tokio::cbsk_log::model::log_size::LogSize;
use cbsk_log_tokio::config::FileSplitTrait;
use cbsk_log_tokio::packer::zip_packer::ZipPacker;

#[tokio::main]
async fn main() {
    let config = Config::default().file_split("E:\\logs\\", LogSize::KB(5), ZipPacker::pack_end(|zip_path| {
        Box::pin(async move {
            println!("{zip_path}");
        })
    }));
    cbsk_log_tokio::init(config).unwrap();
    for i in 1..10000 {
        log::info!("hello world, {i}");
    }

    log::logger().flush();
}
```

### console example

Cargo.toml:

```toml
cbsk_base = "2.1.0"
cbsk_log_tokio = "2.1.0"
```

main.rs:

```rust
use cbsk_base::{log, tokio};
use cbsk_log_tokio::cbsk_log::config::Config;

#[tokio::main]
async fn main() {
    cbsk_log_tokio::init(Config::default().console()).unwrap();
    for i in 1..10000 {
        log::info!("hello world, {i}");
    }

    log::logger().flush();
}

```