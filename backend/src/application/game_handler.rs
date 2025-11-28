use std::sync::Arc;
use uuid::Uuid;

use crate::domain::game_model::{ActiveGame, FinishedGame, GameNotifier, GameService, PlayerQueue};
use shared::{game::GameError, ws_messages::ServerMsg};

/// RPS-specific application orchestrator that enriches messages with usernames via AuthHandler.
pub struct GameHandler<G>
    where G: ActiveGame
{
    pub player_queue: Arc<dyn PlayerQueue>,
    pub game_service: Arc<dyn GameService<G>>,
    pub notifier: Arc<dyn GameNotifier>,
}

impl<G> GameHandler<G> where G: ActiveGame
{
    pub fn new(game_service: Arc<dyn GameService<G>>,
               player_queue: Arc<dyn PlayerQueue>,
               notifier: Arc<dyn GameNotifier>)
               -> Self
    {
        Self { game_service,
               player_queue,
               notifier }
    }

    pub async fn join(&self, user_id: Uuid) -> Result<(), GameError>
    {
        if self.game_service.has_active_game(user_id).await {
            let game = self.game_service.get_game(user_id).await.unwrap();
            let opp_id = game.get_opp(&user_id).unwrap();

            let player_name = self.notifier
                                  .get_name(user_id)
                                  .await
                                  .ok_or(GameError::Disconnected)?;

            let opp_name = self.notifier
                               .get_name(opp_id)
                               .await
                               .ok_or(GameError::Disconnected)?;

            let msg = game.into_msg(user_id, &player_name, &opp_name);

            self.notifier.notify(user_id, msg).await;
            return Err(GameError::AlreadyInGame);
        }

        if let Some(opp_id) = self.player_queue.add(user_id).await {
            let active_game = self.game_service.start(user_id, opp_id).await;

            let player_name = self.notifier
                                  .get_name(user_id)
                                  .await
                                  .ok_or(GameError::Disconnected)?;

            let opp_name = self.notifier
                               .get_name(opp_id)
                               .await
                               .ok_or(GameError::Disconnected)?;

            let msg = active_game.into_msg(user_id, &player_name, &opp_name);

            self.notifier.notify(user_id, msg.clone()).await;
            self.notifier.notify(opp_id, msg).await;
        }

        Ok(())
    }

    pub async fn submit(&self, user_id: Uuid, mv: G::Move) -> Result<(), GameError>
    {
        let curr_game = self.game_service.submit_move(user_id, mv).await?;
        let opp_id = curr_game.get_opp(&user_id).unwrap();

        let player_name = self.notifier
                              .get_name(user_id)
                              .await
                              .ok_or(GameError::Disconnected)?;

        let opp_name = self.notifier
                           .get_name(opp_id)
                           .await
                           .ok_or(GameError::Disconnected)?;

        if curr_game.is_ready() {
            let fin_game = self.game_service.try_resolve(user_id).await.unwrap();
            let msg = fin_game.into_msg(user_id, &player_name, &opp_name);

            self.notifier.notify(user_id, msg.clone()).await;
            self.notifier.notify(opp_id, msg).await;
        }

        Ok(())
    }

    pub async fn leave(&self, user_id: Uuid) -> Result<(), GameError>
    {
        if let Some(game) = self.game_service.get_game(user_id).await {
            let opp_id = game.get_opp(&user_id).unwrap();

            self.game_service.drop_for(user_id).await?;

            let msg = ServerMsg::GameErrorMsg(GameError::Disconnected);
            self.notifier.notify(opp_id, msg.clone()).await;
        } else {
            self.player_queue.remove(user_id).await;
        }

        Ok(())
    }
}
