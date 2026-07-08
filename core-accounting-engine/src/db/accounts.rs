use crate::models::{AccountResponse, CreateAccountRequest, UpdateAccountRequest};
use sqlx::PgPool;

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
