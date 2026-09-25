//! Case detail with the masked identity card and the reveal dialog.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{ApiClient, CaseDetail};
use crate::labels;
use crate::session::{Session, SessionState, can_reveal};
use crate::workbench::{WorkbenchState, format_local};

/// One case: header, identity card, and facts.
#[component]
pub fn CaseDetailView(
    wb: WorkbenchState,
    session_state: SessionState,
    session: Session,
    detail: CaseDetail,
) -> impl IntoView {
    let api = ApiClient::new(session_state.api_base.get(), session.token.clone());
    let reveal_open = RwSignal::new(false);
    let may_reveal = can_reveal(&session.user.role);
    let case_id = detail.case_id.clone();

    view! {
        <div class="detail">
            <div class="panel-toolbar">
                <button class="button-secondary" on:click=move |_| wb.close_case()>
                    "โ เธเธฅเธฑเธเนเธเธเธดเธง"
                </button>
                {may_reveal
                    .then(|| {
                        view! {
                            <button
                                class="button-primary"
                                on:click=move |_| reveal_open.set(true)
                            >
                                "เน€เธเธดเธ”เน€เธเธขเธ•เธฑเธงเธ•เธเธเธนเนเธเนเธงเธข"
                            </button>
                        }
                    })}
            </div>

            <header class="case-header">
                <span class="case-header__id mono">{detail.case_id.clone()}</span>
                <span class=format!("chip {}", labels::status_class(&detail.status))>
                    {labels::status_label(&detail.status)}
                </span>
                <span class=format!("chip {}", labels::risk_class(&detail.risk_level))>
                    {labels::risk_label(&detail.risk_level)}
                </span>
            </header>

            {move || {
                if let Some(revealed) = wb.revealed.get() {
                    view! {
                        <section class="identity-card identity-card--revealed">
                            <p class="identity-card__marker">"เน€เธเธดเธ”เน€เธเธขเนเธฅเนเธง ยท เธ–เธนเธเธเธฑเธเธ—เธถเธ"</p>
                            <dl class="facts">
                                <div>
                                    <dt>"เธเธทเนเธญ"</dt>
                                    <dd>{revealed.name}</dd>
                                </div>
                                <div>
                                    <dt>"HN"</dt>
                                    <dd class="mono">{revealed.hn}</dd>
                                </div>
                                <div>
                                    <dt>"เน€เธฅเธเธเธฑเธ•เธฃเธเธฃเธฐเธเธฒเธเธ"</dt>
                                    <dd class="mono">{revealed.cid}</dd>
                                </div>
                            </dl>
                            <button
                                class="button-secondary"
                                on:click=move |_| wb.revealed.set(None)
                            >
                                "เธเนเธญเธ"
                            </button>
                        </section>
                    }
                        .into_any()
                } else {
                    view! {
                        <section class="identity-card">
                            <dl class="facts">
                                <div>
                                    <dt>"เธเธทเนเธญ"</dt>
                                    <dd>{detail.patient.name.clone()}</dd>
                                </div>
                                <div>
                                    <dt>"HN"</dt>
                                    <dd class="mono">{detail.patient.hn.clone()}</dd>
                                </div>
                                <div>
                                    <dt>"เน€เธฅเธเธเธฑเธ•เธฃเธเธฃเธฐเธเธฒเธเธ"</dt>
                                    <dd class="mono">{detail.patient.cid.clone()}</dd>
                                </div>
                            </dl>
                            <p class="muted">"เธ•เธฑเธงเธ•เธเธ–เธนเธเธเธดเธ”เนเธงเนเธ•เธฒเธกเธเนเธฒเน€เธฃเธดเนเธกเธ•เนเธ"</p>
                        </section>
                    }
                        .into_any()
                }
            }}

            <section class="facts-card">
                <dl class="facts">
                    <div>
                        <dt>"เธเธฃเธฐเน€เธ เธ—เน€เธซเธ•เธธ"</dt>
                        <dd>{labels::incident_label(&detail.incident_type)}</dd>
                    </div>
                    <div>
                        <dt>"เธซเธเนเธงเธขเธเธฒเธเน€เธเนเธฒเธเธญเธเน€เธเธช"</dt>
                        <dd>{labels::department_label(&detail.owner_department)}</dd>
                    </div>
                    <div>
                        <dt>"เธฃเธฑเธเนเธเนเธเน€เธกเธทเนเธญ"</dt>
                        <dd>{format_local(detail.reported_at)}</dd>
                    </div>
                    <div>
                        <dt>"เน€เธเธดเธ”เน€เธซเธ•เธธเน€เธกเธทเนเธญ"</dt>
                        <dd>
                            {detail
                                .incident_at
                                .map(format_local)
                                .unwrap_or_else(|| "-".to_string())}
                        </dd>
                    </div>
                    <div>
                        <dt>"เธเธฑเธเธ—เธถเธเธเนเธญเธกเธนเธฅเธเธนเนเธเนเธงเธขเน€เธกเธทเนเธญ"</dt>
                        <dd>{format_local(detail.patient.snapshot_at)}</dd>
                    </div>
                </dl>
            </section>
        </div>

        <RevealModal open=reveal_open wb=wb api=api case_id=case_id />
    }
}

/// The mandated reason dialog: no reason, no reveal.
#[component]
fn RevealModal(
    open: RwSignal<bool>,
    wb: WorkbenchState,
    api: ApiClient,
    case_id: String,
) -> impl IntoView {
    let reason = RwSignal::new("เธเธฒเธฃเธฃเธฑเธเธฉเธฒ".to_string());
    let note = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let do_reveal: Arc<dyn Fn() + Send + Sync> = Arc::new({
        let api = api.clone();
        let case_id = case_id.clone();
        move || {
            if busy.get() {
                return;
            }
            let base_reason = reason.get();
            let note_value = note.get().trim().to_string();
            let full = if note_value.is_empty() {
                base_reason
            } else {
                format!("{base_reason}: {note_value}")
            };
            let api = api.clone();
            let case_id = case_id.clone();
            error.set(None);
            busy.set(true);
            spawn_local(async move {
                match api.reveal(&case_id, &full).await {
                    Ok(revealed) => {
                        wb.revealed.set(Some(revealed));
                        note.set(String::new());
                        open.set(false);
                    }
                    Err(err) => error.set(Some(err.thai_message().to_string())),
                }
                busy.set(false);
            });
        }
    });

    view! {
        {move || {
            let do_reveal = Arc::clone(&do_reveal);
            open.get()
                .then(move || {
                    view! {
                        <div class="modal-backdrop">
                            <div
                                class="modal"
                                role="dialog"
                                aria-modal="true"
                                aria-label="เน€เธเธดเธ”เน€เธเธขเธ•เธฑเธงเธ•เธเธเธนเนเธเนเธงเธข"
                            >
                                <h2 class="modal__title">"เน€เธเธดเธ”เน€เธเธขเธ•เธฑเธงเธ•เธเธเธนเนเธเนเธงเธข"</h2>
                                <p class="muted">
                                    "เธเธฒเธฃเน€เธเธดเธ”เน€เธเธขเธเธตเนเธเธฐเธ–เธนเธเธเธฑเธเธ—เธถเธเนเธฅเธฐเธกเธญเธเน€เธซเนเธเนเธ”เนเนเธ”เธขเธซเธฑเธงเธซเธเนเธฒ OSCC"
                                </p>

                                <label class="form-field">
                                    <span class="form-field__label">"เน€เธซเธ•เธธเธเธฅ"</span>
                                    <select
                                        class="form-field__input"
                                        on:change=move |ev| reason.set(event_target_value(&ev))
                                    >
                                        {["เธเธฒเธฃเธฃเธฑเธเธฉเธฒ", "เธเธฃเธฐเธชเธฒเธเธเธฒเธเธเธ”เธต", "เธเธงเธฒเธกเธเธฅเธญเธ”เธ เธฑเธข", "เธญเธทเนเธ เน"]
                                            .into_iter()
                                            .map(|value| view! { <option value=value>{value}</option> })
                                            .collect_view()}
                                    </select>
                                </label>

                                <label class="form-field">
                                    <span class="form-field__label">"เธซเธกเธฒเธขเน€เธซเธ•เธธ (เนเธกเนเธเธฑเธเธเธฑเธ)"</span>
                                    <input
                                        class="form-field__input"
                                        autocomplete="off"
                                        prop:value=move || note.get()
                                        on:input=move |ev| note.set(event_target_value(&ev))
                                    />
                                </label>

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
                                        on:click=move |_| do_reveal()
                                        disabled=move || busy.get()
                                    >
                                        "เธขเธทเธเธขเธฑเธเนเธฅเธฐเน€เธเธดเธ”เน€เธเธข"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                })
        }}
    }
}
