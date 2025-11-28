use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum GameResult
{
    Win,
    Defeat,
    Draw,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GameError
{
    NotFound,
    InvalidMove,
    Disconnected,
    AlreadyInGame,
}

impl GameResult
{
    pub fn reverse(&self) -> Self
    {
        use GameResult::*;
        match self {
            Win => Defeat,
            Defeat => Win,
            Draw => Draw,
        }
    }
}
