//! Configuration and errors for the HOSxP connector.

use std::fmt;

use secrecy::SecretString;
use thiserror::Error;

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
}

/// HOSxP connection settings, read once at server start.
///
/// The password is a [`SecretString`]: zeroized on drop and redacted from
/// [`Debug`], because AGENTS.md §2 rule 2 keeps credentials out of logs.
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
