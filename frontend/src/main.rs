use leptos::mount::mount_to_body;
mod api;
mod app;
mod components;
mod hooks;
mod pages;
use app::App;

fn main()
{
    mount_to_body(App);
}
