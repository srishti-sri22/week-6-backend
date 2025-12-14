use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use std::env;
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn create_token(username: &str) -> AppResult<String> {
    let secret = env::var("SESSION_SECRET")
        .map_err(|_| AppError::InternalError("SESSION_SECRET must be set in .env".to_string()))?;
    
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .ok_or_else(|| AppError::InternalError("Failed to calculate token expiration".to_string()))?
        .timestamp();

    let claims = Claims {
        sub: username.to_string(),
        exp: expiration as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::AuthenticationError(format!("Failed to create token: {}", e)))
}

pub fn verify_token(token: &str) -> AppResult<Claims> {
    let secret = env::var("SESSION_SECRET")
        .map_err(|_| AppError::InternalError("SESSION_SECRET must be set in .env".to_string()))?;
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::AuthenticationError(format!("Invalid token: {}", e)))
}