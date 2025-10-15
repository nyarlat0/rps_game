use serde::{Deserialize, Serialize};

use crate::forum::ForumPost;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
pub enum ClientMsg
{
    GetStats,
    ForumMsg(ForumCmd),
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
pub enum ServerMsg
{
    StatsMsg(StatsInfo),
    PostMsg
    {
        post: ForumPost,
        liked: bool,
        disliked: bool,
    },
    Error(WsError),
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StatsInfo
{
    pub online: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ForumCmd
{
    MakePost(String),
    LikePost
    {
        id: u32,
    },
    DislikePost
    {
        id: u32,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum WsError
{
    MsgError,
    UnAuth,
    DataError,
}
