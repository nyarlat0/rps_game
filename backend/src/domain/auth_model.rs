use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::auth::{Credentials, UserInfo};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct User
{
    pub id: Uuid,
    pub name: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub role: String,
}

impl From<User> for UserInfo
{
    fn from(user: User) -> Self
    {
        Self { username: user.name }
    }
}

#[async_trait]
pub trait AuthService: Send + Sync
{
    async fn register(&self,
                      creds: Credentials)
                      -> Result<(), AuthError>;
    async fn login(&self,
                   creds: Credentials)
                   -> Result<String, AuthError>;
    async fn get_userinfo(&self,
                          id: Uuid)
                          -> Result<UserInfo, AuthError>;
}

pub enum AuthError
{
    InvalidCredentials,
    AlreadyExists,
    TokenError,
    HashingError,
    DatabaseError,
}

#[derive(Serialize, Deserialize)]
pub struct Claims
{
    pub sub: String,
    pub exp: usize,
}
