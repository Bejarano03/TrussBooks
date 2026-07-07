use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

// Import our shared models and our new DB function!
use crate::models::{AppState, CreateJournalEntry};
use crate::db::save_journal_entry;

/// Simple health check route to verify the API is alive.
pub async fn health_check(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "TrussBooks Engine is UP and running natively!")
}

/// Receives a transaction, validates it mathematically, and saves it to the ledger.
pub async fn create_journal_entry(
    State(state): State<AppState>,
    Json(payload): Json<CreateJournalEntry>,
) -> impl IntoResponse {
    
    // 1. Math Validation (Check Debits vs Credits)
    if let Err(validation_error) = payload.validate() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "status": "rejected",
            "reason": validation_error
        })));
    }

    // 2. Execute the ACID Database Transaction
    match save_journal_entry(&state.db, &payload).await {
        Ok(entry_id) => {
            // The transaction was permanently written to PostgreSQL
            (StatusCode::CREATED, Json(serde_json::json!({
                "status": "success",
                "message": "Transaction permanently saved to ledger.",
                "journal_entry_id": entry_id
            })))
        }
        Err(db_error) => {
            // Catches database rules, like submitting an account code that doesn't exist
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "status": "db_rejected",
                "reason": db_error
            })))
        }
    }
}