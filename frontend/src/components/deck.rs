use leptos::prelude::*;
use leptos_fluent::tr;

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
                    style="--cluster-justify: space-evenly; --cluster-gap: var(--s1)"
                >
                    <a href="/about" title=move || tr!("deck-about")>
                        <svg class="icon" style="--size: 2cap;" alt=move || tr!("deck-about")>
                            <use href="/icons.svg#info"></use>
                        </svg>
                    </a>
                    <a
                    href="https://github.com/nyarlat0/rps_game"
                    target="_blank"
                    rel="noopener noreferrer"
                    title=move || tr!("deck-source")
                    >
                        <svg class="icon" style="--size: 2cap;" alt=move || tr!("deck-source")>
                            <use href="/icons.svg#github"></use>
                        </svg>
                    </a>
                    <a href="/contact" title=move || tr!("deck-contact")>
                        <svg class="icon" style="--size: 2cap;" alt=move || tr!("deck-contact")>
                            <use href="/icons.svg#mail"></use>
                        </svg>
                    </a>
            </nav>
            </div>

            <For
            each=move || 1..10
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
