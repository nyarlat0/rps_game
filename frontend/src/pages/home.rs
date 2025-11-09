use leptos::prelude::*;

use chrono::Local;
use leptos_use::core::ConnectionReadyState;
use leptos_use::use_interval_fn;
use shared::auth::UserInfo;
use shared::ws_messages::*;

use crate::hooks::WebsocketContext;

#[component]
pub fn AuthHome() -> impl IntoView
{
    let ws = expect_context::<WebsocketContext>();
    let user_info = expect_context::<UserInfo>();
    let (online_count, set_online_count) = signal::<u32>(0);

    Effect::new(move |_| {
        if let Some(msg) = ws.message.get() {
            if let ServerMsg::StatsMsg(stats_info) = msg {
                set_online_count.set(stats_info.online);
            }
        };
    });

    Effect::new({
        let ws = ws.clone();
        move |_| {
            if ws.state.get() == ConnectionReadyState::Open {
                ws.send(ClientMsg::GetStats);
            }
        }
    });

    use_interval_fn(move || {
                        if ws.state.get() == ConnectionReadyState::Open {
                            ws.send(ClientMsg::GetStats);
                        }
                    },
                    10_000);

    view! {
        <div class="stack fill-page card">

        <h1>"Dashboard"</h1>

        <h2>"Welcome, " {user_info.username} "!"</h2>
        <p style="color: var(--success);">"Users online: "{online_count}</p>
        <p>"Account created at: "{
            user_info.created_at
                .with_timezone(&Local)
                .format("%d.%m.%Y %H:%M").to_string()
        }</p>

        <a href = "/game" class="button" style ="margin-block-start: var(--s1);">
            "Play"
        </a>

        <form method="POST" action="/api/auth/logout" class="stack">
            <button type="submit" class="secondary destructive">"Logout"</button>
        </form>

        </div>
    }
}

#[component]
pub fn UnAuthHome() -> impl IntoView
{
    view! {
        <div class="stack fill-page card">

        <h1>"Welcome!"</h1>

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
