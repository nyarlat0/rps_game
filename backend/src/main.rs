use crate::auth::{application::*, infrastructure::*};
use crate::game::{application::*, infrastructure::*};
use actix_files as fs;
use actix_web::{web, App, HttpServer, Responder};
use sqlx::SqlitePool;
use std::sync::Arc;

pub mod auth;
pub mod game;

async fn fallback() -> impl Responder
{
    actix_files::NamedFile::open_async("./frontend/dist/index.html")
        .await
        .unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    let pool = SqlitePool::connect("sqlite://./backend/users.db")
        .await
        .expect("Failed to connect to DB");
    let auth_service =
        Arc::new(SqliteAuthService { db: pool.clone() });
    let auth_handler =
        web::Data::new(AuthHandler { auth_service });

    let game_rep = Arc::new(InMemoryGameRepo::new());
    let player_qu = Arc::new(InMemoryPlayerQueue::new());
    let handler = WsHandler { game_rep,
                              player_qu };
    let shared_handler = web::Data::new(handler);

    HttpServer::new(move || {
        App::new()
            .app_data(auth_handler.clone())
            .app_data(shared_handler.clone())
            .service(web::scope("/api/auth").configure(configure_auth))
            .service(web::scope("/api/game").configure(configure_game))
            .service(fs::Files::new("/", "./frontend/dist").index_file("index.html"))
            .default_service(web::get().to(fallback))
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
