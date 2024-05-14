use cbsk_base::convert::datetime::DateTimeSerialize;
use cbsk_base::log::Level;
use crate::model::cbsk_record::CbskRecord;

/// default log format
#[derive(Default)]
pub struct DefaultFormat {}

/// support log format
impl super::LogFormat for DefaultFormat {
    fn format(&self, record: &CbskRecord) -> String {
        let path =
            match record.level {
                Level::Debug | Level::Info | Level::Warn => { record.short_module_path().into() }
                _ => { format!("{}:{}", record.file, record.line.unwrap_or_default()) }
            };
        format!("{} [{}] [{path}] {}\n", record.time.yyyy_mm_dd_hh_mm_ss_n(), record.level, record.args)
    }
}
