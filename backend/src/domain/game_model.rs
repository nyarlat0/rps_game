use async_trait::async_trait;
use shared::game::GameError;
use uuid::Uuid;

pub type GameId = u32;

/// Abstract matchmaking queue that can be backed by any async runtime or actor system.
#[async_trait]
pub trait PlayerQueue: Send + Sync
{
    async fn contains(&self, user_id: Uuid) -> bool;
    /// Put a player into the queue. If another player was waiting return their id so a match can start.
    async fn add(&self, user_id: Uuid) -> Option<Uuid>;
    async fn try_take(&self) -> Option<Uuid>;
    async fn remove(&self, user_id: Uuid);
}

/// Core game workflow independent from transport or storage concerns.
#[async_trait]
pub trait GameService: Send + Sync
{
    type Move: Send + Sync + Clone;
    type ActiveGame: Send + Sync + Clone;
    type FinishedGame: Send + Sync + Clone;

    async fn has_active_game(&self, user_id: Uuid) -> bool;
    async fn start(&self, user_id: Uuid, opp_id: Uuid) -> GameId;
    async fn submit_move(&self,
                         game_id: GameId,
                         user_id: Uuid,
                         mv: Self::Move)
                         -> Result<Option<Self::FinishedGame>, GameError>;
    async fn opponent_for(&self, game_id: GameId, user_id: Uuid) -> Option<Uuid>;
    async fn drop_for(&self, game_id: GameId, user_id: Uuid) -> Result<(), GameError>;
}
