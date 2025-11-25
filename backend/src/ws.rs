use actix::prelude::*;
use actix_web::{get, rt, web, HttpRequest, HttpResponse, Responder};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;
use shared::game::GameReq;
use shared::ws_messages::*;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Instant, MissedTickBehavior};
use uuid::Uuid;

use crate::application::auth_handler::AuthHandler;
use crate::domain::users_actor::{self, UsersActor};
use crate::infrastructure::auth::extract_id;
use crate::infrastructure::game::GameHandler;

#[get("/ws")]
pub async fn ws_route(req: HttpRequest,
                      body: web::Payload,
                      game_handler: web::Data<GameHandler>,
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

    let conn_id = users_actor.send(users_actor::Joined { tx, user_id })
                             .await
                             .unwrap();

    rt::spawn(async move {
        let mut hb = interval(Duration::from_secs(10));
        hb.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut last_pong = Instant::now();
        const CLIENT_TIMEOUT: Duration = Duration::from_secs(20);

        let mut current_game = None::<usize>;
        let mut current_opp = None::<Uuid>;

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
                                        &mut current_opp,
                                        &mut current_game,
                                        user_id,
                                        &users_actor,
                                        &game_handler,
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
        let _ = session.close(None).await;
    });

    return Ok(response);
}

async fn handle_client_text(text: String,
                            current_opp: &mut Option<Uuid>,
                            current_game: &mut Option<usize>,
                            user_id: Uuid,
                            users_actor: &Addr<UsersActor>,
                            game_handler: &GameHandler,
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
        ClientMsg::GameMsg(game_req) => match game_req {
            GameReq::Start => {
                if let Some((game_id, opp_id)) = game_handler.start(user_id).await {
                    *current_game = Some(game_id);
                    *current_opp = Some(opp_id);

                    users_actor.do_send(users_actor::SendToUser {
                        user_id: opp_id,
                        msg: ServerMsg::InternalGameMsg((game_id, opp_id))
                    });
                }
                true
            }
            GameReq::Submit(mv) => true,
            _ => true,
        },
        _ => true,
    }
}
