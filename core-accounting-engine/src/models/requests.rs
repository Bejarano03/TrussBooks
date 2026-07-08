use rust_decimal::Decimal;
use rust_decimal::prelude::Zero;
use serde::Deserialize;

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

#[derive(Debug, Deserialize)]
pub struct CreateBusinessRequest {
    pub business_name: String,
    pub legal_name: Option<String>,
    pub business_type: Option<String>,
    pub industry: Option<String>,
    pub tax_id: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub logo_url: Option<String>,
    pub brand_color: Option<String>,
    pub invoice_footer_text: Option<String>,
    pub default_payment_terms_days: Option<i16>,
    pub currency_code: Option<String>,
    pub timezone: Option<String>,
    pub fiscal_year_start_month: Option<i16>,
    pub fiscal_year_start_day: Option<i16>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBusinessRequest {
    pub business_name: Option<String>,
    pub legal_name: Option<String>,
    pub business_type: Option<String>,
    pub industry: Option<String>,
    pub tax_id: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub logo_url: Option<String>,
    pub brand_color: Option<String>,
    pub invoice_footer_text: Option<String>,
    pub default_payment_terms_days: Option<i16>,
    pub currency_code: Option<String>,
    pub timezone: Option<String>,
    pub fiscal_year_start_month: Option<i16>,
    pub fiscal_year_start_day: Option<i16>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContactRequest {
    pub contact_type: Option<String>,
    pub display_name: String,
    pub legal_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_id: Option<String>,
    pub payment_terms_days: Option<i16>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContactRequest {
    pub contact_type: Option<String>,
    pub display_name: Option<String>,
    pub legal_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_id: Option<String>,
    pub payment_terms_days: Option<i16>,
    pub notes: Option<String>,
    pub is_active: Option<bool>,
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

impl CreateBusinessRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.business_name.trim().is_empty() {
            return Err("Business name cannot be empty.");
        }
        Ok(())
    }
}

impl UpdateBusinessRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self
            .business_name
            .as_ref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(false)
        {
            return Err("Business name cannot be empty.");
        }
        Ok(())
    }
}

impl CreateContactRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.display_name.trim().is_empty() {
            return Err("Contact display name cannot be empty.");
        }
        Ok(())
    }
}

impl UpdateContactRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self
            .display_name
            .as_ref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(false)
        {
            return Err("Contact display name cannot be empty.");
        }
        Ok(())
    }
}

impl CreateJournalEntry {
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
