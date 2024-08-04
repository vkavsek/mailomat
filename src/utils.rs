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

#[derive(Debug)]
pub enum Error {
    B64uDecode(String),
}
// Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
