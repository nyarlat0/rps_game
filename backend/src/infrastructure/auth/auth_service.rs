use async_trait::async_trait;
use shared::auth::Credentials;
use shared::auth::UserInfo;
use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::domain::auth_model::*;
use crate::infrastructure::auth::*;

pub struct PsqlAuthService
{
    pub db: PgPool,
}

#[async_trait]
impl AuthService for PsqlAuthService
{
    async fn register(&self,
                      creds: Credentials)
                      -> Result<(), AuthError>
    {
        let hashed = hash_password(&creds.password)?;
        sqlx::query!(
            "INSERT INTO users (name, password_hash) VALUES ($1, $2)",
            creds.username,
            hashed
        )
        .execute(&self.db)
        .await
        .map_err(|_| AuthError::AlreadyExists)?;

        Ok(())
    }

    async fn login(&self,
                   creds: Credentials)
                   -> Result<String, AuthError>
    {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE name = $1",
            creds.username
        ).fetch_optional(&self.db)
                   .await
                   .map_err(|_| AuthError::DatabaseError)?;

        let user =
            user.ok_or(AuthError::InvalidCredentials)?;

        verify_password(&creds.password,
                        &user.password_hash)?;
        generate_jwt(&user.id)
    }

    async fn get_userinfo(&self,
                          id: Uuid)
                          -> Result<UserInfo, AuthError>
    {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            id
        ).fetch_optional(&self.db)
                   .await
                   .map_err(|_| AuthError::DatabaseError)?;

        let user =
            user.ok_or(AuthError::InvalidCredentials)?;
        Ok(user.into())
    }
}
