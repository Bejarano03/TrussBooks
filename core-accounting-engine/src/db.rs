use crate::models::{
    AccountLedgerLine, AccountResponse, BusinessResponse, ContactResponse, CreateAccountRequest,
    CreateBusinessRequest, CreateContactRequest, CreateJournalEntry, JournalEntriesQuery,
    JournalEntryHeader, JournalEntryResponse, JournalEntrySummary, TrialBalanceLine,
    UpdateAccountRequest, UpdateBusinessRequest, UpdateContactRequest,
};
use sqlx::PgPool;

/// Checks if the Chart of Accounts is empty, and if so, seeds it with
/// standard construction industry accounts.
pub async fn seed_chart_of_accounts(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts")
        .fetch_one(pool)
        .await?;

    if count.0 > 0 {
        return Ok(count.0); // Already seeded
    }

    println!("🌱 Seeding standard Construction Industry Chart of Accounts...");

    let accounts = vec![
        ("1010", "Operating Checking", "asset"),
        ("1200", "Accounts Receivable", "asset"),
        ("1250", "Retention Receivable", "asset"),
        ("2010", "Accounts Payable", "liability"),
        ("3000", "Owner's Equity", "equity"),
        ("4010", "Construction Contract Revenue", "revenue"),
        ("5010", "Job Materials Expense", "expense"),
        ("5020", "Subcontractor Expense", "expense"),
        ("5030", "Equipment Rental Expense", "expense"),
    ];

    for (code, name, acct_type) in accounts {
        sqlx::query("INSERT INTO accounts (code, name, type) VALUES ($1, $2, $3::account_type)")
            .bind(code)
            .bind(name)
            .bind(acct_type)
            .execute(pool) // Note: No .map() here!
            .await?;
    }

    let final_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts")
        .fetch_one(pool)
        .await?;

    Ok(final_count.0)
}

/// Safely inserts a validated journal entry and its lines into the database
/// using an ACID-compliant transaction.
pub async fn save_journal_entry(
    pool: &PgPool,
    payload: &CreateJournalEntry,
) -> Result<String, String> {
    // 1. Start a database transaction
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 2. Insert the Journal Entry Header
    // We let Postgres cast the string to a DATE and return the UUID as a string
    // to keep our Rust types clean.
    let entry_record: (String,) = sqlx::query_as(
        r#"
        INSERT INTO journal_entries (entry_date, description, reference_number)
        VALUES ($1::date, $2, $3)
        RETURNING id::text
        "#,
    )
    .bind(&payload.entry_date)
    .bind(&payload.description)
    .bind(&payload.reference_number)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("Failed to insert journal entry header: {}", e))?;

    let entry_id = entry_record.0;

    // 3. Insert each line item
    for line in &payload.lines {
        // First, look up the internal database UUID for the provided account code
        let account_record: Option<(String,)> =
            sqlx::query_as("SELECT id::text FROM accounts WHERE code = $1")
                .bind(&line.account_code)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

        let account_id = match account_record {
            Some(record) => record.0,
            None => {
                // If the account code is invalid, returning an Err here will
                // automatically abort the transaction and rollback the header!
                return Err(format!(
                    "Invalid account code provided: {}",
                    line.account_code
                ));
            }
        };

        // Insert the line, casting our text UUIDs back to proper DB uuids
        sqlx::query(
            r#"
            INSERT INTO journal_lines (journal_entry_id, account_id, amount, job_code)
            VALUES ($1::uuid, $2::uuid, $3, $4)
            "#,
        )
        .bind(&entry_id)
        .bind(&account_id)
        .bind(&line.amount)
        .bind(&line.job_code)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to insert journal line: {}", e))?;
    }

    // 4. Commit the transaction (Saves everything permanently)
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(entry_id)
}

pub async fn list_accounts(pool: &PgPool) -> Result<Vec<AccountResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            id::text,
            code,
            name,
            type::text AS account_type,
            is_active,
            created_at
        FROM accounts
        ORDER BY code
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn get_account_by_code(
    pool: &PgPool,
    account_code: &str,
) -> Result<Option<AccountResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            id::text,
            code,
            name,
            type::text AS account_type,
            is_active,
            created_at
        FROM accounts
        WHERE code = $1
        "#,
    )
    .bind(account_code)
    .fetch_optional(pool)
    .await
}

pub async fn create_account(
    pool: &PgPool,
    payload: &CreateAccountRequest,
) -> Result<AccountResponse, sqlx::Error> {
    sqlx::query_as(
        r#"
        INSERT INTO accounts (code, name, type)
        VALUES ($1, $2, $3::account_type)
        RETURNING id::text, code, name, type::text AS account_type, is_active, created_at
        "#,
    )
    .bind(&payload.code)
    .bind(&payload.name)
    .bind(&payload.account_type)
    .fetch_one(pool)
    .await
}

pub async fn update_account(
    pool: &PgPool,
    account_code: &str,
    payload: &UpdateAccountRequest,
) -> Result<Option<AccountResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        UPDATE accounts
        SET
            name = COALESCE($2, name),
            type = COALESCE($3::account_type, type),
            is_active = COALESCE($4, is_active)
        WHERE code = $1
        RETURNING id::text, code, name, type::text AS account_type, is_active, created_at
        "#,
    )
    .bind(account_code)
    .bind(&payload.name)
    .bind(&payload.account_type)
    .bind(payload.is_active)
    .fetch_optional(pool)
    .await
}

pub async fn deactivate_account(
    pool: &PgPool,
    account_code: &str,
) -> Result<Option<AccountResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        UPDATE accounts
        SET is_active = FALSE
        WHERE code = $1
        RETURNING id::text, code, name, type::text AS account_type, is_active, created_at
        "#,
    )
    .bind(account_code)
    .fetch_optional(pool)
    .await
}

pub async fn list_businesses(pool: &PgPool) -> Result<Vec<BusinessResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            id::text,
            business_name,
            legal_name,
            business_type::text AS business_type,
            industry,
            tax_id,
            email::text,
            phone,
            website,
            logo_url,
            brand_color,
            invoice_footer_text,
            default_payment_terms_days,
            currency_code,
            timezone,
            fiscal_year_start_month,
            fiscal_year_start_day,
            is_active,
            created_at,
            updated_at
        FROM businesses
        ORDER BY created_at DESC, business_name
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn get_business_by_id(
    pool: &PgPool,
    business_id: &str,
) -> Result<Option<BusinessResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            id::text,
            business_name,
            legal_name,
            business_type::text AS business_type,
            industry,
            tax_id,
            email::text,
            phone,
            website,
            logo_url,
            brand_color,
            invoice_footer_text,
            default_payment_terms_days,
            currency_code,
            timezone,
            fiscal_year_start_month,
            fiscal_year_start_day,
            is_active,
            created_at,
            updated_at
        FROM businesses
        WHERE id = $1::uuid
        "#,
    )
    .bind(business_id)
    .fetch_optional(pool)
    .await
}

pub async fn create_business(
    pool: &PgPool,
    payload: &CreateBusinessRequest,
) -> Result<BusinessResponse, sqlx::Error> {
    sqlx::query_as(
        r#"
        INSERT INTO businesses (
            business_name, legal_name, business_type, industry, tax_id, email, phone, website,
            logo_url, brand_color, invoice_footer_text, default_payment_terms_days, currency_code,
            timezone, fiscal_year_start_month, fiscal_year_start_day
        )
        VALUES (
            $1,
            $2,
            COALESCE($3::business_type, 'other'::business_type),
            $4,
            $5,
            $6,
            $7,
            $8,
            $9,
            $10,
            $11,
            COALESCE($12, 30),
            COALESCE($13, 'USD'),
            COALESCE($14, 'America/Los_Angeles'),
            COALESCE($15, 1),
            COALESCE($16, 1)
        )
        RETURNING
            id::text,
            business_name,
            legal_name,
            business_type::text AS business_type,
            industry,
            tax_id,
            email::text,
            phone,
            website,
            logo_url,
            brand_color,
            invoice_footer_text,
            default_payment_terms_days,
            currency_code,
            timezone,
            fiscal_year_start_month,
            fiscal_year_start_day,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(&payload.business_name)
    .bind(&payload.legal_name)
    .bind(&payload.business_type)
    .bind(&payload.industry)
    .bind(&payload.tax_id)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.website)
    .bind(&payload.logo_url)
    .bind(&payload.brand_color)
    .bind(&payload.invoice_footer_text)
    .bind(payload.default_payment_terms_days)
    .bind(&payload.currency_code)
    .bind(&payload.timezone)
    .bind(payload.fiscal_year_start_month)
    .bind(payload.fiscal_year_start_day)
    .fetch_one(pool)
    .await
}

pub async fn update_business(
    pool: &PgPool,
    business_id: &str,
    payload: &UpdateBusinessRequest,
) -> Result<Option<BusinessResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        UPDATE businesses
        SET
            business_name = COALESCE($2, business_name),
            legal_name = COALESCE($3, legal_name),
            business_type = COALESCE($4::business_type, business_type),
            industry = COALESCE($5, industry),
            tax_id = COALESCE($6, tax_id),
            email = COALESCE($7, email),
            phone = COALESCE($8, phone),
            website = COALESCE($9, website),
            logo_url = COALESCE($10, logo_url),
            brand_color = COALESCE($11, brand_color),
            invoice_footer_text = COALESCE($12, invoice_footer_text),
            default_payment_terms_days = COALESCE($13, default_payment_terms_days),
            currency_code = COALESCE($14, currency_code),
            timezone = COALESCE($15, timezone),
            fiscal_year_start_month = COALESCE($16, fiscal_year_start_month),
            fiscal_year_start_day = COALESCE($17, fiscal_year_start_day),
            is_active = COALESCE($18, is_active),
            updated_at = NOW()
        WHERE id = $1::uuid
        RETURNING
            id::text,
            business_name,
            legal_name,
            business_type::text AS business_type,
            industry,
            tax_id,
            email::text,
            phone,
            website,
            logo_url,
            brand_color,
            invoice_footer_text,
            default_payment_terms_days,
            currency_code,
            timezone,
            fiscal_year_start_month,
            fiscal_year_start_day,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(business_id)
    .bind(&payload.business_name)
    .bind(&payload.legal_name)
    .bind(&payload.business_type)
    .bind(&payload.industry)
    .bind(&payload.tax_id)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.website)
    .bind(&payload.logo_url)
    .bind(&payload.brand_color)
    .bind(&payload.invoice_footer_text)
    .bind(payload.default_payment_terms_days)
    .bind(&payload.currency_code)
    .bind(&payload.timezone)
    .bind(payload.fiscal_year_start_month)
    .bind(payload.fiscal_year_start_day)
    .bind(payload.is_active)
    .fetch_optional(pool)
    .await
}

pub async fn list_contacts_by_business(
    pool: &PgPool,
    business_id: &str,
) -> Result<Vec<ContactResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            id::text,
            business_id::text,
            contact_type::text AS contact_type,
            display_name,
            legal_name,
            email::text,
            phone,
            tax_id,
            payment_terms_days,
            notes,
            is_active,
            created_at,
            updated_at
        FROM contacts
        WHERE business_id = $1::uuid
        ORDER BY created_at DESC, display_name
        "#,
    )
    .bind(business_id)
    .fetch_all(pool)
    .await
}

pub async fn get_contact_by_id(
    pool: &PgPool,
    contact_id: &str,
) -> Result<Option<ContactResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            id::text,
            business_id::text,
            contact_type::text AS contact_type,
            display_name,
            legal_name,
            email::text,
            phone,
            tax_id,
            payment_terms_days,
            notes,
            is_active,
            created_at,
            updated_at
        FROM contacts
        WHERE id = $1::uuid
        "#,
    )
    .bind(contact_id)
    .fetch_optional(pool)
    .await
}

pub async fn create_contact(
    pool: &PgPool,
    business_id: &str,
    payload: &CreateContactRequest,
) -> Result<ContactResponse, sqlx::Error> {
    sqlx::query_as(
        r#"
        INSERT INTO contacts (
            business_id, contact_type, display_name, legal_name, email, phone, tax_id,
            payment_terms_days, notes
        )
        VALUES (
            $1::uuid,
            COALESCE($2::contact_type, 'customer'::contact_type),
            $3,
            $4,
            $5,
            $6,
            $7,
            COALESCE($8, 30),
            $9
        )
        RETURNING
            id::text,
            business_id::text,
            contact_type::text AS contact_type,
            display_name,
            legal_name,
            email::text,
            phone,
            tax_id,
            payment_terms_days,
            notes,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(business_id)
    .bind(&payload.contact_type)
    .bind(&payload.display_name)
    .bind(&payload.legal_name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.tax_id)
    .bind(payload.payment_terms_days)
    .bind(&payload.notes)
    .fetch_one(pool)
    .await
}

pub async fn update_contact(
    pool: &PgPool,
    contact_id: &str,
    payload: &UpdateContactRequest,
) -> Result<Option<ContactResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        UPDATE contacts
        SET
            contact_type = COALESCE($2::contact_type, contact_type),
            display_name = COALESCE($3, display_name),
            legal_name = COALESCE($4, legal_name),
            email = COALESCE($5, email),
            phone = COALESCE($6, phone),
            tax_id = COALESCE($7, tax_id),
            payment_terms_days = COALESCE($8, payment_terms_days),
            notes = COALESCE($9, notes),
            is_active = COALESCE($10, is_active),
            updated_at = NOW()
        WHERE id = $1::uuid
        RETURNING
            id::text,
            business_id::text,
            contact_type::text AS contact_type,
            display_name,
            legal_name,
            email::text,
            phone,
            tax_id,
            payment_terms_days,
            notes,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(contact_id)
    .bind(&payload.contact_type)
    .bind(&payload.display_name)
    .bind(&payload.legal_name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.tax_id)
    .bind(payload.payment_terms_days)
    .bind(&payload.notes)
    .bind(payload.is_active)
    .fetch_optional(pool)
    .await
}

pub async fn deactivate_contact(
    pool: &PgPool,
    contact_id: &str,
) -> Result<Option<ContactResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        UPDATE contacts
        SET is_active = FALSE,
            updated_at = NOW()
        WHERE id = $1::uuid
        RETURNING
            id::text,
            business_id::text,
            contact_type::text AS contact_type,
            display_name,
            legal_name,
            email::text,
            phone,
            tax_id,
            payment_terms_days,
            notes,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(contact_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_journal_entries(
    pool: &PgPool,
    query: &JournalEntriesQuery,
) -> Result<Vec<JournalEntrySummary>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            je.id::text,
            je.entry_date,
            je.description,
            je.reference_number,
            COUNT(jl.id)::bigint AS line_count,
            COALESCE(SUM(CASE WHEN jl.amount > 0 THEN jl.amount ELSE 0 END), 0) AS debit_total,
            ABS(COALESCE(SUM(CASE WHEN jl.amount < 0 THEN jl.amount ELSE 0 END), 0)) AS credit_total,
            je.created_at
        FROM journal_entries je
        JOIN journal_lines jl ON jl.journal_entry_id = je.id
        JOIN accounts a ON a.id = jl.account_id
        WHERE ($3::text IS NULL OR a.code = $3)
          AND ($4::text IS NULL OR jl.job_code = $4)
          AND ($5::date IS NULL OR je.entry_date >= $5::date)
          AND ($6::date IS NULL OR je.entry_date <= $6::date)
        GROUP BY je.id, je.entry_date, je.description, je.reference_number, je.created_at
        ORDER BY je.entry_date DESC, je.created_at DESC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(query.normalized_limit())
    .bind(query.normalized_offset())
    .bind(&query.account_code)
    .bind(&query.job_code)
    .bind(&query.start_date)
    .bind(&query.end_date)
    .fetch_all(pool)
    .await
}

pub async fn get_journal_entry(
    pool: &PgPool,
    entry_id: &str,
) -> Result<Option<JournalEntryResponse>, sqlx::Error> {
    let header: Option<JournalEntryHeader> = sqlx::query_as(
        r#"
        SELECT
            id::text,
            entry_date,
            description,
            reference_number,
            created_at
        FROM journal_entries
        WHERE id = $1::uuid
        "#,
    )
    .bind(entry_id)
    .fetch_optional(pool)
    .await?;

    let Some(header) = header else {
        return Ok(None);
    };

    let lines = sqlx::query_as(
        r#"
        SELECT
            jl.id::text,
            a.id::text AS account_id,
            a.code AS account_code,
            a.name AS account_name,
            a.type::text AS account_type,
            jl.amount,
            jl.job_code,
            jl.created_at
        FROM journal_lines jl
        JOIN accounts a ON a.id = jl.account_id
        WHERE jl.journal_entry_id = $1::uuid
        ORDER BY jl.created_at, jl.id
        "#,
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await?;

    Ok(Some(JournalEntryResponse::from_parts(header, lines)))
}

pub async fn get_trial_balance(pool: &PgPool) -> Result<Vec<TrialBalanceLine>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            a.id::text AS account_id,
            a.code AS account_code,
            a.name AS account_name,
            a.type::text AS account_type,
            COALESCE(SUM(jl.amount), 0) AS balance
        FROM accounts a
        LEFT JOIN journal_lines jl ON jl.account_id = a.id
        WHERE a.is_active = TRUE
        GROUP BY a.id, a.code, a.name, a.type
        ORDER BY a.code
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn get_account_ledger(
    pool: &PgPool,
    account_code: &str,
) -> Result<Vec<AccountLedgerLine>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            je.id::text AS journal_entry_id,
            jl.id::text AS journal_line_id,
            je.entry_date,
            je.description,
            je.reference_number,
            jl.amount,
            jl.job_code,
            SUM(jl.amount) OVER (
                ORDER BY je.entry_date, je.created_at, jl.created_at, jl.id
            ) AS running_balance
        FROM journal_lines jl
        JOIN journal_entries je ON je.id = jl.journal_entry_id
        JOIN accounts a ON a.id = jl.account_id
        WHERE a.code = $1
        ORDER BY je.entry_date, je.created_at, jl.created_at, jl.id
        "#,
    )
    .bind(account_code)
    .fetch_all(pool)
    .await
}
