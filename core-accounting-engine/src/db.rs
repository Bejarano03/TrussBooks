use crate::models::{
    AccountLedgerLine, AccountResponse, CreateJournalEntry, JournalEntriesQuery,
    JournalEntryHeader, JournalEntryResponse, JournalEntrySummary, TrialBalanceLine,
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
