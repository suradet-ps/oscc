//! Configuration for the HOSxP connector.
//!
//! Two ways in: `OSCC_HOSXP_*` environment variables (CI and quick dev), or
//! an encrypted on-disk file — every field AES-256-GCM encrypted via
//! `encryptman`, with the master key held in the OS keychain via
//! `encryptman-keyring` (the AllerX pattern, service `oscc`). Plaintext
//! values exist only in memory; a database dump of the config file is
//! useless without the keychain.

use std::fmt;
use std::path::Path;

use encryptman::{MasterKey, decrypt, encrypt};
use secrecy::{ExposeSecret, SecretBox, SecretString};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Keychain service name — used as both the keyring identifier and the
/// HKDF context (domain isolation for OSCC).
pub const SERVICE_NAME: &str = "oscc";

/// Default file name of the encrypted settings file.
pub const CONFIG_FILE_NAME: &str = "hosxp.config.json";

/// Version of the on-disk config format. Bump (with migration) if the
/// layout changes.
const CONFIG_VERSION: u32 = 1;

/// Why the connector configuration is unusable.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    /// A required environment variable is absent or empty.
    #[error("missing required environment variable {0}")]
    Missing(&'static str),
    /// The port variable is not a valid port number.
    #[error("invalid TCP port for {key}: {value}")]
    InvalidPort {
        /// The variable name.
        key: &'static str,
        /// The offending value.
        value: String,
    },
    /// The config file is missing, unreadable, or not writable.
    #[error("config file I/O failed: {0}")]
    Io(String),
    /// The config file is not valid JSON, or uses an unsupported version.
    #[error("config file is corrupt: {0}")]
    Corrupt(String),
    /// Decryption failed (wrong key, tampered file).
    #[error("decryption of connection settings failed: {0}")]
    Decrypt(String),
    /// Encryption failed.
    #[error("encryption of connection settings failed: {0}")]
    Encrypt(String),
    /// The OS keychain rejected the operation.
    #[error("OS keychain access failed: {0}")]
    Keyring(String),
}

/// Encrypts/decrypts strings on behalf of [`HosxConfig`].
///
/// Implemented by [`VaultKeyStore`] in production (OS keychain) and by
/// [`MasterKeyStore`] in tests (in-memory key, no keychain needed).
pub trait KeyStore: Send + Sync {
    /// Encrypts `plaintext` for storage.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Encrypt`] when the backing crypto fails.
    fn encrypt(&self, plaintext: &str) -> Result<String, ConfigError>;
    /// Decrypts `ciphertext` back to plaintext.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Decrypt`] when the key does not match or the
    /// data is tampered with.
    fn decrypt(&self, ciphertext: &str) -> Result<String, ConfigError>;
}

/// Production [`KeyStore`] backed by the OS keychain (Windows Credential
/// Manager, macOS Keychain, Linux Secret Service).
#[derive(Debug)]
pub struct VaultKeyStore(pub encryptman_keyring::Vault);

impl KeyStore for VaultKeyStore {
    fn encrypt(&self, plaintext: &str) -> Result<String, ConfigError> {
        self.0
            .encrypt(plaintext)
            .map_err(|e| ConfigError::Encrypt(e.to_string()))
    }

    fn decrypt(&self, ciphertext: &str) -> Result<String, ConfigError> {
        self.0
            .decrypt(ciphertext)
            .map_err(|e| ConfigError::Decrypt(e.to_string()))
    }
}

/// Loads (creating on first use) the keychain-backed vault for OSCC.
///
/// # Errors
///
/// Returns [`ConfigError::Keyring`] when the OS credential store is
/// unavailable (e.g. headless environments, or a Windows service running
/// under an account without a credential store).
pub fn load_vault() -> Result<VaultKeyStore, ConfigError> {
    encryptman_keyring::Vault::new(SERVICE_NAME)
        .map(VaultKeyStore)
        .map_err(|e| ConfigError::Keyring(e.to_string()))
}

/// Test [`KeyStore`] from an in-memory [`MasterKey`] — no OS keychain
/// involved. Production code should use [`load_vault`].
#[derive(Debug)]
pub struct MasterKeyStore(pub MasterKey);

impl KeyStore for MasterKeyStore {
    fn encrypt(&self, plaintext: &str) -> Result<String, ConfigError> {
        encrypt(&self.0, plaintext).map_err(|e| ConfigError::Encrypt(e.to_string()))
    }

    fn decrypt(&self, ciphertext: &str) -> Result<String, ConfigError> {
        decrypt(&self.0, ciphertext).map_err(|e| ConfigError::Decrypt(e.to_string()))
    }
}

/// HOSxP connection settings. Plaintext in memory **only** — never
/// serialized to disk in this form.
///
/// The password is a [`SecretString`]: its heap buffer is zeroized on drop,
/// and [`Debug`] renders it redacted. One documented residual remains:
/// sqlx's pool keeps its own plaintext copy for reconnects, for the
/// lifetime of the pool.
#[derive(Clone)]
pub struct HosxConfig {
    /// Database host (LAN address of the HOSxP server).
    pub host: String,
    /// Database port; HOSxP conventionally uses 3306.
    pub port: u16,
    /// Database (schema) name.
    pub database: String,
    /// Dedicated read-only user.
    pub user: String,
    /// That user's password.
    pub password: SecretString,
}

impl HosxConfig {
    /// Creates a new in-memory settings value.
    pub fn new(
        host: String,
        port: u16,
        database: String,
        user: String,
        password: SecretString,
    ) -> Self {
        Self {
            host,
            port,
            database,
            user,
            password,
        }
    }

    /// Reads the configuration from the process environment.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when a required variable is missing or the
    /// port is not a number.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_vars(|key| std::env::var(key).ok())
    }

    /// Reads the configuration through an injectable getter (testable).
    ///
    /// Required: `OSCC_HOSXP_HOST`, `OSCC_HOSXP_DATABASE`, `OSCC_HOSXP_USER`,
    /// `OSCC_HOSXP_PASSWORD`. Optional: `OSCC_HOSXP_PORT` (default 3306).
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when a required variable is missing or the
    /// port is not a number.
    pub fn from_vars(get: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let required = |key: &'static str| -> Result<String, ConfigError> {
            get(key)
                .filter(|value| !value.trim().is_empty())
                .ok_or(ConfigError::Missing(key))
        };

        let host = required("OSCC_HOSXP_HOST")?;
        let database = required("OSCC_HOSXP_DATABASE")?;
        let user = required("OSCC_HOSXP_USER")?;
        let password = required("OSCC_HOSXP_PASSWORD")?;
        let port = match get("OSCC_HOSXP_PORT").filter(|value| !value.trim().is_empty()) {
            Some(raw) => raw.parse::<u16>().map_err(|_| ConfigError::InvalidPort {
                key: "OSCC_HOSXP_PORT",
                value: raw,
            })?,
            None => 3306,
        };

        Ok(Self {
            host,
            port,
            database,
            user,
            password: SecretString::from(password),
        })
    }
}

impl PartialEq for HosxConfig {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host
            && self.port == other.port
            && self.database == other.database
            && self.user == other.user
            && self.password.expose_secret() == other.password.expose_secret()
    }
}

impl fmt::Debug for HosxConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HosxConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .field("user", &self.user)
            .field("password", &"***")
            .finish()
    }
}

/// On-disk layout: every field holds an encrypted value.
#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    version: u32,
    hosx: EncryptedHosxFields,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedHosxFields {
    host: String,
    port: String,
    database: String,
    user: String,
    password: String,
}

/// Encrypts every field of `cfg` and writes it to `config_path`, creating
/// parent directories as needed.
///
/// # Errors
///
/// Returns [`ConfigError::Encrypt`] on crypto failures and
/// [`ConfigError::Io`] when the file cannot be written.
pub async fn save_encrypted(
    config_path: &Path,
    store: &dyn KeyStore,
    cfg: &HosxConfig,
) -> Result<(), ConfigError> {
    let file = ConfigFile {
        version: CONFIG_VERSION,
        hosx: EncryptedHosxFields {
            host: store.encrypt(&cfg.host)?,
            port: store.encrypt(&cfg.port.to_string())?,
            database: store.encrypt(&cfg.database)?,
            user: store.encrypt(&cfg.user)?,
            password: store.encrypt(cfg.password.expose_secret())?,
        },
    };
    let json =
        serde_json::to_string_pretty(&file).map_err(|e| ConfigError::Corrupt(e.to_string()))?;
    if let Some(parent) = config_path.parent()
        && !parent.as_os_str().is_empty()
    {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| ConfigError::Io(e.to_string()))?;
    }
    tokio::fs::write(config_path, json)
        .await
        .map_err(|e| ConfigError::Io(e.to_string()))
}

/// Loads and decrypts the settings file.
///
/// Returns `Ok(None)` when the file does not exist yet (not configured).
///
/// # Errors
///
/// Returns [`ConfigError::Io`] for read failures, [`ConfigError::Corrupt`]
/// for invalid JSON or an unsupported version, and [`ConfigError::Decrypt`]
/// when the key does not match the stored data.
pub async fn load(
    config_path: &Path,
    store: &dyn KeyStore,
) -> Result<Option<HosxConfig>, ConfigError> {
    if !config_path.exists() {
        return Ok(None);
    }
    let raw = tokio::fs::read_to_string(config_path)
        .await
        .map_err(|e| ConfigError::Io(e.to_string()))?;
    let file: ConfigFile =
        serde_json::from_str(&raw).map_err(|e| ConfigError::Corrupt(e.to_string()))?;
    if file.version != CONFIG_VERSION {
        return Err(ConfigError::Corrupt(format!(
            "unsupported config version {}",
            file.version
        )));
    }
    let fields = file.hosx;
    let port = store
        .decrypt(&fields.port)?
        .parse()
        .map_err(|_| ConfigError::Decrypt("stored port is not a number".into()))?;
    Ok(Some(HosxConfig {
        host: store.decrypt(&fields.host)?,
        port,
        database: store.decrypt(&fields.database)?,
        user: store.decrypt(&fields.user)?,
        password: SecretBox::from(store.decrypt(&fields.password)?),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use encryptman::generate_master_key;
    use tempfile::tempdir;

    fn vars(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
        let pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |key: &str| pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    fn complete() -> Vec<(&'static str, &'static str)> {
        vec![
            ("OSCC_HOSXP_HOST", "10.0.0.5"),
            ("OSCC_HOSXP_DATABASE", "hos"),
            ("OSCC_HOSXP_USER", "oscc_ro"),
            ("OSCC_HOSXP_PASSWORD", "s3cret!p@ss"),
        ]
    }

    fn test_store() -> MasterKeyStore {
        // encryptman 0.3 made key generation fallible (RNG failure is no
        // longer a panic inside the crate); the OS RNG is always available
        // under `cargo test`, so expect here with that invariant.
        MasterKeyStore(generate_master_key().expect("os RNG available during tests"))
    }

    fn sample_config() -> HosxConfig {
        HosxConfig::new(
            "10.0.0.5".into(),
            3306,
            "hosxp".into(),
            "oscc_ro".into(),
            SecretBox::from("s3cret!p@ss".to_string()),
        )
    }

    #[test]
    fn from_vars_uses_default_port_when_absent() {
        let cfg = HosxConfig::from_vars(vars(&complete())).expect("valid config");
        assert_eq!(cfg.port, 3306);
        assert_eq!(cfg.host, "10.0.0.5");
    }

    #[test]
    fn from_vars_reads_explicit_port() {
        let mut pairs = complete();
        pairs.push(("OSCC_HOSXP_PORT", "3307"));
        let cfg = HosxConfig::from_vars(vars(&pairs)).expect("valid config");
        assert_eq!(cfg.port, 3307);
    }

    #[test]
    fn missing_required_var_returns_error() {
        for key in [
            "OSCC_HOSXP_HOST",
            "OSCC_HOSXP_DATABASE",
            "OSCC_HOSXP_USER",
            "OSCC_HOSXP_PASSWORD",
        ] {
            let pairs: Vec<(&str, &str)> =
                complete().into_iter().filter(|(k, _)| *k != key).collect();
            assert_eq!(
                HosxConfig::from_vars(vars(&pairs)).expect_err("missing variable must fail"),
                ConfigError::Missing(key),
                "missing {key}"
            );
        }
    }

    #[test]
    fn invalid_port_returns_error() {
        let mut pairs = complete();
        pairs.push(("OSCC_HOSXP_PORT", "not-a-port"));
        assert!(matches!(
            HosxConfig::from_vars(vars(&pairs)),
            Err(ConfigError::InvalidPort { .. })
        ));
    }

    #[test]
    fn debug_redacts_the_password() {
        let cfg = HosxConfig::from_vars(vars(&complete())).expect("valid config");
        let debug = format!("{cfg:?}");
        assert!(!debug.contains("s3cret!p@ss"));
        assert!(debug.contains("***"));
    }

    #[tokio::test]
    async fn round_trip_encrypts_and_decrypts_all_fields() {
        let dir = tempdir().expect("tempdir in test");
        let path = dir.path().join(CONFIG_FILE_NAME);
        let store = test_store();

        save_encrypted(&path, &store, &sample_config())
            .await
            .expect("save succeeds");
        let loaded = load(&path, &store).await.expect("load succeeds");

        assert_eq!(loaded, Some(sample_config()));
    }

    #[tokio::test]
    async fn missing_file_returns_none() {
        let dir = tempdir().expect("tempdir in test");
        let path = dir.path().join(CONFIG_FILE_NAME);
        assert_eq!(
            load(&path, &test_store()).await.expect("load succeeds"),
            None
        );
    }

    #[tokio::test]
    async fn corrupt_file_returns_error() {
        let dir = tempdir().expect("tempdir in test");
        let path = dir.path().join(CONFIG_FILE_NAME);
        tokio::fs::write(&path, "not json at all")
            .await
            .expect("write in test");
        assert!(load(&path, &test_store()).await.is_err());
    }

    #[tokio::test]
    async fn wrong_key_fails_to_decrypt() {
        let dir = tempdir().expect("tempdir in test");
        let path = dir.path().join(CONFIG_FILE_NAME);
        save_encrypted(&path, &test_store(), &sample_config())
            .await
            .expect("save succeeds");
        assert!(matches!(
            load(&path, &test_store()).await,
            Err(ConfigError::Decrypt(_))
        ));
    }

    #[tokio::test]
    async fn stored_file_contains_no_plaintext() {
        let dir = tempdir().expect("tempdir in test");
        let path = dir.path().join(CONFIG_FILE_NAME);
        save_encrypted(&path, &test_store(), &sample_config())
            .await
            .expect("save succeeds");

        let raw = tokio::fs::read_to_string(&path)
            .await
            .expect("read in test");
        for secret in ["s3cret!p@ss", "oscc_ro", "10.0.0.5"] {
            assert!(
                !raw.contains(secret),
                "plaintext {secret:?} must not appear on disk"
            );
        }
    }
}
