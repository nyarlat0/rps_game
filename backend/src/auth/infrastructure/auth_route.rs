use crate::auth::application::AuthHandler;
use crate::auth::domain::AuthError;
use crate::auth::infrastructure::extract_username;
use actix_web::cookie::{time::Duration, Cookie, SameSite};
use actix_web::{
    get, post, web, HttpRequest, HttpResponse, Responder,
};
use shared::auth::*;

pub fn configure_auth(cfg: &mut web::ServiceConfig)
{
    cfg.service(register)
       .service(login)
       .service(logout)
       .service(whoami);
}

#[post("/register")]
pub async fn register(handler: web::Data<AuthHandler>,
                      form: web::Json<Credentials>)
                      -> impl Responder
{
    match handler.register_user(form.into_inner()).await {
        Ok(_) => HttpResponse::Ok()
            .body("User registered!"),

        Err(AuthError::AlreadyExists) => HttpResponse::Conflict()
            .body("Username already taken!"),

        Err(_) => HttpResponse::InternalServerError()
            .body("Registration failed."),
    }
}

#[post("/login")]
pub async fn login(handler: web::Data<AuthHandler>,
                   creds: web::Json<Credentials>)
                   -> impl Responder
{
    match handler.login_user(creds.into_inner()).await {
        Ok(token) => {
            let cookie = Cookie::build("auth_token", token)
                .path("/")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Lax)
                .max_age(Duration::days(7))
                .finish();

            HttpResponse::Ok()
                .cookie(cookie)
                .body("Successfully logged in!")
        }

        Err(AuthError::InvalidCredentials) => {
            HttpResponse::Unauthorized()
                .body("Wrong username or password!")
        }

        Err(_) => {
            HttpResponse::InternalServerError()
                .body("Login failed.")
        }
    }
}

#[post("/logout")]
pub async fn logout() -> impl Responder
{
    let expired = Cookie::build("auth_token", "")
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0)) // Expire immediately
        .finish();

    HttpResponse::SeeOther().append_header(("Location",
                                            "/"))
                            .cookie(expired)
                            .finish()
}

#[get("/me")]
pub async fn whoami(req: HttpRequest) -> impl Responder
{
    if let Some(username) = extract_username(&req) {
        HttpResponse::Ok().json(UserInfo { username })
    } else {
        HttpResponse::Unauthorized().body("Not logged in")
    }
}
