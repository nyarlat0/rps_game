use crate::auth::domain::*;
use shared::auth::Credentials;
use std::sync::Arc;

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
}
