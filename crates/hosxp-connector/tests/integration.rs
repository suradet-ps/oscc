//! Live-instance tests against a **test/staging** HOSxP only (AGENTS.md §8).
//! Never part of the default run — gate with:
//!
//! ```text
//! cargo test -p oscc-hosxp-connector --features integration-tests
//! ```
//!
//! Requires `OSCC_HOSXP_*` variables pointing at the test instance.

#![cfg(feature = "integration-tests")]

use oscc_hosxp_connector::config::HosxConfig;
use oscc_hosxp_connector::pool;

#[tokio::test]
async fn ping_round_trips_against_the_test_instance() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = HosxConfig::from_env()?;
    let pool = pool::connect(&cfg).await?;
    pool::ping(&pool).await?;
    Ok(())
}
