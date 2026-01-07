use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

const JWT_SECRET: &[u8] = b"your_very_secret_key_change_me"; // In production, load from .env

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,    // User ID
    pub exp: usize,   // Expiration timestamp
}

// Hash password using Argon2id
pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(password.as_bytes(), &salt)
        .expect("Error hashing password")
        .to_string()
}

// Verify password against stored hash
pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).expect("Invalid hash format");
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
}

// Create a JWT token for a user
pub fn create_jwt(user_id: Uuid) -> Result<String, StatusCode> {
    let expiration = Utc::now() + Duration::hours(24);
    let claims = Claims {
        sub: user_id,
        exp: expiration.timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// Middleware to protect routes
pub async fn authorize(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    match auth_header {
        Some(token) => {
            let token_data = decode::<Claims>(
                token,
                &DecodingKey::from_secret(JWT_SECRET),
                &Validation::default(),
            ).map_err(|_| StatusCode::UNAUTHORIZED)?;

            // Attach user_id to the request for handlers to use
            req.extensions_mut().insert(token_data.claims.sub);
            Ok(next.run(req).await)
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}