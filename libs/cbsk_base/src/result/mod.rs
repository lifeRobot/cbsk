pub mod base_result;

/// build result function
#[macro_export]
#[cfg(feature = "macro")]
macro_rules! build_result {
    ($result:tt)=>{
        /// build base function
        impl<T: Serialize> $result<T> {
            /// build only code and data result
            pub fn build_data(code: i64, data: T) -> Self {
                Self { code, msg: None, data: Some(data) }
            }

            /// build custom result
            pub fn build_msg_data(code: i64, msg: impl Into<String>, data: T) -> Self{
                Self { code, msg: Some(msg.into()), data: Some(data) }
            }

            /// build success data result
            pub fn ok_data(data: T) -> Self {
                Self::build_msg_data(0, "Success", data)
            }

            /// build success msg and data result
            pub fn ok_msg_data(msg: impl Into<String>, data: T) -> Self {
                Self::build_msg_data(0, msg, data)
            }

            /// build fail data result
            pub fn fail_data(data: T) -> Self {
                Self::fail_msg_data("Fail", data)
            }

            /// build fail msg and data result
            pub fn fail_msg_data(msg: impl Into<String>, data: T) -> Self {
                Self::build_msg_data(1, msg, data)
            }
        }

        /// build base function to any type
        impl<T> $result<T> {
            /// build only code and msg result
            pub fn build_msg(code: i64, msg: impl Into<String>) -> Self{
                Self { code, msg: Some(msg.into()), data: None }
            }

            /// build success result
            pub fn ok() -> Self {
                Self::ok_msg("Success")
            }

            /// build success msg result
            pub fn ok_msg(msg: impl Into<String>) -> Self {
                Self::build_msg(0, msg)
            }

            /// build fail result
            pub fn fail() -> Self {
                Self::fail_msg("Fail")
            }

            /// build fail msg result
            pub fn fail_msg(msg: impl Into<String>) -> Self {
                Self::build_msg(1, msg)
            }

            /// is the result successful
            pub fn is_ok(&self) -> bool {
                self.code == 0
            }
        }
    }
}