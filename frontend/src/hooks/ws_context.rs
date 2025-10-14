use leptos::prelude::*;
use leptos_use::core::ConnectionReadyState;
use shared::ws_messages::*;
use std::sync::Arc;

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
