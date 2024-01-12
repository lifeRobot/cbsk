use std::net::{IpAddr, SocketAddr};
use cbsk_base::serde::Deserialize;

#[cfg(any(feature = "tcp_client", feature = "ws_client"))]
pub mod re_conn;

/// socket config
#[derive(Deserialize, Default)]
#[serde(crate = "cbsk_base::serde")]
pub struct SocketConfig {
    /// ip
    #[serde(default)]
    pub ip: [u8; 4],
    /// port
    #[serde(default)]
    pub port: u16,
}

/// custom method
impl SocketConfig {
    /// Converts the given value to a SocketAddr.
    pub fn to_addr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::from(self.ip), self.port)
    }
}
