use cbsk_base::fastdate::DateTime;
use cbsk_base::log::{Level, Record};

/// cbsk log record
pub struct CbskRecord {
    /// log level
    pub level: Level,
    /// log target
    pub target: String,
    /// log args
    pub args: String,
    /// log module path
    pub module_path: String,
    /// short module path
    short_module_path: String,
    /// log file path
    pub file: String,
    /// log line
    pub line: Option<u32>,
    /// log date
    pub time: DateTime,
}

/// support [Record] into [CbskRecord]
impl From<&Record<'_>> for CbskRecord {
    fn from(value: &Record) -> Self {
        let module_path = value.module_path().unwrap_or_default();
        let short_module_path = Self::build_short_module_path(module_path);
        Self {
            level: value.level(),
            target: value.target().into(),
            args: value.args().to_string(),
            module_path: module_path.into(),
            short_module_path: short_module_path.into(),
            file: value.file().unwrap_or_default().into(),
            line: value.line(),
            time: DateTime::now(),
        }
    }
}

/// custom method
impl CbskRecord {
    /// get short_module_path
    pub fn short_module_path(&self) -> &str {
        self.short_module_path.as_str()
    }

    /// build short module path
    fn build_short_module_path(module_path: &str) -> &str {
        let i = cbsk_base::match_some_return!(module_path.find("::"),module_path);
        &module_path[0..i]
    }
}
