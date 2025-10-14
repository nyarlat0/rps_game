use codee::string::{FromToStringCodec, JsonSerdeCodec};
use leptoaster::*;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use leptos_use::{
    storage::*, use_websocket, UseWebSocketReturn,
};
use shared::{auth::UserInfo, ws_messages::*};
use std::sync::Arc;

use crate::api::fetch_user_info;
use crate::components::*;
use crate::hooks::WebsocketContext;
use crate::pages::*;

#[component]
pub fn App() -> impl IntoView
{
    provide_toaster();
    let (visible_forum, set_visible_forum) = signal(false);

    let (light, set_light, _) = use_local_storage::<i32, FromToStringCodec>("lightness");
    let (hue, set_hue, _) =
        use_local_storage::<i32, FromToStringCodec>("hue");
    let info =
        LocalResource::new(move || fetch_user_info());
    provide_context(info);

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
            <NavBar visible_forum set_visible_forum/>
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
                    Some(None) => view!{<UnAuthView visible_forum />}.into_any(),

                    // user authenticated
                    Some(Some(user_info)) => view!{<AuthView visible_forum user_info />}.into_any(),
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

            <Settings light set_light hue set_hue/>
        </Router>
    }
}

#[component]
fn UnAuthView(visible_forum: ReadSignal<bool>)
              -> impl IntoView
{
    view! {
        <main class="cover-center">
        <Routes fallback=|| "Not found.">
            <Route path=path!("/login") view=Login />
            <Route path=path!("/register") view=Register />
            <Route path=path!("/*any") view=UnAuthHome />
        </Routes>
        </main>

        <Forum visible_forum />
    }
}

#[component]
fn AuthView(visible_forum: ReadSignal<bool>,
            user_info: UserInfo)
            -> impl IntoView
{
    let UseWebSocketReturn {
        ready_state,
        message,
        send,
        ..
    } = use_websocket::<ClientMsg, ServerMsg, JsonSerdeCodec>("/api/ws");

    provide_context(WebsocketContext::new(ready_state, message, Arc::new(send.clone())));
    provide_context(user_info);

    view! {
        <main class="cover-center">
        <Routes fallback=|| "Not found.">
            <Route path=path!("/") view=AuthHome/>
            <Route path=path!("/login") view=|| {view! {<Redirect path="/" />}} />
            <Route path=path!("/register") view=|| {view! {<Redirect path="/" />}} />
            <Route path=path!("/game") view=Game />
        </Routes>
        </main>

        <Forum visible_forum />
    }
}
