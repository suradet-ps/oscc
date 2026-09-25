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

## Storage and the first account (M1)

Windows step-by-step with the actual commands used (install, role, database,
migration, seed, verification): [postgresql-setup.md](postgresql-setup.md).

The API reads `OSCC_DATABASE_URL` (PostgreSQL) and applies migrations on
start; without the variable it runs with the database reported absent.

```
createdb oscc
OSCC_DATABASE_URL=postgres://oscc_app:***@127.0.0.1:5432/oscc cargo run -p oscc-server
```

Seed the first account on a trusted machine (the password arrives on
stdin, never in argv or shell history):

```
printf '%s' 'the-password' | cargo run -p oscc-server --example hash_password
psql "$OSCC_DATABASE_URL" -c "INSERT INTO users (username, display_name, role, password_hash) \
  VALUES ('nurse.a', 'พยาบาล ก', 'er_nurse', '<hash>');"
```

Roles: `er_nurse`, `forensic_physician`, `social_worker`, `psychologist`,
`oscc_lead`, `admin` (matrix in docs/rbac.md).

Database-backed tests run only when pointed at a disposable database:

```
OSCC_TEST_DATABASE_URL=postgres://oscc_app:***@127.0.0.1:5432/oscc_test cargo test -p oscc-server
```

## Constitution maintenance

`AGENTS-RUST.md` is tool-managed: upgrade with
`cargo agentforge init --template tauri,wasm,axum,library` after reviewing
`cargo agentforge diff`. Project deviations go in its §14 as
`[OVERRIDE §<section>]` lines; `.agentforge.json` is never hand-edited.

## HOSxP credentials (encrypted)

The connector never reads credentials from the repo. Two sources, in
order of preference:

1. **Encrypted file** — every field AES-256-GCM encrypted with
   `encryptman`, master key in the OS keychain (service `oscc`). The
   password arrives on stdin, never argv:

   ```
   printf '%s' '<HOSXP_RO_PASSWORD>' | cargo run -p oscc-server --example set_hosxp_config -- <host> <port> <database> <user>
   ```

   Writes to `$OSCC_HOSXP_CONFIG`, or `hosxp.config.json` in the working
   directory. The file is gitignored and contains no plaintext.

2. **Environment** — `OSCC_HOSXP_HOST`, `OSCC_HOSXP_PORT` (default 3306),
   `OSCC_HOSXP_DATABASE`, `OSCC_HOSXP_USER`, `OSCC_HOSXP_PASSWORD` for CI
   and quick dev.

The server prefers the encrypted file when both are present. For local
development without HOSxP, `OSCC_PATIENT_SOURCE=fake` serves one synthetic
patient — never use it in a real deployment.

### TLS on the HOSxP link

The pilot HOSxP server has TLS disabled, so the link is unencrypted on the
hospital LAN by default (`ssl-mode=Preferred`: TLS when the server offers
it, plaintext otherwise). This is a documented residual; the enforced
boundaries are the dedicated `GRANT SELECT` account, the read-only session,
and the SQL guard. When the DBA enables TLS, raise the bar without a code
change:

```
OSCC_HOSXP_SSL_MODE=required          # encrypt, no certificate validation
OSCC_HOSXP_SSL_MODE=verify_ca         # + verify the chain
OSCC_HOSXP_SSL_MODE=verify_identity   # + verify the hostname
```

Testing against a live instance is feature-gated:

```
cargo test -p oscc-hosxp-connector --features integration-tests
```

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
