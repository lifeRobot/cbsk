pub mod console_actuator;
pub mod file_split_actuator;

/// log actuator
pub trait Actuator {
    /// execute the log actuator
    fn exec(&self, record: &str);
}