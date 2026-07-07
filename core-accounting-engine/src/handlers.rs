use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

// Import our shared models
use crate::models::{AppState, CreateJournalEntry};

/// Simple health check route to verify the API is alive.
pub async fn health_check(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "TrussBooks Engine is UP and running natively!")
}

/// Receives a transaction, validates it, and acknowledges success.
pub async fn create_journal_entry(
    State(_state): State<AppState>,
    Json(payload): Json<CreateJournalEntry>,
) -> impl IntoResponse {
    
    // Pass the payload through our airtight mathematical validation
    if let Err(validation_error) = payload.validate() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "status": "rejected",
            "reason": validation_error
        })));
    }

    // Next step in the future: Execute the DB insert transaction here!
    
    (StatusCode::CREATED, Json(serde_json::json!({
        "status": "validated",
        "message": "Transaction structurally sound and balances perfectly to zero."
    })))
}