use sqlx::PgPool;
use rust_decimal::Decimal;
use rust_decimal::prelude::Zero;
use serde::Deserialize;

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
    pub amount: Decimal, 
    pub job_code: Option<String>,
}

// 3. AIRTIGHT VALIDATION LOGIC
impl CreateJournalEntry {
    /// Validates the transaction payload against rigorous double-entry rules.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.description.trim().is_empty() {
            return Err("Transaction rejected: Description cannot be empty.");
        }

        if self.lines.len() < 2 {
            return Err("Transaction rejected: A valid journal entry must contain at least 2 lines (splits).");
        }

        let mut total_sum = Decimal::zero();
        for line in &self.lines {
            if line.amount.is_zero() {
                return Err("Transaction rejected: Individual line amounts cannot be zero.");
            }
            total_sum += line.amount;
        }

        if !total_sum.is_zero() {
            return Err("Transaction rejected: Ledger out of balance. Total Debits must match Total Credits.");
        }

        Ok(())
    }
}