use serde::{Deserialize, Serialize};

use crate::game::GameResult;
use nolan_chess::{Color, Engine as ChessEngine, Move as ChessMove};

#[derive(Serialize, Deserialize, Clone)]
pub enum ChessGameReq
{
    Start,
    Submit(ChessMove),
    Leave,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChessGameInfo
{
    pub players: [String; 2],
    pub engine: ChessEngine,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ChessGameState
{
    Game
    {
        players: [String; 2],
        last_move: Option<(Color, ChessMove)>,
        turn: Color,
    },
    Finished(ChessGameInfo),
}

impl ChessGameInfo
{
    pub fn resolve(&self) -> GameResult
    {
        if let Some(mated_side) = self.engine.mate() {
            match mated_side {
                Color::Black => GameResult::Win,
                Color::White => GameResult::Defeat,
            }
        } else {
            GameResult::Draw
        }
    }
}
