use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::http::header::{SET_COOKIE, HeaderValue, COOKIE};
use axum::extract::Request;
use crate::utils::{session, error::{AppError, AppResult}};

pub async fn logout(request: Request) -> AppResult<Response> {
    
    let cookies_header = request.headers().get(COOKIE);

    if let Some(cookies) = cookies_header {
        if let Ok(cookie_str) = cookies.to_str() {

            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some((name, value)) = cookie.split_once('=') {
                    if name == "session_token" {
                        let display_len = 20.min(value.len());                        
                        if let Ok(claims) = session::verify_token(value) {
                            println!("=== Logout for: {} ===", claims.sub);
                        }
                    }
                }
            }
        }
    }

    let cookie_value = "session_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    
    let mut resp = Json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })).into_response();
    
    resp.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(cookie_value)
            .map_err(|e| AppError::InternalError(format!("Failed to create cookie header: {}", e)))?
    );

    Ok(resp)
}