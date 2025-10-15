use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ForumPost
{
    id: u64,
    created_at: DateTime<Utc>,
    author: String,
    contents: String,
    likes: u32,
    dislikes: u32,
}
