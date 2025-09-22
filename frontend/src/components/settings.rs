use leptos::prelude::*;

#[component]
pub fn Settings() -> impl IntoView
{
    view! {
        <div id="settings" popover="auto" class="stack fill-page">

            <h1>"Settings"</h1>

            <label for="light" style="font-weight: 700;">"Light:"</label>
            <div class="stack" style="--stack-gap: var(--s-2);">
            <input name="light" id="light" type="range" min="0" max="100" value="0" />
            </div>

            <label for="hue" style="font-weight: 700;">"Hue:"</label>
            <div class="stack" style="--stack-gap: var(--s-2);">
            <input name="hue" id="hue" type="range" min="0" max="100" value="0" />
            </div>

        </div>
    }
}
