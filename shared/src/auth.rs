use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfo
{
    pub username: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Credentials
{
    pub username: String,
    pub password: String,
}
