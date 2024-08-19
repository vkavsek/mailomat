use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
// ###################################
// ->   Base64 utils
// ###################################
/// Encode to Base64 String
pub fn b64_encode(v: impl AsRef<[u8]>) -> String {
    STANDARD.encode(v)
}

/// Base64 decode
pub fn b64_decode(v: &str) -> Result<Vec<u8>> {
    STANDARD
        .decode(v)
        .map_err(|er| B64DecodeError::B64Decode(er.to_string()))
}

/// Base64 decode to String
pub fn b64_decode_to_string(v: &str) -> Result<String> {
    String::from_utf8(b64_decode(v)?).map_err(|er| B64DecodeError::B64Decode(er.to_string()))
}

/// Encode to Base64-URL String with no padding for use in URLs
pub fn b64u_encode(v: impl AsRef<[u8]>) -> String {
    URL_SAFE_NO_PAD.encode(v)
}

/// Base64-URL decode
pub fn b64u_decode(v: &str) -> Result<Vec<u8>> {
    URL_SAFE_NO_PAD
        .decode(v)
        .map_err(|er| B64DecodeError::B64uDecode(er.to_string()))
}

// ###################################
// ->   ERROR
// ###################################
pub type Result<T> = core::result::Result<T, B64DecodeError>;

#[derive(Debug, thiserror::Error)]
pub enum B64DecodeError {
    #[error("base64 decoding error: {0}")]
    B64Decode(String),
    #[error("base64-url decoding error: {0}")]
    B64uDecode(String),
}
