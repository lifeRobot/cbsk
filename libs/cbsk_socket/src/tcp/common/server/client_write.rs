use std::io;
use std::io::Write;
use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use cbsk_base::anyhow;
use cbsk_base::tokio::io::AsyncWriteExt;
use cbsk_base::tokio::net::tcp::OwnedWriteHalf;
use cbsk_mut_data::mut_data_obj::MutDataObj;

/// client write enum
pub enum ClientWrite {
    /// tokio_runtime tcp client write
    Tokio(OwnedWriteHalf),
    /// system tcp client write
    System(Arc<MutDataObj<TcpStream>>),
}

/// custom method
impl ClientWrite {
    /// try send byte to tcp server/client
    pub async fn try_send_bytes(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        match self {
            ClientWrite::Tokio(write) => {
                write.write_all(bytes).await?;
                write.flush().await?;
            }
            ClientWrite::System(write) => {
                let mut write = write.as_mut();
                write.write_all(bytes)?;
                write.flush()?;
            }
        }
        Ok(())
    }

    /// shutdown tcp
    pub async fn shutdown(&mut self) -> io::Result<()> {
        match self {
            ClientWrite::Tokio(write) => {
                write.shutdown().await
            }
            ClientWrite::System(write) => {
                write.shutdown(Shutdown::Both)
            }
        }
    }
}

/// support OwnedWriteHalf to client write
impl From<OwnedWriteHalf> for ClientWrite {
    fn from(value: OwnedWriteHalf) -> Self {
        Self::Tokio(value)
    }
}

/// support TcpStream to client write
impl From<Arc<MutDataObj<TcpStream>>> for ClientWrite {
    fn from(value: Arc<MutDataObj<TcpStream>>) -> Self {
        Self::System(value)
    }
}
