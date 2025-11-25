use shared::rps_game::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct RpsPlayer
{
    pub id: Uuid,
    pub current_move: Option<RpsMove>,
}

#[derive(Clone)]
pub struct RpsGame
{
    pub players: [RpsPlayer; 2],
}

#[derive(Clone)]
pub struct FinishedRpsGame
{
    pub players_id: [Uuid; 2],
    pub moves: [RpsMove; 2],
}

impl RpsGame
{
    pub fn new(player: Uuid, opponent: Uuid) -> Self
    {
        let pl = RpsPlayer { id: player,
                             current_move: None };
        let op = RpsPlayer { id: opponent,
                             current_move: None };
        Self { players: [pl, op] }
    }

    pub fn set_move(&mut self, player_id: &Uuid, mv: RpsMove)
    {
        let [p1, p2] = &mut self.players;

        if p1.id == *player_id {
            p1.current_move.get_or_insert(mv);
        } else if p2.id == *player_id {
            p2.current_move.get_or_insert(mv);
        }
    }

    pub fn has_player(&self, player_id: &Uuid) -> bool
    {
        let [p1, p2] = &self.players;
        p1.id == *player_id || p2.id == *player_id
    }

    pub fn get_opp(&self, player_id: &Uuid) -> Option<RpsPlayer>
    {
        let [p1, p2] = &self.players;

        if p1.id == *player_id {
            Some(p2.clone())
        } else if p2.id == *player_id {
            Some(p1.clone())
        } else {
            None
        }
    }

    pub fn is_ready(&self) -> bool
    {
        self.players.iter().all(|p| p.current_move.is_some())
    }

    pub fn try_resolve(&self) -> Option<FinishedRpsGame>
    {
        if !self.is_ready() {
            return None;
        }
        let [p1, p2] = &self.players;

        Some(FinishedRpsGame { players_id: [p1.id, p2.id],
                               moves: [p1.current_move.clone()?,
                                       p2.current_move.clone()?] })
    }
}
