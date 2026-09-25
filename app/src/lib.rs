//! OSCC desktop client (Leptos 0.8, CSR).
//!
//! M0 ships the workbench shell: top bar, nav rail, and an empty case
//! queue. Case data, RBAC, and the deadline rail arrive in M1-M3
//! (AGENTS.md §7). No PII is ever rendered by a placeholder.

use leptos::prelude::*;

/// Mounts the app into the document body.
pub fn run() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

/// Three-region workbench shell (DESIGN.md "Workbench structure").
#[component]
fn App() -> impl IntoView {
    view! {
        <div class="app">
            <TopBar />
            <div class="app__body">
                <NavRail />
                <main class="workspace">
                    <EmptyState />
                </main>
            </div>
        </div>
    }
}

/// Top bar: app identity and the connection state (DESIGN.md "Top Bar").
#[component]
fn TopBar() -> impl IntoView {
    view! {
        <header class="top-bar">
            <div class="top-bar__brand">
                <span class="top-bar__title">"OSCC"</span>
                <span class="top-bar__subtitle">"ศูนย์พึ่งได้"</span>
            </div>
            <div class="top-bar__status">
                <span class="status-dot status-dot--disconnected" aria-hidden="true"></span>
                <span>"ยังไม่ได้เชื่อมต่อเซิร์ฟเวอร์"</span>
            </div>
        </header>
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
                "ระบบจะแสดงเคสตามกำหนดเวลาเมื่อเชื่อมต่อเซิร์ฟเวอร์สำเร็จ"
            </p>
        </div>
    }
}
