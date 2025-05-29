use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NewMatch
{
    pub player_1: String,
    pub player_2: String,
}

#[derive(Serialize, Deserialize)]
pub struct MoveSubmission
{
    pub match_id: i32,
    pub player: String,
    pub player_move: String, // "rock", "paper", "scissors"
}

#[derive(Serialize, Deserialize)]
pub struct MatchRecord
{
    pub id: i64,
    pub player1: String,
    pub player2: String,
    pub result: String,
}
