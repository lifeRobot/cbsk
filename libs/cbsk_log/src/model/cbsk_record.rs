use cbsk_base::fastdate::DateTime;
use cbsk_base::log::{Level, Record};
use cbsk_mut_data::mut_data_obj::MutDataObj;

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
    short_module_path: MutDataObj<String>,
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
        Self {
            level: value.level(),
            target: value.target().into(),
            args: value.args().to_string(),
            module_path: value.module_path().unwrap_or_default().into(),
            short_module_path: MutDataObj::default(),
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
        if !self.short_module_path.is_empty() {
            return self.short_module_path.as_str();
        }

        let i = cbsk_base::match_some_return!(self.module_path.find("::"),{
            self.short_module_path.set(self.module_path.clone());
            self.short_module_path.as_str()
        });

        self.short_module_path.set(self.module_path[0..i].into());
        self.short_module_path.as_str()
    }
}
