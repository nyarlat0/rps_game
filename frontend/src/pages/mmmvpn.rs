use crate::hooks::MyToaster;
use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_use::storage::*;

#[component]
pub fn MMMVPN() -> impl IntoView
{
    let (scam, set_scam, _) =
        use_local_storage::<bool, FromToStringCodec>("scammed");

    view! {
        <div class="card stack">
        {
            move || if !scam.get() {
                view! {<UnScammed set_scam=set_scam />}.into_any()
            } else {
                view! {<Scammed />}.into_any()
            }
        }
        </div>
    }
}

#[component]
fn UnScammed(set_scam: WriteSignal<bool>) -> impl IntoView
{
    let on_click = move |_| {
        set_scam.set(true);
    };

    view! {
        <h1>"Welcome to MMM VPN!"</h1>
        <h2>
            "Press the button to start using complitely free and secure VPN!"
        </h2>
        <button
        on:click=on_click
        >
            "Press me :3"
        </button>
    }
}

#[component]
fn Scammed() -> impl IntoView
{
    let toaster = MyToaster::new();
    toaster.success("Thanks a lot! Your data, property and soul now belong to me! Tee-hee :3");

    view! {
        <h1>"Haha! Get scammed, looser!"</h1>
        <h2>"Thanks for mining bitcoins for me from now on! :p"</h2>
    }
}
