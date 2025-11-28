use actix::Addr;
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::game_model::GameNotifier;
use crate::domain::users_actor::{self, UsersActor};
use shared::ws_messages::ServerMsg;

#[derive(Clone)]
pub struct WsGameNotifier
{
    pub users_actor: Addr<UsersActor>,
}

impl WsGameNotifier
{
    pub fn new(users_actor: Addr<UsersActor>) -> Self
    {
        Self { users_actor }
    }
}

#[async_trait]
impl GameNotifier for WsGameNotifier
{
    async fn notify(&self, user_id: Uuid, msg: ServerMsg)
    {
        let _ = self.users_actor
                    .send(users_actor::SendToUser { user_id, msg })
                    .await;
    }

    async fn get_name(&self, user_id: Uuid) -> Option<String>
    {
        self.users_actor
            .send(users_actor::GetName { user_id })
            .await
            .unwrap()
    }
}
