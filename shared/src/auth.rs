use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfo
{
    pub username: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Credentials
{
    pub username: String,
    pub password: String,
}
