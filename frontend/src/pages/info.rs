use leptos::prelude::*;
use leptos_fluent::tr;

#[component]
pub fn About() -> impl IntoView
{
    view! {
        <div class="card stack fill-page">
            <h1>{ move || tr!("info-title") }</h1>
            <p>{ move || tr!("info-desc-1") }</p>
            <p>{ move || tr!("info-desc-2") }</p>
            <p>{ move || tr!("info-desc-3") }</p>
            <p>{ move || tr!("info-desc-4") }</p>
            <p>{ move || tr!("info-desc-5") }</p>
            <p>{ move || tr!("info-desc-6") }</p>
            <p>{ move || tr!("info-desc-7") }</p>
            <a href = "/" class="button secondary" style ="margin-block-start: var(--s2);">
                { move || tr!("games-hub-home") }
            </a>
        </div>
    }
}

#[component]
pub fn Contact() -> impl IntoView
{
    view! {
        <div class="card stack fill-page">
            <h1>{ move || tr!("contact-title") }</h1>
            <div class="cluster">
            <p>
                { move || tr!("contact-email-intro") }

            </p>
            <a
            href="mailto:nyarlat@nyarlat.org"
            class="mention-name"
            title=move || tr!("contact-email-title")
            >
                "nyarlat@nyarlat.org"
            </a>
            </div>
            <a href = "/" class="button secondary" style ="margin-block-start: var(--s1); margin-top: auto;">
                { move || tr!("games-hub-home") }
            </a>
        </div>
    }
}
