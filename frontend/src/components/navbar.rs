use leptos::prelude::*;
use leptos_use::{
    use_color_mode, use_cycle_list_with_options, ColorMode,
    UseColorModeReturn, UseCycleListOptions,
    UseCycleListReturn,
};

#[component]
pub fn NavBar(set_visible: WriteSignal<bool>)
              -> impl IntoView
{
    let UseColorModeReturn { mode, set_mode, .. } =
        use_color_mode();

    let UseCycleListReturn {next: next_theme, ..} = use_cycle_list_with_options(
        vec![ColorMode::Dark, ColorMode::Light],
        UseCycleListOptions::default().initial_value(Some((mode, set_mode).into())),
    );

    view! {
        <div class="cluster navbar" style="justify-content: space-between">
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

            <div class="cluster">
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
                on:click=move |_| set_visible.update(|value| *value = !*value)
            >
                <svg class="icon" alt="Toggle forum">
                    <use href="icons.svg#message-square"></use>
                </svg>
            </button>
            </div>
        </div>
    }
}
