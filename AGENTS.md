# AGENTS.md - OSCC

This file gives coding agents (and future contributors) the context needed to
work on this repo correctly and safely. Read it, `AGENTS-RUST.md` (the Rust
constitution installed via `cargo agentforge`), and `DESIGN.md` (visual/UI
design system) before writing any code. This file covers product context,
architecture, and process; `AGENTS-RUST.md` covers Rust-specific style and
workflow rules; `DESIGN.md` covers visual design only.

Phase 1 is deliberately narrow: **internal paperless workflow only**. Anything
touching forensic evidence, external legal documents, or e-signature is out of
scope until the phases named in §1 and the legal gates in §2 are satisfied.

## 1. What this project is

OSCC (One Stop Crisis Center / "ศูนย์พึ่งได้") case management for a Thai
hospital. A desktop client (Tauri 2 + Leptos 0.8 CSR/WASM) used by ER nurses,
forensic physicians, social workers, psychologists, and the OSCC lead across
department PCs on the hospital LAN. A central API + PostgreSQL runs on a
hospital server; HOSxP is read-only and used only to pull patient
demographics/visits.

**Phase 1 scope (current):** the internal paperless workflow.

- Intake / screening with a structured form
- Case registry with status and assignment
- Authentication, RBAC, need-to-know, break-glass, append-only audit
- Timers and alerts (HIV PEP <= 72h, emergency contraception <= 120h,
  STI/HBV prophylaxis, follow-up 1/4 weeks)
- Follow-up tasks and internal referral tracking
- PII-free dashboard and indicators

**Non-goals (deferred, do not build yet):**

- Forensic evidence, chain of custody, photo/attachment vault (Phase 2)
- External documents for police / พมจ. / courts and electronic signatures
  (Phase 2-3, require DPO + legal sign-off)
- Any writing to HOSxP; this is not an EMR/HIS
- Internet-facing services, LINE/email/cloud channels for case data
- Claims of "100% paperless" - external documents stay hybrid by design

## 2. Hard rules (non-negotiable)

1. **Never write to HOSxP.** Read-only end to end: dedicated `GRANT SELECT`
   user, `SET SESSION TRANSACTION READ ONLY`, an application-level guard that
   accepts only a single `SELECT`/`WITH`, and parameterized queries. The same
   enforcement layers as AllerX, for the same reason.
2. **Never log PII.** No name, CID, HN, phone, address, incident narrative, or
   free-text note in logs, errors, metrics, URLs, or panic messages. Log case
   IDs, status codes, and durations only. Never `Debug`-print patient structs
   without an explicit redacting implementation.
3. **All OSCC data flows through the central API.** Desktop clients never hold
   database credentials and never connect to PostgreSQL or HOSxP directly.
4. **Audit is append-only and mandatory.** Every case open/view/reveal/edit/
   print/export writes an audit entry: who, what, when, and why (break-glass).
   Application roles cannot `UPDATE` or `DELETE` audit rows.
5. **RBAC + need-to-know + break-glass.** Default screens show the case ID,
   not the patient's identity. Revealing identity requires permission and is
   audited. Break-glass access requires a reason and notifies the OSCC lead.
6. **No real patient data outside production.** Development, tests, demos, and
   screenshots use synthetic data only.
7. **LAN only.** No public exposure, no third-party SaaS, no cloud sync. TLS
   inside the LAN (hospital internal CA, or a pinned certificate).
8. **Encryption:** TLS is mandatory for all API traffic; encrypt the DB volume
   at rest (BitLocker or equivalent) and field-encrypt the most sensitive
   identifiers where the platform cannot be trusted.
9. **Legal sensitivity.** Case data includes PDPA-sensitive categories (health,
   sexual behavior, criminal record, minors) and is governed by child
   protection and domestic violence law. Consult the hospital DPO/legal before
   expanding scope - not after.
10. **Nothing ships without tests and audit coverage.** A case action without
    an audit path is a bug, not a feature request.

## 3. Architecture

Client-server, deliberately boring:

```
[ER PC] [Forensic PC] [Social work PC] [OSCC lead PC]
    |         |               |                |
    +------ HTTPS over LAN (internal CA / pinned cert) ---+
                           |
                    [Central API - axum]
             auth + RBAC + audit + timers + case logic
                       |             |
               (write) |             | (read-only)
                       v             v
                 [OSCC DB]      [HOSxP MySQL]
                 PostgreSQL     GRANT SELECT,
                 (encrypted)    session read-only,
                                SQL guard
```

- **Start on one hospital Windows host** (API + PostgreSQL together). Clients
  only ever know the **API URL**; the API knows the DB DSN from its
  configuration. Splitting the DB onto another machine later must not require
  a client change.
- **Why central:** RBAC, audit, and credentials stay in one place; department
  PCs stay disposable and credential-free.
- HOSxP reads happen **at intake only** to link the patient; snapshot what was
  read plus the timestamp, because HOSxP data changes but the record must not.

## 4. Workspace layout

```
oscc/
├── src-tauri/              # Tauri 2 shell - thin IPC adapters, no business logic
├── app/                    # Leptos 0.8 CSR/WASM frontend (Thai UI)
├── crates/
│   ├── models/             # Domain types. No sqlx, no I/O.
│   ├── oscc-core/          # Case state machine, timer rules, RBAC decisions,
│   │                       # audit event shapes. Pure logic, fully unit-testable.
│   └── hosxp-connector/    # The only crate allowed to talk to HOSxP (SELECT-only)
├── server/                 # axum API + sqlx/PostgreSQL; the only writer of OSCC data
└── docs/                   # architecture, data map, RBAC matrix, retention, runbooks
```

Dependency direction: `app` -> `oscc-core` -> `models`; `server` ->
`oscc-core` + `hosxp-connector`; `src-tauri` -> `app` only. The client never
depends on sqlx or on `hosxp-connector`.

## 5. Domain model (Phase 1 sketch)

Names are indicative; shape decisions belong in `docs/architecture.md` with
tests in `oscc-core`.

```rust
pub struct Case {
    pub case_id: String,          // non-identifying, e.g. OSCC-2026-0042
    pub status: CaseStatus,       // Intake | Active | FollowUp | Closed
    pub incident_type: IncidentType,
    pub incident_at: Option<NaiveDateTime>,
    pub reported_at: NaiveDateTime,
    pub risk_level: RiskLevel,    // drives timers and alerting
    pub owner_department: Department,
}

pub struct PatientLink {          // pulled from HOSxP at intake, snapshot only
    pub hn: String,
    pub cid_masked: String,       // full CID only through an audited reveal
    pub name_snapshot: String,    // never logged
    pub snapshot_at: NaiveDateTime,
}

pub struct CaseTask {             // timers and follow-ups
    pub kind: TaskKind,           // Pep72h | Ec120h | StiProphylaxis | Hbv
                                  // | FollowUp1w | FollowUp4w
    pub due_at: NaiveDateTime,
    pub state: TaskState,         // Pending | Done | Missed | Waived
}
```

Also required: `User`, `Role`, `Department`, `CaseAssignment`, `BreakGlass`,
`Referral` (internal), `FollowUp`, and `AuditEntry` (append-only, hash-chained
so tampering is detectable).

Case state transitions, timer windows, and RBAC decisions live in
`oscc-core` as pure functions with exhaustive unit tests - not in handlers.

## 6. Security and privacy

- **Secrets:** session tokens in the OS keychain on the client; server secrets
  from service configuration/env, never the repo. No secret in any lockfile,
  log, or error message.
- **Sessions:** short TTL, idle auto-lock, no "remember me" on shared PCs.
- **Reveal / print / export:** permissioned, audited, watermarked, and
  minimized. Printing patient-identifying pages is a logged event.
- **Errors:** typed `thiserror` errors in English inside crates; Thai
  user-facing translation happens only at the client/server boundary layer,
  matching the AllerX convention.
- **Backups:** encrypted, off-box, with a documented restore drill. A backup
  that has never been restored is a rumour.
- **Downtime:** ER works 24/7. Phase 1 must have a paper fallback and a
  documented re-entry path - do not design as if the server is always up.

## 7. Milestones (Phase 1)

| Milestone | Scope |
|---|---|
| **M0** | Workspace scaffold, AGENTS/docs, CI (fmt, clippy, tests, deny) |
| **M1** | Server auth + RBAC + audit skeleton; client login + lock; HOSxP connector with read-only enforcement |
| **M2** | Case registry + intake form; patient link via read-only HOSxP lookup (snapshot) |
| **M3** | Timer rules (PEP/EC/STI/HBV) + follow-up tasks + in-app alerts |
| **M4** | Internal referrals; PII-free dashboard/indicators |
| **M5** | Hardening: audit review, backup/restore drill, downtime runbook, pilot |

Do not start a milestone before the previous one has tests and docs merged.

## 8. Testing expectations

- `oscc-core`: pure unit tests for state machine, timer windows, RBAC
  decisions, and audit hash chaining. Must run without a database.
- `server`: API tests against a disposable PostgreSQL (env-gated, never
  production), including "audit row written" assertions for every case action.
- `app`: wasm tests for critical UI logic (masking, role gating).
- `hosxp-connector`: every statement has a guard test asserting it is a single
  read statement, and a test that the referenced tables stay on the documented
  access surface.
- No test, fixture, or screenshot may contain real patient data.

## 9. Documentation-first workflow

Before implementing a milestone, confirm the relevant doc is current; if a
change alters architecture or design, update the doc in the same commit/PR.
`AGENTS-RUST.md` is tool-managed: keep it current with `cargo agentforge check`
(non-zero when stale), audit issues with `cargo agentforge validate`, and put
genuine project deviations in its §14 as `[OVERRIDE §<section>] <rule>` lines -
never by editing baseline rules. `.agentforge.json` is the rule manifest (body
checksums); do not hand-edit it.

Planned docs:

- `DESIGN.md` (repo root) - visual design system: tokens, components, privacy
  and audit UI patterns
- `docs/architecture.md` - architecture decisions and trade-offs
- `docs/data-map.md` - every PII field, where it lives, who can see it
- `docs/rbac.md` - role x department x action matrix, break-glass policy
- `docs/retention.md` - retention and legal hold per case type
- `docs/runbook.md` - install, backup/restore, downtime paper process

## 10. Style and conventions

- Rust 2024, typed `thiserror` errors per layer, no ad-hoc `String` errors.
- Thai UI strings live in `app/`; backend code, comments, and identifiers stay
  English.
- Prefer small explicit functions over clever abstractions - a nurse or
  pharmacist developer should be able to re-read this in six months.
- Naming: identifiers must not overpromise. A field called `name_snapshot` is
  exactly that; anything derived or masked says so.

## 11. What NOT to do

- Don't add evidence/chain-of-custody, photo vault, e-signature, or external
  document generation yet.
- Don't use LINE, email, or cloud services to move case data.
- Don't give clients DB credentials or a direct DB path "just for a feature".
- Don't cache PII to disk on clients; keep it in session memory only.
- Don't build reports that identify individuals by default.
- Don't write to HOSxP - not in code, not in tests, not "to try something".
- Don't log, `Debug`-print, or put patient data in error messages.

## 12. Open decisions (resolve before M2)

1. Windows edition of the server host, and whether AD/LDAP login is available.
2. Server location (dedicated PC vs hospital VM) and who owns backups.
3. Retention periods per case type, and the legal-hold trigger process
   (needs DPO/legal).
4. Who may unmask identity, and who receives break-glass notifications.
5. Timer policy: exact windows and escalation path for PEP/EC/STI/HBV.
6. Intake taxonomy and the indicator set the OSCC lead actually reports.
7. Whether free-text narrative exists in Phase 1 or is deferred to Phase 2.

Resolve these in `docs/architecture.md` (one short section each) before writing
M2 code, so the schema does not have to be re-cut later.
