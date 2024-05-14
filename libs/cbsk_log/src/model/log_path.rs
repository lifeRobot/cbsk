use std::path::PathBuf;

/// log path
pub struct LogPath {
    /// log name prefix
    pub name_prefix: String,
    /// log name suffix
    pub name_suffix: String,
    /// log path, absolute path
    pub path: PathBuf,
    /// log file path
    pub dir: String,
}

/// custom method
impl LogPath {
    /// create log path
    pub fn new(name_prefix: String, name_suffix: String, path: PathBuf, dir: String) -> Self {
        Self { name_prefix, name_suffix, path, dir }
    }
}
