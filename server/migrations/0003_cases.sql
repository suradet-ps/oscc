-- 0003_cases.sql — case registry and patient snapshots (AGENTS.md §5).
--
-- patient_links stores the intake snapshot: HOSxP keeps changing, the
-- record of who the case was about must not. Full identity lives here and
-- is only returned through the audited reveal path.

CREATE SEQUENCE case_number_seq;

CREATE TABLE cases (
    id               BIGSERIAL PRIMARY KEY,
    case_id          TEXT        NOT NULL UNIQUE,
    status           TEXT        NOT NULL,
    incident_type    TEXT        NOT NULL,
    incident_at      TIMESTAMPTZ,
    reported_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    risk_level       TEXT        NOT NULL,
    owner_department TEXT        NOT NULL,
    created_by       BIGINT      NOT NULL REFERENCES users(id)
);

CREATE INDEX cases_status_idx ON cases (status, reported_at DESC);

CREATE TABLE patient_links (
    case_pk       BIGINT      PRIMARY KEY REFERENCES cases(id),
    hn            TEXT        NOT NULL,
    cid           TEXT        NOT NULL,
    name_snapshot TEXT        NOT NULL,
    snapshot_at   TIMESTAMPTZ NOT NULL
);
