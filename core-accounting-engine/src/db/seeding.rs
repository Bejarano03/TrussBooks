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
            .execute(pool)
            .await?;
    }

    let final_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts")
        .fetch_one(pool)
        .await?;

    Ok(final_count.0)
}
