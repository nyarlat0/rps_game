use actix::Addr;
use uuid::Uuid;

use crate::domain::game_model::PlayerQueue;
use crate::infrastructure::game::players_actor::{self, PlayersQueueActor};

#[derive(Clone)]
pub struct ActorPlayerQueue
{
    addr: Addr<PlayersQueueActor>,
}

impl ActorPlayerQueue
{
    pub fn new(addr: Addr<PlayersQueueActor>) -> Self
    {
        Self { addr }
    }
}

#[async_trait::async_trait]
impl PlayerQueue for ActorPlayerQueue
{
    async fn contains(&self, user_id: Uuid) -> bool
    {
        self.addr
            .send(players_actor::Contains { user_id })
            .await
            .unwrap_or(false)
    }

    async fn add(&self, user_id: Uuid) -> Option<Uuid>
    {
        self.addr
            .send(players_actor::Join { user_id })
            .await
            .unwrap_or(None)
    }

    async fn try_take(&self) -> Option<Uuid>
    {
        self.addr.send(players_actor::TryTake).await.unwrap_or(None)
    }

    async fn remove(&self, user_id: Uuid)
    {
        let _ = self.addr
                    .send(players_actor::Disconnected { user_id })
                    .await;
    }
}
