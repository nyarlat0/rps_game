use crate::{api::register_user, hooks::MyToaster};
use leptos::{prelude::*, task::spawn_local};
use leptos_fluent::tr;
use leptos_router::hooks::use_navigate;
use shared::auth::Credentials;
use web_sys::SubmitEvent;

#[component]
pub fn Register() -> impl IntoView
{
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());

    let navigate = use_navigate();
    let toaster = MyToaster::new();

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let name = username.get();

        if name.chars().count() > 20 {
            toaster.error(&tr!("register-username-too-long"));
            return;
        };

        let creds = Credentials { username: name,
                                  password: password.get() };

        let toaster = toaster.clone();
        let navigate = navigate.clone();

        spawn_local(async move {
            match register_user(&creds).await {
                Ok(msg) => {
                    toaster.success(&msg);
                    navigate("/login", Default::default());
                }
                Err(msg) => {
                    toaster.error(&msg);
                }
            }
        });
    };

    view! {
        <form on:submit=on_submit class="stack fill-page card">
            <h1>{ move || tr!("register") }</h1>

            <label for="username">{ move || tr!("register-username-label") }</label>
            <div class="stack" style="--stack-gap: var(--s-1)">
            <input
                id="username"
                type="text"
                placeholder=move || tr!("register-username-placeholder")
                autocomplete="username"
                required=true
                prop:value=username
                on:input=move |ev|{
                    set_username.set(event_target_value(&ev));
                }
            />
            </div>
            <label for="password">{ move || tr!("register-password-label") }</label>
            <div class="stack" style="--stack-gap: var(--s-1)">
            <input
                id="password"
                type="password"
                placeholder=move || tr!("register-password-placeholder")
                autocomplete="new-password"
                required=true
                prop:value=password
                on:input=move |ev|{
                    set_password.set(event_target_value(&ev));
                }
            />
            </div>
            <div class="stack" style="--stack-gap: var(--s2); margin-top: auto;">
            <button type="submit">
                { move || tr!("register-submit") }
            </button>
            </div>

            <div class="stack">
            <a href="/login" class="button secondary">
                { move || tr!("register-login-link") }
            </a>
            </div>
        </form>
    }
}
