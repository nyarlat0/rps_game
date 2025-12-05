use std::collections::HashMap;

use actix::prelude::*;
use shared::ws_messages::*;
use slab::Slab;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct UsersActor
{
    pub users_online: HashMap<Uuid, Slab<UnboundedSender<ServerMsg>>>,
    pub user_names: HashMap<Uuid, String>,
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

// ---- Mesages for UsersActor

#[derive(Message)]
#[rtype(result = "usize")]
pub struct Joined
{
    pub tx: UnboundedSender<ServerMsg>,
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnected
{
    pub user_id: Uuid,
    pub conn_id: usize,
}

#[derive(Message)]
#[rtype(result = "usize")]
pub struct GetOnline;

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendToUser
{
    pub user_id: Uuid,
    pub msg: ServerMsg,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Broadcast
{
    pub msg: ServerMsg,
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct IsOnline
{
    pub user_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "Option<String>")]
pub struct GetName
{
    pub user_id: Uuid,
}

// ---- Handlers for UsersActor

impl Handler<Joined> for UsersActor
{
    type Result = usize;
    fn handle(&mut self, msg: Joined, _ctx: &mut Self::Context) -> Self::Result
    {
        if self.users_online.contains_key(&msg.user_id) {
            let conns = self.users_online.get_mut(&msg.user_id).unwrap();
            let conn_id = conns.insert(msg.tx);
            return conn_id;
        } else {
            let mut conns = Slab::new();
            let conn_id = conns.insert(msg.tx);

            self.users_online.insert(msg.user_id, conns);
            self.user_names.insert(msg.user_id, msg.username);

            return conn_id;
        }
    }
}

impl Handler<Disconnected> for UsersActor
{
    type Result = ();
    fn handle(&mut self, msg: Disconnected, _ctx: &mut Self::Context) -> Self::Result
    {
        let conns = self.users_online.get_mut(&msg.user_id).unwrap();
        conns.remove(msg.conn_id);

        if conns.is_empty() {
            self.users_online.remove(&msg.user_id);
            self.user_names.remove(&msg.user_id);
        };
    }
}

impl Handler<GetOnline> for UsersActor
{
    type Result = usize;
    fn handle(&mut self, _msg: GetOnline, _ctx: &mut Self::Context) -> Self::Result
    {
        self.users_online.len()
    }
}

impl Handler<SendToUser> for UsersActor
{
    type Result = ();

    fn handle(&mut self, msg: SendToUser, _ctx: &mut Self::Context) -> Self::Result
    {
        if let Some(slab) = self.users_online.get_mut(&msg.user_id) {
            for (_, tx) in slab {
                tx.send(msg.msg.clone()).ok();
            }
        };
    }
}

impl Handler<Broadcast> for UsersActor
{
    type Result = ();

    fn handle(&mut self, msg: Broadcast, _ctx: &mut Self::Context) -> Self::Result
    {
        for (_, slab) in self.users_online.iter() {
            for (_, tx) in slab {
                tx.send(msg.msg.clone()).ok();
            }
        }
    }
}

impl Handler<GetName> for UsersActor
{
    type Result = Option<String>;

    fn handle(&mut self, msg: GetName, _ctx: &mut Self::Context) -> Self::Result
    {
        self.user_names.get(&msg.user_id).cloned()
    }
}

impl Handler<IsOnline> for UsersActor
{
    type Result = bool;
    fn handle(&mut self, msg: IsOnline, _ctx: &mut Self::Context) -> Self::Result
    {
        self.users_online.contains_key(&msg.user_id)
    }
}
