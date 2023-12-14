#[cfg(feature = "tokio")]
pub use tokio;
#[cfg(feature = "anyhow")]
pub use anyhow;
#[cfg(feature = "once_cell")]
pub use once_cell;
#[cfg(feature = "serde")]
pub use serde;
#[cfg(feature = "serde_json")]
pub use serde_json;
#[cfg(feature = "log")]
pub use log;
#[cfg(feature = "async-trait")]
pub use async_trait;
#[cfg(feature = "async-recursion")]
pub use async_recursion;

#[cfg(feature = "macro")]
pub mod r#macro;
#[cfg(feature = "serde")]
pub mod json;
#[cfg(feature = "result")]
pub mod result;

/// get the directory where the program is located
pub fn root_path() -> String {
    let mut root_path = String::new();

    // get default path
    if let Ok(exe) = std::env::current_exe() {
        if let Some(path) = exe.parent() {
            if let Some(path_str) = path.to_str() {
                root_path = path_str.to_string();
            }
        }
    }

    root_path
}
