#![allow(async_fn_in_trait)]

pub(crate) mod r#macro;
#[cfg(any(feature = "tcp_server", feature = "tcp_client"))]
pub mod tcp;
#[cfg(any(feature = "ws_server", feature = "ws_client"))]
pub mod ws;
