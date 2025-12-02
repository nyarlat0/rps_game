use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_fluent::I18n;
use leptos_use::storage::use_local_storage;

use crate::hooks::{SettingsCtx, UserResCtx};

#[component]
pub fn Settings() -> impl IntoView
{
    let UserResCtx(user_res) = expect_context::<UserResCtx>();

    let role = move || {
        user_res.get()
                .flatten()
                .map(|ui| ui.role)
                .unwrap_or("user".to_string())
    };

    let (light, set_light, _) = use_local_storage::<i32, FromToStringCodec>("lightness");
    let (hue, set_hue, _) = use_local_storage::<i32, FromToStringCodec>("hue");

    let (admin, set_admin) = expect_context::<SettingsCtx>().admin_control;

    let (lang_open, set_lang_open) = signal(false);

    let toggle_lang = move |_| set_lang_open.update(|v| *v = !*v);

    let i18n = expect_context::<I18n>();
    let menu_items = move || {
        i18n.languages
            .iter()
            .map(|lang| {
                let _code = lang.id;
                let name = lang.name;

                view! {
                    <button
                        class="lang-item"
                        on:click=move |_| {
                            i18n.language.set(lang);
                            set_lang_open.set(false);
                        }
                    >
                        {name}
                    </button>
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div id="settings" popover="auto" class="stack fill-page">

            <h1>"Settings"</h1>

            <div
            class="cluster"
            style=move || if (role() == "admin") || (role() == "moderator") {
                "--cluster-align: baseline;"
            } else {
                "display: none;"
            }
            >
            <input
            name="admin" id="admin" type="checkbox"
            style="inline-size: 0.9rem; block-size: 0.9rem;"
            prop:checked=admin
            on:change=move |ev| {
                set_admin.set(event_target_checked(&ev));
            }
            />
            <label for="admin" style="font-weight: 700;">
                "Admin Controls"
            </label>
            </div>

            <div class="cluster">
            <svg class="icon" alt="Language" style="margin: 0;">
                <use href="/icons.svg#globe"></use>
            </svg>
            <div class="lang-switcher">

            <button on:click=toggle_lang class="lang-button" class:pressed=lang_open>
                { move || i18n.language.get().name }
            </button>

            <div class="lang-menu cluster" class:active=lang_open>
                {menu_items}
            </div>
            </div>
            </div>

            <label for="light" style="font-weight: 700;">
                {move || format!("Light: {:+}%", light.get())}
            </label>
            <div class="stack" style="--stack-gap: var(--s-2);">
            <input
            style="padding: 0;"
            name="light" id="light" type="range"
            min="-5" max="5" step="1"
            prop:value=light
            on:input=move |ev|{
                set_light.set(event_target_value(&ev).parse::<i32>().unwrap());
            }
            />
            </div>

            <label for="hue" style="font-weight: 700;">"Hue: +"{hue}"°"</label>
            <div class="stack" style="--stack-gap: var(--s-2);">
            <input
            style="padding: 0;"
            name="hue" id="hue" type="range"
            min="0" max="70" step="1"
            prop:value=hue
            on:input=move |ev|{
                set_hue.set(event_target_value(&ev).parse::<i32>().unwrap());
            }
            />
            </div>

            <div class="stack" style="--stack-gap: var(--s1)">
            <hr />
            <div class="cluster test-layout" style="--cluster-justify: space-evenly;">
            <div class="toaster-test" id="info-test">"Info"</div>
            <div class="toaster-test" id="success-test">"Success"</div>
            <div class="toaster-test" id="error-test">"Error"</div>
            </div>
            </div>

        </div>
    }
}
