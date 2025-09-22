use crate::api::fetch_user_info;
use leptos::prelude::*;
use shared::auth::UserInfo;

#[component]
pub fn Home() -> impl IntoView
{
    let info =
        LocalResource::new(move || fetch_user_info());
    view! {
        //<div class="cover center">
            {
                move || match info.get() {
                    Some(Some(user_info)) => view! {<AuthView user_info />}.into_any(),
                    Some(None) => view! {<UnAuthView />}.into_any(),
                    None => view! {<div class="loading-spinner"></div>}.into_any(),
                }
            }
        //</div>
    }
}

#[component]
pub fn AuthView(user_info: UserInfo) -> impl IntoView
{
    view! {
        <div class="card stack center">

        <h1 class="text-center">"Dashboard"</h1>

        <h2 class="text-center">"Welcome, " {user_info.username} "!"</h2>

        <a href = "/game" class="button">
            "Play"
        </a>

        <form method="POST" action="/api/auth/logout" class="stack">
            <button type="submit">"Logout"</button>
        </form>

        </div>
    }
}

#[component]
pub fn UnAuthView() -> impl IntoView
{
    view! {
        <div class="center stack card">

        <h1>"Dashboard"</h1>

        <h2>"Please log in or register"</h2>

        <a href="/login" class="button">
            "Login"
        </a>

        <a href = "/register" class="button">
            "Register"
        </a>

        </div>
    }
}
