use axum::http::{Request, StatusCode, header::COOKIE};
use crate::utils::session;

pub fn extract_username_from_request<T>(request: &Request<T>) -> Result<String, StatusCode> {
    let cookies_header = request.headers()
        .get(COOKIE)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let cookie_str = cookies_header
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    for cookie in cookie_str.split(';') {
        let cookie = cookie.trim();
        if let Some((name, value)) = cookie.split_once('=') {
            if name == "session_token" {
                let claims = session::verify_token(value) 
                    .map_err(|_| StatusCode::UNAUTHORIZED)?;
                return Ok(claims.sub);
            }
        }
    }
    
    Err(StatusCode::UNAUTHORIZED)
}