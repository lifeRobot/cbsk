/// verify result data
#[derive(Debug)]
pub struct VerifyData {
    /// verify fail frame
    pub error_frame: Vec<u8>,
    /// verify success frame
    pub data_frame: Vec<u8>,
    /// verify first frame success, but the remaining bytes are too short<br />
    /// the bytes with insufficient length are stored here
    pub too_short_frame: Vec<u8>,
    /// loop verification failed, data that needs to be verified next time
    pub next_verify_frame: Vec<u8>,
}

/// custom method
impl VerifyData {
    /// just verify fail
    pub fn fail(error_frame: Vec<u8>) -> Self {
        Self::new(error_frame, Vec::new())
    }

    /// just verify success
    pub fn success(data_frame: Vec<u8>) -> Self {
        Self::new(Vec::new(), data_frame)
    }

    /// just too short
    pub fn too_short(error_frame: Vec<u8>, too_short_frame: Vec<u8>) -> Self {
        Self::new_with_too_short_frame(error_frame, Vec::new(), too_short_frame)
    }

    /// just next next verify
    pub fn next_verify(error_frame: Vec<u8>, next_verify_frame: Vec<u8>) -> Self {
        Self { error_frame, data_frame: Vec::new(), too_short_frame: Vec::new(), next_verify_frame }
    }

    /// new verify data
    pub fn new(error_frame: Vec<u8>, data_frame: Vec<u8>) -> Self {
        Self::new_with_too_short_frame(error_frame, data_frame, Vec::new())
    }

    pub fn new_with_too_short_frame(error_frame: Vec<u8>, data_frame: Vec<u8>, too_short_frame: Vec<u8>) -> Self {
        Self { error_frame, data_frame, too_short_frame, next_verify_frame: Vec::new() }
    }
}
