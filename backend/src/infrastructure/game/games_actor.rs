use std::collections::HashMap;

use actix::prelude::*;
use shared::game::Move;
use slab::Slab;
use uuid::Uuid;

use crate::domain::game_model::{ActiveGame, FinishedGame};

#[derive(Clone, Default)]
pub struct GamesActor
{
    games: Slab<ActiveGame>,
    players: HashMap<Uuid, usize>,
}

impl GamesActor
{
    pub fn new() -> Self
    {
        Self::default()
    }
}

impl Actor for GamesActor
{
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "usize")]
pub struct Create
{
    pub game: ActiveGame,
}

#[derive(Message)]
#[rtype(result = "Option<FinishedGame>")]
pub struct Submit
{
    pub game_id: usize,
    pub user_id: Uuid,
    pub mv: Move,
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct Contains
{
    pub user_id: Uuid,
}

impl Handler<Create> for GamesActor
{
    type Result = usize;
    fn handle(&mut self, msg: Create, _ctx: &mut Self::Context) -> Self::Result
    {
        let game_id = self.games.insert(msg.game.clone());
        for p in msg.game.players {
            self.players.insert(p.id, game_id);
        }
        game_id
    }
}

impl Handler<Submit> for GamesActor
{
    type Result = Option<FinishedGame>;
    fn handle(&mut self, msg: Submit, _ctx: &mut Self::Context) -> Self::Result
    {
        if let Some(game) = self.games.get_mut(msg.game_id) {
            game.set_move(&msg.user_id, msg.mv);
            if game.is_ready() {
                let res = game.try_resolve();
                self.games.remove(msg.game_id);
                res
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Handler<Contains> for GamesActor
{
    type Result = bool;
    fn handle(&mut self, msg: Contains, _ctx: &mut Self::Context) -> Self::Result
    {
        self.players.contains_key(&msg.user_id)
    }
}
