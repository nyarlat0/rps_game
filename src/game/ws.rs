use actix_rt::time::interval;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::{handle, Message, Session};
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use std::time::Duration;
use std::{collections::HashMap, sync::Mutex};

use super::auth::extract_username;
use super::logic::game_eval;

// Global queue for matchmaking
pub static WS_GAME_QUEUE: Lazy<Mutex<HashMap<String,
                                               Session>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub static WS_GAME_PLAY: Lazy<Mutex<Vec<GameInfo>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub struct GameInfo
{
    players: (String, String),
    sessions: (Session, Session),
    moves: (Option<String>, Option<String>),
}
/// WebSocket handler for matchmaking
pub async fn ws_join(req: HttpRequest,
                     stream: web::Payload)
                     -> actix_web::Result<HttpResponse>
{
    // Extract username from cookie (or use query param)
    let username = match extract_username(req.clone()) {
        Some(name) => name,
        None => {
            return Ok(HttpResponse::Unauthorized().body("Not logged in!"));
        }
    };

    // Corrected: `handle` now returns (response, session, message stream)
    let (response, mut session, mut msg_stream) =
        handle(&req, stream)?;

    let mut ping_session = session.clone();
    actix_rt::spawn(async move {
        let mut interval =
            interval(Duration::from_secs(10));
        loop {
            interval.tick().await;

            // Send ping and check if connection is alive
            if ping_session.ping(b"").await.is_err() {
                break;
            }
        }
    });

    // Spawn async task to handle messages
    actix_rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    if text == "join" {
                        let mut queue =
                            WS_GAME_QUEUE.lock().unwrap();

                        if let Some((waiting_user,
                                     waiting_session)) =
                            queue.clone().iter().next()
                        {
                            // We have an opponent! Generate a game ID.
                            if *waiting_user != username {
                                let response =
                                    format!("Playing against {}", waiting_user);
                                let waiting_response =
                                    format!("Playing against {}", username);
                                session.text(response)
                                       .await
                                       .ok();
                                waiting_session
                                    .clone()
                                    .text(waiting_response)
                                    .await.ok();
                                let mut save_game =
                                    WS_GAME_PLAY.lock()
                                                .unwrap();
                                save_game.push(GameInfo{
                                    players: (username.clone(), waiting_user.clone()),
                                    sessions: (session.clone(), waiting_session.clone()),
                                    moves: (None, None)
                                });
                                queue.remove(waiting_user);
                            } else {
                                session.text("Waiting for opponent..")
                                   .await
                                   .ok();
                            }
                        } else {
                            // No waiting player; add this username to the queue.
                            queue.insert(username.clone(),
                                         session.clone());
                            session.text("Waiting for opponent..")
                                   .await
                                   .ok();
                        }
                    }
                    if (text == "rock")
                       || (text == "paper")
                       || (text == "scissors")
                    {
                        let mut get_game =
                            WS_GAME_PLAY.lock().unwrap();

                        if let Some(game) =
                            get_game.iter_mut().find(|x| {
                                               x.players.0
                                               == username
                                           })
                        {
                            (*game).moves.0.get_or_insert(text.to_string());
                            session.text(format!("You played {}", text)).await.ok();
                        }
                        else if let Some(game) =
                            get_game.iter_mut().find(|x| {
                                               x.players.1
                                               == username
                                           })
                        {
                            (*game).moves.1.get_or_insert(text.to_string());
                            session.text(format!("You played {}", text)).await.ok();
                        }
                    }
                    if text == "result" {
                        let mut get_game =
                            WS_GAME_PLAY.lock().unwrap();

                        if let Some(game) =
                            get_game.iter_mut().find(|x| {
                                               x.players.0
                                               == username
                                           })
                        {
                            if let Some(ref move_0) = (*game).moves.0 {
                                if let Some (ref move_1) = (*game).moves.1 {
                                    let opponent = &(*game).players.1;
                                    let mut opp_session = (*game).sessions.1.clone();

                                    let result = game_eval(&move_0, &move_1);
                                    session.text(
                                        format!(
                                            "You played {}\n{} played {}\n{}",
                                            move_0,
                                            opponent,
                                            move_1,
                                            result,
                                            )
                                        ).await.ok();

                                    let result = game_eval(&move_1, &move_0);
                                    opp_session.text(
                                        format!(
                                            "You played {}\n{} played {}\n{}",
                                            move_1,
                                            username,
                                            move_0,
                                            result,
                                            )
                                        ).await.ok();

                                    get_game.retain(|x| x.players.0 != username);
                                }
                            }
                        }
                        else if let Some(game) =
                            get_game.iter_mut().find(|x| {
                                               x.players.1
                                               == username
                                           })
                        {
                            if let Some(ref move_0) = (*game).moves.1 {
                                if let Some (ref move_1) = (*game).moves.0 {
                                    let opponent = &(*game).players.0;
                                    let mut opp_session = (*game).sessions.0.clone();

                                    let result = game_eval(&move_0, &move_1);
                                    session.text(
                                        format!(
                                            "You played {}\n{} played {}\n{}",
                                            move_0,
                                            opponent,
                                            move_1,
                                            result,
                                            )
                                        ).await.ok();

                                    let result = game_eval(&move_1, &move_0);
                                    opp_session.text(
                                        format!(
                                            "You played {}\n{} played {}\n{}",
                                            move_1,
                                            username,
                                            move_0,
                                            result,
                                            )
                                        ).await.ok();

                                    get_game.retain(|x| x.players.1 != username);
                                }
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    let mut queue =
                        WS_GAME_QUEUE.lock().unwrap();

                    let mut games =
                        WS_GAME_PLAY.lock().unwrap();

                    // Try to find and message opponent before removing the game
                    if let Some(pos) = games.iter()
                                            .position(|g| {
                                                g.players.0
                                == username
                                || g.players.1
                                == username
                                            })
                    {
                        let game = games.remove(pos);
                        let (p1, _p2) = &game.players;
                        let mut s1 =
                            game.sessions.0.clone();
                        let mut s2 =
                            game.sessions.1.clone();

                        if p1 == &username {
                            s2.text("Opponent disconnected.").await.ok();
                        } else {
                            s1.text("Opponent disconnected.").await.ok();
                        }
                    }
                    queue.remove(&username);
                    println!("Closing websocket!!!");
                    session.close(None).await.ok();
                    break;
                } // Stop on close
                _ => {}
            }
        }
    });

    Ok(response)
}
