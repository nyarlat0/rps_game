use crate::auth::domain::*;
use crate::auth::infrastructure::*;
use async_trait::async_trait;
use shared::auth::Credentials;
use sqlx::sqlite::SqlitePool;

pub struct SqliteAuthService
{
    pub db: SqlitePool,
}

#[async_trait]
impl AuthService for SqliteAuthService
{
    async fn register(&self,
                      creds: Credentials)
                      -> Result<(), AuthError>
    {
        let hashed = hash_password(&creds.password)?;
        sqlx::query!(
            "INSERT INTO users (username, password) VALUES (?, ?)",
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
            "SELECT * FROM users WHERE username = ?",
            creds.username
        ).fetch_optional(&self.db)
                   .await
                   .map_err(|_| AuthError::DatabaseError)?;

        let user =
            user.ok_or(AuthError::InvalidCredentials)?;

        verify_password(&creds.password, &user.password)?;
        generate_jwt(&user.username)
    }
}
