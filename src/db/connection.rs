use mongodb::{options::ClientOptions, Client, Database};
use std::env;
use dotenvy::dotenv;

pub async fn init_db() -> Database {
dotenv().ok(); 

let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI must be set in .env");
let db_name = env::var("DB_NAME").expect("DB_NAME must be set in .env");

let mut client_options = ClientOptions::parse(&mongo_uri)
.await
.expect("Failed to parse MongoDB URI");

client_options.app_name = Some("PollingApp".to_string());

let client = Client::with_options(client_options).expect("Failed to initialize MongoDB client");
println!("Database connection sucessfull.");
return client.database(&db_name);
}
