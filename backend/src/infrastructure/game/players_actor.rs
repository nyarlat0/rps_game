use std::collections::VecDeque;

use actix::prelude::*;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct PlayersQueueActor
{
    pub players: VecDeque<Uuid>,
}

impl PlayersQueueActor
{
    pub fn new() -> Self
    {
        Self::default()
    }
}

impl Actor for PlayersQueueActor
{
    type Context = Context<Self>;
}

// ---- Mesages for PlayersQueueActor

#[derive(Message)]
#[rtype(result = "Option<Uuid>")]
pub struct Join
{
    pub user_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "Option<Uuid>")]
pub struct TryTake;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnected
{
    pub user_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct Contains
{
    pub user_id: Uuid,
}

impl Handler<Join> for PlayersQueueActor
{
    type Result = Option<Uuid>;
    fn handle(&mut self, msg: Join, _ctx: &mut Self::Context) -> Self::Result
    {
        if !self.players.contains(&msg.user_id) {
            if !self.players.is_empty() {
                let opp_id = self.players.pop_front()?;
                Some(opp_id)
            } else {
                self.players.push_back(msg.user_id);
                None
            }
        } else {
            None
        }
    }
}

impl Handler<Disconnected> for PlayersQueueActor
{
    type Result = ();
    fn handle(&mut self, msg: Disconnected, _ctx: &mut Self::Context) -> Self::Result
    {
        if let Some(pos) = self.players.iter().position(|x| *x == msg.user_id) {
            self.players.remove(pos);
        }
    }
}

impl Handler<TryTake> for PlayersQueueActor
{
    type Result = Option<Uuid>;

    fn handle(&mut self, _msg: TryTake, _ctx: &mut Self::Context) -> Self::Result
    {
        self.players.pop_front()
    }
}

impl Handler<Contains> for PlayersQueueActor
{
    type Result = bool;

    fn handle(&mut self, msg: Contains, _ctx: &mut Self::Context) -> Self::Result
    {
        self.players.contains(&msg.user_id)
    }
}
