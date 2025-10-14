use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Clone)]
pub struct ForumPost
{
    id: u64,
    created_at: OffsetDateTime,
    author: String,
    contents: String,
    likes: u32,
    dislikes: u32,
}
