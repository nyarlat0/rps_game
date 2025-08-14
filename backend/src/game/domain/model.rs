use async_trait::async_trait;
use shared::*;
use std::sync::Arc;

pub struct WsClosed;
#[derive(Debug)]
pub struct GameError;

#[async_trait]
pub trait WsSession: Send + Sync {
    async fn send(&self, msg: &str) -> Result<(), WsClosed>;
}

#[derive(Clone)]
pub struct Player {
    pub name: String,
    pub session: Arc<dyn WsSession>,
    pub current_move: Option<Move>,
}

pub struct ActiveGame {
    pub players: [Player; 2],
}

impl ActiveGame {
    pub fn new(player: Player, opponent: Player) -> ActiveGame {
        ActiveGame {
            players: [player, opponent],
        }
    }

    pub fn set_move(&mut self, player_name: &str, mv: Move) {
        let [p1, p2] = &mut self.players;

        if p1.name == player_name {
            p1.current_move.get_or_insert(mv);
        } else if p2.name == player_name {
            p2.current_move.get_or_insert(mv);
        }
    }

    pub fn has_player(&self, player_name: &str) -> bool {
        let [p1, p2] = &self.players;
        p1.name == player_name || p2.name == player_name
    }

    pub fn get_opp(&self, player_name: &str) -> Option<Player> {
        let [p1, p2] = &self.players;

        if p1.name == player_name {
            Some(p2.clone())
        } else if p2.name == player_name {
            Some(p1.clone())
        } else {
            None
        }
    }

    pub fn is_ready(&self) -> bool {
        self.players.iter().all(|p| p.current_move.is_some())
    }

    pub fn resolve_for(&self, player_name: &str) -> Option<GameInfo> {
        if !self.is_ready() {
            return None;
        }
        let [p1, p2] = &self.players;

        if p1.name == player_name {
            Some(GameInfo {
                you: p1.name.clone(),
                opponent: p2.name.clone(),
                your_move: p1.current_move.clone()?,
                opp_move: p2.current_move.clone()?,
                result: GameResult::from_moves(p1.current_move.clone()?, p2.current_move.clone()?),
            })
        } else if p2.name == player_name {
            Some(GameInfo {
                you: p2.name.clone(),
                opponent: p1.name.clone(),
                your_move: p2.current_move.clone()?,
                opp_move: p1.current_move.clone()?,
                result: GameResult::from_moves(p2.current_move.clone()?, p1.current_move.clone()?),
            })
        } else {
            None
        }
    }
}
