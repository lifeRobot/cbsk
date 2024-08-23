use cbsk_base::log;
use cbsk_base::log::{Log, Metadata, Record};
use cbsk_log::config::Config;

mod runtime;

/// cbsk log
pub struct CbskLog {}

/// support log
impl Log for CbskLog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
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

    // start log timer
    runtime::LogRuntime::default().start(config);
    Ok(())
}
