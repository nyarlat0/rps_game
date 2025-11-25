use actix::prelude::*;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use shared::forum::*;
use shared::ws_messages::ServerMsg;

use crate::application::{auth_handler::AuthHandler, forum_handler::ForumHandler};
use crate::domain::auth_model::AuthError;
use crate::domain::users_actor::{Broadcast, UsersActor};
use crate::infrastructure::auth::extract_id;

#[post("/forum")]
pub async fn forum_control(forum_handler: web::Data<ForumHandler>,
                           auth_handler: web::Data<AuthHandler>,
                           users_actor: web::Data<Addr<UsersActor>>,
                           req: HttpRequest,
                           forum_cmd: web::Json<ForumCmd>)
                           -> impl Responder
{
    if let Some(user_id) = extract_id(&req) {
        match auth_handler.get_user(user_id).await {
            Ok(user) => match forum_cmd.into_inner() {
                ForumCmd::MakePost(post_contents) => {
                    let result = forum_handler.make_post(user, &post_contents).await;

                    if let Ok(post) = result.as_ref() {
                        users_actor.do_send(Broadcast { msg:
                                                            ServerMsg::NewPostMsg(post.clone()) });
                    };
                    HttpResponse::Ok().json(result)
                }

                ForumCmd::FetchPosts => {
                    let result = forum_handler.fetch_posts(user_id).await;
                    HttpResponse::Ok().json(result)
                }

                ForumCmd::FetchPostsBy { start_id, end_id } => {
                    let result = forum_handler.fetch_posts_by(user_id, start_id, end_id)
                                              .await;
                    HttpResponse::Ok().json(result)
                }

                ForumCmd::LikePost { post_id } => {
                    let result = forum_handler.like_post(user_id, post_id).await;
                    HttpResponse::Ok().json(result)
                }

                ForumCmd::DislikePost { post_id } => {
                    let result = forum_handler.dislike_post(user_id, post_id).await;
                    HttpResponse::Ok().json(result)
                }

                ForumCmd::UndoReaction { post_id } => {
                    let result = forum_handler.undo_reacton(user_id, post_id).await;
                    HttpResponse::Ok().json(result)
                }

                ForumCmd::DeletePost { post_id } => {
                    if (user.role == "admin") || (user.role == "moderator") {
                        let result = forum_handler.delete_post(post_id).await;
                        HttpResponse::Ok().json(result)
                    } else {
                        HttpResponse::Unauthorized().body("Not admin!")
                    }
                }
            },

            Err(AuthError::InvalidCredentials) => {
                HttpResponse::Unauthorized().body("Wrong username or password!")
            }

            Err(_) => HttpResponse::InternalServerError().body("Login failed."),
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in")
    }
}
