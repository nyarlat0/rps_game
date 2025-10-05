use std::sync::Mutex;

use crate::application::ws_handler::*;
use crate::domain::game_model::*;
use shared::game::*;

#[derive(Default)]
pub struct InMemoryGameRepo
{
    pub games: Mutex<Vec<ActiveGame>>,
}

impl InMemoryGameRepo
{
    pub fn new() -> Self
    {
        Self { games: Mutex::new(Vec::new()) }
    }
}

impl GameRepository for InMemoryGameRepo
{
    fn start(&self, player: Player, opp: Player)
    {
        let game = ActiveGame::new(player, opp);
        self.games.lock().unwrap().push(game);
    }

    fn submit_move(&self,
                   player_name: &str,
                   mv: Move)
                   -> Result<(), GameError>
    {
        let mut games = self.games.lock().unwrap();
        let game = games.iter_mut()
                        .find(|g| g.has_player(player_name))
                        .ok_or(GameError)?;
        game.set_move(player_name, mv);
        Ok(())
    }

    fn get_opp_for(&self,
                   player_name: &str)
                   -> Option<Player>
    {
        let mut games = self.games.lock().unwrap();
        let game =
            games.iter_mut()
                 .find(|g| g.has_player(player_name))?;
        game.get_opp(player_name)
    }

    fn resolve_for(&self,
                   player_name: &str)
                   -> Option<GameInfo>
    {
        let mut games = self.games.lock().unwrap();
        let game =
            games.iter_mut()
                 .find(|g| g.has_player(player_name))?;
        game.resolve_for(player_name)
    }

    fn remove_for(&self,
                  player_name: &str)
                  -> Result<(), GameError>
    {
        let mut games = self.games.lock().unwrap();
        let index =
            games.iter()
                 .position(|g| g.has_player(player_name))
                 .ok_or(GameError)?;
        games.remove(index);
        Ok(())
    }
}
