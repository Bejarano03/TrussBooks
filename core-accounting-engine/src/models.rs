use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::Zero;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

// 1. SHARED APPLICATION STATE
// We keep this here so any route or database handler can easily import it.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

// 2. DOMAIN STRUCTS (Incoming JSON Payloads)
#[derive(Debug, Deserialize)]
pub struct CreateJournalEntry {
    pub entry_date: String,
    pub description: String,
    pub reference_number: Option<String>,
    pub lines: Vec<CreateJournalLine>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJournalLine {
    pub account_code: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    pub job_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JournalEntriesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub account_code: Option<String>,
    pub job_code: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl JournalEntriesQuery {
    pub fn normalized_limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 200)
    }

    pub fn normalized_offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub code: String,
    pub name: String,
    pub account_type: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub account_type: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct AccountResponse {
    pub id: String,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl CreateAccountRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.code.trim().is_empty() {
            return Err("Account code cannot be empty.");
        }
        if self.name.trim().is_empty() {
            return Err("Account name cannot be empty.");
        }
        if self.account_type.trim().is_empty() {
            return Err("Account type cannot be empty.");
        }

        Ok(())
    }
}

impl UpdateAccountRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self
            .name
            .as_ref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(false)
        {
            return Err("Account name cannot be empty.");
        }
        if self
            .account_type
            .as_ref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(false)
        {
            return Err("Account type cannot be empty.");
        }

        Ok(())
    }
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

// 3. AIRTIGHT VALIDATION LOGIC
impl CreateJournalEntry {
    /// Validates the transaction payload against rigorous double-entry rules.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.description.trim().is_empty() {
            return Err("Transaction rejected: Description cannot be empty.");
        }

        if self.lines.len() < 2 {
            return Err(
                "Transaction rejected: A valid journal entry must contain at least 2 lines (splits).",
            );
        }

        let mut total_sum = Decimal::zero();
        for line in &self.lines {
            if line.amount.is_zero() {
                return Err("Transaction rejected: Individual line amounts cannot be zero.");
            }
            total_sum += line.amount;
        }

        if !total_sum.is_zero() {
            return Err(
                "Transaction rejected: Ledger out of balance. Total Debits must match Total Credits.",
            );
        }

        Ok(())
    }
}
