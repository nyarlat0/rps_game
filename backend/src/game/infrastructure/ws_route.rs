use crate::auth::infrastructure::extract_username;
use crate::game::application::WsHandler;
use crate::game::domain::{Player, WsClosed, WsSession};
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_ws::{handle, Message, Session};
use async_trait::async_trait;
use futures_util::StreamExt;
use std::sync::Arc;

pub fn configure_game(cfg: &mut web::ServiceConfig)
{
    cfg.service(game_route);
}

impl From<actix_ws::Closed> for WsClosed
{
    fn from(_: actix_ws::Closed) -> Self
    {
        WsClosed
    }
}

#[async_trait]
impl WsSession for Session
{
    async fn send(&self, msg: &str)
                  -> Result<(), WsClosed>
    {
        Ok(self.clone().text(msg).await?)
    }
}

#[get("/start")]
pub async fn game_route(
    req: HttpRequest,
    stream: web::Payload,
    handler: web::Data<WsHandler>)
    -> actix_web::Result<HttpResponse>
{
    // Extract username from cookie (or use query param)
    let username = match extract_username(&req) {
        Some(name) => name,
        None => {
            return Ok(HttpResponse::Unauthorized().body("Not logged in!"));
        }
    };

    let (response, session, mut msg_stream) =
        handle(&req, stream)?;

    let player = Player { name: username,
                          session: Arc::new(session),
                          current_move: None };

    actix_rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    let _ = handler.handle_message(player.clone(), &text).await;
                }

                Message::Close(_) => {
                    let _ = handler.handle_disconnect(player.clone()).await;
                    break;
                }

                _ => {}
            }
        }
    });

    Ok(response)
}
