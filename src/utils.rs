use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

pub fn b64u_encode(v: impl AsRef<[u8]>) -> String {
    URL_SAFE_NO_PAD.encode(v)
}

pub fn b64u_decode(v: &str) -> Result<Vec<u8>> {
    URL_SAFE_NO_PAD
        .decode(v)
        .map_err(|er| Error::B64uDecode(er.to_string()))
}

pub fn b64u_decode_to_string(v: &str) -> Result<String> {
    String::from_utf8(b64u_decode(v)?).map_err(|er| Error::B64uDecode(er.to_string()))
}

// ###################################
// ->   ERROR
// ###################################
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Base64-URL decoding error: {0}")]
    B64uDecode(String),
}
