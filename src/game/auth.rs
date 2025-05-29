use crate::Claims;
use crate::SECRET_KEY;
use actix_web::HttpRequest;
use jsonwebtoken::{decode, DecodingKey, Validation};

pub fn extract_username(req: HttpRequest)
                        -> Option<String>
{
    if let Some(cookie) = req.cookie("auth_token") {
        let token_data = decode::<Claims>(
            cookie.value(),
            &DecodingKey::from_secret(SECRET_KEY),
            &Validation::default(),
        );

        if let Ok(data) = token_data {
            return Some(data.claims.sub);
        }
    }
    return None;
}
