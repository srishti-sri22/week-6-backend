use webauthn_rs::prelude::*;
use std::sync::Arc;
use crate::utils::error::{AppError, AppResult};

pub fn init_webauthn() -> AppResult<Arc<Webauthn>> {
    let rp_id = if cfg!(debug_assertions) {
        "localhost" 
    } else {
        "yourdomain.com" 
    };
    
    let rp_origin = if cfg!(debug_assertions) {
        Url::parse("http://localhost:3000")
            .map_err(|e| AppError::WebauthnError(format!("Invalid URL: {}", e)))?
    } else {
        Url::parse("https://yourdomain.com")
            .map_err(|e| AppError::WebauthnError(format!("Invalid URL: {}", e)))?
    };
    
    let mut builder = WebauthnBuilder::new(rp_id, &rp_origin)
        .map_err(|e| AppError::WebauthnError(format!("Invalid configuration: {}", e)))?
        .rp_name("Polling App")
        .allow_subdomains(false);
    
    if cfg!(debug_assertions) {
        builder = builder.allow_any_port(true);
    }
    
    let webauthn = builder.build()
        .map_err(|e| AppError::WebauthnError(format!("Failed to build Webauthn: {}", e)))?;
    
    Ok(Arc::new(webauthn))
}