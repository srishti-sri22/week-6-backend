use webauthn_rs::prelude::*;
use std::sync::Arc;

pub fn init_webauthn() -> Arc<Webauthn> {
    let rp_id = if cfg!(debug_assertions) {
        "localhost" 
    } else {
        "yourdomain.com" 
    };
    
    let rp_origin = if cfg!(debug_assertions) {
        Url::parse("http://localhost:3000").expect("Invalid URL")
    } else {
        Url::parse("https://yourdomain.com").expect("Invalid URL")
    };
    
    let mut builder = WebauthnBuilder::new(rp_id, &rp_origin)
        .expect("Invalid configuration")
        .rp_name("Polling App")
        .allow_subdomains(false);
    
    if cfg!(debug_assertions) {
        builder = builder
            .allow_any_port(true);
    }
    
    Arc::new(builder.build().expect("Failed to build Webauthn"))
}