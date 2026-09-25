# data-map.md — where PII lives and who can see it

> Every PII field must appear here with its storage location, visibility, and
> audit requirement (AGENTS.md §9). Phase 1 M0 stores nothing yet: this file
> inventories the fields the domain already models.

## Field inventory

| Field | Type | Classification | Storage (planned) | Visibility | Audit |
|---|---|---|---|---|---|
| `CaseId` | `oscc-models::CaseId` | Non-identifying | `cases.case_id` | All case roles | No (safe label) |
| `PatientLink.hn` | `oscc-models::Hn` | PII | `patient_links.hn` (encrypted at rest) | Case roles after assignment; masked list views do not show it | View logged with the case |
| `PatientLink.cid` | `oscc-models::Cid` | Sensitive PII | `patient_links.cid` (field-encrypted) | Only via reveal (reason + audit) | Reveal/remask audited |
| `PatientLink.name_snapshot` | `String` | PII | `patient_links.name_snapshot` (encrypted) | Same as CID reveal | Reveal audited |
| `PatientLink.snapshot_at` | `DateTime<Utc>` | Metadata | `patient_links.snapshot_at` | Case roles | No |
| `Case.incident_type`, `risk_level` | enums | Sensitive (health/violence) | `cases.*` | Case roles | Edits audited |
| `CaseTask.kind`, `due_at`, `state` | `oscc-models` | Sensitive (health) | `case_tasks.*` | Case roles | Completions/waivers audited |
| Audit actor (name, role) | server-side | PII (staff) | `audit_entries.actor_*` | OSCC lead; own entries for staff | Append-only |
| Break-glass reason | server-side | Sensitive | `audit_entries.reason` | OSCC lead | Append-only |

## Rules

1. Nothing in this table may appear in logs, error messages, metrics, URLs,
   window titles, notifications, or `Debug` output. `Cid`, `Hn`, and
   `PatientLink` redact their `Debug` implementations for this reason.
2. The full CID and name exist in exactly two places: the database column
   and the audited reveal dialog. Everywhere else uses `Cid::masked()`.
3. A HOSxP read happens at intake only and stores a snapshot; the case
   record never silently follows later HOSxP changes.
4. Screenshots, demos, and tests use synthetic data only (AGENTS.md §2
   rule 6).

## To fill in M1 (before storage exists)

- Column-level encryption choice (application-level vs disk-level only) and
  key management owner.
- Backup contents: are PII columns included, and who can restore them?
- Log scrubbing policy for the server (what request metadata is retained).
