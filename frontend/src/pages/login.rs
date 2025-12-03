use std::time::Duration;

use crate::{
    api::login_user,
    hooks::{MyToaster, UserResCtx},
};
use leptos::{prelude::*, task::spawn_local};
use leptos_fluent::tr;
use leptos_router::hooks::use_navigate;
use shared::auth::Credentials;
use web_sys::SubmitEvent;

#[component]
pub fn Login() -> impl IntoView
{
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());

    let navigate = use_navigate();
    let toaster = MyToaster::new();

    let UserResCtx(info_resource) = expect_context::<UserResCtx>();
    let cookie_msg = untrack(|| tr!("login-cookie-info"));

    Effect::new({
        let toaster = toaster.clone();
        let cookie_msg = cookie_msg.clone();

        move |_| {
            let toaster = toaster.clone();
            let cookie_msg = cookie_msg.clone();

            set_timeout(move || toaster.info(&cookie_msg),
                        Duration::from_millis(600));
        }
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let creds = Credentials { username: username.get(),
                                  password: password.get() };

        let toaster = toaster.clone();
        let navigate = navigate.clone();

        let success_msg = tr!("login-success");

        spawn_local(async move {
            match login_user(&creds).await {
                Ok(_msg) => {
                    info_resource.refetch();
                    toaster.success(&success_msg);
                    navigate("/", Default::default());
                }
                Err(msg) => {
                    toaster.error(&msg);
                }
            }
        });
    };

    view! {
        <form on:submit=on_submit class="stack fill-page card">
            <h1>{ move || tr!("login-title") }</h1>

            <label for="username">{ move || tr!("login-username-label") }</label>
            <div class="stack" style="--stack-gap: var(--s-1)">
            <input
                id="username"
                type="text"
                placeholder=move || tr!("login-username-placeholder")
                autocomplete="username"
                required=true
                prop:value=username
                on:input=move |ev|{
                    set_username.set(event_target_value(&ev));
                }
            />
            </div>
            <label for="password">{ move || tr!("login-password-label") }</label>
            <div class="stack" style="--stack-gap: var(--s-1)">
            <input
                id="password"
                type="password"
                placeholder=move || tr!("login-password-placeholder")
                autocomplete="current-password"
                required=true
                prop:value=password
                on:input=move |ev|{
                    set_password.set(event_target_value(&ev));
                }
            />
            </div>

            <div class="stack" style="--stack-gap: var(--s2); margin-top: auto;">
            <button type="submit">
                { move || tr!("login-submit") }
            </button>
            </div>

            <div class="stack">
            <a href="/register" class="button secondary">
                { move || tr!("login-register-link") }
            </a>
            </div>
        </form>
    }
}
