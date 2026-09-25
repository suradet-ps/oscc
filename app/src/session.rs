//! Client-side session state: who is signed in, whether the screen is
//! locked, and how long the session has left.
//!
//! The token lives in memory only. On shared department PCs that is the
//! safer default: closing the app signs the operator out (AGENTS.md §6).

use chrono::{DateTime, Utc};
use leptos::prelude::*;

use crate::api::{DEFAULT_API_BASE, UserInfo};

/// One signed-in session.
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    /// Opaque bearer token.
    pub token: String,
    /// The signed-in user.
    pub user: UserInfo,
    /// Absolute expiry (server-owned).
    pub expires_at: DateTime<Utc>,
}

/// Reactive session state shared by the shell.
#[derive(Clone, Copy)]
pub struct SessionState {
    /// `Some` while a session exists; `None` shows the login screen.
    pub session: RwSignal<Option<Session>>,
    /// True while the lock screen is up.
    pub locked: RwSignal<bool>,
    /// API base URL typed on the login screen.
    pub api_base: RwSignal<String>,
    /// Seconds until the session locks, refreshed every second.
    pub remaining_secs: RwSignal<i64>,
}

impl SessionState {
    /// A fresh, signed-out state.
    pub fn new() -> Self {
        Self {
            session: RwSignal::new(None),
            locked: RwSignal::new(false),
            api_base: RwSignal::new(DEFAULT_API_BASE.to_string()),
            remaining_secs: RwSignal::new(0),
        }
    }

    /// Starts a session after a successful login.
    pub fn sign_in(&self, session: Session) {
        self.locked.set(false);
        self.remaining_secs.set(remaining_of(&session));
        self.session.set(Some(session));
    }

    /// Ends the session (logout or API rejection).
    pub fn sign_out(&self) {
        self.session.set(None);
        self.locked.set(false);
        self.remaining_secs.set(0);
    }

    /// Locks the screen without ending the session.
    pub fn lock(&self) {
        self.locked.set(true);
    }

    /// Refreshes the countdown and locks when it reaches zero.
    pub fn tick(&self) {
        let Some(session) = self.session.get() else {
            return;
        };
        let remaining = remaining_of(&session);
        self.remaining_secs.set(remaining);
        if remaining == 0 {
            self.lock();
        }
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

fn remaining_of(session: &Session) -> i64 {
    (session.expires_at - Utc::now()).num_seconds().max(0)
}

/// Thai label for a machine role name (UI strings live in `app/`).
pub fn role_label(role: &str) -> &'static str {
    match role {
        "er_nurse" => "พยาบาล ER",
        "forensic_physician" => "นิติเวช",
        "social_worker" => "สังคมสงเคราะห์",
        "psychologist" => "จิตวิทยา",
        "oscc_lead" => "หัวหน้า OSCC",
        "admin" => "ผู้ดูแลระบบ",
        _ => "ไม่ทราบบทบาท",
    }
}

/// Whether the role may reveal identity. Mirrors `oscc-core::rbac`; the
/// server enforces it regardless, and a `403` is handled as an error.
pub fn can_reveal(role: &str) -> bool {
    matches!(role, "forensic_physician" | "social_worker" | "oscc_lead")
}

/// Whether the role may register cases. Mirrors `oscc-core::rbac`.
pub fn can_create(role: &str) -> bool {
    matches!(
        role,
        "er_nurse" | "forensic_physician" | "social_worker" | "oscc_lead"
    )
}
