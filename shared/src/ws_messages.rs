use serde::{Deserialize, Serialize};

use crate::{
    chess::{ChessGameReq, ChessGameState},
    forum::*,
    game::GameError,
    rps_game::{RpsGameReq, RpsGameState},
};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
pub enum ClientMsg
{
    GetStats,
    RpsGameMsg(RpsGameReq),
    ChessGameMsg(ChessGameReq),
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
pub enum ServerMsg
{
    StatsMsg(StatsInfo),
    NewPostMsg(ForumPost),
    RpsGameMsg(RpsGameState),
    ChessGameMsg(ChessGameState),
    WsErrorMsg(WsError),
    GameErrorMsg(GameError),
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StatsInfo
{
    pub online: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum WsError
{
    MsgError,
    UnAuth,
    DataError,
}
