use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn create_token(username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("SESSION_SECRET").unwrap_or_else(|_| "default-secret-key".to_string());
    
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: username.to_string(),
        exp: expiration as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("SESSION_SECRET").unwrap_or_else(|_| "default-secret-key".to_string());
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

pub async fn extract_session_from_cookie(
    cookie_header: Option<&str>
) -> Result<Claims, &'static str> {
    let cookie_header = cookie_header.ok_or("No cookie header")?;
    
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if cookie.starts_with("session_token=") {
            let token = cookie.strip_prefix("session_token=")
                .ok_or("Invalid cookie format")?;
            
            return verify_token(token)
                .map_err(|_| "Invalid or expired token");
        }
    }
    
    Err("Session token not found in cookies")
}