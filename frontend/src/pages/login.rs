use crate::{
    api::login_user,
    hooks::{MyToaster, UserResCtx},
};
use leptos::{prelude::*, task::spawn_local};
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

    toaster.info("By logging in you argee to our cookie policy.");

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let creds = Credentials { username: username.get(),
                                  password: password.get() };

        let toaster = toaster.clone();
        let navigate = navigate.clone();

        spawn_local(async move {
            match login_user(&creds).await {
                Ok(msg) => {
                    info_resource.refetch();
                    toaster.success(&msg);
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
            <h1>"Log In"</h1>

            <label for="username">"Username:"</label>
            <div class="stack" style="--stack-gap: var(--s-1)">
            <input
                id="username"
                type="text"
                placeholder="Username"
                required=true
                prop:value=username
                on:input=move |ev|{
                    set_username.set(event_target_value(&ev));
                }
            />
            </div>
            <label for="password">"Password:"</label>
            <div class="stack" style="--stack-gap: var(--s-1)">
            <input
                id="password"
                type="password"
                placeholder="Password"
                required=true
                prop:value=password
                on:input=move |ev|{
                    set_password.set(event_target_value(&ev));
                }
            />
            </div>

            <div class="stack" style="--stack-gap: var(--s2)">
            <button type="submit">
                "Log in"
            </button>
            </div>

            <div class="stack">
            <div class="grid">
            <a href="/register" class="button">
                "Register"
            </a>
            <a href="/" class="button">
                "Home"
            </a>
            </div>
            </div>
        </form>
    }
}
