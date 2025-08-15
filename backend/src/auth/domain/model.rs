use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shared::auth::Credentials;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct User
{
    pub id: i64,
    pub username: String,
    pub password: String,
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
