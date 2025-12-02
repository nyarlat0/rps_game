use leptos::prelude::*;
use leptos_fluent::tr;

#[component]
pub fn GamesHub() -> impl IntoView
{
    view! {
        <div class="stack fill-page card">
            <h1>{ move || tr!("games-hub-title") }</h1>
            <h2>{ move || tr!("games-hub-subtitle") }</h2>
            <a href = "/games/rps" class="button"
            style ="margin-block-start: var(--s1); margin-top: auto; margin-bottom: auto;"
            >
                { move || tr!("games-hub-rps") }
            </a>
            <a href = "/" class="button secondary" style ="margin-block-start: var(--s1); margin-top: auto;">
                { move || tr!("games-hub-home") }
            </a>
        </div>
    }
}
