use std::sync::Arc;
use crate::model::log_path::LogPath;

pub mod zip_packer;

/// log packer
pub trait Packer: Sync + Send {
    /// do pack
    fn pack(&self, split_name: String, split_path: String, log_path: Arc<LogPath>);
}