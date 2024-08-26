pub mod console_actuator;

/// log actuator
pub trait Actuator {
    /// execute the log actuator
    fn exec(&self, record: &str);
}