use crate::models::{BusinessResponse, CreateBusinessRequest, UpdateBusinessRequest};
use sqlx::PgPool;

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
