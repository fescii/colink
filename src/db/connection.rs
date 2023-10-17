use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use actix_web::web;
use crate::AppState;


pub async fn establish_connection() -> PgConnection{
  let app_state = web::Data<(Arc<AppState>);

  // Get database url from env file
  let database_url = env::var("DATABASE_URL")
    .expect("Database url must be set in .env file");

  PgConnection::establish(&database_url)
    .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}