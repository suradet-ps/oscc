//! Provisions the encrypted HOSxP credentials file: every field is
//! AES-256-GCM encrypted with `encryptman`, and the master key lives in the
//! OS keychain (`encryptman-keyring`, service `oscc`).
//!
//! Usage (password arrives on stdin, never argv):
//!
//! ```text
//! cargo run -p oscc-server --example set_hosxp_config -- <host> <port> <database> <user>
//! ```
//!
//! Writes to `$OSCC_HOSXP_CONFIG`, or `hosxp.config.json` in the working
//! directory when the variable is unset.

use std::io::Read;
use std::path::PathBuf;

use oscc_hosxp_connector::config::{self, CONFIG_FILE_NAME, HosxConfig, load_vault};
use secrecy::SecretString;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [host, port, database, user] = args.as_slice() else {
        return Err(
            "usage: set_hosxp_config <host> <port> <database> <user> (password on stdin)".into(),
        );
    };
    let port: u16 = port.parse()?;

    let mut password = String::new();
    std::io::stdin().read_to_string(&mut password)?;
    let password = password.trim_end_matches(['\r', '\n']).to_string();
    if password.is_empty() {
        return Err("password must not be empty".into());
    }

    let path = std::env::var("OSCC_HOSXP_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(CONFIG_FILE_NAME));

    let store = load_vault()?;
    let cfg = HosxConfig::new(
        host.clone(),
        port,
        database.clone(),
        user.clone(),
        SecretString::from(password),
    );
    config::save_encrypted(&path, &store, &cfg).await?;

    println!("wrote encrypted HOSxP credentials to {}", path.display());
    println!("host={host} port={port} database={database} user={user} password=***");
    Ok(())
}
