use leptos::prelude::*;

#[component]
pub fn Forum(visible: ReadSignal<bool>) -> impl IntoView
{
    let (message, set_message) = signal(String::new());

    let on_submit = move |_| {
        let msg = message.get().trim().to_string();
        if !msg.is_empty() {
            // TODO: send to backend
            set_message.set("".to_string());
        }
    };

    view! {
        <div
            class="card stack forum"
            class:is-open=move || visible.get()
        >
            <h3>"Forum"</h3>

            <textarea
                name="message"
                placeholder="Write your message..."
                prop:value=move || message.get()
                on:input=move |ev| set_message.set(event_target_value(&ev))
                />
                <button on:click=on_submit>"Send"</button>
        </div>
    }
}
