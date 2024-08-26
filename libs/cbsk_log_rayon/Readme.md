cbsk_log is a log tool, the inspiration for this log writing library comes
from [fast_log](https://crates.io/crates/fast_log)

### file split example

Cargo.toml:

```toml
cbsk_base = "2.0.1"
cbsk_log_rayon = "2.0.1"
```

main.rs:

```rust
use cbsk_base::log;
use cbsk_log_rayon::cbsk_log::config::Config;
use cbsk_log_rayon::cbsk_log::model::log_size::LogSize;
use cbsk_log_rayon::config::FileSplitTrait;
use cbsk_log_rayon::packer::zip_packer::ZipPacker;

fn main() {
    let config = Config::default().console().file_split("E:\\logs\\", LogSize::KB(5), ZipPacker::pack_end(|zip_path| {
        println!("{zip_path}");
    }));
    cbsk_log_rayon::init(config).unwrap();
    for i in 1..10000 {
        log::info!("hello world, {i}");
    }

    log::logger().flush();
}
```

### console example

Cargo.toml:

```toml
cbsk_base = "2.0.1"
cbsk_log_rayon = "2.0.1"
```

main.rs:

```rust
use cbsk_base::log;
use cbsk_log_rayon::cbsk_log::config::Config;

fn main() {
    cbsk_log_rayon::init(Config::default().console()).unwrap();
    for i in 1..10000 {
        log::info!("hello world, {i}");
    }

    log::logger().flush();
}
```