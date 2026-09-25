# OSCC

```
 ██████╗ ███████╗ ██████╗ ██████╗
██╔═══██╗██╔════╝██╔════╝██╔════╝
██║   ██║███████╗██║     ██║
██║   ██║╚════██║██║     ██║
╚██████╔╝███████╗╚██████╗╚██████╗
 ╚═════╝ ╚══════╝ ╚═════╝ ╚═════╝
```

---

## ◆ PULSE

One case, one clock. OSCC is a desktop workbench for a hospital's
One Stop Crisis Center ("ศูนย์พึ่งได้") - the team that receives, screens,
and follows up on violence and abuse cases under pressure, around the
clock. It keeps the internal workflow paperless: intake, case registry,
deadlines (HIV PEP ≤ 72 h, emergency contraception ≤ 120 h), follow-up
tasks, and an append-only audit trail - while patient identity stays
masked until someone with a reason unmasks it.

Nothing writes to HOSxP. Nothing identifies a patient in a log. Nothing
opens outside the hospital LAN.

| M0 ▣ | M1 ▣ | M2-M5 ☐ | Phase 2 ☐ | Phase 3 ☐ |
|---|---|---|---|---|

*The workspace, the Rust constitution, the design system, and the CI gates
are sealed — and so is the M1 trust layer: local accounts, RBAC
enforcement, the append-only audit chain, and the TLS-required read-only
HOSxP connector. Case intake, the patient link, and the deadline rail
stand open; forensic evidence and electronic signatures wait behind legal
sign-off.*

> Built with Tauri 2 + Leptos 0.8, served by an axum API, decided by
> `oscc-core`, read from HOSxP by `hosxp-connector` - never a write,
> never a leak.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

Two installs, one test, one launch.

```
⟫ rustup target add wasm32-unknown-unknown
⟫ cargo install trunk --locked
⟫ cargo test --workspace        # DB-free: core logic and the HOSxP guard
⟫ cargo run -p oscc-server      # loopback by default; bind the LAN explicitly
⟫ cargo run -p oscc-tauri       # desktop shell; trunk serves the frontend
```

<details>
<summary>The constitution</summary>

`AGENTS-RUST.md` is installed and kept current with
[agentforge-rs](https://github.com/suradet-ps/agentforge-rs):

```
⟫ cargo install --git https://github.com/suradet-ps/agentforge-rs cargo-agentforge --locked
⟫ cargo agentforge validate
⟫ cargo agentforge check
```

Project-specific deviations belong in its §14 as `[OVERRIDE §<section>]`
lines - the baseline is never edited by hand.

</details>

---

## ◆ ANATOMY

Five crates, three laws: HOSxP is read-only, identity is masked by
default, and the audit trail only ever grows.

- **Records** - `models` holds the inert domain: case id, lifecycle,
  deadlines, roles, and the patient snapshot. `Cid`, `Hn`, and
  `PatientLink` redact their `Debug` output, because logs are not a place
  for people.
- **Decides** - `oscc-core` is pure logic: lifecycle transitions, deadline
  windows and urgency, and the role matrix. Every rule is tested without a
  database, a server, or a clock.
- **Serves** - `server` is the only writer of OSCC data and the only
  component allowed to read HOSxP. It owns Argon2id accounts, opaque
  hashed sessions, the append-only audit chain, and RBAC enforcement;
  case routes arrive in M2.
- **Asks** - `app` is the Leptos workbench: top bar with role and
  connection state, nav rail with non-identifying queue counts, and the
  deadline rail - the only element allowed to shout.
- **Guards** - `hosxp-connector` is the only door to HOSxP: a dedicated
  `GRANT SELECT` user, `SET SESSION TRANSACTION READ ONLY`, a SQL guard
  that accepts a single read statement, and parameterized queries on top.
- **Seals** - identity is masked by default and re-masks after 60 seconds,
  on navigation, and on lock; reveal and break-glass require a reason and
  are audited. `Admin` holds no case permissions. The API binds loopback
  unless a deployment says otherwise, TLS is required, and no PII reaches
  a log, a toast, or a window title.

---

## ◆ RITUALS

**The core ceremony** - one case, one deadline rail:

1. Register the case at intake. The queue shows `OSCC-2026-0042`, never a
   name.
2. The deadline rail appears under the case header: **PEP**, **EC**,
   **STI**, **HBV**, follow-ups - each a chip that turns amber, then red,
   on its own schedule.
3. Record what happened: a dose given, an appointment set, a reason for
   waiving. Each entry lands in the audit trail.
4. Follow up until the case closes. Closed is closed - nothing reopens it
   without a new case and its own record.

**The ceremony of privacy** - identity is not shown, it is requested.
Reveal is a two-step action with a recorded reason; the revealed state
carries a persistent "เปิดเผยแล้ว · ถูกบันทึก" marker; the audit says who
saw what, when, and why. Break-glass opens a case outside your assignment
and cannot be hidden afterwards.

**The ceremony of accountability** - every open, view, reveal, edit, and
close writes an append-only entry. The access trail sits next to the
record it describes, not in a settings page - transparency is part of the
case, not an admin feature.

**The ceremony of downtime** - the ER works 24/7, so the system assumes it
will not. When the API is unreachable, writes stop, the paper fallback
takes over, and anything re-entered later is flagged as a late entry.
Nothing is silently back-dated.

---

## ◆ ECHOES

**Where this artifact is heading**

```
Phase 1 ▸ internal paperless: intake, registry, RBAC, audit, timers      ▸ M0-M1 sealed, M2-M5 open
Phase 2 ▸ forensic evidence + chain of custody, document generation      ▸ ahead
Phase 3 ▸ e-signature + external documents (DPO/legal gates)             ▸ ahead
```

**Raising the artifact** - read `AGENTS.md` first, especially the hard
rules: never write to HOSxP, never log PII, all data through the API,
audit everything, mask by default. `DESIGN.md` holds the visual system
and its privacy patterns; `docs/architecture.md` records every decision
and open question; `docs/rbac.md` is the provisional role matrix;
`docs/runbook.md` covers install, backup, and downtime.

**Status** - CI gates every change: fmt, `clippy -D warnings`, tests,
`cargo doc`, `cargo-deny`, a trunk WASM build, and the agentforge
validate/check pair. [Watch the gates](.github/workflows).

> ⚠️ OSCC coordinates a care process; it is not a legal record and not
> medical advice. Forensic evidence, disclosure, and signatures follow
> hospital policy and the law - the tool never replaces a clinician's
> judgement.

---

```
  ─────────────────────────────────────────
   A record that is visible by default
   cannot be made private later.
  ─────────────────────────────────────────
```

OSCC is distributed under the [MIT License](LICENSE).
