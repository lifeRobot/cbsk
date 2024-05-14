cbsk_log is a log tool, the inspiration for this log writing library comes from [fast_log](https://crates.io/crates/fast_log)

### file split example

Cargo.toml:

```toml
cbsk_base = "1.2.0"
cbsk_log = "1.2.0"
```

main.rs:

```rust
use cbsk_base::log;
use cbsk_log::config::Config;
use cbsk_log::filter::module_filter::ModuleFilter;
use cbsk_log::model::log_size::LogSize;
use cbsk_log::packer::zip_packer::ZipPacker;

pub fn main() {
    let conf = Config::default()
        .push_filter(ModuleFilter::default().push("test"))
        .file_split("/logs/", LogSize::MB(5), ZipPacker::default().pack_end(|pack_name| {
            println!("pack name is {pack_name}");
        }));

    cbsk_log::init(conf).unwrap();
    log::info!("hello world");

    // wait log flush
    log::logger().flush();
}
```

### console example

Cargo.toml:

```toml
cbsk_base = "1.2.0"
cbsk_log = "1.2.0"
```

main.rs:

```rust
use cbsk_base::log;
use cbsk_log::config::Config;
use cbsk_log::filter::module_filter::ModuleFilter;

pub fn main() {
    let conf = Config::default()
        .push_filter(ModuleFilter::default().push("test"))
        .console();

    cbsk_log::init(conf).unwrap();
    log::info!("hello world");

    // wait log flush
    log::logger().flush();
}
```