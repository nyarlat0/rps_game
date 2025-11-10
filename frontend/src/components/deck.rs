use leptos::prelude::*;

#[component]
pub fn Deck() -> impl IntoView
{
    view! {
        <div class="deck">
            <div
                class="deck-card upper-deck-card card fill-page site-footer"
                style="transform: translateY(30px); z-index: 5;"
            >
            <nav
                    aria-label="Footer"
                    class="cluster"
                    style="--cluster-justify: center; --cluster-gap: var(--s1)"
                >
                    <a href="/donate">"Donate"</a>
                    <a href="/about">"About"</a>
                    <a href="/contact">"Contact"</a>
            </nav>
            </div>

            <For
            each=move || (1..10)
            key=|i| *i
            children=move |i| {
                view! {
                    <div
                        class="card deck-card fill-page"
                        style=move || format!("
                        transform: translateY({}px);
                        ", i*3)
                    >
                    </div>
                }
            }
            />
        </div>
    }
}
