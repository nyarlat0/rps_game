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
use std::time::Duration;

use crate::api::fetch_user_info;
use crate::components::*;
use crate::hooks::*;
use crate::pages::*;

use fluent_templates::static_loader;
use leptos_fluent::leptos_fluent;

static_loader! {
    pub static TRANSLATIONS = {
        locales: "./locales",
        fallback_language: "en",
    };
}

#[component]
fn I18nProvider(children: Children) -> impl IntoView
{
    leptos_fluent! {
        children: children(),
        translations: [TRANSLATIONS],
        default_language: "en",
        sync_html_tag_lang: true,
        initial_language_from_navigator: true,
        set_language_to_local_storage: true,
        local_storage_key: "language",
        initial_language_from_local_storage: true,
        locales: "./locales",
    }
}

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

    let load_anim = Signal::derive(move || info.get().is_some());

    view! {
        <I18nProvider>
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

            <main class="cover-center vt-page">
            <AnimatedShow
                when=load_anim
                show_class="load-anim-in"
                hide_class="load-anim-out"
                hide_delay=Duration::from_millis(300)
            >
            {
                move || match info.get() {
                    // loading user info
                    None => view! {
                        <main class="cover-center vt-page">
                            <div class="loading-spinner"></div>
                        </main>
                    }.into_any(),

                    // user unauthenticated
                    Some(None) => view!{<UnAuthView />}.into_any(),

                    // user authenticated
                    Some(Some(user_info)) => view!{<AuthView user_info />}.into_any(),
                }
            }
            </AnimatedShow>
            </main>

            <footer>
            <Deck />
            </footer>

            <Forum />
            <Settings />
        </Router>
        </I18nProvider>
    }
}

#[component]
fn UnAuthView() -> impl IntoView
{
    view! {
        <Routes transition=true fallback=|| "Not found.">
            <Route path=path!("/login") view=Login />
            <Route path=path!("/register") view=Register />
            <Route path=path!("/about") view=About />
            <Route path=path!("/contact") view=Contact />
            <Route path=path!("/*any") view=Login />
        </Routes>
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
        <Routes transition=true fallback=|| "Not found.">
            <Route path=path!("/") view=AuthHome/>
            <Route path=path!("/login") view=|| {view! {<Redirect path="/" />}} />
            <Route path=path!("/register") view=|| {view! {<Redirect path="/" />}} />
            <Route path=path!("/about") view=About />
            <Route path=path!("/contact") view=Contact />
            <ParentRoute path=path!("/games") view=|| {view! {<Outlet />}} >
                <Route path=path!("") view=GamesHub />
                <Route path=path!("rps") view=RpsGame />
            </ParentRoute>
        </Routes>
    }
}
