// 1. DECLARE OUR MODULES
// This tells the Rust compiler to look for these files in the src/ folder.
mod app;
mod db;
mod handlers;
mod models;

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;

// 2. IMPORT FROM OUR MODULES
use crate::app::build_app;
use crate::db::seed_chart_of_accounts;
use crate::models::AppState;

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

    println!(
        "📊 Ledger State: {} accounts loaded in Chart of Accounts.",
        total_accounts
    );

    // Build application state via our models module
    let state = AppState { db: pool };
    let app = build_app(state);

    // Start Axum server
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3001);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    println!("🚀 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
