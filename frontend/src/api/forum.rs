use gloo_net::http::{Request, Response};
use shared::forum::*;

pub async fn fetch_posts() -> Option<Vec<UserForumPost>>
{
    let response =
        send_forum_cmd(ForumCmd::FetchPosts).await?;
    let result = response.json::<Result<Vec<UserForumPost>,
                                ForumError>>()
                         .await
                         .ok()?;
    return result.ok();
}

pub async fn fetch_posts_by(start_id: i64,
                            end_id: i64)
                            -> Option<Vec<UserForumPost>>
{
    let response =
        send_forum_cmd(ForumCmd::FetchPostsBy { start_id, end_id }).await?;
    let result = response.json::<Result<Vec<UserForumPost>,
                                ForumError>>()
                         .await
                         .ok()?;
    return result.ok();
}

pub async fn create_post(msg: String)
                         -> Result<ForumPost, ForumError>
{
    let response =
        send_forum_cmd(ForumCmd::MakePost(msg)).await.ok_or(ForumError::NetworkError)?;

    let result =
        response.json::<Result<ForumPost, ForumError>>()
                .await
                .map_err(|_| ForumError::NetworkError)?;

    return result;
}

pub async fn like_post(post_id: i64)
                       -> Result<(), ForumError>
{
    let response =
        send_forum_cmd(ForumCmd::LikePost{post_id}).await.ok_or(ForumError::NetworkError)?;

    let result =
        response.json::<Result<(), ForumError>>()
                .await
                .map_err(|_| ForumError::NetworkError)?;

    return result;
}

pub async fn dislike_post(post_id: i64)
                          -> Result<(), ForumError>
{
    let response =
        send_forum_cmd(ForumCmd::DislikePost {post_id}).await.ok_or(ForumError::NetworkError)?;

    let result =
        response.json::<Result<(), ForumError>>()
                .await
                .map_err(|_| ForumError::NetworkError)?;

    return result;
}

pub async fn undo_reaction(post_id: i64)
                           -> Result<(), ForumError>
{
    let response =
        send_forum_cmd(ForumCmd::UndoReaction{post_id}).await.ok_or(ForumError::NetworkError)?;

    let result =
        response.json::<Result<(), ForumError>>()
                .await
                .map_err(|_| ForumError::NetworkError)?;

    return result;
}

async fn send_forum_cmd(cmd: ForumCmd) -> Option<Response>
{
    Request::post("/api/forum").json(&cmd)
                               .unwrap()
                               .send()
                               .await
                               .ok()
}
