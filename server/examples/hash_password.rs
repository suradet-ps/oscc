//! Prints an Argon2id PHC hash for a password, for seeding the first
//! account (docs/runbook.md "First user").
//!
//! The password arrives on stdin, not argv: arguments leak into shell
//! history and process lists.
//!
//! Usage:
//!
//! ```text
//! printf '%s' 'the-password' | cargo run -p oscc-server --example hash_password
//! ```

use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut password = String::new();
    std::io::stdin().read_to_string(&mut password)?;
    let hash = oscc_server::auth::password::hash_password(password.trim_end_matches(['\r', '\n']))
        .map_err(|err| format!("hash failed: {err}"))?;
    println!("{hash}");
    Ok(())
}
