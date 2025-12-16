use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;

use crate::utils::{error::AppError, session::verify_token};

pub async fn jwt_auth(
    cookie_jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| AppError::AuthenticationError("No token found".to_string()))?;

    let claims = verify_token(&token)?;
    
    req.extensions_mut().insert(claims);
    
    Ok(next.run(req).await)
}