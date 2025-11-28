use async_trait::async_trait;
use shared::forum::*;
use uuid::Uuid;

use crate::domain::auth_model::User;

#[async_trait]
pub trait ForumService: Send + Sync {
    async fn make_post(&self, user: User, post_contents: &str) -> Result<ForumPost, ForumError>;
    async fn delete_post(&self, post_id: i64) -> Result<(), ForumError>;
    async fn like_post(&self, user_id: Uuid, post_id: i64) -> Result<(), ForumError>;
    async fn dislike_post(&self, user_id: Uuid, post_id: i64) -> Result<(), ForumError>;
    async fn undo_reaction(&self, user_id: Uuid, post_id: i64) -> Result<(), ForumError>;
    async fn fetch_posts(&self, user_id: Uuid) -> Result<Vec<UserForumPost>, ForumError>;
    async fn fetch_posts_by(
        &self,
        user_id: Uuid,
        start_id: i64,
        end_id: i64,
    ) -> Result<Vec<UserForumPost>, ForumError>;
}
