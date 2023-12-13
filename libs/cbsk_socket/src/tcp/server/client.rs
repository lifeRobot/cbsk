use std::net::SocketAddr;
use std::sync::Arc;
use cbsk_base::tokio::net::tcp::OwnedWriteHalf;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::tcp::server::config::TcpServerConfig;
use crate::tcp::write_trait::WriteTrait;

/// tcp client
pub struct TcpServerClient {
    /// tcp client addr
    pub addr: SocketAddr,
    /// internal log name
    pub log_head: String,
    /// tcp client write
    writer: Arc<MutDataObj<OwnedWriteHalf>>,
}

/// custom method
impl TcpServerClient {
    /// create tcp server client
    pub(crate) fn new(addr: SocketAddr, conf: &TcpServerConfig, writer: OwnedWriteHalf) -> Self {
        let log_head = format!("{}TCP客户端[{}]", conf.name, addr);
        Self { addr, log_head, writer: MutDataObj::new(writer).into() }
    }
}

/// support writer trait
impl WriteTrait for TcpServerClient {
    fn try_get_write(&self) -> cbsk_base::anyhow::Result<&MutDataObj<OwnedWriteHalf>> {
        Ok(self.writer.as_ref())
    }

    fn get_log_head(&self) -> &str {
        self.log_head.as_str()
    }
}
