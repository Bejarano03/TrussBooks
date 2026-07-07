use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::net::SocketAddr;

// Hold shared application state, so all API routes safely access PostgreSQL
#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load env variables
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("FATAL: DATABASE_URL environment variable is not set in .env");
    
        println!("Initializing TrussBooks Core Accounting Engine...");

    // PostgreSQL Connection Pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("FATAL: Failed to connect to PostgreSQL database.");
    
        println!("Successfully connected to PostgreSQL!");

    // Verify Database Schema
    let account_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts")
        .fetch_one(&pool)
        .await
        .expect("FATAL: Could not query the accounts table. Did you run the SQL migration?");

    println!("Current Chart of Accounts size: {} accounts loaded.", account_count.0);

    // Build application state
    let state = AppState{ db: pool };

    // Define API routes
    let app = Router::new()
        .route("/health", get(health_check))
        .with_state(state);

    // Start Axum
    let addr = SocketAddr::from(([127, 0, 0 ,1], 3000));
    println!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Simple health check
async fn health_check(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "TrussBooks Engine is UP and running natively!")
}