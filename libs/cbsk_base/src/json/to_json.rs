use serde::Serialize;

/// if data impl Serialize, they will auto support to_json
pub trait ToJson {
    /// support to_json
    fn to_json(&self) -> serde_json::Result<serde_json::Value>;
}

/// any impl Serialize data support to_json
impl<T: Serialize> ToJson for T {
    fn to_json(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::to_value(self)
    }
} 
