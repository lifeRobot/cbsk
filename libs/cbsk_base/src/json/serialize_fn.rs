#[cfg(feature = "serde_json")]
use serde::{Deserialize, Deserializer};
#[cfg(feature = "serde_json")]
use serde_json::Number;

/// default true
pub fn default_true() -> bool { true }

/// if number equal 1, return Ok(true), else return Ok(false)<br />
/// if value not number, will be return Err
#[cfg(feature = "serde_json")]
pub fn number_to_bool<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    let num = Number::deserialize(deserializer)?;
    if let Some(num) = num.as_i64() {
        return Ok(num == 1);
    }
    if let Some(num) = num.as_u64() {
        return Ok(num == 1);
    }
    // Note that there may be bugs in floating-point, so floating-point judgment is placed last
    if let Some(num) = num.as_f64() {
        return Ok(num == 1.0);
    }

    // default return false
    Ok(false)
}

/// if str equal 1, return Ok(true), else return Ok(false)<br />
/// if value not str, will be return Err
#[cfg(feature = "serde_json")]
pub fn str_to_bool<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(String::deserialize(deserializer)?.eq("1"))
}

/// if deserialize err, return Ok("".to_string()), else return Ok(String)
#[cfg(feature = "serde_json")]
pub fn str_or_default<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(String::deserialize(deserializer).unwrap_or_default())
}
