use crate::models::{ContactResponse, CreateContactRequest, UpdateContactRequest};
use sqlx::PgPool;

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
