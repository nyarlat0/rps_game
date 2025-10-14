use std::collections::HashMap;

use actix::prelude::*;
use actix_web::{
    get, rt, web, HttpRequest, HttpResponse, Responder,
};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;
use shared::ws_messages::*;
use slab::Slab;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::time::{
    interval, Duration, Instant, MissedTickBehavior,
};
use uuid::Uuid;

use crate::application::auth_handler::AuthHandler;
use crate::infrastructure::auth::extract_id;

#[derive(Clone, Default)]
pub struct UsersActor
{
    users_online:
        HashMap<Uuid, Slab<UnboundedSender<ServerMsg>>>,
}

impl UsersActor
{
    pub fn new() -> Self
    {
        Self::default()
    }
}

impl Actor for UsersActor
{
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "usize")]
struct Joined
{
    tx: UnboundedSender<ServerMsg>,
    user_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Disconnected
{
    user_id: Uuid,
    conn_id: usize,
}

impl Handler<Joined> for UsersActor
{
    type Result = usize;
    fn handle(&mut self,
              msg: Joined,
              _ctx: &mut Self::Context)
              -> Self::Result
    {
        if self.users_online.contains_key(&msg.user_id) {
            let conns = self.users_online
                            .get_mut(&msg.user_id)
                            .unwrap();
            let conn_id = conns.insert(msg.tx);
            return conn_id;
        } else {
            let mut conns = Slab::new();
            let conn_id = conns.insert(msg.tx);
            self.users_online.insert(msg.user_id, conns);
            return conn_id;
        }
    }
}

impl Handler<Disconnected> for UsersActor
{
    type Result = ();
    fn handle(&mut self,
              msg: Disconnected,
              _ctx: &mut Self::Context)
              -> Self::Result
    {
        let conns = self.users_online
                        .get_mut(&msg.user_id)
                        .unwrap();
        conns.remove(msg.conn_id);

        if conns.is_empty() {
            self.users_online.remove(&msg.user_id);
        };
    }
}

#[derive(Message)]
#[rtype(result = "usize")]
struct GetOnline;

impl Handler<GetOnline> for UsersActor
{
    type Result = usize;
    fn handle(&mut self,
              _msg: GetOnline,
              _ctx: &mut Self::Context)
              -> Self::Result
    {
        self.users_online.len()
    }
}

pub fn configure_ws(cfg: &mut web::ServiceConfig)
{
    cfg.service(ws_route);
}

#[get("/ws")]
pub async fn ws_route(
    req: HttpRequest,
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

    let userinfo = match auth_handler.get_userinfo(user_id)
                                     .await
    {
        Ok(info) => info,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().body("Not logged in!"));
        }
    };

    let (response, mut session, stream) =
        actix_ws::handle(&req, body)?;

    let mut stream =
        stream.aggregate_continuations()
              // aggregate continuation frames up to 1MiB
              .max_continuation_size(2_usize.pow(20));

    let (tx, mut rx) =
        mpsc::unbounded_channel::<ServerMsg>();

    let conn_id = users_actor.send(Joined { tx, user_id })
                             .await
                             .unwrap();

    rt::spawn(async move {
        let mut hb = interval(Duration::from_secs(10));
        hb.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut last_pong = Instant::now();
        const CLIENT_TIMEOUT: Duration =
            Duration::from_secs(20);

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

        users_actor.do_send(Disconnected { conn_id,
                                           user_id });
        let _ = session.close(None).await;
    });

    return Ok(response);
}
