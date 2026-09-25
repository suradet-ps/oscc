//! Lock screen (DESIGN.md): no case data, password re-entry, sign out.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::session::{Session, SessionState, role_label};

/// Shown when the session is locked; unlocking re-enters the password.
#[component]
pub fn LockScreen(state: SessionState, session: Session) -> impl IntoView {
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let username = session.user.username.clone();
    let display_name = session.user.display_name.clone();
    let role = role_label(&session.user.role);

    let do_unlock = {
        let username = username.clone();
        move || {
            if busy.get() {
                return;
            }
            let base = state.api_base.get();
            let pass = password.get();
            error.set(None);
            busy.set(true);
            let username = username.clone();
            spawn_local(async move {
                match api::login(&base, &username, &pass).await {
                    Ok(response) => {
                        password.set(String::new());
                        state.sign_in(Session {
                            token: response.token,
                            user: response.user,
                            expires_at: response.expires_at,
                        });
                    }
                    Err(err) => error.set(Some(err.thai_message().to_string())),
                }
                busy.set(false);
            });
        }
    };

    let do_sign_out = {
        let token = session.token.clone();
        move || {
            let base = state.api_base.get();
            let token = token.clone();
            spawn_local(async move {
                let _ = api::logout(&base, &token).await;
                state.sign_out();
            });
        }
    };

    view! {
        <div class="auth-screen">
            <form
                class="auth-card"
                on:submit=move |ev| {
                    ev.prevent_default();
                    do_unlock();
                }
            >
                <h1 class="auth-card__title">"ล็อกอยู่"</h1>
                <p class="auth-card__subtitle">{display_name} " · " {role}</p>

                <label class="form-field">
                    <span class="form-field__label">"รหัสผ่านเพื่อปลดล็อก"</span>
                    <input
                        class="form-field__input"
                        type="password"
                        autocomplete="off"
                        prop:value=move || password.get()
                        on:input=move |ev| password.set(event_target_value(&ev))
                    />
                </label>

                {move || {
                    error
                        .get()
                        .map(|message| view! { <p class="form-error">{message}</p> })
                }}

                <button class="button-primary" type="submit" disabled=move || busy.get()>
                    "ปลดล็อก"
                </button>
                <button class="button-secondary" type="button" on:click=move |_| do_sign_out()>
                    "ออกจากระบบ"
                </button>
            </form>
        </div>
    }
}
