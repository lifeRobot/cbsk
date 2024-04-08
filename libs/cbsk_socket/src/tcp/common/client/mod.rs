#[cfg(all(feature = "tcp_runtime_tokio", not(feature = "tcp_runtime_thread")))]
pub use r#async::*;

pub mod config;
#[cfg(feature = "tcp_runtime_tokio")]
pub mod r#async;
#[cfg(any(feature = "tcp_runtime_thread", feature = "tcp_runtime_rayon"))]
pub mod sync;
