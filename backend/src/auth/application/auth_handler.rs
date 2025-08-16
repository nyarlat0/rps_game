use crate::auth::domain::*;
use shared::auth::{Credentials, UserInfo};
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthHandler
{
    pub auth_service: Arc<dyn AuthService>,
}

impl AuthHandler
{
    pub async fn register_user(&self,
                               creds: Credentials)
                               -> Result<(), AuthError>
    {
        self.auth_service.register(creds).await
    }

    pub async fn login_user(&self,
                            creds: Credentials)
                            -> Result<String, AuthError>
    {
        self.auth_service.login(creds).await
    }

    pub async fn get_userinfo(
        &self,
        id: Uuid)
        -> Result<UserInfo, AuthError>
    {
        self.auth_service.get_userinfo(id).await
    }
}
