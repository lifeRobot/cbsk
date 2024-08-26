use std::io;
use cbsk_log::config::Config;
use cbsk_log::model::log_size::LogSize;
use crate::actuator::file_split_actuator::FileSplitActuator;
use crate::packer::Packer;

/// file split trait
pub trait FileSplitTrait: Sized {
    /// try add output logs in file split
    fn try_file_split(self, log_dir: impl Into<String>, log_size: LogSize, packer: impl Packer + 'static) -> io::Result<Self>;

    /// output logs in file split<br />
    /// ## Panic
    /// if create log file fail, will panic
    fn file_split(self, log_dir: impl Into<String>, log_size: LogSize, packer: impl Packer + 'static) -> Self {
        self.try_file_split(log_dir, log_size, packer).expect("create file split fail")
    }
}

/// let config support file split trait
impl FileSplitTrait for Config {
    fn try_file_split(self, log_dir: impl Into<String>, log_size: LogSize, packer: impl Packer + 'static) -> io::Result<Self> {
        self.actuators.push(Box::new(FileSplitActuator::new(log_dir, log_size, packer)?));
        Ok(self)
    }
}
