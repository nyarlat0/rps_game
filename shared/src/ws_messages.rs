use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ClientMsg
{
    GetStats,
}

#[derive(Serialize, Deserialize)]
pub struct StatsInfo
{
    pub online: u32,
}
