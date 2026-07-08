// 1. DECLARE OUR MODULES
// This tells the Rust compiler to look for these files in the src/ folder.
mod db;
mod handlers;
mod models;

use axum::{Router, routing::get};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;

// 2. IMPORT FROM OUR MODULES
use crate::db::seed_chart_of_accounts;
use crate::handlers::{
    account_ledger, create_account_handler, create_business_contact_handler,
    create_business_handler, create_journal_entry, deactivate_account_handler,
    deactivate_contact_handler, get_account, get_accounts, get_business, get_business_contacts,
    get_businesses, get_contact, get_journal_entries, get_journal_entry_by_id, health_check,
    trial_balance, update_account_handler, update_business_handler, update_contact_handler,
};
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

    // Define API routes via our handlers module
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/accounts", get(get_accounts).post(create_account_handler))
        .route(
            "/accounts/:account_code",
            get(get_account).patch(update_account_handler),
        )
        .route(
            "/accounts/:account_code/deactivate",
            axum::routing::delete(deactivate_account_handler),
        )
        .route("/accounts/:account_code/ledger", get(account_ledger))
        .route(
            "/businesses",
            get(get_businesses).post(create_business_handler),
        )
        .route(
            "/businesses/:business_id",
            get(get_business).patch(update_business_handler),
        )
        .route(
            "/businesses/:business_id/contacts",
            get(get_business_contacts).post(create_business_contact_handler),
        )
        .route(
            "/contacts/:contact_id",
            get(get_contact).patch(update_contact_handler),
        )
        .route(
            "/contacts/:contact_id/deactivate",
            axum::routing::delete(deactivate_contact_handler),
        )
        .route(
            "/journal-entries",
            get(get_journal_entries).post(create_journal_entry),
        )
        .route("/journal-entries/:entry_id", get(get_journal_entry_by_id))
        .route("/reports/trial-balance", get(trial_balance))
        .with_state(state);

    // Start Axum server
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    println!("🚀 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
