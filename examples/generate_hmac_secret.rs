use mailomat::utils::b64_encode;
use rand::{rng, RngCore};

fn main() {
    let mut buf = [0u8; 64];
    rng().fill_bytes(&mut buf);
    let buf_str = b64_encode(buf);
    println!("{buf_str}")
}
