use crate::models::{AccountTemplateAccountResponse, AccountTemplateResponse};
use sqlx::PgPool;

pub async fn list_account_templates(
    pool: &PgPool,
) -> Result<Vec<AccountTemplateResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            at.id::text,
            at.template_key,
            at.template_name,
            at.business_type::text AS business_type,
            at.industry,
            at.description,
            at.is_default,
            COUNT(ata.id)::bigint AS account_count,
            at.created_at,
            at.updated_at
        FROM account_templates at
        LEFT JOIN account_template_accounts ata ON ata.template_id = at.id
        GROUP BY at.id
        ORDER BY at.is_default DESC, at.template_name
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_account_template_accounts(
    pool: &PgPool,
    template_key: &str,
) -> Result<Vec<AccountTemplateAccountResponse>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            ata.code,
            ata.name,
            ata.type::text AS account_type,
            ata.sort_order,
            ata.is_required
        FROM account_templates at
        JOIN account_template_accounts ata ON ata.template_id = at.id
        WHERE at.template_key = $1
        ORDER BY ata.sort_order, ata.code
        "#,
    )
    .bind(template_key)
    .fetch_all(pool)
    .await
}
