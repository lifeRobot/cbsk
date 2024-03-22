cbsk_run is async pool tool  
the main functions include async pool and signal::run

### async pool example

Cargo.toml file :

```toml
cbsk_base = { version = "0.1.7" }
cbsk_run = { version = "0.1.10" }
```

main.rs file :

```rust
use std::time::Duration;
use cbsk_base::tokio;

#[tokio::main]
async fn main() {
    // println hello world
    cbsk_run::async_pool::push(async {
        loop {
            println!("hello world");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    // println hi
    cbsk_run::async_pool::push(async {
        loop {
            println!("hi!");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });

    // wait for async to end
    cbsk_run::async_pool::listener().await.unwrap();
}
```

### signal::run example

Cargo.toml file :

```toml
cbsk_base = { version = "0.1.7" }
cbsk_run = { version = "0.1.10" }
```

main.rs file :

```rust
use std::time::Duration;
use cbsk_base::{anyhow, tokio};
use cbsk_base::tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    cbsk_run::run::signal::run(runnable()).await
}

fn runnable() -> anyhow::Result<Vec<JoinHandle<()>>> {
    Ok(vec![hello_world(), say_hi()])
}

fn hello_world() -> JoinHandle<()> {
    tokio::spawn(async {
        loop {
            println!("hello world");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
}

fn say_hi() -> JoinHandle<()> {
    tokio::spawn(async {
        loop {
            println!("hi!");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    })
}
```

running results :

> E:\work\github\rust\cbsk_test>cargo run  
> Compiling cbsk_run v0.1.0 (E:\work\github\rust\cbsk\libs\cbsk_run)  
> Compiling cbsk_test v0.1.0 (E:\work\github\rust\cbsk_test)  
> Finished dev [unoptimized + debuginfo] target(s) in 4.21s  
> Running `E:\work\cache\rust\github\target\debug\cbsk_test.exe`  
> hello world  
> hi!  
> hello world  
> hello world  
> hi!  
> hello world  
> hello world  
> hi!  
> hello world  
> hello world  
> hi!  
> hello world  
> hello world  
> hi!  
> hello world  
> hello world  
> hi!  
> hello world  
> hi!  
> hello world  
> hello world  
> hello world  
> hi!  
> hello world
>
> E:\work\github\rust\cbsk_test>