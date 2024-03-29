pub use super::common::tcp_write_trait;

#[cfg(feature = "tcp_server")]
pub mod server;
#[cfg(feature = "tcp_client")]
pub mod client;
pub(crate) mod tokio_tcp_read_trait;