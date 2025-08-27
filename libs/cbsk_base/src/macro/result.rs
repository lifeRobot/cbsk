/// match Result <br />
/// if is Ok, will be return value <br />
/// if is Err, will be return custom value and exit method
#[macro_export]
macro_rules! match_result_return {
    ($result:expr) => {
        $crate::match_some_return!($result,())
    };
    ($result:expr,$r:expr) => {
        $crate::match_result_exec!($result,return $r)
    };
}

/// match Result <br />
/// if is Ok, will be return value and keep loop <br />
/// if is Err, will be continue the loop
#[macro_export]
macro_rules! match_result_continue {
    ($result:expr) => {
        $crate::match_result_exec!($result,continue)
    };
}

/// match Result <br />
/// if is Ok, will be return value and keep loop <br />
/// if is Err, will be return custom value and break loop<br />
/// need to be used in loop to break value, invalid in for and while
#[macro_export]
macro_rules! match_result_break {
    ($result:expr) => {
        $crate::match_result_break!($result,())
    };
    ($result:expr,$r:expr) => {
        $crate::match_result_exec!($result,break $r)
    };
}

/// match Result <br />
/// if is Ok, will be return value <br />
/// if is Err, will be return custom value
#[macro_export]
macro_rules! match_result_exec {
    ($result:expr,$exec:expr) => {
        match $result {
            Ok(s) => s,
            Err(_e) => $exec,
        }
    };
}
