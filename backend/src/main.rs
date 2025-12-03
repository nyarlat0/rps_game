use actix::Actor;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod ws;

use crate::application::game_handler::GameHandler;
use crate::application::{auth_handler::*, forum_handler::*};
use crate::domain::rps_model::RpsGame;
use crate::domain::users_actor::UsersActor;
use crate::infrastructure::{auth::*, forum::*, game::*};
use crate::ws::ws_route;

#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    dotenvy::from_path("/etc/rps_game/.env").ok();
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("Database URL isn't set");

    let pool = PgPool::connect(&db_url).await
                                       .expect("Failed to connect to DB");

    let auth_service = Arc::new(PsqlAuthService { db: pool.clone() });
    let auth_handler = web::Data::new(AuthHandler { auth_service });

    let forum_service = Arc::new(PsqlForumService { db: pool.clone() });
    let forum_handler = web::Data::new(ForumHandler { forum_service });

    let users_actor = UsersActor::new().start();
    let sh_users_actor = web::Data::new(users_actor.clone());

    let players_actor = PlayersQueueActor::new().start();
    let rps_player_qu = Arc::new(ActorPlayerQueue::new(players_actor));
    let rps_service = InMemoryGameService::<RpsGame>::new();
    let notifier = Arc::new(WsGameNotifier::new(users_actor));
    let game_recorder = Arc::new(PsqlGameRecorder { db: pool.clone() });

    let rps_game_handler = web::Data::new(GameHandler::<RpsGame>::new(rps_service,
                                                                      rps_player_qu,
                                                                      notifier,
                                                                      game_recorder));

    HttpServer::new(move || {
        App::new().app_data(auth_handler.clone())
                  .app_data(forum_handler.clone())
                  .app_data(sh_users_actor.clone())
                  .app_data(rps_game_handler.clone())
                  .service(web::scope("/api").configure(configure_auth)
                                             .service(ws_route)
                                             .service(forum_control))
    }).bind("127.0.0.1:8081")?
      .run()
      .await
}
