//! Case queue and the intake modal.

use std::sync::Arc;

use chrono::{Local, NaiveDateTime, TimeZone, Utc};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{ApiClient, CreateCaseInput, PatientCandidate};
use crate::labels;
use crate::session::{Session, SessionState, can_create};
use crate::workbench::{WorkbenchState, format_local, visible_cases};

/// The queue list with its toolbar.
#[component]
pub fn CaseQueue(
    wb: WorkbenchState,
    session_state: SessionState,
    session: Session,
) -> impl IntoView {
    let api = ApiClient::new(session_state.api_base.get(), session.token.clone());
    let intake_open = RwSignal::new(false);
    let may_create = can_create(&session.user.role);

    view! {
        <div class="queue">
            <div class="panel-toolbar">
                <h2 class="panel-title">{move || wb.queue.get().label()}</h2>
                {may_create
                    .then(|| {
                        view! {
                            <button
                                class="button-primary"
                                on:click=move |_| intake_open.set(true)
                            >
                                "เธฅเธเธ—เธฐเน€เธเธตเธขเธเน€เธเธชเนเธซเธกเน"
                            </button>
                        }
                    })}
            </div>

            {move || {
                wb.error
                    .get()
                    .map(|message| view! { <p class="form-error">{message}</p> })
            }}

            {move || {
                if wb.loading.get() {
                    view! { <p class="muted">"เธเธณเธฅเธฑเธเนเธซเธฅเธ”..."</p> }.into_any()
                } else {
                    let rows = visible_cases(&wb.cases.get(), wb.queue.get());
                    if rows.is_empty() {
                        view! {
                            <div class="empty-state">
                                <p class="empty-state__title">"เธขเธฑเธเนเธกเนเธกเธตเน€เธเธชเนเธเธเธดเธงเธเธตเน"</p>
                                <p class="empty-state__hint">
                                    "เน€เธเธชเธ—เธตเนเธฅเธเธ—เธฐเน€เธเธตเธขเธเนเธฅเนเธงเธเธฐเนเธชเธ”เธเธ—เธตเนเธเธตเนเธ•เธฒเธกเธเธณเธซเธเธ”เน€เธงเธฅเธฒ"
                                </p>
                            </div>
                        }
                        .into_any()
                    } else {
                        view! {
                            <ul class="case-list">
                                {rows
                                    .into_iter()
                                    .map(|case| {
                                        let case_id = case.case_id.clone();
                                        let api = api.clone();
                                        view! {
                                            <li>
                                                <button
                                                    class="case-row"
                                                    on:click=move |_| {
                                                        wb.open_case(api.clone(), case_id.clone())
                                                    }
                                                >
                                                    <span class="case-row__id mono">
                                                        {case.case_id.clone()}
                                                    </span>
                                                    <span class=format!(
                                                        "chip {}",
                                                        labels::status_class(&case.status),
                                                    )>{labels::status_label(&case.status)}</span>
                                                    <span class="case-row__meta">
                                                        {labels::incident_label(&case.incident_type)}
                                                    </span>
                                                    <span class=format!(
                                                        "chip {}",
                                                        labels::risk_class(&case.risk_level),
                                                    )>{labels::risk_label(&case.risk_level)}</span>
                                                    <span class="case-row__meta">
                                                        {labels::department_label(&case.owner_department)}
                                                    </span>
                                                    <span class="case-row__time">
                                                        {format_local(case.reported_at)}
                                                    </span>
                                                </button>
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        }
                        .into_any()
                    }
                }
            }}
        </div>

        <IntakeModal
            open=intake_open
            wb=wb
            session_state=session_state
            session=session
        />
    }
}

/// The intake form: find the patient in HOSxP, then register the case.
#[component]
fn IntakeModal(
    open: RwSignal<bool>,
    wb: WorkbenchState,
    session_state: SessionState,
    session: Session,
) -> impl IntoView {
    let api = ApiClient::new(session_state.api_base.get(), session.token.clone());
    let lookup_input = RwSignal::new(String::new());
    let lookup_key = RwSignal::new(Option::<(bool, String)>::None);
    let candidate = RwSignal::new(Option::<PatientCandidate>::None);
    let incident_type = RwSignal::new("other".to_string());
    let risk_level = RwSignal::new("medium".to_string());
    let department = RwSignal::new("emergency".to_string());
    let incident_at = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let do_lookup: Arc<dyn Fn() + Send + Sync> = Arc::new({
        let api = api.clone();
        move || {
            let trimmed = lookup_input.get().trim().to_string();
            if trimmed.is_empty() {
                error.set(Some("เธเธฃเธญเธ HN เธซเธฃเธทเธญเน€เธฅเธเธเธฑเธ•เธฃเธเธฃเธฐเธเธฒเธเธเธเนเธญเธเธเนเธเธซเธฒ".to_string()));
                return;
            }
            let is_cid = trimmed.len() == 13 && trimmed.chars().all(|c| c.is_ascii_digit());
            let api = api.clone();
            error.set(None);
            busy.set(true);
            spawn_local(async move {
                let result = if is_cid {
                    api.lookup_cid(&trimmed).await
                } else {
                    api.lookup_hn(&trimmed).await
                };
                match result {
                    Ok(rows) => match rows.into_iter().next() {
                        Some(first) => {
                            candidate.set(Some(first));
                            lookup_key.set(Some((is_cid, trimmed)));
                        }
                        None => {
                            candidate.set(None);
                            lookup_key.set(None);
                            error.set(Some(
                                "เนเธกเนเธเธเธเธนเนเธเนเธงเธขเนเธ HOSxP".to_string(),
                            ));
                        }
                    },
                    Err(err) => {
                        candidate.set(None);
                        lookup_key.set(None);
                        error.set(Some(err.thai_message().to_string()));
                    }
                }
                busy.set(false);
            });
        }
    });

    let do_submit: Arc<dyn Fn() + Send + Sync> = Arc::new({
        let api = api.clone();
        move || {
            let Some((is_cid, value)) = lookup_key.get() else {
                error.set(Some("เธเนเธเธซเธฒเนเธฅเธฐเธขเธทเธเธขเธฑเธเธเธนเนเธเนเธงเธขเธเนเธญเธเธเธฑเธเธ—เธถเธ".to_string()));
                return;
            };
            if busy.get() {
                return;
            }
            let input = CreateCaseInput {
                hn: if is_cid { None } else { Some(value.clone()) },
                cid: if is_cid { Some(value) } else { None },
                incident_type: incident_type.get(),
                incident_at: parse_local_datetime(&incident_at.get()),
                risk_level: risk_level.get(),
                owner_department: department.get(),
            };
            let api = api.clone();
            error.set(None);
            busy.set(true);
            spawn_local(async move {
                match api.create_case(&input).await {
                    Ok(detail) => {
                        let case_id = detail.case_id.clone();
                        open.set(false);
                        lookup_input.set(String::new());
                        lookup_key.set(None);
                        candidate.set(None);
                        wb.reload(api.clone());
                        wb.open_case(api, case_id);
                    }
                    Err(err) => error.set(Some(err.thai_message().to_string())),
                }
                busy.set(false);
            });
        }
    });

    view! {
        {move || {
            let do_lookup = Arc::clone(&do_lookup);
            let do_submit = Arc::clone(&do_submit);
            open.get()
                .then(move || {
                    view! {
                        <div class="modal-backdrop">
                            <div
                                class="modal"
                                role="dialog"
                                aria-modal="true"
                                aria-label="เธฅเธเธ—เธฐเน€เธเธตเธขเธเน€เธเธชเนเธซเธกเน"
                            >
                                <h2 class="modal__title">"เธฅเธเธ—เธฐเน€เธเธตเธขเธเน€เธเธชเนเธซเธกเน"</h2>

                                <div class="form-field">
                                    <span class="form-field__label">
                                        "HN เธซเธฃเธทเธญเน€เธฅเธเธเธฑเธ•เธฃเธเธฃเธฐเธเธฒเธเธ 13 เธซเธฅเธฑเธ"
                                    </span>
                                    <div class="form-row">
                                        <input
                                            class="form-field__input"
                                            autocomplete="off"
                                            prop:value=move || lookup_input.get()
                                            on:input=move |ev| {
                                                lookup_input.set(event_target_value(&ev))
                                            }
                                        />
                                        <button
                                            class="button-secondary"
                                            on:click=move |_| do_lookup()
                                            disabled=move || busy.get()
                                        >
                                            "เธเนเธเธซเธฒ"
                                        </button>
                                    </div>
                                </div>

                                {move || {
                                    candidate
                                        .get()
                                        .map(|c| {
                                            view! {
                                                <section class="identity-card">
                                                    <dl class="facts">
                                                        <div>
                                                            <dt>"เธเธทเนเธญ"</dt>
                                                            <dd>{c.name}</dd>
                                                        </div>
                                                        <div>
                                                            <dt>"HN"</dt>
                                                            <dd class="mono">{c.hn}</dd>
                                                        </div>
                                                        <div>
                                                            <dt>"เน€เธฅเธเธเธฑเธ•เธฃเธเธฃเธฐเธเธฒเธเธ"</dt>
                                                            <dd class="mono">{c.cid}</dd>
                                                        </div>
                                                    </dl>
                                                </section>
                                            }
                                        })
                                }}

                                <div class="form-grid">
                                    <label class="form-field">
                                        <span class="form-field__label">"เธเธฃเธฐเน€เธ เธ—เน€เธซเธ•เธธ"</span>
                                        <select
                                            class="form-field__input"
                                            on:change=move |ev| {
                                                incident_type.set(event_target_value(&ev))
                                            }
                                        >
                                            {[
                                                "other",
                                                "domestic_violence",
                                                "sexual_assault",
                                                "child_abuse",
                                                "physical_assault",
                                                "trafficking",
                                            ]
                                                .into_iter()
                                                .map(|value| {
                                                    view! {
                                                        <option value=value>
                                                            {labels::incident_label(value)}
                                                        </option>
                                                    }
                                                })
                                                .collect_view()}
                                        </select>
                                    </label>

                                    <label class="form-field">
                                        <span class="form-field__label">"เธฃเธฐเธ”เธฑเธเธเธงเธฒเธกเน€เธชเธตเนเธขเธ"</span>
                                        <select
                                            class="form-field__input"
                                            on:change=move |ev| {
                                                risk_level.set(event_target_value(&ev))
                                            }
                                        >
                                            {["medium", "low", "high", "critical"]
                                                .into_iter()
                                                .map(|value| {
                                                    view! {
                                                        <option value=value>
                                                            {labels::risk_label(value)}
                                                        </option>
                                                    }
                                                })
                                                .collect_view()}
                                        </select>
                                    </label>

                                    <label class="form-field">
                                        <span class="form-field__label">"เธซเธเนเธงเธขเธเธฒเธเน€เธเนเธฒเธเธญเธเน€เธเธช"</span>
                                        <select
                                            class="form-field__input"
                                            on:change=move |ev| {
                                                department.set(event_target_value(&ev))
                                            }
                                        >
                                            {["emergency", "forensic", "social_work", "psychology", "oscc"]
                                                .into_iter()
                                                .map(|value| {
                                                    view! {
                                                        <option value=value>
                                                            {labels::department_label(value)}
                                                        </option>
                                                    }
                                                })
                                                .collect_view()}
                                        </select>
                                    </label>

                                    <label class="form-field">
                                        <span class="form-field__label">
                                            "เธงเธฑเธเน€เธเธดเธ”เน€เธซเธ•เธธ (เธ–เนเธฒเธ—เธฃเธฒเธ)"
                                        </span>
                                        <input
                                            class="form-field__input"
                                            type="datetime-local"
                                            prop:value=move || incident_at.get()
                                            on:input=move |ev| {
                                                incident_at.set(event_target_value(&ev))
                                            }
                                        />
                                    </label>
                                </div>

                                {move || {
                                    error
                                        .get()
                                        .map(|message| {
                                            view! { <p class="form-error">{message}</p> }
                                        })
                                }}

                                <div class="modal__actions">
                                    <button
                                        class="button-secondary"
                                        on:click=move |_| open.set(false)
                                    >
                                        "เธขเธเน€เธฅเธดเธ"
                                    </button>
                                    <button
                                        class="button-primary"
                                        on:click=move |_| do_submit()
                                        disabled=move || busy.get()
                                    >
                                        "เธเธฑเธเธ—เธถเธเน€เธเธช"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                })
        }}
    }
}

/// Converts a `datetime-local` value (workstation local time) to UTC.
fn parse_local_datetime(raw: &str) -> Option<chrono::DateTime<Utc>> {
    if raw.trim().is_empty() {
        return None;
    }
    let naive = NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M").ok()?;
    let local = Local.from_local_datetime(&naive).single()?;
    Some(local.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_local_datetime_handles_empty_and_invalid_input() {
        assert!(parse_local_datetime("").is_none());
        assert!(parse_local_datetime("not-a-date").is_none());
    }
}
