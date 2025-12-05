use std::sync::Arc;
use uuid::Uuid;

use crate::domain::game_model::{
    ActiveGame, FinishedGame, GameNotifier, GameRecorder, GameService, PlayerQueue,
};
use shared::{game::GameError, ws_messages::ServerMsg};

/// RPS-specific application orchestrator that enriches messages with usernames via AuthHandler.
pub struct GameHandler<G>
    where G: ActiveGame
{
    pub player_queue: Arc<dyn PlayerQueue>,
    pub game_service: Arc<dyn GameService<G>>,
    pub notifier: Arc<dyn GameNotifier>,
    pub recorder: Arc<dyn GameRecorder<G>>,
}

impl<G> GameHandler<G> where G: ActiveGame
{
    pub fn new(game_service: Arc<dyn GameService<G>>,
               player_queue: Arc<dyn PlayerQueue>,
               notifier: Arc<dyn GameNotifier>,
               recorder: Arc<dyn GameRecorder<G>>)
               -> Self
    {
        Self { game_service,
               player_queue,
               notifier,
               recorder }
    }

    pub async fn join(&self, user_id: Uuid) -> Result<(), GameError>
    {
        if self.game_service.has_active_game(user_id).await {
            if let Some(game) = self.game_service.get_game(user_id).await {
                let opp_id = game.get_opp(&user_id).ok_or(GameError::NotFound)?;

                let player_name = match self.notifier.get_name(user_id).await {
                    None => {
                        self.game_service.drop_for(user_id).await?;
                        return Err(GameError::Disconnected);
                    }
                    Some(name) => name,
                };

                if let Some(opp_name) = self.notifier.get_name(opp_id).await {
                    let msg = game.into_msg(user_id, &player_name, &opp_name);

                    self.notifier.notify(user_id, msg).await;
                    return Err(GameError::AlreadyInGame);
                } else {
                    self.game_service.drop_for(user_id).await?;
                };
            }
        }

        while let Some(opp_id) = self.player_queue.try_take().await {
            if !self.notifier.is_online(opp_id).await || opp_id == user_id {
                continue;
            }

            let active_game = self.game_service.start(user_id, opp_id).await;

            let player_name = match self.notifier.get_name(user_id).await {
                None => {
                    self.game_service.drop_for(user_id).await?;
                    return Err(GameError::Disconnected);
                }
                Some(name) => name,
            };

            let opp_name = match self.notifier.get_name(opp_id).await {
                None => {
                    self.game_service.drop_for(user_id).await?;
                    continue;
                }
                Some(name) => name,
            };

            let msg = active_game.into_msg(user_id, &player_name, &opp_name);

            self.notifier.notify(user_id, msg.clone()).await;
            self.notifier.notify(opp_id, msg).await;
            return Ok(());
        }

        self.player_queue.add(user_id).await;
        Ok(())
    }

    pub async fn submit(&self, user_id: Uuid, mv: G::Move) -> Result<(), GameError>
    {
        let curr_game = self.game_service.submit_move(user_id, mv).await?;
        let opp_id = curr_game.get_opp(&user_id).ok_or(GameError::NotFound)?;

        let player_name = match self.notifier.get_name(user_id).await {
            None => {
                self.game_service.drop_for(user_id).await?;
                return Err(GameError::Disconnected);
            }
            Some(name) => name,
        };

        let opp_name = match self.notifier.get_name(opp_id).await {
            None => {
                self.game_service.drop_for(user_id).await?;
                return Err(GameError::Disconnected);
            }
            Some(name) => name,
        };

        if curr_game.is_ready() {
            let fin_game = self.game_service
                               .try_resolve(user_id)
                               .await
                               .ok_or(GameError::NotFound)?;
            let _ = self.recorder.record(fin_game.clone()).await?;

            let msg = fin_game.into_msg(user_id, &player_name, &opp_name);

            self.notifier.notify(user_id, msg.clone()).await;
            self.notifier.notify(opp_id, msg).await;
        } else {
            let msg = curr_game.into_msg(user_id, &player_name, &opp_name);

            self.notifier.notify(user_id, msg.clone()).await;
            self.notifier.notify(opp_id, msg).await;
        }

        Ok(())
    }

    pub async fn leave(&self, user_id: Uuid) -> Result<(), GameError>
    {
        if let Some(game) = self.game_service.get_game(user_id).await {
            let opp_id = game.get_opp(&user_id).ok_or(GameError::NotFound)?;

            self.game_service.drop_for(user_id).await?;

            let msg = ServerMsg::GameErrorMsg(GameError::Disconnected);
            self.notifier.notify(opp_id, msg.clone()).await;
        } else {
            self.player_queue.remove(user_id).await;
        }

        Ok(())
    }
}
