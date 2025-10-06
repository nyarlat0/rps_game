use leptos::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct WebsocketContext
{
    pub message: Signal<Option<String>>,
    send: Arc<dyn Fn(&String) + Send + Sync>,
}

impl WebsocketContext
{
    pub fn new(message: Signal<Option<String>>,
               send: Arc<dyn Fn(&String) + Send + Sync>)
               -> Self
    {
        Self { message, send }
    }

    // create a method to avoid having to use parantheses around the field
    #[inline(always)]
    pub fn send(&self, message: &str)
    {
        (self.send)(&message.to_string())
    }
}
