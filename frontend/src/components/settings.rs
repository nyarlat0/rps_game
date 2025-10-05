use leptos::prelude::*;

#[component]
pub fn Settings(light: Signal<i32>,
                set_light: WriteSignal<i32>,
                hue: Signal<i32>,
                set_hue: WriteSignal<i32>)
                -> impl IntoView
{
    view! {
        <div id="settings" popover="auto" class="stack fill-page">

            <h1>"Settings"</h1>

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
