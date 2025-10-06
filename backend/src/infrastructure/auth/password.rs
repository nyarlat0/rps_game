use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher,
        PasswordVerifier, SaltString,
    },
    Algorithm, Argon2, Params, Version,
};
use std::sync::OnceLock;

use crate::domain::auth_model::*;

static ARGON2: OnceLock<Argon2<'static>> = OnceLock::new();

fn argon2() -> &'static Argon2<'static>
{
    ARGON2.get_or_init(|| {
              let params = Params::new(124 * 512,
                                       3,
                                       1,
                                       None).unwrap();
              Argon2::new(Algorithm::Argon2id,
                          Version::V0x13,
                          params)
          })
}

pub fn hash_password(password: &str)
                     -> Result<String, AuthError>
{
    let salt = SaltString::generate(&mut OsRng);
    argon2().hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthError::HashingError)
            .map(|h| h.to_string())
}

pub fn verify_password(password: &str,
                       hashed: &str)
                       -> Result<(), AuthError>
{
    let parsed = PasswordHash::new(hashed).map_err(|_| {
                     AuthError::InvalidCredentials
                 })?;
    argon2().verify_password(password.as_bytes(), &parsed)
            .map_err(|_| AuthError::InvalidCredentials)
}
