use super::ws::ws_join;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig)
{
    cfg.service(
        web::scope("/api/game")
        .route("/start", web::get().to(ws_join))
    );
}
