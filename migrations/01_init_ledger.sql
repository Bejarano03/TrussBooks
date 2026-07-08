CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- 1. Create a custom enum for the main financial categories
CREATE TYPE account_type AS ENUM ('asset', 'liability', 'equity', 'revenue', 'expense');

-- 2. The Chart of Accounts Table
CREATE TABLE accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) UNIQUE NOT NULL,         -- e.g., "1010" for Main Checking, "4010" for Construction Income
    name VARCHAR(255) NOT NULL,                -- e.g., "Operating Checking Account"
    type account_type NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- 3. The Journal Entries Table (The Header/Wrapper)
-- This acts as the envelope for a financial transaction event.
CREATE TABLE journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_date DATE NOT NULL,                  -- The actual date the transaction took place
    description TEXT NOT NULL,                 -- e.g., "Home Depot - Project #102 Drywall Materials"
    reference_number VARCHAR(100),            -- Check number, invoice number, or receipt ID
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- 4. The Journal Lines Table (The Entry Details)
-- This holds the split items. A single entry must have at least 2 lines.
CREATE TABLE journal_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    journal_entry_id UUID NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts(id),
    
    -- Financial Data: Using NUMERIC to prevent floating point errors.
    -- Storing up to 4 decimal places ensures precision for partial unit costs or tax distributions.
    amount NUMERIC(19, 4) NOT NULL, 
    
    -- Direction: Instead of separate debit/credit columns, standard double-entry engines 
    -- frequently use a strict mathematical approach: Positives are DEBITS, Negatives are CREDITS.
    -- Alternatively, you can use an enum. Let's use explicit math here for easy summation.
    
    -- Construction Job Costing Hook
    job_code VARCHAR(100),                     -- Optional tag referencing your specific job/project ID
    
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- 5. Data Integrity: Indexing for fast reporting
CREATE INDEX idx_journal_lines_entry ON journal_lines(journal_entry_id);
CREATE INDEX idx_journal_lines_account ON journal_lines(account_id);
CREATE INDEX idx_journal_lines_job ON journal_lines(job_code) WHERE job_code IS NOT NULL;
