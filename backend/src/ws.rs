use actix::prelude::*;
use actix_web::{get, rt, web, HttpRequest, Responder};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;
use shared::ws_messages::*;

#[derive(Clone, Default)]
pub struct SiteStats
{
    online: u32,
}

impl SiteStats
{
    pub fn new() -> Self
    {
        Self::default()
    }
}

impl Actor for SiteStats
{
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
enum StatsChange
{
    Joined,
    Disconnected,
}

impl From<SiteStats> for StatsInfo
{
    fn from(value: SiteStats) -> Self
    {
        Self { online: value.online }
    }
}

impl Handler<StatsChange> for SiteStats
{
    type Result = ();
    fn handle(&mut self,
              msg: StatsChange,
              _ctx: &mut Self::Context)
              -> Self::Result
    {
        match msg {
            StatsChange::Joined => self.online += 1,
            StatsChange::Disconnected => self.online -= 1,
        };
    }
}

#[derive(Message)]
#[rtype(result = "Option<StatsInfo>")]
struct ActorGetStats;

impl Handler<ActorGetStats> for SiteStats
{
    type Result = Option<StatsInfo>;
    fn handle(&mut self,
              _msg: ActorGetStats,
              _ctx: &mut Self::Context)
              -> Self::Result
    {
        Some(self.clone().into())
    }
}

pub fn configure_ws(cfg: &mut web::ServiceConfig)
{
    cfg.service(ws_route);
}

#[get("/ws")]
pub async fn ws_route(
    req: HttpRequest,
    body: web::Payload,
    stats_actor: web::Data<Addr<SiteStats>>)
    -> actix_web::Result<impl Responder>
{
    let (response, mut session, stream) =
        actix_ws::handle(&req, body)?;

    stats_actor.do_send(StatsChange::Joined);
    let a = stats_actor.send(ActorGetStats)
                       .await
                       .unwrap()
                       .unwrap()
                       .online;
    println!("Ws opened! Current online is {a}");

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

                AggregatedMessage::Text(text) => {
                    if let Ok(msg) = serde_json::from_str::<ClientMsg>(&text) {
                        match msg {
                            ClientMsg::GetStats => {
                                let stats = stats_actor.send(ActorGetStats).await.unwrap().unwrap();
                                let out = serde_json::to_string(&stats).unwrap();
                                if session.text(out).await.is_err() { break }
                            }
                        }

                    } else {session.text("Wrong command!").await.ok();}
                }

                _ => break,
            }
        }

        let _ = session.close(None).await;
        stats_actor.do_send(StatsChange::Disconnected);
    });

    Ok(response)
}
