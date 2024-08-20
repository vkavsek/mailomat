use std::io::stdin;

use mailomat::utils::b64u_encode;

fn main() -> anyhow::Result<()> {
    println!("Input string:");
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    let res = b64u_encode(trimmed);
    println!("ENCODED:\n\t{res}");

    Ok(())
}
