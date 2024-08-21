#![allow(async_fn_in_trait)]

#[cfg(feature = "cbsk_socket")]
pub use cbsk_socket;
#[cfg(feature = "tokio-tungstenite")]
pub use tokio_tungstenite;
#[cfg(feature = "futures-util")]
pub use futures_util;

pub(crate) mod r#macro;
#[cfg(any(feature = "tcp_server", feature = "tcp_client"))]
pub mod tcp;
#[cfg(any(feature = "ws_server", feature = "ws_client"))]
pub mod ws;
