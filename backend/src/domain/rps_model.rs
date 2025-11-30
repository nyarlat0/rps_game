use chrono::{DateTime, Duration, Utc};
use shared::{rps_game::*, ws_messages::ServerMsg};
use uuid::Uuid;

use crate::domain::game_model::{ActiveGame, FinishedGame};

#[derive(Clone)]
pub struct RpsPlayer
{
    pub id: Uuid,
    pub current_move: Option<RpsMove>,
}

#[derive(Clone)]
pub struct RpsGame
{
    pub players: [RpsPlayer; 2],
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct FinishedRpsGame
{
    pub players_id: [Uuid; 2],
    pub moves: [RpsMove; 2],
}

impl FinishedGame for FinishedRpsGame
{
    fn into_msg(&self, player_id: Uuid, player_name: &str, opp_name: &str) -> ServerMsg
    {
        let info = if player_id == self.players_id[0] {
            RpsGameInfo { players: [player_name.to_string(),
                                    opp_name.to_string()],
                          moves: self.moves.clone() }
        } else {
            RpsGameInfo { players: [opp_name.to_string(),
                                    player_name.to_string()],
                          moves: self.moves.clone() }
        };

        ServerMsg::RpsGameMsg(RpsGameState::Finished(info))
    }
}

impl ActiveGame for RpsGame
{
    type Move = RpsMove;
    type FinishedGame = FinishedRpsGame;

    fn new(player: Uuid, opponent: Uuid) -> Self
    {
        let pl = RpsPlayer { id: player,
                             current_move: None };
        let op = RpsPlayer { id: opponent,
                             current_move: None };
        let created_at = Utc::now();

        Self { players: [pl, op],
               created_at }
    }

    fn is_spoiled(&self) -> bool
    {
        (Utc::now() - self.created_at) >= Duration::minutes(2)
    }

    fn set_move(&mut self, player_id: &Uuid, mv: RpsMove) -> Self
    {
        let [p1, p2] = &mut self.players;

        if p1.id == *player_id {
            p1.current_move.get_or_insert(mv);
        } else if p2.id == *player_id {
            p2.current_move.get_or_insert(mv);
        }
        self.clone()
    }

    fn has_player(&self, player_id: &Uuid) -> bool
    {
        let [p1, p2] = &self.players;
        p1.id == *player_id || p2.id == *player_id
    }

    fn get_opp(&self, player_id: &Uuid) -> Option<Uuid>
    {
        let [p1, p2] = &self.players;

        if p1.id == *player_id {
            Some(p2.id.clone())
        } else if p2.id == *player_id {
            Some(p1.id.clone())
        } else {
            None
        }
    }

    fn is_ready(&self) -> bool
    {
        self.players.iter().all(|p| p.current_move.is_some())
    }

    fn try_resolve(&self) -> Option<FinishedRpsGame>
    {
        if !self.is_ready() {
            return None;
        }
        let [p1, p2] = &self.players;

        Some(FinishedRpsGame { players_id: [p1.id, p2.id],
                               moves: [p1.current_move.clone()?,
                                       p2.current_move.clone()?] })
    }

    fn into_msg(&self, player_id: Uuid, player_name: &str, opp_name: &str) -> ServerMsg
    {
        if !self.is_ready() {
            let submitted = [self.players[0].current_move.is_some(),
                             self.players[1].current_move.is_some()];

            let state = if self.players[0].id == player_id {
                RpsGameState::Game { players: [player_name.to_string(),
                                               opp_name.to_string()],
                                     submitted }
            } else {
                RpsGameState::Game { players: [opp_name.to_string(),
                                               player_name.to_string()],
                                     submitted }
            };

            ServerMsg::RpsGameMsg(state)
        } else {
            self.try_resolve()
                .unwrap()
                .into_msg(player_id, player_name, opp_name)
        }
    }
}
