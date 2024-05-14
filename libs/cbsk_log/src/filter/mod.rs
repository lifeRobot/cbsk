use crate::model::cbsk_record::CbskRecord;

pub mod module_filter;

/// log filter
pub trait Filter {
    /// filter<br />
    /// will not execute the log actuator when return true<br />
    /// will execute log actuator when return false
    fn filter(&self, record: &CbskRecord) -> bool;
}
