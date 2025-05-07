use mailomat::utils::b64_encode;
use rand::Fill;
use secrecy::SecretString;

fn main() -> anyhow::Result<()> {
    let mut bytes = [0u8; 64];
    let mut rng = rand::rng();
    bytes.fill(&mut rng);
    let pass = b64_encode(bytes);

    println!("The password:\n\t'{}'", pass);
    let hashed = mailomat::web::auth::password::hash_new_to_string(SecretString::from(pass))?;
    println!("was hashed into:\n\t'{hashed}'");

    Ok(())
}
