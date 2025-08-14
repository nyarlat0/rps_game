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
    let (visible, set_visible) = signal(false);

    view! {
        <Toaster />
        <Router>
            <div class="navbar-overlay">
            <NavBar set_visible />
            <Forum visible />
            </div>
            <main>
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/register") view=Register />
                    <Route path=path!("/login") view=Login />
                    <Route path=path!("/game") view=Game />
                    <Route path=path!("/mmmvpn") view=MMMVPN />
                </Routes>
            </main>
        </Router>
    }
}
