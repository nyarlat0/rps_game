use chrono::{DateTime, Duration, Utc};
use nolan_chess::{Color, Engine as ChessEngine, Move as ChessMove};
use shared::{chess::ChessGameInfo, game::GameResult, ws_messages::ServerMsg};
use uuid::Uuid;

use crate::domain::game_model::{ActiveGame, FinishedGame};
use shared::chess::ChessGameState;

#[derive(Clone)]
pub struct ChessGame
{
    pub players: [Uuid; 2],
    pub engine: ChessEngine,
    pub last_move: Option<(Color, ChessMove)>,
    pub created_at: DateTime<Utc>,
}

impl FinishedGame for ChessGame
{
    fn into_msg(&self, player_id: Uuid, player_name: &str, opp_name: &str) -> ServerMsg
    {
        let players = if player_id == self.players[0] {
            [player_name.to_string(),
             opp_name.to_string()]
        } else {
            [opp_name.to_string(),
             player_name.to_string()]
        };

        ServerMsg::ChessGameMsg(ChessGameState::Finished(ChessGameInfo { players,
                                                                         engine: self.engine
                                                                                     .clone() }))
    }
    fn reverse(&mut self)
    {
        self.players.reverse();
    }
    fn resolve(&self) -> GameResult
    {
        if self.engine.is_mated(Color::White) {
            GameResult::Win
        } else {
            GameResult::Defeat
        }
    }
}

impl ActiveGame for ChessGame
{
    type Move = ChessMove;
    type FinishedGame = ChessGame;

    fn new(player: Uuid, opponent: Uuid) -> Self
    {
        ChessGame { players: [player, opponent],
                    engine: ChessEngine::new(),
                    last_move: None,
                    created_at: Utc::now() }
    }

    fn is_spoiled(&self) -> bool
    {
        (Utc::now() - self.created_at) >= Duration::hours(4)
    }

    fn set_move(&mut self, player_id: &Uuid, mv: ChessMove) -> Self
    {
        let [p1, p2] = &mut self.players;

        if *p1 == *player_id {
            let _ = self.engine.apply_move(mv);
            self.last_move = Some((Color::White, mv));
        } else if *p2 == *player_id {
            let _ = self.engine.apply_move(mv);
            self.last_move = Some((Color::Black, mv));
        }
        self.clone()
    }

    fn has_player(&self, player_id: &Uuid) -> bool
    {
        let [p1, p2] = &self.players;
        *p1 == *player_id || *p2 == *player_id
    }

    fn get_opp(&self, player_id: &Uuid) -> Option<Uuid>
    {
        let [p1, p2] = &self.players;

        if *p1 == *player_id {
            Some(p2.clone())
        } else if *p2 == *player_id {
            Some(p1.clone())
        } else {
            None
        }
    }

    fn is_ready(&self) -> bool
    {
        self.engine.mate().is_some() || self.engine.stalemate()
    }

    fn try_resolve(&self) -> Option<Self>
    {
        if !self.is_ready() {
            return None;
        }
        Some(self.clone())
    }

    fn into_msg(&self, player_id: Uuid, player_name: &str, opp_name: &str) -> ServerMsg
    {
        if self.is_ready() {
            FinishedGame::into_msg(&self.try_resolve().unwrap(),
                                   player_id,
                                   player_name,
                                   opp_name)
        } else {
            ServerMsg::ChessGameMsg(ChessGameState::Game { players: [player_name.to_string(),
                                                                     opp_name.to_string()],
                                                           last_move: self.last_move,
                                                           turn: self.engine.turn })
        }
    }
}
