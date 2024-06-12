cbsk_base is a locked version cargo crates  
you can use cbsk_base lock commonly used cargo crates versions  
cbsk_base also supports some custom trait, like ToJson,FromJson and some macro

#### now locked version

| name                                                        | git                                                 | version |  
|-------------------------------------------------------------|-----------------------------------------------------|---------|
| [tokio](https://crates.io/crates/tokio)                     | [github](https://github.com/tokio-rs/tokio)         | 1.38.0  |
| [anyhow](https://crates.io/crates/anyhow)                   | [github](https://github.com/dtolnay/anyhow)         | 1.0.86  |
| [once_cell](https://crates.io/crates/once_cell)             | [github](https://github.com/matklad/once_cell)      | 1.19.0  |
| [serde](https://crates.io/crates/serde)                     | [github](https://github.com/serde-rs/serde)         | 1.0.203 |
| [serde_json](https://crates.io/crates/serde_json)           | [github](https://github.com/serde-rs/json)          | 1.0.117 |
| [log](https://crates.io/crates/log)                         | [github](https://github.com/rust-lang/log)          | 0.4.21  |
| [async-trait](https://crates.io/crates/async-trait)         | [github](https://github.com/dtolnay/async-trait)    | 0.1.80  |
| [async-recursion](https://crates.io/crates/async-recursion) | [github](https://github.com/dcchut/async-recursion) | 1.1.1   |
| [parking_lot](https://crates.io/crates/parking_lot)         | [github](https://github.com/Amanieu/parking_lot)    | 0.12.3  |
| [fastdate](https://crates.io/crates/fastdate)               | [github](https://github.com/rbatis/fastdate)        | 0.3.28  |

### serde example

use serde_derive_json,   
the struct impl Serialize, will auto impl ToJson  
the struct impl Deserialize, will auto impl FromJson

Cargo.toml file :

```toml
cbsk_base = { version = "1.3.5", features = ["serde_derive_json"] }
```

main.rs file :

```rust
use cbsk_base::json::from_json::FromJson;
use cbsk_base::json::to_json::ToJson;
use cbsk_base::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(crate = "cbsk_base::serde")]
struct A {
    data: String,
}

fn main() {
    let a = A::default();
    let json = a.to_json();
    println!("a is {json:?}");// a is Ok(Object {"data": String("")})

    let a = A::from_json(json.unwrap());
    println!("a is {a:?}");// a is Ok(A { data: "" })
}
```

### option macro example

Cargo.toml file :

```toml
cbsk_base = { version = "1.3.5", features = ["macro", "anyhow"] }
```

main.rs file :

```rust
use cbsk_base::anyhow;

fn main() {
    let a = try_get_option();
    println!("a is {a:?}");// a is Ok("hello world")
    exec_option();
}

fn try_get_option() -> anyhow::Result<String> {
    let a = Some("hello world".to_string());
    // match Option if is Some,
    // will be return value if is Nome,
    // will be return custom value and exit method
    Ok(cbsk_base::match_some_return!(a,Err(anyhow::anyhow!("a is none"))))
}

fn exec_option() {
    let a = Some("hello world".to_string());
    // match Option if is Some,
    // will be return value if is Nome,
    // will be return custom value
    let a = cbsk_base::match_some_exec!(a,{
        // do something, you can exit method, or break String
        println!("a is None");// will not output, because a not None
        return;
    });
    println!("a is {a}");// a is hello world
}
```

### root_path example

Cargo.toml file:

```toml
cbsk_base = { version = "1.3.5", features = ["root_path"] }
```

main.rs file:

```rust
fn main() {
    // print the path where the program is located
    let root_path = cbsk_base::root_path();
    println!("root path is {root_path}");
}
```
