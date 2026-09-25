//! Tauri 2 shell — thin IPC adapters only (AGENTS.md §3).
//!
//! No SQL and no business logic live here: the client talks to the central
//! API over HTTPS, and HOSxP is read by the server, never by this shell.
//! M0 registers no commands yet.

/// Entry point used by `main.rs` and the mobile entry point.
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect(
            "invariant: tauri::run() fails only when the platform cannot launch the app window — a GUI app has no fallback path",
        );
}
