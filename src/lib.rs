#[allow(async_fn_in_trait)]
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;
pub mod business;
pub mod data;