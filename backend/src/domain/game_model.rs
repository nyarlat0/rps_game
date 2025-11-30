use async_trait::async_trait;
use shared::game::GameError;
use shared::ws_messages::ServerMsg;
use uuid::Uuid;

pub type GameId = usize;

pub trait FinishedGame: Send + Sync + Clone
{
    fn into_msg(&self, player_id: Uuid, player_name: &str, opp_name: &str) -> ServerMsg;
}

pub trait ActiveGame: Send + Sync + Clone
{
    type Move: Send + Sync + Clone;
    type FinishedGame: FinishedGame;

    fn new(player: Uuid, opponent: Uuid) -> Self;
    fn set_move(&mut self, player_id: &Uuid, mv: Self::Move) -> Self;
    fn has_player(&self, player_id: &Uuid) -> bool;
    fn get_opp(&self, player_id: &Uuid) -> Option<Uuid>;
    fn is_ready(&self) -> bool;
    fn try_resolve(&self) -> Option<Self::FinishedGame>;
    fn into_msg(&self, player_id: Uuid, player_name: &str, opp_name: &str) -> ServerMsg;
    fn is_spoiled(&self) -> bool;
}

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
pub trait GameService<G>: Send + Sync
    where G: ActiveGame
{
    async fn has_active_game(&self, user_id: Uuid) -> bool;
    async fn start(&self, user_id: Uuid, opp_id: Uuid) -> G;
    async fn submit_move(&self, user_id: Uuid, mv: G::Move) -> Result<G, GameError>;
    async fn opponent_for(&self, user_id: Uuid) -> Option<Uuid>;
    async fn drop_for(&self, user_id: Uuid) -> Result<(), GameError>;
    async fn try_resolve(&self, user_id: Uuid) -> Option<G::FinishedGame>;
    async fn get_game(&self, user_id: Uuid) -> Option<G>;
    async fn clear_spoiled(&self);
}

/// Port for pushing game events to clients (e.g., via websockets).
#[async_trait]
pub trait GameNotifier: Send + Sync
{
    async fn notify(&self, user_id: Uuid, msg: ServerMsg);
    async fn get_name(&self, user_id: Uuid) -> Option<String>;
}
