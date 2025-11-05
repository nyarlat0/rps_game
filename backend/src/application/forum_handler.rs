use crate::domain::{auth_model::User, forum_model::*};
use shared::forum::*;
use std::sync::Arc;
use uuid::Uuid;

pub struct ForumHandler
{
    pub forum_service: Arc<dyn ForumService>,
}

impl ForumHandler
{
    pub async fn make_post(
        &self,
        user: User,
        post_contents: &str)
        -> Result<ForumPost, ForumError>
    {
        self.forum_service
            .make_post(user, post_contents)
            .await
    }
    pub async fn delete_post(&self,
                             post_id: i64)
                             -> Result<(), ForumError>
    {
        self.forum_service.delete_post(post_id).await
    }

    pub async fn like_post(&self,
                           user_id: Uuid,
                           post_id: i64)
                           -> Result<(), ForumError>
    {
        self.forum_service.like_post(user_id, post_id).await
    }
    pub async fn dislike_post(&self,
                              user_id: Uuid,
                              post_id: i64)
                              -> Result<(), ForumError>
    {
        self.forum_service
            .dislike_post(user_id, post_id)
            .await
    }
    pub async fn undo_reacton(&self,
                              user_id: Uuid,
                              post_id: i64)
                              -> Result<(), ForumError>
    {
        self.forum_service
            .undo_reaction(user_id, post_id)
            .await
    }
    pub async fn fetch_posts(
        &self,
        user_id: Uuid)
        -> Result<Vec<UserForumPost>, ForumError>
    {
        self.forum_service.fetch_posts(user_id).await
    }
    pub async fn fetch_posts_by(
        &self,
        user_id: Uuid,
        start_id: i64,
        end_id: i64)
        -> Result<Vec<UserForumPost>, ForumError>
    {
        self.forum_service
            .fetch_posts_by(user_id, start_id, end_id)
            .await
    }
}
