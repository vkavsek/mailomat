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
        .map_err(|er| UtilsError::B64Decode(er.to_string()))
}

/// Base64 decode to String
pub fn b64_decode_to_string(v: &str) -> Result<String> {
    String::from_utf8(b64_decode(v)?).map_err(|er| UtilsError::B64Decode(er.to_string()))
}

/// Encode to Base64-URL String with no padding for use in URLs
pub fn b64u_encode(v: impl AsRef<[u8]>) -> String {
    URL_SAFE_NO_PAD.encode(v)
}

/// Base64-URL decode
pub fn b64u_decode(v: &str) -> Result<Vec<u8>> {
    URL_SAFE_NO_PAD
        .decode(v)
        .map_err(|er| UtilsError::B64uDecode(er.to_string()))
}

/// Base64-URL decode to String
pub fn b64u_decode_to_string(v: &str) -> Result<String> {
    String::from_utf8(b64u_decode(v)?).map_err(|er| UtilsError::B64uDecode(er.to_string()))
}
// ###################################
// ->   Hex encode / decode
// ###################################
/// encodes to padded lowercase hexadecimal representation
pub fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    let bytes = bytes.as_ref();
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut hex = String::with_capacity(bytes.len() * 2);

    for &byte in bytes {
        // encode the top 4 bits
        let high = HEX_CHARS[(byte >> 4) as usize] as char;
        // encode the bottom 4 bits
        let low = HEX_CHARS[(byte & 0x0F) as usize] as char;
        hex.extend([high, low]);
    }

    hex
}

/// decodes from padded hexadecimal representation
pub fn hex_decode(hex: impl AsRef<str>) -> Result<Vec<u8>> {
    let hex = hex.as_ref();
    if hex.len() % 2 == 1 {
        return Err(UtilsError::HexDecode("invalid input length".to_string()));
    }

    let mut res = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let n = u8::from_str_radix(&hex[i..i + 2], 16)
            .map_err(|e| UtilsError::HexDecode(e.to_string()))?;
        res.push(n);
    }
    assert_eq!(hex.len() / 2, res.len());

    Ok(res)
}

// ###################################
// ->   ERROR
// ###################################
pub type Result<T> = core::result::Result<T, UtilsError>;

#[derive(Debug, thiserror::Error)]
pub enum UtilsError {
    #[error("base64 decoding error: {0}")]
    B64Decode(String),
    #[error("base64-url decoding error: {0}")]
    B64uDecode(String),
    #[error("hex decoding error: {0}")]
    HexDecode(String),
}

#[cfg(test)]
mod test {
    use super::*;

    use derive_more::Deref;
    use quickcheck_macros::quickcheck;
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use std::ops::Deref;

    #[derive(Clone, Debug, Deref)]
    struct Bytes(Vec<u8>);

    impl quickcheck::Arbitrary for Bytes {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let bytes = (0..u8::arbitrary(g))
                .map(|_| rng.random())
                .collect::<Vec<_>>();
            Self(bytes)
        }
    }

    #[quickcheck]
    fn hex_encode_fuzz_works(bytes: Bytes) {
        let fixture = bytes.deref();
        let expected_hex = fixture
            .iter()
            .map(|&c| format!("{:02x}", c))
            .collect::<String>();

        let hex = hex_encode(fixture);

        assert_eq!(expected_hex, hex);
    }

    #[quickcheck]
    fn hex_decode_fuzz_works(bytes: Bytes) -> anyhow::Result<()> {
        let fixture = bytes.deref();
        let hex = hex_encode(fixture);
        let expected_hex = fixture
            .iter()
            .map(|&c| format!("{:02x}", c))
            .collect::<String>();

        let decoded = hex_decode(&hex)?;
        let expected_decoded = hex_decode(&expected_hex)?;
        assert_eq!(expected_decoded, decoded);
        assert_eq!(*fixture, decoded);

        Ok(())
    }

    #[test]
    fn test_hex_encode_basic() {
        let input = b"hello";
        let expected = "68656c6c6f";
        assert_eq!(hex_encode(input), expected);
    }

    #[test]
    fn test_hex_encode_empty() {
        let input: &[u8] = b"";
        let expected = "";
        assert_eq!(hex_encode(input), expected);
    }

    #[test]
    fn test_hex_decode_basic() {
        let input = "68656c6c6f";
        let expected = b"hello".to_vec();
        assert_eq!(hex_decode(input).unwrap(), expected);
    }

    #[test]
    fn test_hex_decode_uppercase() {
        let input = "48656C6C6F";
        let expected = b"Hello".to_vec();
        assert_eq!(hex_decode(input).unwrap(), expected);
    }

    #[test]
    fn test_hex_decode_empty() {
        let input = "";
        let expected: Vec<u8> = vec![];
        assert_eq!(hex_decode(input).unwrap(), expected);
    }

    #[test]
    fn test_hex_decode_invalid_length() {
        let input = "abc"; // Odd number of characters
        assert!(hex_decode(input).is_err());
    }

    #[test]
    fn test_hex_decode_invalid_chars() {
        let input = "zzzz";
        assert!(hex_decode(input).is_err());
    }
}
