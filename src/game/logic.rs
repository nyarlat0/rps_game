use std::fmt::{Display, Formatter, Result};
pub enum GameResult
{
    Win,
    Defeat,
    Draw,
}

impl Display for GameResult
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        match self {
            GameResult::Win => {
                write!(f, "You win!")
            }
            GameResult::Defeat => {
                write!(f, "You lose!")
            }
            GameResult::Draw => {
                write!(f, "Draw!")
            }
        }
    }
}

pub fn game_eval(player1: &str, player2: &str)
                 -> GameResult
{
    match (player1, player2) {
        ("rock", "scissors")
        | ("paper", "rock")
        | ("scissors", "paper") => GameResult::Win,
        ("scissors", "rock")
        | ("rock", "paper")
        | ("paper", "scissors") => GameResult::Defeat,
        _ => GameResult::Draw,
    }
}
