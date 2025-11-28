use actix::prelude::*;
use actix_web::{get, rt, web, HttpRequest, HttpResponse, Responder};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;
use shared::{rps_game::RpsGameReq, ws_messages::*};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Instant, MissedTickBehavior};
use uuid::Uuid;

use crate::application::auth_handler::AuthHandler;
use crate::application::game_handler::GameHandler;
use crate::domain::rps_model::RpsGame;
use crate::domain::users_actor::{self, UsersActor};
use crate::infrastructure::auth::extract_id;

#[get("/ws")]
pub async fn ws_route(req: HttpRequest,
                      body: web::Payload,
                      rps_handler: web::Data<GameHandler<RpsGame>>,
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

    let userinfo = match auth_handler.get_userinfo(user_id).await {
        Ok(info) => info,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().body("Not logged in!"));
        }
    };

    let username = userinfo.username;

    let (response, mut session, stream) = actix_ws::handle(&req, body)?;

    let mut stream = stream.aggregate_continuations()
                           // aggregate continuation frames up to 1MiB
                           .max_continuation_size(2_usize.pow(20));

    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMsg>();

    let conn_id = users_actor.send(users_actor::Joined { tx,
                                                         user_id,
                                                         username })
                             .await
                             .unwrap();

    let gh = rps_handler.clone();

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
                                    if !handle_client_text(
                                        text.to_string(),
                                        user_id,
                                        &users_actor,
                                        &gh,
                                        &mut session)
                                        .await {break;}
                                },
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

        users_actor.do_send(users_actor::Disconnected { conn_id, user_id });
        // Drop from queue / game if still present.
    });

    return Ok(response);
}

async fn handle_client_text(text: String,
                            user_id: Uuid,
                            users_actor: &Addr<UsersActor>,
                            rps_handler: &GameHandler<RpsGame>,
                            session: &mut actix_ws::Session)
                            -> bool
{
    let parsed = match serde_json::from_str::<ClientMsg>(&text) {
        Ok(m) => m,
        Err(_) => {
            // best-effort error reply, but don't break connection
            let _ = session.text("Wrong command!").await;
            return true;
        }
    };

    match parsed {
        ClientMsg::GetStats => {
            let online = users_actor.send(users_actor::GetOnline).await.unwrap();

            let serv_msg = ServerMsg::StatsMsg(StatsInfo { online: online as u32 });
            let out = serde_json::to_string(&serv_msg).unwrap();
            session.text(out).await.is_ok()
        }
        ClientMsg::RpsGameMsg(game_req) => match game_req {
            RpsGameReq::Start => {
                if let Err(err) = rps_handler.join(user_id).await {
                    let msg = ServerMsg::GameErrorMsg(err);
                    let out = serde_json::to_string(&msg).unwrap();
                    session.text(out).await.is_ok()
                } else {
                    true
                }
            }

            RpsGameReq::Submit(mv) => {
                if let Err(err) = rps_handler.submit(user_id, mv).await {
                    let msg = ServerMsg::GameErrorMsg(err);
                    let out = serde_json::to_string(&msg).unwrap();
                    session.text(out).await.is_ok()
                } else {
                    true
                }
            }

            RpsGameReq::Leave => {
                if let Err(err) = rps_handler.leave(user_id).await {
                    let msg = ServerMsg::GameErrorMsg(err);
                    let out = serde_json::to_string(&msg).unwrap();
                    session.text(out).await.is_ok()
                } else {
                    true
                }
            }
        },
        _ => true,
    }
}
