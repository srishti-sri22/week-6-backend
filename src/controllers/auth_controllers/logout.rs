use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::http::header::{SET_COOKIE, HeaderValue, COOKIE};
use axum::http::StatusCode;
use axum::extract::Request;
use crate::utils::session;

pub async fn logout(
    request: Request,
) -> Result<Response, StatusCode> {
    
    println!("=== Logout called ===");
    
    let cookies_header = request.headers().get(COOKIE);
    println!("Cookie header: {:?}", cookies_header);

    if let Some(cookies) = cookies_header {
        if let Ok(cookie_str) = cookies.to_str() {
            println!("Cookies: {}", cookie_str);
            
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some((name, value)) = cookie.split_once('=') {
                    if name == "session_token" {
                        println!("Found session_token: {}", &value[..20.min(value.len())]);
                        
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
        HeaderValue::from_str(cookie_value).unwrap()
    );

    Ok(resp)
}