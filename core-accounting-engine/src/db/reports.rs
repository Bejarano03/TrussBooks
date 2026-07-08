use crate::models::{AccountLedgerLine, TrialBalanceLine};
use sqlx::PgPool;

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
