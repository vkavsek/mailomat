use std::fmt::Display;

use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    println!("\nADMIN USER SETUP UTILITY\n");
    println!("For all the fields you are asked to provide note that leading and trailing whitespaces are FORBIDDEN and will be removed.\n");
    println!("provide a username:\n");
    let mut username_buf = String::with_capacity(256);
    std::io::stdin().read_line(&mut username_buf)?;
    let username = username_buf.trim().to_string();

    println!("provide a password:\n");
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
            "\n\nADMIN:\n\n'{}'\n'{}'\n'{}'",
            self.user_id, self.username, self.password_hash
        )
    }
}
