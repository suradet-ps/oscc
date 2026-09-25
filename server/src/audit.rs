//! Append-only audit entries with a hash chain (AGENTS.md §2 rule 4).
//!
//! Every auditable action writes one entry inside a transaction, chaining
//! `hash = sha256(prev_hash || canonical_payload)`. The canonical payload
//! is field-ordered text, so verification never depends on JSON key order.
//! The `detail` value must stay non-identifying (AGENTS.md §2 rule 2).

use oscc_models::{AuditAction, Role};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

/// Advisory lock id that serializes chain appends on this database.
const CHAIN_LOCK_ID: i64 = 0x05CC_A0D1;

/// One auditable action.
#[derive(Debug, Clone)]
pub struct AuditEvent<'a> {
    /// What happened.
    pub action: AuditAction,
    /// Username of the actor (never a patient identifier). For a failed
    /// sign-in this is the attempted username.
    pub actor: &'a str,
    /// Role the actor held, when it is known (a failed sign-in has none).
    pub actor_role: Option<Role>,
    /// Case the action touched, when it touched one.
    pub case_id: Option<&'a str>,
    /// Reason for reasoned actions (reveal, waiver, break-glass).
    pub reason: Option<&'a str>,
    /// Non-identifying structured detail (counts, flags, ids).
    pub detail: serde_json::Value,
}

/// The stored actor role; failed sign-ins are recorded as `unknown`.
pub fn stored_actor_role(event: &AuditEvent<'_>) -> &'static str {
    event.actor_role.map_or("unknown", |role| role.as_str())
}

/// Builds the canonical, field-ordered payload that gets hashed.
///
/// `serde_json::Value` maps are BTree-ordered, so the rendered detail is
/// deterministic.
pub fn canonical_payload(event: &AuditEvent<'_>) -> String {
    format!(
        "action={};actor={};role={};case={};reason={};detail={}",
        event.action.as_str(),
        event.actor,
        event.actor_role.map_or("", |role| role.as_str()),
        event.case_id.unwrap_or(""),
        event.reason.unwrap_or(""),
        serde_json::to_string(&event.detail).unwrap_or_else(|_| "null".to_string()),
    )
}

/// `sha256(prev_hash || canonical_payload)` in lowercase hex.
///
/// The genesis entry uses the literal `genesis` in place of the previous
/// hash, so no entry can ever have a null input.
pub fn entry_hash(prev_hash: Option<&str>, canonical_payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.unwrap_or("genesis").as_bytes());
    hasher.update(canonical_payload.as_bytes());
    hex::encode(hasher.finalize())
}

/// Appends one entry, chaining it to the current head.
///
/// The transaction takes an advisory lock first, so two concurrent appends
/// cannot build the same parent. Callers must not swallow the error: an
/// action without its audit entry did not happen (AGENTS.md §2 rule 4).
///
/// # Errors
///
/// Returns [`sqlx::Error`] when the transaction fails.
pub async fn append(pool: &PgPool, event: &AuditEvent<'_>) -> Result<String, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(CHAIN_LOCK_ID)
        .execute(&mut *tx)
        .await?;

    let prev_hash: Option<String> =
        sqlx::query_scalar("SELECT hash FROM audit_entries ORDER BY id DESC LIMIT 1")
            .fetch_optional(&mut *tx)
            .await?;

    let payload = canonical_payload(event);
    let hash = entry_hash(prev_hash.as_deref(), &payload);

    sqlx::query(
        "INSERT INTO audit_entries \
         (action, actor, actor_role, case_id, reason, detail, prev_hash, hash) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(event.action.as_str())
    .bind(event.actor)
    .bind(stored_actor_role(event))
    .bind(event.case_id)
    .bind(event.reason)
    .bind(&event.detail)
    .bind(prev_hash.as_deref())
    .bind(&hash)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event<'a>(actor: &'a str, case_id: Option<&'a str>) -> AuditEvent<'a> {
        AuditEvent {
            action: AuditAction::CaseViewed,
            actor,
            actor_role: Some(Role::ErNurse),
            case_id,
            reason: None,
            detail: serde_json::json!({ "queue": "my_cases" }),
        }
    }

    #[test]
    fn canonical_payload_is_field_ordered_and_stable() {
        let payload = canonical_payload(&event("nurse.a", Some("OSCC-2026-0042")));
        assert_eq!(
            payload,
            "action=case_viewed;actor=nurse.a;role=er_nurse;case=OSCC-2026-0042;reason=;detail={\"queue\":\"my_cases\"}"
        );
        assert_eq!(
            payload,
            canonical_payload(&event("nurse.a", Some("OSCC-2026-0042")))
        );
    }

    #[test]
    fn canonical_payload_includes_reason_when_present() {
        let mut e = event("lead", Some("OSCC-2026-0042"));
        e.action = AuditAction::BreakGlassOpened;
        e.reason = Some("emergency coverage");
        let payload = canonical_payload(&e);
        assert!(payload.contains("action=break_glass_opened"));
        assert!(payload.contains("reason=emergency coverage"));
    }

    #[test]
    fn hash_is_deterministic_and_sensitive_to_input() {
        let payload = canonical_payload(&event("nurse.a", None));
        let first = entry_hash(None, &payload);
        assert_eq!(first, entry_hash(None, &payload));
        assert_ne!(first, entry_hash(Some("deadbeef"), &payload));
        assert_ne!(first, entry_hash(None, "other-payload"));
        assert_eq!(first.len(), 64);
    }

    #[test]
    fn failed_sign_in_has_no_role_and_stores_unknown() {
        let mut e = event("nurse.a", None);
        e.action = AuditAction::LoginFailed;
        e.actor_role = None;
        let payload = canonical_payload(&e);
        assert!(payload.contains("action=login_failed"));
        assert!(payload.contains("role=;"));
        assert_eq!(stored_actor_role(&e), "unknown");
    }

    #[test]
    fn chain_links_differ_per_entry() {
        let a = canonical_payload(&event("nurse.a", Some("OSCC-2026-0001")));
        let b = canonical_payload(&event("nurse.b", Some("OSCC-2026-0002")));
        let h1 = entry_hash(None, &a);
        let h2 = entry_hash(Some(&h1), &b);
        assert_ne!(h1, h2);
    }
}
