# rbac.md — roles, permissions, and break-glass

> Provisional matrix (AGENTS.md §12 item 4). It must stay in sync with
> `crates/oscc-core/src/rbac.rs`; the code is the source of truth and its
> tests assert the rules below.

## Role × permission matrix

| Permission | ER nurse | Forensic physician | Social worker | Psychologist | OSCC lead | Admin |
|---|---|---|---|---|---|---|
| ViewCase | yes | yes | yes | yes | yes | no |
| RevealIdentity | no | yes | yes | no | yes | no |
| UpdateCase | yes | yes | yes | yes | yes | no |
| CompleteTask | yes | yes | yes | yes | yes | no |
| CloseCase | no | yes | no | no | yes | no |
| ViewDashboard | yes | yes | yes | yes | yes | yes |
| ExportAggregate | no | no | no | no | yes | no |
| OpenBreakGlass | yes | no | no | no | yes | no |

Permissions are necessary but not sufficient: the server additionally checks
the case assignment. A role only ever opens cases the user is assigned to,
unless break-glass is taken.

## Reason-required actions

`RevealIdentity`, `ExportAggregate`, and `OpenBreakGlass` require a recorded
reason and a two-step confirmation (DESIGN.md privacy rules). Reveal
re-masks automatically after 60 seconds, on navigation, and on lock.

## Break-glass

- Who: ER nurse and OSCC lead (provisional).
- Why: required free-text reason, stored in the audit entry.
- Effect: a persistent banner on the case shows who opened it, why, and
  when; it cannot be dismissed while the case is open.
- Aftermath: every break-glass entry is surfaced to the OSCC lead for
  review. Review tracking arrives with the audit module in M1.

## Sign-off required before M1

- Confirm the ER nurse may break-glass, or restrict it to the OSCC lead.
- Confirm who may close a case (currently forensic physician + OSCC lead).
- Confirm whether psychologists ever need identity reveal.
