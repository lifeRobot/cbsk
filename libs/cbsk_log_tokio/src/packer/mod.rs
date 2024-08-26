use std::sync::Arc;
use cbsk_base::async_trait::async_trait;
use cbsk_log::model::log_path::LogPath;

#[cfg(feature = "zip")]
pub mod zip_packer;

/// log packer
#[async_trait]
pub trait Packer: Sync + Send {
    /// do pack
    async fn pack(&self, split_name: String, split_path: String, log_path: Arc<LogPath>);
}
