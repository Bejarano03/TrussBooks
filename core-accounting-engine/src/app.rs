use axum::{
    Router,
    routing::{delete, get},
};

use crate::handlers::{
    account_ledger, create_account_handler, create_business_contact_handler,
    create_business_handler, create_journal_entry, deactivate_account_handler,
    deactivate_contact_handler, get_account, get_account_template, get_account_templates,
    get_accounts, get_business, get_business_contacts, get_businesses, get_contact,
    get_journal_entries, get_journal_entry_by_id, health_check, trial_balance,
    update_account_handler, update_business_handler, update_contact_handler,
};
use crate::models::AppState;

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/account-templates", get(get_account_templates))
        .route(
            "/account-templates/:template_key",
            get(get_account_template),
        )
        .route("/accounts", get(get_accounts).post(create_account_handler))
        .route(
            "/accounts/:account_code",
            get(get_account).patch(update_account_handler),
        )
        .route(
            "/accounts/:account_code/deactivate",
            delete(deactivate_account_handler),
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
            delete(deactivate_contact_handler),
        )
        .route(
            "/journal-entries",
            get(get_journal_entries).post(create_journal_entry),
        )
        .route("/journal-entries/:entry_id", get(get_journal_entry_by_id))
        .route("/reports/trial-balance", get(trial_balance))
        .with_state(state)
}
