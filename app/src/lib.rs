//! OSCC desktop client (Leptos 0.8, CSR).
//!
//! M2: the case queue, intake, and audited reveal on top of the M1 trust
//! layer. Identity stays masked until someone reveals it with a reason.

pub mod api;
pub mod components;
pub mod labels;
pub mod session;
pub mod workbench;

use std::time::Duration;

use leptos::prelude::*;

use api::ApiClient;
use components::case_detail::CaseDetailView;
use components::case_queue::CaseQueue;
use components::lock_screen::LockScreen;
use components::login::LoginScreen;
use session::{Session, SessionState, role_label};
use workbench::{QueueFilter, WorkbenchState, count_for};

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

    let wb = WorkbenchState::new();
    let api = ApiClient::new(state.api_base.get(), session.token.clone());

    // Load the queue once when the workbench mounts.
    Effect::new(move |_| {
        wb.reload(api.clone());
    });

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
            <NavRail wb=wb />
            <main class="workspace">
                {move || {
                    if let Some(detail) = wb.selected.get() {
                        view! {
                            <CaseDetailView
                                wb=wb
                                session_state=state
                                session=session.clone()
                                detail=detail
                            />
                        }
                            .into_any()
                    } else {
                        view! {
                            <CaseQueue wb=wb session_state=state session=session.clone() />
                        }
                            .into_any()
                    }
                }}
            </main>
        </div>
    }
}

/// Nav rail: queues with non-identifying counts.
#[component]
fn NavRail(wb: WorkbenchState) -> impl IntoView {
    view! {
        <nav class="nav-rail" aria-label="เมนูหลัก">
            <ul class="nav-rail__list">
                {QueueFilter::ALL
                    .into_iter()
                    .map(|filter| {
                        let enabled = filter.enabled();
                        view! {
                            <li>
                                <button
                                    class="nav-item"
                                    class=("nav-item--active", move || wb.queue.get() == filter)
                                    disabled=!enabled
                                    on:click=move |_| {
                                        if enabled {
                                            wb.select_queue(filter);
                                        }
                                    }
                                >
                                    <span>{filter.label()}</span>
                                    <span class="nav-item__count">
                                        {move || {
                                            if enabled {
                                                count_for(&wb.cases.get(), filter).to_string()
                                            } else {
                                                "—".to_string()
                                            }
                                        }}
                                    </span>
                                </button>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </nav>
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
