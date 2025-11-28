use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use slab::Slab;
use tokio::sync::Mutex;
use uuid::Uuid;

use shared::game::GameError;

use crate::domain::game_model::{ActiveGame, GameId, GameService};

/// In-memory GameService backed by a slab of active games and a player -> game index.
#[derive(Clone, Default)]
pub struct InMemoryGameService<G>
    where G: ActiveGame
{
    active_games: Arc<Mutex<Slab<G>>>,
    player_to_game: Arc<Mutex<HashMap<Uuid, GameId>>>,
}

impl<G> InMemoryGameService<G> where G: ActiveGame
{
    pub fn new() -> Self
    {
        Self { active_games: Arc::new(Mutex::new(Slab::new())),
               player_to_game: Arc::new(Mutex::new(HashMap::new())) }
    }
}

#[async_trait]
impl<G> GameService<G> for InMemoryGameService<G> where G: ActiveGame
{
    async fn has_active_game(&self, user_id: Uuid) -> bool
    {
        let map = self.player_to_game.lock().await;
        map.contains_key(&user_id)
    }

    async fn get_game(&self, user_id: Uuid) -> Option<G>
    {
        let game_id = {
            let map = self.player_to_game.lock().await;
            *map.get(&user_id)?
        };

        let mut games = self.active_games.lock().await;
        let game = games.get_mut(game_id)?;

        Some(game.clone())
    }

    async fn start(&self, user_id: Uuid, opp_id: Uuid) -> G
    {
        let game = G::new(user_id, opp_id);

        let game_id = {
            let mut games = self.active_games.lock().await;
            let key = games.insert(game.clone());
            key
        };

        let mut map = self.player_to_game.lock().await;
        map.insert(user_id, game_id);
        map.insert(opp_id, game_id);

        game
    }

    async fn submit_move(&self, user_id: Uuid, mv: G::Move) -> Result<G, GameError>
    {
        let game_id = {
            let map = self.player_to_game.lock().await;
            *map.get(&user_id).ok_or(GameError::NotFound)?
        };

        let curr_game = {
            let mut games = self.active_games.lock().await;
            let game = games.get_mut(game_id).ok_or(GameError::NotFound)?;

            game.set_move(&user_id, mv);
            game.clone()
        };

        Ok(curr_game)
    }

    async fn opponent_for(&self, user_id: Uuid) -> Option<Uuid>
    {
        let game_id = {
            let map = self.player_to_game.lock().await;
            *map.get(&user_id)?
        };

        let games = self.active_games.lock().await;
        games.get(game_id).and_then(|g| g.get_opp(&user_id))
    }

    async fn drop_for(&self, user_id: Uuid) -> Result<(), GameError>
    {
        let game_id = {
            let map = self.player_to_game.lock().await;
            *map.get(&user_id).ok_or(GameError::NotFound)?
        };

        let curr_game = {
            let games = self.active_games.lock().await;
            let game = games.get(game_id).ok_or(GameError::NotFound)?;

            if !game.has_player(&user_id) {
                return Err(GameError::NotFound);
            }

            game.clone()
        };
        let opp_id = curr_game.get_opp(&user_id).unwrap();

        let mut games = self.active_games.lock().await;
        let mut map = self.player_to_game.lock().await;

        games.remove(game_id);
        map.remove(&user_id);
        map.remove(&opp_id);

        Ok(())
    }

    async fn try_resolve(&self, user_id: Uuid) -> Option<G::FinishedGame>
    {
        let game_id = {
            let map = self.player_to_game.lock().await;
            *map.get(&user_id)?
        };

        let (finished, opp_id) = {
            let mut games = self.active_games.lock().await;
            let game = games.get_mut(game_id)?;

            let opp_id = game.get_opp(&user_id).unwrap();
            (game.try_resolve(), opp_id)
        };

        if finished.is_some() {
            let mut games = self.active_games.lock().await;
            games.remove(game_id);

            let mut map = self.player_to_game.lock().await;
            map.remove(&user_id);

            map.remove(&opp_id);
        }

        finished
    }
}
