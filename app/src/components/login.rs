//! Login screen: server address, username, password (DESIGN.md forms).

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::session::{Session, SessionState};

/// The first screen: nothing about a patient exists until sign-in succeeds.
#[component]
pub fn LoginScreen(state: SessionState) -> impl IntoView {
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let do_submit = move || {
        if busy.get() {
            return;
        }
        let base = state.api_base.get();
        let user = username.get();
        let pass = password.get();
        error.set(None);
        busy.set(true);
        spawn_local(async move {
            match api::login(&base, &user, &pass).await {
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
    };

    view! {
        <div class="auth-screen">
            <form
                class="auth-card"
                on:submit=move |ev| {
                    ev.prevent_default();
                    do_submit();
                }
            >
                <h1 class="auth-card__title">"OSCC"</h1>
                <p class="auth-card__subtitle">"ศูนย์พึ่งได้"</p>

                <label class="form-field">
                    <span class="form-field__label">"ที่อยู่เซิร์ฟเวอร์"</span>
                    <input
                        class="form-field__input"
                        type="url"
                        autocomplete="off"
                        prop:value=move || state.api_base.get()
                        on:input=move |ev| state.api_base.set(event_target_value(&ev))
                    />
                </label>

                <label class="form-field">
                    <span class="form-field__label">"ชื่อผู้ใช้"</span>
                    <input
                        class="form-field__input"
                        type="text"
                        autocomplete="off"
                        prop:value=move || username.get()
                        on:input=move |ev| username.set(event_target_value(&ev))
                    />
                </label>

                <label class="form-field">
                    <span class="form-field__label">"รหัสผ่าน"</span>
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
                    {move || if busy.get() { "กำลังเข้าสู่ระบบ..." } else { "เข้าสู่ระบบ" }}
                </button>
            </form>
        </div>
    }
}
