use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct User
{
    pub id: i64,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfo
{
    pub username: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Credentials
{
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Move
{
    Rock,
    Paper,
    Scissors,
}

#[derive(Serialize, Deserialize)]
pub enum GameReq
{
    Start,
    Submit(Move),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameInfo
{
    pub you: String,
    pub opponent: String,
    pub your_move: Move,
    pub opp_move: Move,
    pub result: GameResult,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum GameResult
{
    Win,
    Defeat,
    Draw,
}

#[derive(Serialize, Deserialize)]
pub enum GameState
{
    Waiting,
    Matched
    {
        opponent: String,
    },
    Submitted
    {
        opponent: String,
        your_move: Move,
    },
    Finished(GameInfo),
    Disconnected,
}

impl GameResult
{
    pub fn from_moves(your_move: Move,
                      opp_move: Move)
                      -> Self
    {
        use Move::*;
        match (your_move, opp_move) {
            (Rock, Scissors)
            | (Paper, Rock)
            | (Scissors, Paper) => Self::Win,
            (Rock, Paper)
            | (Paper, Scissors)
            | (Scissors, Rock) => Self::Defeat,
            _ => Self::Draw,
        }
    }

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

impl GameInfo
{
    pub fn reverse(self) -> GameInfo
    {
        GameInfo { you: self.opponent,
                   opponent: self.you,
                   your_move: self.opp_move,
                   opp_move: self.your_move,
                   result: self.result.reverse() }
    }
}
