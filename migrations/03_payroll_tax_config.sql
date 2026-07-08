CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Payroll tax configuration, versioned by year and customizable by business.
CREATE TYPE payroll_tax_scope AS ENUM (
    'federal',
    'state',
    'local',
    'company'
);

CREATE TYPE payroll_tax_basis AS ENUM (
    'gross_pay',
    'taxable_wages',
    'wages',
    'flat_amount',
    'custom'
);

CREATE TABLE payroll_tax_years (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tax_year SMALLINT NOT NULL UNIQUE,
    label VARCHAR(50) NOT NULL,
    effective_from DATE NOT NULL,
    effective_to DATE,
    source_name VARCHAR(255),
    source_version VARCHAR(100),
    raw_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE TABLE payroll_tax_jurisdictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tax_year_id UUID NOT NULL REFERENCES payroll_tax_years(id) ON DELETE CASCADE,
    business_id UUID REFERENCES businesses(id) ON DELETE CASCADE,
    scope payroll_tax_scope NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    country CHAR(2) NOT NULL DEFAULT 'US',
    state_region VARCHAR(100),
    locality VARCHAR(120),
    raw_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_payroll_tax_jurisdictions_year ON payroll_tax_jurisdictions(tax_year_id);
CREATE INDEX idx_payroll_tax_jurisdictions_business ON payroll_tax_jurisdictions(business_id);
CREATE INDEX idx_payroll_tax_jurisdictions_scope ON payroll_tax_jurisdictions(scope);
CREATE UNIQUE INDEX ux_payroll_tax_jurisdictions_scope_code
    ON payroll_tax_jurisdictions (
        tax_year_id,
        scope,
        code,
        COALESCE(business_id, '00000000-0000-0000-0000-000000000000'::uuid)
    );

CREATE TABLE payroll_tax_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tax_year_id UUID NOT NULL REFERENCES payroll_tax_years(id) ON DELETE CASCADE,
    jurisdiction_id UUID NOT NULL REFERENCES payroll_tax_jurisdictions(id) ON DELETE CASCADE,
    business_id UUID REFERENCES businesses(id) ON DELETE CASCADE,
    rule_code VARCHAR(100) NOT NULL,
    rule_name VARCHAR(255) NOT NULL,
    tax_basis payroll_tax_basis NOT NULL DEFAULT 'taxable_wages',
    employee_rate NUMERIC(9, 6) NOT NULL DEFAULT 0,
    employer_rate NUMERIC(9, 6) NOT NULL DEFAULT 0,
    wage_base_limit NUMERIC(19, 4),
    minimum_income NUMERIC(19, 4),
    maximum_income NUMERIC(19, 4),
    withholding_formula TEXT,
    effective_from DATE NOT NULL,
    effective_to DATE,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    raw_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_payroll_tax_rules_year ON payroll_tax_rules(tax_year_id);
CREATE INDEX idx_payroll_tax_rules_jurisdiction ON payroll_tax_rules(jurisdiction_id);
CREATE INDEX idx_payroll_tax_rules_business ON payroll_tax_rules(business_id);
CREATE INDEX idx_payroll_tax_rules_active ON payroll_tax_rules(is_active);
CREATE UNIQUE INDEX ux_payroll_tax_rules_versioned
    ON payroll_tax_rules (
        tax_year_id,
        jurisdiction_id,
        rule_code,
        effective_from,
        COALESCE(business_id, '00000000-0000-0000-0000-000000000000'::uuid)
    );

CREATE TABLE payroll_tax_brackets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payroll_tax_rule_id UUID NOT NULL REFERENCES payroll_tax_rules(id) ON DELETE CASCADE,
    bracket_order INTEGER NOT NULL,
    lower_bound NUMERIC(19, 4) NOT NULL DEFAULT 0,
    upper_bound NUMERIC(19, 4),
    employee_rate NUMERIC(9, 6),
    employer_rate NUMERIC(9, 6),
    flat_amount NUMERIC(19, 4),
    raw_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    UNIQUE (payroll_tax_rule_id, bracket_order)
);

CREATE INDEX idx_payroll_tax_brackets_rule ON payroll_tax_brackets(payroll_tax_rule_id);

CREATE TABLE payroll_tax_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tax_year_id UUID NOT NULL REFERENCES payroll_tax_years(id) ON DELETE CASCADE,
    business_id UUID REFERENCES businesses(id) ON DELETE CASCADE,
    imported_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    source_filename VARCHAR(255),
    raw_json JSONB NOT NULL,
    notes TEXT,
    imported_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_payroll_tax_snapshots_year ON payroll_tax_snapshots(tax_year_id);
CREATE INDEX idx_payroll_tax_snapshots_business ON payroll_tax_snapshots(business_id);
CREATE INDEX idx_payroll_tax_snapshots_user ON payroll_tax_snapshots(imported_by_user_id);
