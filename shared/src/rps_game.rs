use serde::{Deserialize, Serialize};

use crate::game::GameResult;

#[derive(Serialize, Deserialize, Clone)]
pub enum RpsMove
{
    Rock,
    Paper,
    Scissors,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum RpsGameReq
{
    Start,
    Submit(RpsMove),
    Leave,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RpsGameInfo
{
    pub players: [String; 2],
    pub moves: [RpsMove; 2],
}

#[derive(Serialize, Deserialize, Clone)]
pub enum RpsGameState
{
    Waiting,
    Game
    {
        players: [String; 2],
        submitted: [bool; 2],
    },
    Finished(RpsGameInfo),
}

impl RpsGameInfo
{
    pub fn resolve(&self) -> GameResult
    {
        let [your_move, opp_move] = &self.moves;
        use RpsMove::*;
        match (your_move, opp_move) {
            (Rock, Scissors) | (Paper, Rock) | (Scissors, Paper) => GameResult::Win,
            (Rock, Paper) | (Paper, Scissors) | (Scissors, Rock) => GameResult::Defeat,
            _ => GameResult::Draw,
        }
    }
}
