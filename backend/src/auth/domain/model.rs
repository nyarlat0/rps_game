use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::auth::Credentials;
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
