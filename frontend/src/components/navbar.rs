use leptos::prelude::*;
use leptos_use::{
    use_color_mode, use_cycle_list_with_options, ColorMode,
    UseColorModeReturn, UseCycleListOptions,
    UseCycleListReturn,
};

#[component]
pub fn NavBar(set_visible_forum: WriteSignal<bool>)
              -> impl IntoView
{
    let UseColorModeReturn { mode, set_mode, .. } =
        use_color_mode();

    let UseCycleListReturn {next: next_theme, ..} = use_cycle_list_with_options(
        vec![ColorMode::Dark, ColorMode::Light],
        UseCycleListOptions::default().initial_value(Some((mode, set_mode).into())),
    );

    view! {
        <nav
        class="cluster navbar"
        style="--cluster-justify: space-between"
        aria-label="Primary">
            <a href="/">
            { move || match mode.get() {
                ColorMode::Dark => view! {
                    <img class="icon" alt="Logo" src="images/logo_light.png"/>
                }.into_any(),

                ColorMode::Light => view! {
                    <img class="icon" alt="Logo" src="images/logo_dark.png"/>
                }.into_any(),

                _ => view! {}.into_any(),
            }
            }
            </a>

            //<div class="loading-spinner"></div>

            <div
            class="cluster"
            style="--cluster-gap: 0;"
            >
            <button
                class="icon-btn"
                title="Toggle theme"
                aria-label="Toggle theme"
                on:click=move |_| next_theme()
            >
            { move || match mode.get() {
                ColorMode::Dark => view! {
                    <svg class="icon" alt="Toggle theme">
                        <use href="icons.svg#sun"></use>
                    </svg>
                }.into_any(),

                ColorMode::Light => view! {
                    <svg class="icon" alt="Toggle theme">
                        <use href="icons.svg#moon"></use>
                    </svg>
                }.into_any(),

                _ => view! {}.into_any(),
            }
            }
            </button>
            <button
                class="icon-btn"
                title="Toggle forum"
                aria-label="Toggle forum"
                on:click=move |_| set_visible_forum.update(|value| *value = !*value)
            >
                <svg class="icon" alt="Toggle forum">
                    <use href="icons.svg#message-square"></use>
                </svg>
            </button>
            <button
                class="icon-btn"
                title="Toggle settings"
                aria-label="Toggle settings"
                popovertarget="settings"
            >
                <svg class="icon" alt="Toggle settings">
                    <use href="icons.svg#settings"></use>
                </svg>
            </button>
            </div>
        </nav>
    }
}
