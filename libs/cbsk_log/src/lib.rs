use cbsk_base::log;
use cbsk_base::log::{Log, Metadata, Record};
use crate::config::Config;

pub mod model;
pub mod config;
pub mod actuator;
pub mod format;
pub mod packer;
pub mod filter;
mod runtime;

/// cbsk log
pub struct CbskLog {}

/// support log
impl Log for CbskLog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= runtime::log_conf.level
    }

    fn log(&self, record: &Record) {
        runtime::cache::push(record.into());
    }

    fn flush(&self) {
        runtime::LogRuntime::wait_flush();
    }
}

/// init cbsk log
pub fn init(config: Config) -> Result<(), log::SetLoggerError> {
    // set log logger
    log::set_logger(&CbskLog {})?;
    log::set_max_level(config.level);

    // set globol config and start log timer
    runtime::log_conf.set(config);
    runtime::LogRuntime::default().start();
    Ok(())
}

