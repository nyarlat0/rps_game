use actix::Actor;
use actix_files as fs;
use actix_web::{web, App, HttpServer, Responder};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod ws;

use crate::application::{auth_handler::*, forum_handler::*};
use crate::domain::games_actor::GamesActor;
use crate::domain::players_actor::PlayersQueueActor;
use crate::domain::users_actor::UsersActor;
use crate::infrastructure::{auth::*, forum::*, game::*};
use crate::ws::ws_route;

async fn fallback() -> impl Responder
{
    actix_files::NamedFile::open_async("./frontend/dist/index.html").await
                                                                    .unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("Database URL isn't set");

    let pool = PgPool::connect(&db_url).await
                                       .expect("Failed to connect to DB");

    let auth_service = Arc::new(PsqlAuthService { db: pool.clone() });
    let auth_handler = web::Data::new(AuthHandler { auth_service });

    let forum_service = Arc::new(PsqlForumService { db: pool.clone() });
    let forum_handler = web::Data::new(ForumHandler { forum_service });

    let player_qu = PlayersQueueActor::new().start();
    let games_actor = GamesActor::new().start();
    let game_handler = web::Data::new(GameHandler { player_qu,
                                                    games_actor });

    let users_actor = web::Data::new(UsersActor::new().start());

    HttpServer::new(move || {
        App::new().app_data(auth_handler.clone())
                  .app_data(forum_handler.clone())
                  .app_data(users_actor.clone())
                  .app_data(game_handler.clone())
                  .service(web::scope("/api").configure(configure_auth)
                                             .service(ws_route)
                                             .service(forum_control))
                  .service(fs::Files::new("/", "./frontend/dist").index_file("index.html"))
                  .default_service(web::get().to(fallback))
    }).bind("127.0.0.1:8081")?
      .run()
      .await
}
