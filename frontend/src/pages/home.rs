use leptos::prelude::*;
use leptos_fluent::{move_tr, tr};

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

        <h1>{ move || tr!("auth-home-title") }</h1>

        <h2>{ move_tr!("auth-home-welcome", {"username" => user_info.username.clone()}) }</h2>
        <p style="color: var(--success);">{ move_tr!("auth-home-online", {"count" => online_count.get()}) }</p>
        <p>{ move_tr!("auth-home-created-at",
            {"date" => user_info
                             .created_at
                             .with_timezone(&Local)
                             .format("%d.%m.%Y %H:%M")
                             .to_string()}) }</p>

        <a href = "/games" class="button" style ="margin-block-start: var(--s1); margin-top: auto;">
            { move || tr!("auth-home-play") }
        </a>

        <form method="POST" action="/api/auth/logout" class="stack">
            <button type="submit" class="secondary destructive">{ move || tr!("auth-home-logout") }</button>
        </form>

        </div>
    }
}
