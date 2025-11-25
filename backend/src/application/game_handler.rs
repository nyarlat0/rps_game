use std::sync::Arc;
use uuid::Uuid;

use crate::domain::game_model::GameService;

pub struct GameHandler
{
    pub auth_service: Arc<dyn GameService>,
}

impl GameHandler {}
