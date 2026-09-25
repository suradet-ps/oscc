//! OSCC desktop client (Leptos 0.8, CSR).
//!
//! M1: sign-in, session countdown, and lock. Case data, RBAC-driven views,
//! and the deadline rail arrive in M2-M3 (AGENTS.md §7). No PII is ever
//! rendered by a placeholder.

pub mod api;
pub mod components;
pub mod session;

use std::time::Duration;

use leptos::prelude::*;

use components::lock_screen::LockScreen;
use components::login::LoginScreen;
use session::{Session, SessionState, role_label};

/// Mounts the app into the document body.
pub fn run() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

/// Session-aware shell: login, lock screen, or the workbench.
#[component]
fn App() -> impl IntoView {
    let state = SessionState::new();

    // One-second ticker: refreshes the countdown and locks on expiry. One
    // interval for the app's lifetime; the handle is deliberately not kept.
    Effect::new(move |_| {
        let _ = set_interval_with_handle(move || state.tick(), Duration::from_secs(1));
    });

    view! {
        <div class="app">
            {move || match state.session.get() {
                None => view! { <LoginScreen state=state /> }.into_any(),
                Some(session) => {
                    if state.locked.get() {
                        view! { <LockScreen state=state session=session /> }.into_any()
                    } else {
                        view! { <Workbench state=state session=session /> }.into_any()
                    }
                }
            }}
        </div>
    }
}

/// The three-region workbench shell (DESIGN.md "Workbench structure").
#[component]
fn Workbench(state: SessionState, session: Session) -> impl IntoView {
    let display_name = session.user.display_name.clone();
    let role = role_label(&session.user.role);
    let remaining = state.remaining_secs;

    view! {
        <header class="top-bar">
            <div class="top-bar__brand">
                <span class="top-bar__title">"OSCC"</span>
                <span class="top-bar__subtitle">"ศูนย์พึ่งได้"</span>
            </div>
            <div class="top-bar__status">
                <span class="top-bar__user">{display_name} " · " {role}</span>
                <span class="top-bar__timer">{move || format_remaining(remaining.get())}</span>
                <button class="button-secondary" on:click=move |_| state.lock()>
                    "ล็อก"
                </button>
            </div>
        </header>
        <div class="app__body">
            <NavRail />
            <main class="workspace">
                <EmptyState />
            </main>
        </div>
    }
}

/// Nav rail: case queues and the dashboard (counts are non-identifying).
#[component]
fn NavRail() -> impl IntoView {
    let queues = ["รับแจ้งใหม่", "เคสของฉัน", "ติดตามวันนี้", "ทั้งหมด", "แดชบอร์ด"];
    view! {
        <nav class="nav-rail" aria-label="เมนูหลัก">
            <ul class="nav-rail__list">
                {queues
                    .into_iter()
                    .map(|label| {
                        view! {
                            <li class="nav-item">
                                <span>{label}</span>
                                <span class="nav-item__count">"0"</span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </nav>
    }
}

/// Empty queue state (DESIGN.md "Empty & Loading States").
#[component]
fn EmptyState() -> impl IntoView {
    view! {
        <div class="empty-state">
            <p class="empty-state__title">"ยังไม่มีเคสในคิว"</p>
            <p class="empty-state__hint">
                "ระบบจะแสดงเคสตามกำหนดเวลาเมื่อมีข้อมูลเคสในระบบ"
            </p>
        </div>
    }
}

/// `h:mm:ss` (or `mm:ss` under an hour) for the session countdown.
fn format_remaining(secs: i64) -> String {
    let secs = secs.max(0);
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}
