use crate::auth::domain::*;
use actix_web::HttpRequest;
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey,
    Header, Validation,
};
use std::env;
use std::sync::OnceLock;
use uuid::Uuid;

static ENCODING_KEY: OnceLock<EncodingKey> =
    OnceLock::new();
static DECODING_KEY: OnceLock<DecodingKey> =
    OnceLock::new();

fn encoding_key() -> &'static EncodingKey
{
    ENCODING_KEY.get_or_init(|| {
        dotenv().ok();
        let s =
        env::var("JWT_SECRET").expect("JWT_SECRET not set");
        EncodingKey::from_base64_secret(&s).expect("JWT_SECRET must be base64")
    })
}

fn decoding_key() -> &'static DecodingKey
{
    DECODING_KEY.get_or_init(|| {
                    dotenv().ok();
                    let s =
        env::var("JWT_SECRET").expect("JWT_SECRET not set");
                    DecodingKey::from_base64_secret(&s)
            .expect("JWT_SECRET must be base64")
                })
}

pub fn generate_jwt(id: &Uuid)
                    -> Result<String, AuthError>
{
    let expiration = Utc::now() + Duration::hours(24);

    let claims = Claims { sub: id.to_string(),
                          exp: expiration.timestamp()
                               as usize };

    let header = Header::new(Algorithm::HS256);

    encode(
        &header,
        &claims,
        encoding_key(),
    )
        .map_err(|_| AuthError::TokenError)
}

pub fn extract_id(req: &HttpRequest) -> Option<Uuid>
{
    let mut v = Validation::new(Algorithm::HS256);
    v.validate_exp = true;
    v.leeway = 30;

    let id_str = req.cookie("auth_token")
                    .and_then(|cookie| {
                        decode::<Claims>(cookie.value(),
                                         decoding_key(),
                                         &v).ok()
                    })
                    .map(|data| data.claims.sub)?;
    Uuid::parse_str(&id_str).ok()
}
