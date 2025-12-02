use leptos::prelude::*;
use leptos_use::{
    use_color_mode_with_options, use_cycle_list_with_options, ColorMode, UseColorModeOptions,
    UseColorModeReturn, UseCycleListOptions, UseCycleListReturn,
};
use leptos_fluent::tr;

use crate::hooks::NavBarCtx;

#[component]
pub fn NavBar() -> impl IntoView
{
    let navctx = expect_context::<NavBarCtx>();
    let (visible_forum, set_visible_forum) = navctx.visible_forum;
    let (new_posts, _) = navctx.new_posts;

    let UseColorModeReturn { mode, set_mode, .. } =
        use_color_mode_with_options(UseColorModeOptions::default().initial_value(ColorMode::Dark)
                                                                  .storage_enabled(true));

    let UseCycleListReturn {next: next_theme, ..} = use_cycle_list_with_options(
        vec![ColorMode::Dark, ColorMode::Light],
        UseCycleListOptions::default().initial_value(Some((mode, set_mode).into())),
    );

    view! {
        <nav
        class="cluster navbar"
        style="--cluster-justify: space-between"
        aria-label=move || tr!("navbar-primary")>
            <a href="/">
            { move || match mode.get() {
                ColorMode::Dark => view! {
                    <img class="icon" alt=move || tr!("navbar-logo-alt") src="/images/logo_light.png"/>
                }.into_any(),

                ColorMode::Light => view! {
                    <img class="icon" alt=move || tr!("navbar-logo-alt") src="/images/logo_dark.png"/>
                }.into_any(),

                _ => view! {}.into_any(),
            }
            }
            </a>

            <div
            class="cluster"
            style="--cluster-gap: 0;"
            >
            <button
                class="icon-btn navbar-icon"
                title=move || tr!("navbar-toggle-theme")
                aria-label=move || tr!("navbar-toggle-theme")
                on:click=move |_| next_theme()
            >
            { move || match mode.get() {
                ColorMode::Dark => view! {
                    <svg class="icon" alt=move || tr!("navbar-toggle-theme")>
                        <use href="/icons.svg#sun"></use>
                    </svg>
                }.into_any(),

                ColorMode::Light => view! {
                    <svg class="icon" alt=move || tr!("navbar-toggle-theme")>
                        <use href="/icons.svg#moon"></use>
                    </svg>
                }.into_any(),

                _ => view! {}.into_any(),
            }
            }
            </button>
            <button
                class="icon-btn navbar-icon"
                class:forum-btn-pressed=move || visible_forum.get()
                class:has-new=move || new_posts.get()
                title=move || tr!("navbar-toggle-forum")
                aria-label=move || tr!("navbar-toggle-forum")
                on:click=move |_| set_visible_forum.update(|value| *value = !*value)
            >
                <svg class="icon" alt=move || tr!("navbar-toggle-forum")>
                    <use href="/icons.svg#message-square"></use>
                </svg>
            </button>
            <button
                class="icon-btn navbar-icon"
                title=move || tr!("navbar-toggle-settings")
                aria-label=move || tr!("navbar-toggle-settings")
                popovertarget="settings"
            >
                <svg class="icon" alt=move || tr!("navbar-toggle-settings")>
                    <use href="/icons.svg#settings"></use>
                </svg>
            </button>
            </div>
        </nav>
    }
}
