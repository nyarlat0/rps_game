use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for GameResult
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let s = match self {
            GameResult::Win => "You win!",
            GameResult::Defeat => "You lose!",
            GameResult::Draw => "Is draw!",
        };
        write!(f, "{s}")
    }
}
