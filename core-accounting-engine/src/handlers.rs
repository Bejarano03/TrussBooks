use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};

// Import our shared models and our new DB function!
use crate::db::{
    create_account, deactivate_account, get_account_by_code, get_account_ledger, get_journal_entry,
    get_trial_balance, list_accounts, list_journal_entries, save_journal_entry, update_account,
};
use crate::models::{
    AppState, CreateAccountRequest, CreateJournalEntry, JournalEntriesQuery, UpdateAccountRequest,
};

/// Simple health check route to verify the API is alive.
pub async fn health_check(State(_state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        "TrussBooks Engine is UP and running natively!",
    )
}

/// Receives a transaction, validates it mathematically, and saves it to the ledger.
pub async fn create_journal_entry(
    State(state): State<AppState>,
    Json(payload): Json<CreateJournalEntry>,
) -> impl IntoResponse {
    // 1. Math Validation (Check Debits vs Credits)
    if let Err(validation_error) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "rejected",
                "reason": validation_error
            })),
        );
    }

    // 2. Execute the ACID Database Transaction
    match save_journal_entry(&state.db, &payload).await {
        Ok(entry_id) => {
            // The transaction was permanently written to PostgreSQL
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "status": "success",
                    "message": "Transaction permanently saved to ledger.",
                    "journal_entry_id": entry_id
                })),
            )
        }
        Err(db_error) => {
            // Catches database rules, like submitting an account code that doesn't exist
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "db_rejected",
                    "reason": db_error
                })),
            )
        }
    }
}

pub async fn get_accounts(State(state): State<AppState>) -> impl IntoResponse {
    match list_accounts(&state.db).await {
        Ok(accounts) => (
            StatusCode::OK,
            Json(serde_json::json!({ "accounts": accounts })),
        ),
        Err(db_error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "reason": db_error.to_string()
            })),
        ),
    }
}

pub async fn get_account(
    State(state): State<AppState>,
    Path(account_code): Path<String>,
) -> impl IntoResponse {
    match get_account_by_code(&state.db, &account_code).await {
        Ok(Some(account)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "account": account })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": "not_found",
                "reason": "Account was not found."
            })),
        ),
        Err(db_error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "query_rejected",
                "reason": db_error.to_string()
            })),
        ),
    }
}

pub async fn create_account_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateAccountRequest>,
) -> impl IntoResponse {
    if let Err(validation_error) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "rejected",
                "reason": validation_error
            })),
        );
    }

    match create_account(&state.db, &payload).await {
        Ok(account) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "status": "success",
                "account": account
            })),
        ),
        Err(db_error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "db_rejected",
                "reason": db_error.to_string()
            })),
        ),
    }
}

pub async fn update_account_handler(
    State(state): State<AppState>,
    Path(account_code): Path<String>,
    Json(payload): Json<UpdateAccountRequest>,
) -> impl IntoResponse {
    if let Err(validation_error) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "rejected",
                "reason": validation_error
            })),
        );
    }

    match update_account(&state.db, &account_code, &payload).await {
        Ok(Some(account)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "account": account
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": "not_found",
                "reason": "Account was not found."
            })),
        ),
        Err(db_error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "db_rejected",
                "reason": db_error.to_string()
            })),
        ),
    }
}

pub async fn deactivate_account_handler(
    State(state): State<AppState>,
    Path(account_code): Path<String>,
) -> impl IntoResponse {
    match deactivate_account(&state.db, &account_code).await {
        Ok(Some(account)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "account": account
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": "not_found",
                "reason": "Account was not found."
            })),
        ),
        Err(db_error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "db_rejected",
                "reason": db_error.to_string()
            })),
        ),
    }
}

pub async fn get_journal_entries(
    State(state): State<AppState>,
    Query(query): Query<JournalEntriesQuery>,
) -> impl IntoResponse {
    match list_journal_entries(&state.db, &query).await {
        Ok(entries) => (
            StatusCode::OK,
            Json(serde_json::json!({ "journal_entries": entries })),
        ),
        Err(db_error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "query_rejected",
                "reason": db_error.to_string()
            })),
        ),
    }
}

pub async fn get_journal_entry_by_id(
    State(state): State<AppState>,
    Path(entry_id): Path<String>,
) -> impl IntoResponse {
    match get_journal_entry(&state.db, &entry_id).await {
        Ok(Some(entry)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "journal_entry": entry })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": "not_found",
                "reason": "Journal entry was not found."
            })),
        ),
        Err(db_error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "query_rejected",
                "reason": db_error.to_string()
            })),
        ),
    }
}

pub async fn trial_balance(State(state): State<AppState>) -> impl IntoResponse {
    match get_trial_balance(&state.db).await {
        Ok(lines) => (
            StatusCode::OK,
            Json(serde_json::json!({ "trial_balance": lines })),
        ),
        Err(db_error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "reason": db_error.to_string()
            })),
        ),
    }
}

pub async fn account_ledger(
    State(state): State<AppState>,
    Path(account_code): Path<String>,
) -> impl IntoResponse {
    match get_account_ledger(&state.db, &account_code).await {
        Ok(lines) => (StatusCode::OK, Json(serde_json::json!({ "ledger": lines }))),
        Err(db_error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "query_rejected",
                "reason": db_error.to_string()
            })),
        ),
    }
}
