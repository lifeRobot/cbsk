/// analysis result data
pub struct AnalysisData {
    /// analysis success frame
    pub data_frame: Vec<u8>,
    /// analysis data too short frame
    pub too_short_frame: Vec<u8>,
    /// the length of the data description exceeds the limit length
    pub too_long_byte: Option<u8>,
    /// data that needs to be verified next time
    pub next_verify_frame: Vec<u8>,
}

/// custom method
impl AnalysisData {
    /// just analysis success
    pub fn success(data_frame: Vec<u8>, next_verify_frame: Vec<u8>) -> Self {
        Self::new(data_frame, Vec::new(), None, next_verify_frame)
    }

    /// just too short<br />
    /// if data too short, will be add header data
    pub fn too_short(too_short_frame: Vec<u8>) -> Self {
        Self::new(Vec::new(), too_short_frame, None, Vec::new())
    }

    /// just too long
    pub fn too_long(too_long_byte: u8, next_verify_frame: Vec<u8>) -> Self {
        Self::new(Vec::new(), Vec::new(), Some(too_long_byte), next_verify_frame)
    }

    /// just next verify
    pub fn next_verify(next_verify_frame: Vec<u8>) -> Self {
        Self::success(Vec::new(), next_verify_frame)
    }

    /// new analysis data<br />
    /// if too short frame is not empty, will be add header data
    pub fn new(data_frame: Vec<u8>, mut too_short_frame: Vec<u8>, too_long_byte: Option<u8>, next_verify_frame: Vec<u8>) -> Self {
        if !too_short_frame.is_empty() {
            let mut heander = super::default_header();
            heander.append(&mut too_short_frame);
            too_short_frame = heander;
        }

        Self { data_frame, too_short_frame, too_long_byte, next_verify_frame }
    }
}
