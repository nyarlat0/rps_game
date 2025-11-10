use leptos::prelude::*;
use shared::auth::UserInfo;

use crate::app::AdminToggleCtx;

#[component]
pub fn Settings(light: Signal<i32>,
                set_light: WriteSignal<i32>,
                hue: Signal<i32>,
                set_hue: WriteSignal<i32>)
                -> impl IntoView
{
    let role = use_context::<UserInfo>().map(|ui| ui.role)
                                        .unwrap_or("user".to_string());

    let AdminToggleCtx(admin, set_admin) = expect_context::<AdminToggleCtx>();

    view! {
        <div id="settings" popover="auto" class="stack fill-page">

            <h1>"Settings"</h1>

            <div
            class="cluster"
            style=move || if (role == "admin") || (role == "moderator") {
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
