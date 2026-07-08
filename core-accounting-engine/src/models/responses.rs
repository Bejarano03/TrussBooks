use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct AccountResponse {
    pub id: String,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct BusinessResponse {
    pub id: String,
    pub business_name: String,
    pub legal_name: Option<String>,
    pub business_type: String,
    pub industry: Option<String>,
    pub tax_id: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub logo_url: Option<String>,
    pub brand_color: Option<String>,
    pub invoice_footer_text: Option<String>,
    pub default_payment_terms_days: i16,
    pub currency_code: String,
    pub timezone: String,
    pub fiscal_year_start_month: i16,
    pub fiscal_year_start_day: i16,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct ContactResponse {
    pub id: String,
    pub business_id: String,
    pub contact_type: String,
    pub display_name: String,
    pub legal_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_id: Option<String>,
    pub payment_terms_days: i16,
    pub notes: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct AccountTemplateResponse {
    pub id: String,
    pub template_key: String,
    pub template_name: String,
    pub business_type: Option<String>,
    pub industry: Option<String>,
    pub description: Option<String>,
    pub is_default: bool,
    pub account_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct AccountTemplateAccountResponse {
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub sort_order: i32,
    pub is_required: bool,
}

#[derive(Debug, FromRow, Serialize)]
pub struct JournalEntrySummary {
    pub id: String,
    pub entry_date: NaiveDate,
    pub description: String,
    pub reference_number: Option<String>,
    pub line_count: i64,
    #[serde(with = "rust_decimal::serde::str")]
    pub debit_total: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub credit_total: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct JournalEntryHeader {
    pub id: String,
    pub entry_date: NaiveDate,
    pub description: String,
    pub reference_number: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct JournalLineResponse {
    pub id: String,
    pub account_id: String,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    pub job_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct JournalEntryResponse {
    pub id: String,
    pub entry_date: NaiveDate,
    pub description: String,
    pub reference_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub lines: Vec<JournalLineResponse>,
}

impl JournalEntryResponse {
    pub fn from_parts(header: JournalEntryHeader, lines: Vec<JournalLineResponse>) -> Self {
        Self {
            id: header.id,
            entry_date: header.entry_date,
            description: header.description,
            reference_number: header.reference_number,
            created_at: header.created_at,
            lines,
        }
    }
}

#[derive(Debug, FromRow, Serialize)]
pub struct TrialBalanceLine {
    pub account_id: String,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub balance: Decimal,
}

#[derive(Debug, FromRow, Serialize)]
pub struct AccountLedgerLine {
    pub journal_entry_id: String,
    pub journal_line_id: String,
    pub entry_date: NaiveDate,
    pub description: String,
    pub reference_number: Option<String>,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    pub job_code: Option<String>,
    #[serde(with = "rust_decimal::serde::str")]
    pub running_balance: Decimal,
}
