use futures::{SinkExt, StreamExt};
use gloo_console::log;
use gloo_net::http::Request;
use gloo_net::websocket::{futures::WebSocket, Message};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Serialize, Deserialize)]
struct User
{
    id: i64,
    username: String,
    password: String, // Stored hashed password
}

#[derive(Serialize, Deserialize)]
struct RegisterRequest
{
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct LoginRequest
{
    username: String,
    password: String,
}

#[derive(Deserialize, Debug)]
struct ProtectedResponse
{
    user: String,
}

#[derive(Clone, Routable, PartialEq)]
enum Route
{
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/register")]
    Register,
    #[at("/game")]
    Game,
}

fn emoji(text: &str) -> String
{
    match text {
        "rock" => "✊ Rock".to_string(),
        "paper" => "✋ Paper".to_string(),
        "scissors" => "✌ Scissors".to_string(),
        other => other.to_string(),
    }
}

#[function_component(Home)]
fn home() -> Html
{
    let navigator = use_navigator().unwrap();

    let href = |x: Route| {
        let nav = navigator.clone();
        Callback::from(move |_| nav.push(&x))
    };

    let username = use_state(|| None::<String>);
    {
        let username = username.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let response = Request::get("/api/protected")
                    .credentials(web_sys::RequestCredentials::Include)
                    .send()
                    .await;

                match response {
                    Ok(resp) => match resp.json::<ProtectedResponse>().await {
                        Ok(data) => {
                            username.set(Some(data.user))
                        },
                        Err(_) => username.set(None),
                    },
                    Err(_) => username.set(None),
                }
            });
            || ()
        });
    }

    let logout_but = {
        let username = username.clone();
        Callback::from(move |_| {
            let username = username.clone();
            spawn_local(async move {
                let response = Request::get("/api/logout")
                    .credentials(web_sys::RequestCredentials::Include)
                    .send()
                    .await;

                match response {
                    Ok(_) => username.set(None),
                    Err(_) => (),
                }
            })
        })
    };

    html! {
        <div>
            <h1>{"Welcome to Dashboard!"}</h1>
        {
            match &*username {
                    Some(name) => html! {
                        <div>
                            <p><h2>{ format!("Welcome, {}!", name) }</h2></p>
                            <button onclick={href(Route::Game)}>{ "Play" }</button>
                            <button onclick={logout_but}>{ "Logout" }</button>
                        </div>
                    },
                    None => html! {
                        <div>
                            <button onclick={href(Route::Register)}>{ "Register" }</button>
                            <button onclick={href(Route::Login)}>{ "Login" }</button>
                        </div>
                    },
            }
        }
        </div>
    }
}

#[function_component(Register)]
fn register() -> Html
{
    let name = use_state(|| String::new());
    let password = use_state(|| String::new());
    let message = use_state(|| String::new());

    let navigator = use_navigator().unwrap();

    let href = |x: Route| {
        let nav = navigator.clone();
        Callback::from(move |_| nav.push(&x))
    };

    let on_input = |x: UseStateHandle<String>| {
        let x = x.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement =
                e.target_unchecked_into();
            x.set(input.value());
        })
    };

    let on_submit = {
        let name = name.clone();
        let password = password.clone();
        let message = message.clone();
        let navigator = use_navigator().unwrap();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let username = (*name).trim().to_string();
            let password = (*password).trim().to_string();
            if username.is_empty() || password.is_empty() {
                return;
            };
            let new_user =
                RegisterRequest { username, password };
            let message = message.clone();
            let navigator = navigator.clone();

            spawn_local(async move {
                let req = Request::post("/api/register")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&new_user).unwrap())
                    .unwrap();
                let response = req.send().await;
                match response {
                    Ok(resp) => {
                        match resp.text().await {
                            Ok(text) => {
                                if text == "User registered!"{
                                    message.set(text);
                                    navigator.push(&Route::Login);
                                } else {
                                    message.set(text);
                                }
                            },
                            Err(e) => message.set(format!("Error reading response: {:?}", e)),
                        }
                    }
                    Err(e) => {
                        message.set(format!("Request failed: {:?}", e));
                    }
                };
            });
        })
    };

    html! {
    <div>
        <h1>{ "Register" }</h1>
        <pre><h2>{ (*message).clone() }</h2></pre>
        <form onsubmit={on_submit}>
        <label for="username">{ "Username: " }</label>
        <input type="text" id="username" value={(*name).clone()} oninput={on_input(name)} />
        <label for="password">{ "Password: " }</label>
        <input type="text" id="password" value={(*password).clone()} oninput={on_input(password)} />
        <button type="submit">
            {"Register"}
        </button>
        </form>
        <button onclick={href(Route::Home)}>{ "Home" }</button>
    </div>}
}

#[function_component(Login)]
fn login() -> Html
{
    let name = use_state(|| String::new());
    let password = use_state(|| String::new());
    let message = use_state(|| String::new());

    let navigator = use_navigator().unwrap();

    let href = |x: Route| {
        let nav = navigator.clone();
        Callback::from(move |_| nav.push(&x))
    };

    let on_input = |x: UseStateHandle<String>| {
        let x = x.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement =
                e.target_unchecked_into();
            x.set(input.value());
        })
    };

    let on_submit = {
        let name = name.clone();
        let password = password.clone();
        let message = message.clone();
        let navigator = use_navigator().unwrap();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let username = (*name).trim().to_string();
            let password = (*password).trim().to_string();
            if username.is_empty() || password.is_empty() {
                return;
            };
            let log_user =
                LoginRequest { username, password };
            let message = message.clone();
            let navigator = navigator.clone();

            spawn_local(async move {
                let req = Request::post("/api/login")
                    .credentials(web_sys::RequestCredentials::Include)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&log_user).unwrap())
                    .unwrap();
                let response = req.send().await;
                match response {
                    Ok(resp) => {
                        match resp.text().await {
                            Ok(text) => {
                                if text == "Login successful!"{
                                    message.set(text);
                                    navigator.push(&Route::Home);
                                } else {
                                    message.set(text);
                                }
                            },
                            Err(e) => message.set(format!("Error reading response: {:?}", e)),
                        }
                    }
                    Err(e) => {
                        message.set(format!("Request failed: {:?}", e));
                    }
                };
            });
        })
    };

    html! {
    <div>
        <h1>{ "Log In" }</h1>
        <pre><h2>{ (*message).clone() }</h2></pre>
        <form onsubmit={on_submit}>
        <label for="username">{ "Username: " }</label>
        <input type="text" id="username" value={(*name).clone()} oninput={on_input(name)} />

        <label for="password">{ "Password: " }</label>
        <input type="text" id="password" value={(*password).clone()} oninput={on_input(password)} />

        <button type="submit">
            {"Log in"}
        </button>
        </form>
        <button onclick={href(Route::Home)}>{ "Home" }</button>
    </div>}
}

#[function_component(Game)]
fn game() -> Html
{
    let message = use_state(|| String::new());
    // State to store the MPSC sender for outbound messages.
    let outbound_tx = use_state(|| None::<Sender<String>>);

    {
        let message = message.clone();
        let outbound_tx = outbound_tx.clone();
        // Effect to open WebSocket connection on component mount.
        use_effect_with((), move |_| {
            let (tx, mut rx) = mpsc::channel::<String>(100);
            outbound_tx.set(Some(tx)); // Store the sender for later use.

            // Open the WebSocket connection.
            let ws = WebSocket::open("wss://nyarlat.org/api/game/start")
                    .expect("Failed to connect to WebSocket");

            let (mut sink, stream) = ws.split();

            let send_fut = async move {
                sink.send(Message::Text("join".to_string()))
                .await
                .ok();
                while let Some(msg) = rx.recv().await {
                    sink.send(Message::Text(msg))
                        .await
                        .ok();
                }
            };

            spawn_local(async move {
                send_fut.await;
            });

            // Process incoming messages.

            let (cancel_tx, mut cancel_rx) =
                oneshot::channel();

            let read_fut = async move {
                let mut stream = stream;
                loop {
                    select! {
                        _ = &mut cancel_rx => break,
                        msg = stream.next() => {
                            if let Some(Ok(Message::Text(text))) = msg {
                                message.set(text);
                            }
                        }
                    }
                }
                log!("Closing!");
            };

            spawn_local(async move {
                read_fut.await;
            });

            move || {
                cancel_tx.send(()).ok();
            }
        });
    }

    // Callback to send a message using the stored sender.
    let send_message = move |x: String| {
        let outbound_tx = outbound_tx.clone();
        Callback::from(move |_| {
            let x = x.clone();
            if let Some(tx) = &*outbound_tx {
                let tx = tx.clone();
                spawn_local(async move {
                    tx.send(x).await.ok();
                    tx.send("result".to_string())
                      .await
                      .ok();
                });
            }
        })
    };

    let navigator = use_navigator().unwrap();

    let href = |x: Route| {
        let nav = navigator.clone();
        Callback::from(move |_| nav.push(&x))
    };

    let playing_msg = Regex::new("Playing*").unwrap();

    let parsed_message = {
        let raw = (*message).clone(); // clone is cheap here
        let tokens: Vec<&str> =
            raw.split_whitespace().collect();
        if tokens.len() >= 7
           && tokens[0] == "You"
           && tokens[1] == "played"
        {
            let you = tokens[2].to_string();
            let abobus = tokens[5].to_string();
            let result = tokens[6..].join(" ");
            Some((you, abobus, result))
        } else {
            None
        }
    };

    html! {
        <div>
            <h1>{ "Rock-Paper-Scissors" }</h1>
        {
            if let Some((you, abobus, result)) = parsed_message {
                html! {
                    <div class="message-box">
                        <p>{ format!("🧍 You:     {}", emoji(&you)) }</p>
                        <p>{ format!("🤖 Abobus: {}", emoji(&abobus)) }</p>
                        <p class="result">{ format!("🏆 Result: {}", result) }</p>
                        </div>
                }
            } else {
                html! { <p><h2>{ (*message).clone() }</h2></p> }
            }
        }
        {
            if playing_msg.is_match(&message){
                html!{
                    <div>
                        <button onclick={send_message("rock".to_string())}>{ "✊ Rock" }</button>
                        <button onclick={send_message("paper".to_string())}>{ "✋ Paper" }</button>
                        <button onclick={send_message("scissors".to_string())}>{ "✌ Scissors" }</button>
                        </div>
                }
            } else {html! {}

            }
        }
        <button onclick={href(Route::Home)}>{ "Home" }</button>
            </div>
    }
}

fn switch(routes: Route) -> Html
{
    match routes {
        Route::Home => html! {
            <Home />
        },
        Route::Register => html! {
            <Register />
        },
        Route::Login => html! {
            <Login />
        },
        Route::Game => html! {
            <Game />
        },
    }
}

#[function_component(App)]
fn app() -> Html
{
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} /> // <- must be child of <BrowserRouter>
            </BrowserRouter>
    }
}

fn main()
{
    yew::Renderer::<App>::new().render();
}
