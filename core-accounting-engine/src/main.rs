// 1. DECLARE OUR MODULES
// This tells the Rust compiler to look for these files in the src/ folder.
mod models;
mod db;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;

// 2. IMPORT FROM OUR MODULES
use crate::models::AppState;
use crate::db::seed_chart_of_accounts;
use crate::handlers::{health_check, create_journal_entry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load env variables
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("FATAL: DATABASE_URL environment variable is not set in .env");
    
    println!("⚡ Initializing TrussBooks Core Accounting Engine...");

    // PostgreSQL Connection Pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("FATAL: Failed to connect to PostgreSQL database.");
    
    println!("✅ Successfully connected to PostgreSQL!");

    // Execute the database seed process via our db module
    let total_accounts = seed_chart_of_accounts(&pool)
        .await
        .expect("FATAL: Failed to execute Chart of Accounts seeding.");
    
    println!("📊 Ledger State: {} accounts loaded in Chart of Accounts.", total_accounts);

    // Build application state via our models module
    let state = AppState { db: pool };

    // Define API routes via our handlers module
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/journal-entries", post(create_journal_entry))
        .with_state(state);

    // Start Axum server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}