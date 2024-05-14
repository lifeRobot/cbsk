/// console actuator
pub struct ConsoleActuator {}

/// support actuator
impl super::Actuator for ConsoleActuator {
    fn exec(&self, record: &str) {
        print!("{record}");
    }
}
