use codee::string::{FromToStringCodec, JsonSerdeCodec};
use leptoaster::*;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use leptos_use::use_websocket_with_options;
use leptos_use::UseWebSocketOptions;
use leptos_use::{storage::*, use_websocket, UseWebSocketReturn};
use shared::{auth::UserInfo, ws_messages::*};
use std::sync::Arc;

use crate::api::fetch_user_info;
use crate::components::*;
use crate::hooks::*;
use crate::pages::*;

#[component]
pub fn App() -> impl IntoView
{
    provide_toaster();

    let info = LocalResource::new(move || fetch_user_info());
    provide_context(UserResCtx(info));

    let (new_posts, set_new_posts) = signal(false);
    let (visible_forum, set_visible_forum) = signal(false);

    provide_context(NavBarCtx { visible_forum: (visible_forum, set_visible_forum),
                                new_posts: (new_posts, set_new_posts) });

    let (admin, set_admin) = signal(false);

    provide_context(SettingsCtx { admin_control: (admin, set_admin) });

    let (authed, set_authed) = signal(false);
    provide_context(StateCtx { authed: (authed, set_authed) });

    let (light, _, _) = use_local_storage::<i32, FromToStringCodec>("lightness");
    let (hue, _, _) = use_local_storage::<i32, FromToStringCodec>("hue");

    let ws = use_websocket_with_options::<ClientMsg, ServerMsg, JsonSerdeCodec, _, _>
        ("/api/ws", UseWebSocketOptions::default().immediate(false));

    provide_context(WebsocketContext::new(ws.ready_state, ws.message, Arc::new(ws.send.clone())));

    Effect::new(move |_| {
        if authed.get() {
            (ws.open)();
        } else {
            (ws.close)();
        }
    });

    view! {
        <Toaster />
        <Router>
            <style>{
                move || format!(
                    ":root{{--set-light:{};--set-hue:{};}}",
                    light.get() as f32 / 100.0,
                    hue.get()
                )
            }</style>
            <header class="site-header">
            <NavBar />
            </header>

            {
                move || match info.get() {
                    // loading user info
                    None => view! {
                        <main class="cover-center">
                            <div class="loading-spinner"></div>
                        </main>
                    }.into_any(),

                    // user unauthenticated
                    Some(None) => view!{<UnAuthView />}.into_any(),

                    // user authenticated
                    Some(Some(user_info)) => view!{<AuthView user_info />}.into_any(),
                }
            }


            <footer class="site-footer">
                <nav
                    aria-label="Footer"
                    class="cluster"
                    style="--cluster-justify: flex-end; --cluster-gap: var(--s1)"
                >
                    <a href="/donate">"Donate"</a>
                    <a href="/about">"About"</a>
                    <a href="/contact">"Contact"</a>
                </nav>
            </footer>

            <Forum />
            <Settings />
        </Router>
    }
}

#[component]
fn UnAuthView() -> impl IntoView
{
    view! {
        <main class="cover-center">
        <Routes fallback=|| "Not found.">
            <Route path=path!("/login") view=Login />
            <Route path=path!("/register") view=Register />
            <Route path=path!("/*any") view=UnAuthHome />
        </Routes>
        </main>
    }
}

#[component]
fn AuthView(user_info: UserInfo) -> impl IntoView
{
    let UseWebSocketReturn { ready_state,
                             message,
                             send,
                             .. } =
        use_websocket::<ClientMsg, ServerMsg, JsonSerdeCodec>("/api/ws");

    provide_context(WebsocketContext::new(ready_state, message, Arc::new(send.clone())));
    provide_context(user_info);

    let (_, set_authed) = expect_context::<StateCtx>().authed;
    set_authed.set(true);

    view! {
        <main class="cover-center">
        <Routes fallback=|| "Not found.">
            <Route path=path!("/") view=AuthHome/>
            <Route path=path!("/login") view=|| {view! {<Redirect path="/" />}} />
            <Route path=path!("/register") view=|| {view! {<Redirect path="/" />}} />
            <Route path=path!("/game") view=Game />
        </Routes>
        </main>
    }
}
