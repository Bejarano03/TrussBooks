CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE account_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_key VARCHAR(100) UNIQUE NOT NULL,
    template_name VARCHAR(255) NOT NULL,
    business_type business_type,
    industry VARCHAR(255),
    description TEXT,
    is_default BOOLEAN DEFAULT FALSE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE TABLE account_template_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES account_templates(id) ON DELETE CASCADE,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    type account_type NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_required BOOLEAN DEFAULT TRUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    UNIQUE (template_id, code)
);

CREATE INDEX idx_account_template_accounts_template ON account_template_accounts(template_id);
CREATE INDEX idx_account_templates_default ON account_templates(is_default);

INSERT INTO account_templates (
    template_key,
    template_name,
    business_type,
    industry,
    description,
    is_default
) VALUES (
    'construction_core',
    'Construction Core Chart',
    NULL,
    'construction',
    'Default chart of accounts for construction-oriented businesses.',
    TRUE
);

INSERT INTO account_template_accounts (
    template_id,
    code,
    name,
    type,
    sort_order,
    is_required
)
SELECT
    at.id,
    acct.code,
    acct.name,
    acct.type,
    acct.sort_order,
    acct.is_required
FROM account_templates at
CROSS JOIN (
    VALUES
        ('1010', 'Operating Checking', 'asset'::account_type, 10, TRUE),
        ('1200', 'Accounts Receivable', 'asset'::account_type, 20, TRUE),
        ('1250', 'Retention Receivable', 'asset'::account_type, 30, FALSE),
        ('2010', 'Accounts Payable', 'liability'::account_type, 40, TRUE),
        ('3000', 'Owner''s Equity', 'equity'::account_type, 50, TRUE),
        ('4010', 'Construction Contract Revenue', 'revenue'::account_type, 60, TRUE),
        ('5010', 'Job Materials Expense', 'expense'::account_type, 70, TRUE),
        ('5020', 'Subcontractor Expense', 'expense'::account_type, 80, TRUE),
        ('5030', 'Equipment Rental Expense', 'expense'::account_type, 90, FALSE)
) AS acct(code, name, type, sort_order, is_required)
WHERE at.template_key = 'construction_core';
