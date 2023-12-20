cbsk_mut_data is a ref mut tool  
you can use cbsk_mut_data for static data modification, etc  
the idea for this cargo crate comes
from [dark-std](https://crates.io/crates/dark-std)([github](github.com/darkrpc/dark-std))  
this cargo crate may be unsafe, if you want safe data, you can
use [dark-std](https://crates.io/crates/dark-std)([github](github.com/darkrpc/dark-std))

#### data type

* MutDataObj
* MutDataVec
* MutDataHashMap  
  **more type welcome to submit issue**

#### Simple Example

Cargo.toml file :

```toml
cbsk_mut_data = "0.1.3"
```

main.rs file :

```rust
use cbsk_mut_data::mut_data_obj::MutDataObj;

fn main() {
    let b = MutDataObj::new(true);
    println!("b is {b}");// b is true
    b.set(false);
    println!("b is {b}");// b is false
    b.trigger();
    println!("b is {b}");// b is true
}
```

main.rs file (use struct) :

```rust
use cbsk_mut_data::mut_data_obj::MutDataObj;

#[derive(Default, Debug)]
struct A {
    data: MutDataObj<i32>,
}

fn main() {
    let a = A::default();
    println!("a is {a:?}");// a is A { data: 0 }
    a.data.set(10);
    println!("a is {a:?}");// a is A { data: 10 }
}

```

main.rs file (in struct)

```rust
use cbsk_mut_data::mut_data_obj::MutDataObj;

#[derive(Default, Debug)]
struct A {
    b: MutDataObj<B>,
}

#[derive(Default, Debug)]
struct B {
    data: i32,
}

fn main() {
    let a = A::default();
    println!("a is {a:?}");// a is A { b: B { data: 0 } }
    a.b.as_mut().data = 10;
    println!("a is {a:?}");// a is A { b: B { data: 10 } }
}
```

#### OnceCell Example

Cargo.toml file :

```toml
once_cell = "1.19.0"
cbsk_mut_data = "0.1.3"
```

main.rs file :

```rust
use cbsk_mut_data::mut_data_obj::MutDataObj;
use once_cell::sync::Lazy;

pub static B: Lazy<MutDataObj<bool>> = Lazy::new(MutDataObj::default);

fn main() {
    println!("b is {}", B.as_ref());// b is false
    B.set(true);
    println!("b is {}", B.as_ref());// b is true
    B.trigger();
    println!("b is {}", B.as_ref());// b is false
}
```

#### Unsafe Example

Cargo.toml file :

```toml
cbsk_mut_data = "0.1.3"
```

main.rs file :

```rust
use std::sync::Arc;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use cbsk_mut_data::mut_data_ref::MutDataRef;

fn main() {
    // unsafe example
    let mut a = MutDataRef::new(&mut 0);
    println!("a is {a:?}");// a is 0
    let b = a.clone();
    *a = 10;
    println!("b is {b:?}");// b is 10

    // mut arc
    // mut arc may have many scenarios and can be very useful, but it is not recommended for you to use it this way
    let c = Arc::new(MutDataObj::new(0));
    println!("c is {c:?}");// c is 0
    let d = c.clone();
    c.set(10);
    println!("d is {d:?}");// d is 10
}
```

#### Thanks

* [dark-std](https://crates.io/crates/dark-std)([github](github.com/darkrpc/dark-std))
