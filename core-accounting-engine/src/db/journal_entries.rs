use crate::models::{
    CreateJournalEntry, JournalEntriesQuery, JournalEntryHeader, JournalEntryResponse,
    JournalEntrySummary,
};
use sqlx::PgPool;

pub async fn save_journal_entry(
    pool: &PgPool,
    payload: &CreateJournalEntry,
) -> Result<String, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

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

    for line in &payload.lines {
        let account_record: Option<(String,)> =
            sqlx::query_as("SELECT id::text FROM accounts WHERE code = $1")
                .bind(&line.account_code)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

        let account_id = match account_record {
            Some(record) => record.0,
            None => {
                return Err(format!(
                    "Invalid account code provided: {}",
                    line.account_code
                ));
            }
        };

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

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(entry_id)
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
        "#,
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
