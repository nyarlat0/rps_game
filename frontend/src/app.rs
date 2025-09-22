use crate::components::*;
use crate::pages::*;
use leptoaster::*;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView
{
    provide_toaster();
    let (visible_forum, set_visible_forum) = signal(false);

    view! {
        <Toaster />
        <Router>
            <header class="site-header">
            <NavBar set_visible_forum/>
            </header>

            <main class="center cover-center">
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
                    <a>"Donate"</a>
                    <a>"About"</a>
                    <a>"Contact"</a>
                </nav>
            </footer>

            <Forum visible_forum />
            <Settings />
        </Router>
    }
}
