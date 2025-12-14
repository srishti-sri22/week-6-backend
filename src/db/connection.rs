use mongodb::{options::ClientOptions, Client, Database};
use std::env;
use dotenvy::dotenv;
use crate::utils::error::{AppError, AppResult};

pub async fn init_db() -> AppResult<Database> {
    dotenv().ok();

    let mongo_uri = env::var("MONGO_URI")
        .map_err(|_| AppError::InternalError("MONGO_URI must be set in .env".to_string()))?;
    let db_name = env::var("DB_NAME")
        .map_err(|_| AppError::InternalError("DB_NAME must be set in .env".to_string()))?;

    let mut client_options = ClientOptions::parse(&mongo_uri)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to parse MongoDB URI: {}", e)))?;

    client_options.app_name = Some("PollingApp".to_string());
    
    client_options.connect_timeout = Some(std::time::Duration::from_secs(10));
    client_options.server_selection_timeout = Some(std::time::Duration::from_secs(10));
    client_options.max_pool_size = Some(10);
    client_options.min_pool_size = Some(2);

    let client = Client::with_options(client_options)
        .map_err(|e| AppError::DatabaseError(format!("Failed to initialize MongoDB client: {}", e)))?;
    
    client.database(&db_name)
        .run_command(mongodb::bson::doc! { "ping": 1 })
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to ping MongoDB: {}", e)))?;
    
    println!("Database connection successful.");
    
    Ok(client.database(&db_name))
}