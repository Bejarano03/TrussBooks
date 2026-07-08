CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citext;

-- Core enums for tenant-aware bookkeeping and access control.
CREATE TYPE business_type AS ENUM (
    'sole_proprietorship',
    'llc',
    'corporation',
    'partnership',
    'non_profit',
    'other'
);

CREATE TYPE address_type AS ENUM (
    'billing',
    'shipping',
    'mailing',
    'physical'
);

CREATE TYPE membership_role AS ENUM (
    'owner',
    'admin',
    'accountant',
    'bookkeeper',
    'staff',
    'viewer'
);

CREATE TYPE contact_type AS ENUM (
    'customer',
    'vendor',
    'both',
    'other'
);

CREATE TYPE tax_type AS ENUM (
    'sales_tax',
    'vat',
    'gst',
    'hst',
    'withholding',
    'other'
);

CREATE TYPE invoice_status AS ENUM (
    'draft',
    'sent',
    'partially_paid',
    'paid',
    'void',
    'overdue'
);

-- Authentication and access control.
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email CITEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    email_verified_at TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE TABLE auth_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash TEXT NOT NULL UNIQUE,
    user_agent TEXT,
    ip_address INET,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE TABLE email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_auth_sessions_user ON auth_sessions(user_id);
CREATE INDEX idx_auth_sessions_expires ON auth_sessions(expires_at);
CREATE INDEX idx_password_reset_tokens_user ON password_reset_tokens(user_id);
CREATE INDEX idx_email_verification_tokens_user ON email_verification_tokens(user_id);

-- Business profile and tenant membership.
CREATE TABLE businesses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_name VARCHAR(255) NOT NULL,
    legal_name VARCHAR(255),
    business_type business_type NOT NULL DEFAULT 'other',
    industry VARCHAR(255),
    tax_id VARCHAR(50),
    email CITEXT,
    phone VARCHAR(50),
    website VARCHAR(255),
    logo_url TEXT,
    brand_color VARCHAR(20),
    invoice_footer_text TEXT,
    default_payment_terms_days SMALLINT NOT NULL DEFAULT 30,
    currency_code CHAR(3) NOT NULL DEFAULT 'USD',
    timezone VARCHAR(100) NOT NULL DEFAULT 'America/Los_Angeles',
    fiscal_year_start_month SMALLINT NOT NULL DEFAULT 1,
    fiscal_year_start_day SMALLINT NOT NULL DEFAULT 1,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE TABLE business_memberships (
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role membership_role NOT NULL DEFAULT 'bookkeeper',
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    PRIMARY KEY (business_id, user_id)
);

CREATE INDEX idx_business_memberships_user ON business_memberships(user_id);
CREATE INDEX idx_business_memberships_role ON business_memberships(role);

CREATE TABLE business_addresses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    address_type address_type NOT NULL DEFAULT 'physical',
    label VARCHAR(100),
    line1 VARCHAR(255) NOT NULL,
    line2 VARCHAR(255),
    city VARCHAR(120) NOT NULL,
    state_region VARCHAR(120),
    postal_code VARCHAR(20),
    country CHAR(2) NOT NULL DEFAULT 'US',
    is_primary BOOLEAN DEFAULT FALSE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_business_addresses_business ON business_addresses(business_id);
CREATE INDEX idx_business_addresses_primary ON business_addresses(business_id, is_primary);

-- Customers, vendors, and other contacts for invoicing.
CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    contact_type contact_type NOT NULL DEFAULT 'customer',
    display_name VARCHAR(255) NOT NULL,
    legal_name VARCHAR(255),
    email CITEXT,
    phone VARCHAR(50),
    tax_id VARCHAR(50),
    payment_terms_days SMALLINT NOT NULL DEFAULT 30,
    notes TEXT,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_contacts_business ON contacts(business_id);
CREATE INDEX idx_contacts_business_type ON contacts(business_id, contact_type);
CREATE INDEX idx_contacts_name ON contacts(display_name);

CREATE TABLE contact_addresses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    address_type address_type NOT NULL DEFAULT 'billing',
    label VARCHAR(100),
    line1 VARCHAR(255) NOT NULL,
    line2 VARCHAR(255),
    city VARCHAR(120) NOT NULL,
    state_region VARCHAR(120),
    postal_code VARCHAR(20),
    country CHAR(2) NOT NULL DEFAULT 'US',
    is_primary BOOLEAN DEFAULT FALSE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_contact_addresses_contact ON contact_addresses(contact_id);
CREATE INDEX idx_contact_addresses_primary ON contact_addresses(contact_id, is_primary);

-- Tax setup.
CREATE TABLE tax_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    tax_type tax_type NOT NULL DEFAULT 'sales_tax',
    jurisdiction VARCHAR(255),
    rate NUMERIC(9, 6) NOT NULL CHECK (rate >= 0),
    is_compound BOOLEAN DEFAULT FALSE NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    effective_from DATE,
    effective_to DATE,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_tax_rates_business ON tax_rates(business_id);
CREATE INDEX idx_tax_rates_active ON tax_rates(business_id, is_active);

-- Invoicing and payments.
CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    customer_id UUID NOT NULL REFERENCES contacts(id),
    invoice_number VARCHAR(100) NOT NULL,
    invoice_date DATE NOT NULL,
    due_date DATE,
    status invoice_status NOT NULL DEFAULT 'draft',
    subtotal NUMERIC(19, 4) NOT NULL DEFAULT 0,
    tax_total NUMERIC(19, 4) NOT NULL DEFAULT 0,
    discount_total NUMERIC(19, 4) NOT NULL DEFAULT 0,
    total NUMERIC(19, 4) NOT NULL DEFAULT 0,
    amount_paid NUMERIC(19, 4) NOT NULL DEFAULT 0,
    balance_due NUMERIC(19, 4) NOT NULL DEFAULT 0,
    currency_code CHAR(3) NOT NULL DEFAULT 'USD',
    notes TEXT,
    terms TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    UNIQUE (business_id, invoice_number)
);

CREATE INDEX idx_invoices_business ON invoices(business_id);
CREATE INDEX idx_invoices_customer ON invoices(customer_id);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_due_date ON invoices(due_date);

CREATE TABLE invoice_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    description TEXT NOT NULL,
    account_id UUID REFERENCES accounts(id),
    quantity NUMERIC(19, 4) NOT NULL DEFAULT 1,
    unit_price NUMERIC(19, 4) NOT NULL DEFAULT 0,
    discount_amount NUMERIC(19, 4) NOT NULL DEFAULT 0,
    line_subtotal NUMERIC(19, 4) NOT NULL DEFAULT 0,
    tax_rate_id UUID REFERENCES tax_rates(id) ON DELETE SET NULL,
    tax_amount NUMERIC(19, 4) NOT NULL DEFAULT 0,
    line_total NUMERIC(19, 4) NOT NULL DEFAULT 0,
    job_code VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    UNIQUE (invoice_id, line_number)
);

CREATE INDEX idx_invoice_lines_invoice ON invoice_lines(invoice_id);
CREATE INDEX idx_invoice_lines_account ON invoice_lines(account_id);
CREATE INDEX idx_invoice_lines_tax_rate ON invoice_lines(tax_rate_id);

CREATE TABLE invoice_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    payment_date DATE NOT NULL,
    amount NUMERIC(19, 4) NOT NULL,
    payment_method VARCHAR(50) NOT NULL,
    reference_number VARCHAR(100),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_invoice_payments_invoice ON invoice_payments(invoice_id);
CREATE INDEX idx_invoice_payments_date ON invoice_payments(payment_date);
