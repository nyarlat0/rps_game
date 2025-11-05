use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ForumPost
{
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub author: String,
    pub contents: String,
    pub likes: i32,
    pub dislikes: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserForumPost
{
    pub post: ForumPost,
    pub liked: bool,
    pub disliked: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ForumCmd
{
    MakePost(String),
    FetchPosts,
    FetchPostsBy
    {
        start_id: i64,
        end_id: i64,
    },
    LikePost
    {
        post_id: i64,
    },
    DislikePost
    {
        post_id: i64,
    },
    UndoReaction
    {
        post_id: i64,
    },
    DeletePost
    {
        post_id: i64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ForumError
{
    DbError,
    WrongPostId,
    NetworkError,
}
