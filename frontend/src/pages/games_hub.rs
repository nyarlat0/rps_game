use leptos::prelude::*;

#[component]
pub fn GamesHub() -> impl IntoView
{
    view! {
        <div class="stack fill-page card">
            <h1>"Games"</h1>
            <h2>"Choose game to play:"</h2>
            <a href = "/games/rps" class="button"
            style ="margin-block-start: var(--s1); margin-top: auto; margin-bottom: auto;"
            >
                "Rock-Paper-Scissors"
            </a>
            <a href = "/" class="button secondary" style ="margin-block-start: var(--s1); margin-top: auto;">
                "Home"
            </a>
        </div>
    }
}
