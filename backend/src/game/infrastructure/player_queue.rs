use crate::game::application::*;
use crate::game::domain::{GameError, Player};
use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Default)]
pub struct InMemoryPlayerQueue
{
    pub queue: Mutex<VecDeque<Player>>,
}

impl InMemoryPlayerQueue
{
    pub fn new() -> Self
    {
        Self { queue: Mutex::new(VecDeque::new()) }
    }
}

impl PlayerQueue for InMemoryPlayerQueue
{
    fn add(&self, player: Player)
    {
        self.queue.lock().unwrap().push_back(player);
    }

    fn try_take(&self) -> Option<Player>
    {
        self.queue.lock().unwrap().pop_front()
    }

    fn remove_for(&self,
                  player_name: &str)
                  -> Result<(), GameError>
    {
        let mut queue = self.queue.lock().unwrap();

        if let Some(index) =
            queue.iter().position(|p| p.name == player_name)
        {
            queue.remove(index);
            Ok(())
        } else {
            Err(GameError)
        }
    }
}
