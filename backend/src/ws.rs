use actix_web::{get, rt, web, HttpRequest, Responder};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;

pub fn configure_ws(cfg: &mut web::ServiceConfig)
{
    cfg.service(ws_route);
}

#[get("/ws")]
pub async fn ws_route(
    req: HttpRequest,
    body: web::Payload)
    -> actix_web::Result<impl Responder>
{
    let (response, mut session, stream) =
        actix_ws::handle(&req, body)?;

    let mut stream =
        stream.aggregate_continuations()
              // aggregate continuation frames up to 1MiB
              .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                AggregatedMessage::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }

                AggregatedMessage::Text(msg) => {
                    if session.text(format!("Got text: {msg}")).await.is_err() {
                        return;
                    }
                }

                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}
