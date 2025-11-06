use async_trait::async_trait;
use shared::forum::*;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    auth_model::User, forum_model::ForumService,
};

pub struct PsqlForumService
{
    pub db: PgPool,
}

#[async_trait]
impl ForumService for PsqlForumService
{
    async fn make_post(&self,
                       user: User,
                       post_contents: &str)
                       -> Result<ForumPost, ForumError>
    {
        let rec = sqlx::query!(
                               r#"
            INSERT INTO posts (author_id, body)
            VALUES ($1, $2)
            RETURNING *
            "#,
                               user.id,
                               post_contents
        ).fetch_one(&self.db)
                  .await
                  .map_err(|_| ForumError::DbError)?;

        Ok(ForumPost { id: rec.id,
                       created_at: rec.created_at,
                       author: user.name,
                       contents:
                           post_contents.to_string(),
                       likes: rec.like_count,
                       dislikes: rec.dislike_count })
    }
    async fn delete_post(&self,
                         post_id: i64)
                         -> Result<(), ForumError>
    {
        sqlx::query!("DELETE FROM posts WHERE id = $1",
                     post_id).execute(&self.db)
                             .await
                             .map_err(|_| {
                                 ForumError::WrongPostId
                             })?;
        Ok(())
    }

    async fn like_post(&self,
                       user_id: Uuid,
                       post_id: i64)
                       -> Result<(), ForumError>
    {
        sqlx::query!(
        r#"
        INSERT INTO post_reactions (post_id, user_id, reaction)
        VALUES ($1, $2, 1)
        ON CONFLICT (post_id, user_id) DO UPDATE
        SET reaction = 1;
        "#, post_id, user_id).execute(&self.db)
            .await
            .map_err(|_| ForumError::WrongPostId)?;

        Ok(())
    }
    async fn dislike_post(&self,
                          user_id: Uuid,
                          post_id: i64)
                          -> Result<(), ForumError>
    {
        sqlx::query!(
        r#"
        INSERT INTO post_reactions (post_id, user_id, reaction)
        VALUES ($1, $2, -1)
        ON CONFLICT (post_id, user_id) DO UPDATE
        SET reaction = -1;
        "#, post_id, user_id).execute(&self.db)
            .await
            .map_err(|_| ForumError::WrongPostId)?;

        Ok(())
    }
    async fn undo_reaction(&self,
                           user_id: Uuid,
                           post_id: i64)
                           -> Result<(), ForumError>
    {
        sqlx::query!(
            "DELETE FROM post_reactions WHERE user_id = $1 AND post_id = $2",
            user_id, post_id).execute(&self.db)
            .await
            .map_err(|_| ForumError::WrongPostId)?;

        Ok(())
    }

    async fn fetch_posts(
        &self,
        user_id: Uuid)
        -> Result<Vec<UserForumPost>, ForumError>
    {
        let rows = sqlx::query!(
                                r#"
        SELECT
          p.id,
          p.created_at,
          u.name       AS author,
          p.body           AS contents,
          p.like_count     AS likes,
          p.dislike_count  AS dislikes,
          COALESCE(pr.reaction = 1,  false) AS "liked!",
          COALESCE(pr.reaction = -1, false) AS "disliked!"
        FROM posts p
        JOIN users u ON u.id = p.author_id
        LEFT JOIN post_reactions pr
          ON pr.post_id = p.id AND pr.user_id = $1
        ORDER BY p.id DESC
        LIMIT 25
        "#,
                                user_id
        ).fetch_all(&self.db)
                   .await
                   .map_err(|_| ForumError::DbError)?;

        let mut out: Vec<UserForumPost> =
            rows.into_iter()
                .map(|r| {
                    UserForumPost {
                    post: ForumPost {
                        id: r.id,
                        created_at: r.created_at, // timestamptz -> DateTime<Utc>
                        author: r.author,
                        contents: r.contents,
                        likes: r.likes,
                        dislikes: r.dislikes,
                    },
                    liked: r.liked,
                    disliked: r.disliked,
                }
                })
                .collect::<Vec<UserForumPost>>();
        out.reverse();

        Ok(out)
    }

    async fn fetch_posts_by(
        &self,
        user_id: Uuid,
        start_id: i64,
        end_id: i64)
        -> Result<Vec<UserForumPost>, ForumError>
    {
        let (lo, hi) = if start_id <= end_id {
            (start_id, end_id)
        } else {
            (end_id, start_id)
        };

        let rows = sqlx::query!(
                                r#"
        SELECT
          p.id,
          p.created_at,
          u.name       AS author,
          p.body           AS contents,
          p.like_count     AS likes,
          p.dislike_count  AS dislikes,
          COALESCE(pr.reaction = 1,  false) AS "liked!",
          COALESCE(pr.reaction = -1, false) AS "disliked!"
        FROM posts p
        JOIN users u ON u.id = p.author_id
        LEFT JOIN post_reactions pr
          ON pr.post_id = p.id AND pr.user_id = $1
        WHERE p.id BETWEEN $2 AND $3
        ORDER BY p.id ASC
        "#,
                                user_id,
                                lo,
                                hi
        ).fetch_all(&self.db)
                   .await
                   .map_err(|_| ForumError::DbError)?;

        let out = rows.into_iter()
                      .map(|r| {
                          UserForumPost {
                    post: ForumPost {
                        id: r.id,
                        created_at: r.created_at,
                        author: r.author,
                        contents: r.contents,
                        likes: r.likes,
                        dislikes: r.dislikes,
                    },
                    liked: r.liked,
                    disliked: r.disliked,
                }
                      })
                      .collect();

        Ok(out)
    }
}
