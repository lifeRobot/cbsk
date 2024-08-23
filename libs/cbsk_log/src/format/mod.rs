use crate::model::cbsk_record::CbskRecord;

pub mod default_format;

/// log format
pub trait LogFormat: Send + Sync {
    /// format
    fn format(&self, record: &CbskRecord) -> String;
}