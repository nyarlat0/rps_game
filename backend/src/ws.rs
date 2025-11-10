use actix::prelude::*;
use actix_web::{get, rt, web, HttpRequest, HttpResponse, Responder};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;
use shared::ws_messages::*;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Instant, MissedTickBehavior};

use crate::application::auth_handler::AuthHandler;
use crate::domain::users_actor::*;
use crate::infrastructure::auth::extract_id;

#[get("/ws")]
pub async fn ws_route(req: HttpRequest,
                      body: web::Payload,
                      users_actor: web::Data<Addr<UsersActor>>,
                      auth_handler: web::Data<AuthHandler>)
                      -> actix_web::Result<impl Responder>
{
    let user_id = match extract_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().body("Not logged in!"));
        }
    };

    let _userinfo = match auth_handler.get_userinfo(user_id).await {
        Ok(info) => info,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().body("Not logged in!"));
        }
    };

    let (response, mut session, stream) = actix_ws::handle(&req, body)?;

    let mut stream = stream.aggregate_continuations()
                           // aggregate continuation frames up to 1MiB
                           .max_continuation_size(2_usize.pow(20));

    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMsg>();

    let conn_id = users_actor.send(Joined { tx, user_id }).await.unwrap();

    rt::spawn(async move {
        let mut hb = interval(Duration::from_secs(10));
        hb.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut last_pong = Instant::now();
        const CLIENT_TIMEOUT: Duration = Duration::from_secs(20);

        loop {
            tokio::select! {
                _ = hb.tick() => {
                    if last_pong.elapsed() > CLIENT_TIMEOUT {
                        break;
                    }
                    if session.ping(b"hb").await.is_err() {
                        break;
                    }
                }

                maybe_msg = stream.next() => {
                    match maybe_msg {
                        Some(Ok(msg)) => {
                            match msg {
                                AggregatedMessage::Ping(bytes) => {
                                    if session.pong(&bytes).await.is_err() {
                                        break;
                                    }
                                }

                                AggregatedMessage::Pong(_) => {
                                    last_pong = Instant::now();
                                }

                                AggregatedMessage::Text(text) => {
                                    if let Ok(msg) = serde_json::from_str::<ClientMsg>(&text) {
                                        match msg {
                                            ClientMsg::GetStats => {
                                                let online = users_actor.send(GetOnline).await.unwrap();
                                                let serv_msg = ServerMsg::StatsMsg(StatsInfo{ online: online as u32});
                                                let out = serde_json::to_string(&serv_msg).unwrap();
                                                if session.text(out).await.is_err() { break }
                                            },
                                            _=>{},
                                        }

                                    } else {session.text("Wrong command!").await.ok();}
                                }

                                AggregatedMessage::Close(_) => break,

                                _ => break,
                            }
                        }
                        _ => break,
                    }
                }

                maybe_msg = rx.recv() => {
                    if let Some(msg) = maybe_msg {
                        let out = serde_json::to_string(&msg).unwrap();
                        if session.text(out).await.is_err(){ break }
                    }
                }
            }
        }

        users_actor.do_send(Disconnected { conn_id, user_id });
        let _ = session.close(None).await;
    });

    return Ok(response);
}
