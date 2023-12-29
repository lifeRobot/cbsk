#![allow(async_fn_in_trait)]

pub mod config;
#[cfg(any(feature = "tcp_server", feature = "tcp_client"))]
pub mod tcp;