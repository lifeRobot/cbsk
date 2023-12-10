use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BaseResult<T = ()> {
    /// custom code, default is 0(success)
    pub code: i64,
    /// result msg
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    /// result data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

crate::build_result!(BaseResult);
