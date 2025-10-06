use codee::string::{FromToStringCodec, JsonSerdeCodec};
use leptoaster::*;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use leptos_use::{
    storage::*, use_websocket, UseWebSocketReturn,
};
use std::sync::Arc;

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

    let UseWebSocketReturn {
        message,
        send,
        ..
    } = use_websocket::<String,String, JsonSerdeCodec>("/api/ws");
    provide_context(WebsocketContext::new(message, Arc::new(send.clone())));

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

            <main class="cover-center">
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/register") view=Register />
                    <Route path=path!("/login") view=Login />
                    <Route path=path!("/game") view=Game />
                    <Route path=path!("/mmmvpn") view=MMMVPN />
                </Routes>
            </main>
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

            <Forum visible_forum />
            <Settings light set_light hue set_hue/>
        </Router>
    }
}
