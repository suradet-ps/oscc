# retention.md — how long OSCC data lives

> Phase 1 stores nothing yet. This file records the rules that must be
> signed off by the hospital DPO and legal before M5, and the interim
> behaviour meanwhile.

## Interim behaviour

- Nothing is deleted in Phase 1.
- No automatic purge, no "cleanup" tooling, no hard-delete API.
- Retention is enforced later by a reviewed migration, never by ad-hoc SQL.

## Policies to define (per case type)

| Case type | Retention | Legal basis | Notes |
|---|---|---|---|
| Adult sexual assault | TBD | TBD | Evidence and medical record windows differ |
| Domestic violence | TBD | TBD | Repeat-incident history may need longer |
| Child abuse / neglect | TBD | Child protection law | Often the longest window |
| Trafficking | TBD | TBD | May involve a criminal case |
| Other / unknown | TBD | TBD | Default to the longest applicable window |

## Legal hold

- When a case becomes part of a criminal or civil proceeding, it must be
  flagged as legal hold and excluded from every purge.
- Legal hold can only be set or cleared by the OSCC lead, and both actions
  are audited.
- The hold flag is designed in M2's schema; until then, nothing is deleted
  anyway.

## Audit retention

- Audit entries are retained at least as long as the case they describe.
- Audit entries are never editable; the retention process may only remove
  whole expired cases, never individual entries.

## Sign-off required before M5

- Retention periods per case type, confirmed by DPO/legal.
- Backup retention and rotation consistent with the table above.
- The procedure (and approver) for legally destroying expired records.
