pub mod verify_data;
pub mod analysis_data;

/// cbsk default header
pub fn default_header() -> Vec<u8> {
    vec![b'c', b'b', b's', b'k']
}