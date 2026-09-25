-- 0001_audit.sql — append-only audit trail with a hash chain
-- (AGENTS.md §2 rule 4).
--
-- The application role gets INSERT and SELECT only; UPDATE and DELETE are
-- revoked by the deployment grants in docs/runbook.md. The chain is
-- `hash = sha256(prev_hash || canonical_payload)`.

CREATE TABLE audit_entries (
    id          BIGSERIAL PRIMARY KEY,
    action      TEXT        NOT NULL,
    actor       TEXT        NOT NULL,
    actor_role  TEXT        NOT NULL,
    case_id     TEXT,
    reason      TEXT,
    detail      JSONB       NOT NULL DEFAULT '{}'::jsonb,
    at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    prev_hash   TEXT,
    hash        TEXT        NOT NULL
);

CREATE INDEX audit_entries_at_idx ON audit_entries (at);
CREATE INDEX audit_entries_case_idx ON audit_entries (case_id)
    WHERE case_id IS NOT NULL;
