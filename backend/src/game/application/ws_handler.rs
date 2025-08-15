use crate::game::domain::*;
use shared::game::*;
use std::sync::Arc;

pub enum WsHandleError
{
    Parse,
    Domain(GameError),
    Send,
}

impl From<serde_json::Error> for WsHandleError
{
    fn from(_: serde_json::Error) -> Self
    {
        Self::Parse
    }
}

impl From<GameError> for WsHandleError
{
    fn from(e: GameError) -> Self
    {
        Self::Domain(e)
    }
}

impl From<WsClosed> for WsHandleError
{
    fn from(_: WsClosed) -> Self
    {
        Self::Send
    }
}

pub trait PlayerQueue: Send + Sync
{
    fn add(&self, player: Player);
    fn try_take(&self) -> Option<Player>;
    fn remove_for(&self,
                  player_name: &str)
                  -> Result<(), GameError>;
}

pub trait GameRepository: Send + Sync
{
    fn start(&self, player: Player, opp: Player);
    fn submit_move(&self,
                   player_name: &str,
                   mv: Move)
                   -> Result<(), GameError>;
    fn resolve_for(&self,
                   player_name: &str)
                   -> Option<GameInfo>;
    fn get_opp_for(&self,
                   player_name: &str)
                   -> Option<Player>;
    fn remove_for(&self,
                  player_name: &str)
                  -> Result<(), GameError>;
}

pub struct WsHandler
{
    pub game_rep: Arc<dyn GameRepository>,
    pub player_qu: Arc<dyn PlayerQueue>,
}

impl WsHandler
{
    pub async fn handle_message(
        &self,
        player: Player,
        msg: &str)
        -> Result<(), WsHandleError>
    {
        let req: GameReq = serde_json::from_str(msg)?;

        match req {
            GameReq::Start => {
                let opp = match self.player_qu.try_take() {
                    Some(opp) => opp,
                    None => {
                        let msg = serde_json::to_string(&GameState::Waiting)?;
                        player.session.send(&msg).await?;
                        return Ok(());
                    }
                };

                self.game_rep
                    .start(player.clone(), opp.clone());

                let msg_player = serde_json::to_string(&GameState::Matched { opponent: opp.name })?;
                let msg_opp = serde_json::to_string(&GameState::Matched {
                    opponent: player.name,
                })?;
                player.session.send(&msg_player).await?;
                opp.session.send(&msg_opp).await?;
                Ok(())
            }

            GameReq::Submit(mv) => {
                self.game_rep.submit_move(&player.name,
                                           mv.clone())?;

                let game_info =
                    match self.game_rep
                              .resolve_for(&player.name)
                    {
                        Some(info) => info,
                        None => {
                            let opponent = self.game_rep.get_opp_for(&player.name).ok_or(GameError)?;

                            let msg = serde_json::to_string(&GameState::Submitted {
                            opponent: opponent.name,
                            your_move: mv,
                        })?;

                            player.session
                                  .send(&msg)
                                  .await?;
                            return Ok(());
                        }
                    };

                let opponent =
                    self.game_rep
                        .get_opp_for(&player.name)
                        .unwrap();

                self.game_rep
                    .remove_for(&player.name)
                    .unwrap();

                let msg_player = serde_json::to_string(&GameState::Finished(game_info.clone()))?;
                let msg_opp = serde_json::to_string(&GameState::Finished(game_info.reverse()))?;
                player.session.send(&msg_player).await?;
                opponent.session.send(&msg_opp).await?;
                Ok(())
            }
        }
    }

    pub async fn handle_disconnect(
        &self,
        player: Player)
        -> Result<(), WsHandleError>
    {
        let _ = self.player_qu.remove_for(&player.name);
        if let Some(opp) =
            self.game_rep.get_opp_for(&player.name)
        {
            let _ = self.game_rep.remove_for(&player.name);
            let msg = serde_json::to_string(&GameState::Disconnected)?;
            opp.session.send(&msg).await?;
        }
        Ok(())
    }
}
