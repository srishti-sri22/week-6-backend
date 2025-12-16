use mongodb::Database;
use std::sync::Arc;
use webauthn_rs::prelude::Webauthn;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub webauthn: Arc<Webauthn>,
}

impl AppState {
    pub fn new(db: Arc<Database>, webauthn: Arc<Webauthn>) -> Self {
        Self { db, webauthn }
    }
}