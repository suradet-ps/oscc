# runbook.md — development, deployment, and downtime

> Operational facts only. Everything here is meant to be runnable exactly as
> written, on a clean machine.

## Development setup

Requirements: Rust (edition 2024, rust-version 1.85), the
`wasm32-unknown-unknown` target, and Trunk for the frontend.

```
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Daily checks (AGENTS-RUST.md §3)

```
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
cargo agentforge validate
cargo agentforge check
```

The app workspace is separate: from `app/`, run
`cargo fmt --all -- --check`, `cargo test -p oscc-app`, and
`cargo clippy --target wasm32-unknown-unknown --all-targets -- -D warnings`.

## Running locally

Server (loopback by default, PII-free logs):

```
cargo run -p oscc-server
curl http://127.0.0.1:8080/healthz
```

Binds to the LAN only when explicitly configured, e.g.
`OSCC_BIND=0.0.0.0:8080` for a development machine on a trusted segment.

Desktop shell (starts trunk, then the Tauri window):

```
cargo run -p oscc-tauri
```

## Constitution maintenance

`AGENTS-RUST.md` is tool-managed: upgrade with
`cargo agentforge init --template tauri,wasm,axum,library` after reviewing
`cargo agentforge diff`. Project deviations go in its §14 as
`[OVERRIDE §<section>]` lines; `.agentforge.json` is never hand-edited.

## Deployment (M5 — placeholders, do not run yet)

- Install PostgreSQL as a Windows service; data volume on BitLocker.
- Bind the API to the LAN address; TLS via the hospital CA (or a pinned
  certificate in the client).
- Create the service account and scheduled `pg_dump`/`pg_basebackup` job;
  copy encrypted backups off-box.

## Backup and restore

- A backup that has never been restored is a rumour: the M5 drill records
  who restored, when, and how long it took.
- Restore is performed only on a non-production machine unless the pilot is
  down.

## Downtime (paper fallback)

- ER works 24/7. When the API is unreachable, the client shows the
  paper-fallback instructions and disables writes; staff use the downtime
  paper form and re-enter data afterwards.
- Re-entry is flagged in the audit trail as a late entry; nothing is
  silently back-dated.
