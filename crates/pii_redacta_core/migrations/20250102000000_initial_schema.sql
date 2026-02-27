-- Initial schema for PII Redacta
-- Tables: tiers, users, subscriptions, api_keys, usage_logs, ip_blocks

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================
-- Tiers table (configurable plans)
-- ============================================
CREATE TABLE tiers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(50) UNIQUE NOT NULL,           -- 'trial', 'starter', 'pro', 'enterprise'
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- JSONB configuration for flexible tier definitions
    limits JSONB NOT NULL DEFAULT '{}',
    features JSONB NOT NULL DEFAULT '{}',
    
    -- Pricing (NULL for trial)
    monthly_price_cents INTEGER,
    yearly_price_cents INTEGER,
    
    -- Metadata
    is_public BOOLEAN NOT NULL DEFAULT true,     -- Show on pricing page
    is_active BOOLEAN NOT NULL DEFAULT true,     -- Can new users subscribe?
    sort_order INTEGER NOT NULL DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for active tiers lookup
CREATE INDEX idx_tiers_active ON tiers(is_active, sort_order);

-- ============================================
-- Users table
-- ============================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    email_verified_at TIMESTAMPTZ,
    
    -- Password (Argon2id hash)
    password_hash VARCHAR(255),
    
    -- Profile
    display_name VARCHAR(100),
    company_name VARCHAR(100),
    
    -- Email preferences
    email_notifications_enabled BOOLEAN NOT NULL DEFAULT true,
    
    -- Metadata
    is_admin BOOLEAN NOT NULL DEFAULT false,
    last_login_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ                       -- Soft delete
);

-- Indexes for users
CREATE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_admin ON users(is_admin) WHERE deleted_at IS NULL;

-- ============================================
-- Subscriptions table
-- ============================================
CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tier_id UUID NOT NULL REFERENCES tiers(id),
    
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'trial'
        CHECK (status IN ('trial', 'active', 'past_due', 'cancelled', 'expired')),
    
    -- Billing period
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    
    -- Cancellation
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
    cancelled_at TIMESTAMPTZ,
    
    -- Stripe (optional)
    stripe_customer_id VARCHAR(255),
    stripe_subscription_id VARCHAR(255),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for subscriptions
CREATE INDEX idx_subscriptions_user ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_tier ON subscriptions(tier_id);
CREATE INDEX idx_subscriptions_stripe ON subscriptions(stripe_subscription_id) WHERE stripe_subscription_id IS NOT NULL;

-- Only one active subscription per user
CREATE UNIQUE INDEX idx_subscriptions_active_user ON subscriptions(user_id) WHERE status IN ('trial', 'active', 'past_due');

-- ============================================
-- API Keys table
-- ============================================
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Key identifier (first 8 chars of key for display)
    key_prefix VARCHAR(16) NOT NULL,
    
    -- HMAC hash of the actual key (not stored in plaintext!)
    key_hash VARCHAR(255) NOT NULL,
    
    -- Key metadata
    name VARCHAR(100) NOT NULL DEFAULT 'API Key',
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,                     -- NULL = never expires
    
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    revoked_at TIMESTAMPTZ,
    revoked_reason TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for API keys
CREATE INDEX idx_api_keys_user ON api_keys(user_id);
CREATE INDEX idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX idx_api_keys_active ON api_keys(user_id, is_active) WHERE is_active = true;

-- ============================================
-- Usage Logs table (for analytics and limits)
-- ============================================
CREATE TABLE usage_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    api_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    
    -- Request details
    request_type VARCHAR(50) NOT NULL           -- 'playground', 'api_detect', 'api_redact', etc.
        CHECK (request_type IN ('playground', 'api_detect', 'api_redact', 'api_detect_stream')),
    
    -- Resource usage
    file_name VARCHAR(255),
    file_size_bytes INTEGER,
    file_type VARCHAR(50),                      -- 'pdf', 'docx', 'txt', etc.
    
    -- Processing stats
    processing_time_ms INTEGER,
    page_count INTEGER,
    detections_count INTEGER,
    
    -- Result
    success BOOLEAN NOT NULL,
    error_message TEXT,
    
    -- IP for rate limiting
    ip_address INET,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for usage logs (time-series, partition-ready)
CREATE INDEX idx_usage_logs_user_created ON usage_logs(user_id, created_at DESC);
CREATE INDEX idx_usage_logs_created ON usage_logs(created_at DESC);
CREATE INDEX idx_usage_logs_user_request ON usage_logs(user_id, request_type, created_at DESC);
CREATE INDEX idx_usage_logs_ip ON usage_logs(ip_address, created_at DESC) WHERE ip_address IS NOT NULL;

-- ============================================
-- IP Blocks table (security)
-- ============================================
CREATE TABLE ip_blocks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    ip_address INET NOT NULL,
    ip_range CIDR,                              -- For range blocks
    
    -- Block details
    reason VARCHAR(255) NOT NULL,
    blocked_by UUID REFERENCES users(id),       -- NULL = system block
    
    -- Duration (NULL = permanent)
    expires_at TIMESTAMPTZ,
    
    -- Metadata
    is_active BOOLEAN NOT NULL DEFAULT true,
    hit_count INTEGER NOT NULL DEFAULT 0,       -- How many requests were blocked
    last_hit_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for IP blocks
CREATE UNIQUE INDEX idx_ip_blocks_address ON ip_blocks(ip_address) WHERE ip_range IS NULL AND is_active = true;
CREATE INDEX idx_ip_blocks_range ON ip_blocks USING GIST (ip_range inet_ops) WHERE ip_range IS NOT NULL AND is_active = true;
CREATE INDEX idx_ip_blocks_active ON ip_blocks(is_active, expires_at);

-- ============================================
-- Insert default tiers
-- ============================================

-- Trial tier
INSERT INTO tiers (name, display_name, description, limits, features, monthly_price_cents, yearly_price_cents, sort_order)
VALUES (
    'trial',
    'Trial',
    '14-day free trial with full access. No credit card required.',
    '{
        "api_enabled": true,
        "max_api_keys": 2,
        "max_file_size": 10485760,
        "max_files_per_month": 50,
        "max_pages_per_file": 100,
        "max_total_size": 524288000,
        "playground_max_daily": 5,
        "playground_max_file_size": 1048576,
        "retention_days": 7
    }'::jsonb,
    '{
        "batch_processing": false,
        "custom_rules": false,
        "email_support": false,
        "playground": true,
        "rate_limit_per_minute": 10,
        "sla": null,
        "webhooks": false
    }'::jsonb,
    NULL,
    NULL,
    0
);

-- Starter tier
INSERT INTO tiers (name, display_name, description, limits, features, monthly_price_cents, yearly_price_cents, sort_order)
VALUES (
    'starter',
    'Starter',
    'For individuals and small projects.',
    '{
        "api_enabled": true,
        "max_api_keys": 3,
        "max_file_size": 26214400,
        "max_files_per_month": 500,
        "max_pages_per_file": 500,
        "max_total_size": 5368709120,
        "playground_max_daily": 25,
        "playground_max_file_size": 5242880,
        "retention_days": 30
    }'::jsonb,
    '{
        "batch_processing": false,
        "custom_rules": false,
        "email_support": true,
        "playground": true,
        "rate_limit_per_minute": 30,
        "sla": "99%",
        "webhooks": false
    }'::jsonb,
    999,
    9990,
    1
);

-- Pro tier
INSERT INTO tiers (name, display_name, description, limits, features, monthly_price_cents, yearly_price_cents, sort_order)
VALUES (
    'pro',
    'Pro',
    'For growing teams with higher volume needs.',
    '{
        "api_enabled": true,
        "max_api_keys": 10,
        "max_file_size": 104857600,
        "max_files_per_month": 5000,
        "max_pages_per_file": 1000,
        "max_total_size": 53687091200,
        "playground_max_daily": 100,
        "playground_max_file_size": 10485760,
        "retention_days": 90
    }'::jsonb,
    '{
        "batch_processing": true,
        "custom_rules": true,
        "email_support": true,
        "playground": true,
        "rate_limit_per_minute": 100,
        "sla": "99.9%",
        "webhooks": true
    }'::jsonb,
    4999,
    49990,
    2
);

-- Enterprise tier
INSERT INTO tiers (name, display_name, description, limits, features, monthly_price_cents, yearly_price_cents, sort_order)
VALUES (
    'enterprise',
    'Enterprise',
    'Custom solutions for large organizations. Contact us for pricing.',
    '{
        "api_enabled": true,
        "max_api_keys": null,
        "max_file_size": null,
        "max_files_per_month": null,
        "max_pages_per_file": null,
        "max_total_size": null,
        "playground_max_daily": null,
        "playground_max_file_size": null,
        "retention_days": null
    }'::jsonb,
    '{
        "batch_processing": true,
        "custom_rules": true,
        "email_support": true,
        "playground": true,
        "rate_limit_per_minute": null,
        "sla": "99.99%",
        "webhooks": true
    }'::jsonb,
    NULL,
    NULL,
    3
);

-- ============================================
-- Update timestamps trigger
-- ============================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_tiers_updated_at BEFORE UPDATE ON tiers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_subscriptions_updated_at BEFORE UPDATE ON subscriptions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
