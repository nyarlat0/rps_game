use actix_files as fs;
use actix_web::cookie::{time::*, Cookie, SameSite};
use actix_web::{
    web, App, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher,
        PasswordVerifier, SaltString,
    },
    Argon2,
};
use jsonwebtoken::{
    decode, encode, DecodingKey, EncodingKey, Header,
    Validation,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
mod game;

// JWT Secret Key (change this in production!)
pub const SECRET_KEY: &[u8] = b"your_secret_key";

#[derive(FromRow, Serialize, Deserialize)]
struct User
{
    id: i64,
    username: String,
    password: String, // Stored hashed password
}

#[derive(Deserialize)]
struct RegisterRequest
{
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest
{
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims
{
    sub: String,
    exp: usize, // Expiration timestamp
}

async fn fallback() -> impl Responder
{
    actix_files::NamedFile::open_async("./frontend/dist/index.html").await.unwrap()
}

// 🔐 Hash Password (Argon2)
async fn hash_password(password: &str) -> String
{
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(password.as_bytes(), &salt)
          .unwrap()
          .to_string()
}

// 🔑 Verify Password
async fn verify_password(password: &str,
                         hashed: &str)
                         -> bool
{
    let parsed_hash = PasswordHash::new(hashed).unwrap();
    Argon2::default().verify_password(password.as_bytes(),
                                      &parsed_hash)
                     .is_ok()
}

// 🎟 Generate JWT Token
fn generate_jwt(username: &str) -> String
{
    let expiration =
        UtcDateTime::now() + Duration::hours(24);
    let claims = Claims { sub: username.to_string(),
                          exp: expiration.unix_timestamp()
                               as usize };
    encode(&Header::default(),
           &claims,
           &EncodingKey::from_secret(SECRET_KEY)).unwrap()
}

// ✅ User Registration
async fn register(pool: web::Data<SqlitePool>,
                  form: web::Json<RegisterRequest>)
                  -> impl Responder
{
    let hashed_password =
        hash_password(&form.password).await;

    let result = sqlx::query!(
        "INSERT INTO users (username, password) VALUES (?, ?)",
        form.username,
        hashed_password
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            HttpResponse::Ok().body("User registered!")
        }
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::InternalServerError().body("Registration failed!")
        }
    }
}

// 🔑 User Login
async fn login(pool: web::Data<SqlitePool>,
               form: web::Json<LoginRequest>)
               -> impl Responder
{
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE username = ?", form.username)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap();

    if let Some(user) = user {
        if verify_password(&form.password, &user.password).await {
            let token = generate_jwt(&user.username);
            let cookie = Cookie::build("auth_token", token.clone())
                .path("/")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Lax)
                .max_age(Duration::days(7))
                .finish();

            return HttpResponse::Ok()
                .cookie(cookie)
                .body("Login successful!");
        }
    }

    HttpResponse::Unauthorized().body("Invalid username or password")
}

// 🔐 Protected Route (Requires Auth)
async fn protected(req: HttpRequest) -> impl Responder
{
    if let Some(cookie) = req.cookie("auth_token") {
        let token_data = decode::<Claims>(
            cookie.value(),
            &DecodingKey::from_secret(SECRET_KEY),
            &Validation::default(),
        );

        if let Ok(data) = token_data {
            return HttpResponse::Ok().json(serde_json::json!({"user": data.claims.sub }));
        }
    }
    HttpResponse::Unauthorized().body("Access denied!")
}

async fn logout() -> impl Responder
{
    HttpResponse::Ok()
        .cookie(
            Cookie::build("auth_token", "")
                .path("/")
                .max_age(Duration::seconds(0)) // Expire the cookie immediately
                .finish(),
        )
        .body("Logged out!")
}

// 🏁 Start the Server
#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    let pool = SqlitePool::connect("sqlite:users.db").await.expect("Failed to connect to DB");

    HttpServer::new(move || {
        App::new().app_data(web::Data::new(pool.clone()))
                  .route("/api/register",
                         web::post().to(register))
                  .route("/api/login",
                         web::post().to(login))
                  .route("/api/protected",
                         web::get().to(protected))
                  .route("/api/logout",
                         web::get().to(logout))
                  .configure(game::routes::config)
                  .service(fs::Files::new("/", "./frontend/dist").index_file("index.html"))
                  .default_service(web::get().to(fallback))
    }).bind("127.0.0.1:8081")?
      .run()
      .await
}
