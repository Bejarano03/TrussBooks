use sqlx::PgPool;
use crate::models::CreateJournalEntry;

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

    let accounts = vec!(
        ("1010", "Operating Checking", "asset"),
        ("1200", "Accounts Receivable", "asset"),
        ("1250", "Retention Receivable", "asset"), 
        ("2010", "Accounts Payable", "liability"),
        ("3000", "Owner's Equity", "equity"),
        ("4010", "Construction Contract Revenue", "revenue"),
        ("5010", "Job Materials Expense", "expense"),
        ("5020", "Subcontractor Expense", "expense"),
        ("5030", "Equipment Rental Expense", "expense"),
    );

    for (code, name, acct_type) in accounts {
        sqlx::query(
            "INSERT INTO accounts (code, name, type) VALUES ($1, $2, $3::account_type)"
        )
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
        "#
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
        let account_record: Option<(String,)> = sqlx::query_as(
            "SELECT id::text FROM accounts WHERE code = $1"
        )
        .bind(&line.account_code)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        let account_id = match account_record {
            Some(record) => record.0,
            None => {
                // If the account code is invalid, returning an Err here will 
                // automatically abort the transaction and rollback the header!
                return Err(format!("Invalid account code provided: {}", line.account_code));
            }
        };

        // Insert the line, casting our text UUIDs back to proper DB uuids
        sqlx::query(
            r#"
            INSERT INTO journal_lines (journal_entry_id, account_id, amount, job_code)
            VALUES ($1::uuid, $2::uuid, $3, $4)
            "#
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