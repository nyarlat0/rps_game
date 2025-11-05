use serde::{Deserialize, Serialize};

use crate::forum::*;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
pub enum ClientMsg
{
    GetStats,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
pub enum ServerMsg
{
    StatsMsg(StatsInfo),
    NewPostMsg(ForumPost),
    WsErrorMsg(WsError),
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
