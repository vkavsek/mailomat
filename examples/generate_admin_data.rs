use std::fmt::Display;

use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    println!("\nADMIN USER SETUP UTILITY\n");
    println!("Please note: Any leading or trailing whitespace in the fields you enter will be automatically trimmed and is not allowed.\n");
    println!("Provide a username:\n");
    let mut username_buf = String::with_capacity(256);
    std::io::stdin().read_line(&mut username_buf)?;
    let username = username_buf.trim().to_string();

    println!("Provide a password:\n");
    let mut password_buf = String::with_capacity(256);
    std::io::stdin().read_line(&mut password_buf)?;
    let password = password_buf.trim().to_string();
    let hashed = mailomat::web::auth::password::hash_new_to_string(SecretString::from(password))?;

    let admin = Admin::new(Uuid::new_v4(), username, hashed.expose_secret().to_string());

    println!("{admin}");

    Ok(())
}

struct Admin {
    user_id: Uuid,
    username: String,
    password_hash: String,
}

impl Admin {
    fn new(user_id: Uuid, username: String, password_hash: String) -> Self {
        Self {
            user_id,
            username,
            password_hash,
        }
    }
}

impl Display for Admin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n\nADMIN:\n\n'{}',\n'{}',\n'{}'",
            self.user_id, self.username, self.password_hash
        )
    }
}
