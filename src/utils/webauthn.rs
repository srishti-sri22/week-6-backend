use webauthn_rs::prelude::*;
use std::sync::Arc;
use std::env;
use crate::utils::error::{AppError, AppResult};

pub fn init_webauthn() -> AppResult<Arc<Webauthn>> {
    let rp_id = env::var("RP_ID")
        .unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                "localhost".to_string()
            } else {
                eprintln!("RP_ID must be set in .env for production");
                std::process::exit(1);
            }
        });
    
    let rp_origin_str = env::var("RP_ORIGIN")
        .unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                "http://localhost:3000".to_string()
            } else {
                eprintln!("RP_ORIGIN must be set in .env for production");
                std::process::exit(1);
            }
        });
    
    let rp_origin = Url::parse(&rp_origin_str)
        .map_err(|e| AppError::WebauthnError(format!("Invalid RP_ORIGIN URL: {}", e)))?;
    
    let rp_name = env::var("RP_NAME")
        .unwrap_or_else(|_| "Polling App".to_string());
    
    let mut builder = WebauthnBuilder::new(&rp_id, &rp_origin)
        .map_err(|e| AppError::WebauthnError(format!("Invalid configuration: {}", e)))?
        .rp_name(&rp_name)
        .allow_subdomains(false);
    
    if cfg!(debug_assertions) {
        builder = builder.allow_any_port(true);
    }
    
    let webauthn = builder.build()
        .map_err(|e| AppError::WebauthnError(format!("Failed to build Webauthn: {}", e)))?;
    
    println!("WebAuthn initialized - RP ID: {}, RP Origin: {}", rp_id, rp_origin_str);
    
    Ok(Arc::new(webauthn))
}