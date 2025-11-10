use leptos::prelude::*;
use leptos_use::core::ConnectionReadyState;
use shared::{auth::UserInfo, ws_messages::*};
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct NavBarCtx
{
    pub visible_forum: (ReadSignal<bool>, WriteSignal<bool>),
    pub new_posts: (ReadSignal<bool>, WriteSignal<bool>),
}

#[derive(Clone, Copy)]
pub struct SettingsCtx
{
    pub admin_control: (ReadSignal<bool>, WriteSignal<bool>),
}

#[derive(Clone, Copy)]
pub struct StateCtx
{
    pub authed: (ReadSignal<bool>, WriteSignal<bool>),
}

#[derive(Clone, Copy)]
pub struct UserResCtx(pub LocalResource<Option<UserInfo>>);

#[derive(Clone)]
pub struct WebsocketContext
{
    pub state: Signal<ConnectionReadyState>,
    pub message: Signal<Option<ServerMsg>>,
    send: Arc<dyn Fn(&ClientMsg) + Send + Sync>,
}

impl WebsocketContext
{
    pub fn new(state: Signal<ConnectionReadyState>,
               message: Signal<Option<ServerMsg>>,
               send: Arc<dyn Fn(&ClientMsg) + Send + Sync>)
               -> Self
    {
        Self { state,
               message,
               send }
    }

    // create a method to avoid having to use parantheses around the field
    #[inline(always)]
    pub fn send(&self, message: ClientMsg)
    {
        (self.send)(&message)
    }
}
