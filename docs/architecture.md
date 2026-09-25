# architecture.md — OSCC decisions and trade-offs

> Living document. Every change to the architecture lands here in the same
> commit as the code (AGENTS.md §9). Open decisions must be resolved here
> before M2 code is written.

## Shape

```
[department PCs]
  Tauri 2 shell + Leptos CSR      (no DB credentials, API URL only)
        |
        | HTTPS over hospital LAN (internal CA or pinned cert)
        v
[central API — oscc-server]        (axum; the only OSCC writer)
  auth + RBAC + audit + timers + case logic
        |                    |
        | (write)            | (read-only, intake only)
        v                    v
  [OSCC PostgreSQL]      [HOSxP MySQL]
  encrypted volume       GRANT SELECT, read-only session, SQL guard
```

## Decisions

1. **Client-server, central API.** Role checks, audit, and credentials live
   in one place; department PCs stay disposable and credential-free
   (AGENTS.md §3).
2. **One host to start.** API and PostgreSQL run on the same Windows host.
   Clients only know the API URL, so moving the DB to a second machine later
   is a server-side configuration change, not a client release.
3. **Windows host is acceptable.** PostgreSQL runs natively as a Windows
   service. A dedicated PC (not a clinical workstation) with UPS, BitLocker
   on the data volume, no sleep/hibernate, and scheduled Windows updates.
   If an ER PC must serve, it is dedicated to this role.
4. **HOSxP is read-only, read at intake only.** The connector is the only
   crate allowed to touch MySQL. Patient data is snapshotted at intake with
   a timestamp; later HOSxP edits never rewrite the case record.
5. **Identity masked by default.** Full CID and HN appear only through an
   audited reveal (reason required). `Admin` holds no case permissions.
6. **Audit is append-only.** Every open/view/reveal/edit/print/export writes
   an entry. Hash chaining (`AuditEntry`) arrives in M1; application roles
   can never update or delete audit rows.
7. **LAN only, TLS required.** The server binds loopback by default;
   binding to the LAN is an explicit deployment choice (`OSCC_BIND`).
8. **Deadline policy lives in `oscc-core`.** Windows and warning leads are
   pure functions with tests; the UI only renders what they return.
9. **Local accounts first (M1).** Argon2id password hashes; opaque 256-bit
   session tokens stored only as SHA-256; 8 h absolute and 30 min idle
   limits. The client keeps the token in memory only — on shared PCs,
   closing the app signs the operator out. AD/LDAP remains open decision 1.
10. **Storage is optional at startup.** Without `OSCC_DATABASE_URL` the API
    still runs, the health probe reports `database: false`, and auth answers
    `503` instead of pretending. Migrations run automatically on start.
11. **CORS is an allow-list.** Only the Tauri webview origin and the trunk
    dev server may call the API; credentials are never allowed (tokens
    travel in the `Authorization` header).

## Phases

| Phase | Scope |
|---|---|
| **1 (current)** | Internal paperless workflow: intake, registry, RBAC, audit, timers, follow-up, internal referral, PII-free dashboard |
| 2 | Forensic evidence and chain of custody, document generation (hybrid print), attachments vault |
| 3 | Electronic signatures, external documents for police / พมจ. / courts — requires DPO and legal sign-off |

## Open decisions (AGENTS.md §12)

| # | Decision | Current stance | Needed by |
|---|---|---|---|
| 1 | Server host edition + AD/LDAP login | Local accounts are the M1 baseline; AD availability still unknown | M2 |
| 2 | Server location + backup owner | Dedicated PC or hospital VM; owner unknown | M5 |
| 3 | Retention + legal hold | Not yet defined; nothing is deleted in Phase 1 | M5 (DPO) |
| 4 | Who may unmask; break-glass recipients | Provisional matrix in `docs/rbac.md` | M1 |
| 5 | Timer windows + escalation | Provisional in `oscc-core::timers` (PEP/EC fixed; STI/HBV provisional) | M3 (OSCC lead) |
| 6 | Intake taxonomy + indicators | Provisional enum in `oscc-models` | M2 (OSCC lead) |
| 7 | Free-text narrative in Phase 1 | Deferred: structured fields only for now | M2 |
