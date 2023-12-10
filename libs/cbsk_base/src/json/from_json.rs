use serde::de::DeserializeOwned;
use serde_json::Value;

/// if data impl DeserializeOwned, they will auto support from_json
pub trait FromJson<T> {
    /// from json to data
    fn from_json(json: serde_json::Value) -> serde_json::Result<T>;

    /// from json bytes to data
    fn from_slice(data: &[u8]) -> serde_json::Result<T>;

    /// from json bytes to data
    fn from_vec(data: Vec<u8>) -> serde_json::Result<T>;

    /// from json string to data
    fn from_str(str: &str) -> serde_json::Result<T>;
}

/// any impl DeserializeOwned data support from_json
impl<T: DeserializeOwned> FromJson<T> for T {
    fn from_json(json: Value) -> serde_json::Result<T> {
        serde_json::from_value(json)
    }

    fn from_slice(data: &[u8]) -> serde_json::Result<T> {
        serde_json::from_slice(data)
    }

    fn from_vec(data: Vec<u8>) -> serde_json::Result<T> {
        Self::from_slice(data.as_slice())
    }

    fn from_str(str: &str) -> serde_json::Result<T> {
        serde_json::from_str(str)
    }
}
