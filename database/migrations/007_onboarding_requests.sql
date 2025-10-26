-- Migration 007: Onboarding Request Management
-- Create tables for managing onboarding requests and their DSL configurations

-- Onboarding Request table - high-level onboarding requests
CREATE TABLE IF NOT EXISTS onboarding_request (
    id SERIAL PRIMARY KEY,
    onboarding_id VARCHAR(50) UNIQUE NOT NULL, -- Human-readable ID like "OR-2025-00042"
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'draft', -- draft, compiled, executing, completed, failed
    cbu_id VARCHAR(100), -- References cbu_profile.cbu_id
    created_by VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Onboarding Request DSL table - stores the DSL configuration for each request
CREATE TABLE IF NOT EXISTS onboarding_request_dsl (
    onboarding_request_id INTEGER PRIMARY KEY REFERENCES onboarding_request(id) ON DELETE CASCADE,
    instance_id VARCHAR(100) NOT NULL, -- Same as onboarding_id for now
    products TEXT[], -- Array of product IDs like ['GlobalCustody@v3', 'FundAccounting@v2']
    team_users JSONB DEFAULT '[]'::jsonb, -- JSON array of team users
    cbu_profile JSONB DEFAULT '{}'::jsonb, -- JSON object with CBU-specific config
    workflow_config JSONB DEFAULT '{}'::jsonb, -- Additional workflow configuration
    template_version VARCHAR(50) DEFAULT 'v1',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Onboarding Execution Plans table - stores compiled execution plans
CREATE TABLE IF NOT EXISTS onboarding_execution_plan (
    id SERIAL PRIMARY KEY,
    onboarding_request_id INTEGER NOT NULL REFERENCES onboarding_request(id) ON DELETE CASCADE,
    plan_version INTEGER DEFAULT 1,
    plan_data JSONB NOT NULL, -- The compiled execution plan
    idd_data JSONB, -- Information Dependency Diagram (data gaps)
    bindings_data JSONB, -- Resource bindings
    compiled_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    compiled_by VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    UNIQUE(onboarding_request_id, plan_version)
);

-- Onboarding Execution Log table - tracks execution history
CREATE TABLE IF NOT EXISTS onboarding_execution_log (
    id SERIAL PRIMARY KEY,
    onboarding_request_id INTEGER NOT NULL REFERENCES onboarding_request(id) ON DELETE CASCADE,
    execution_plan_id INTEGER REFERENCES onboarding_execution_plan(id),
    status VARCHAR(50) NOT NULL, -- running, completed, failed
    started_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE,
    execution_log TEXT[], -- Array of log entries
    error_message TEXT,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Indexes for performance
CREATE INDEX idx_onboarding_request_status ON onboarding_request(status);
CREATE INDEX idx_onboarding_request_cbu ON onboarding_request(cbu_id);
CREATE INDEX idx_onboarding_request_created ON onboarding_request(created_at DESC);
CREATE INDEX idx_execution_plan_request ON onboarding_execution_plan(onboarding_request_id);
CREATE INDEX idx_execution_log_request ON onboarding_execution_log(onboarding_request_id);

-- Function to auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_onboarding_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for auto-updating timestamps
CREATE TRIGGER update_onboarding_request_timestamp
    BEFORE UPDATE ON onboarding_request
    FOR EACH ROW
    EXECUTE FUNCTION update_onboarding_updated_at();

CREATE TRIGGER update_onboarding_request_dsl_timestamp
    BEFORE UPDATE ON onboarding_request_dsl
    FOR EACH ROW
    EXECUTE FUNCTION update_onboarding_updated_at();

-- Comments for documentation
COMMENT ON TABLE onboarding_request IS 'High-level onboarding requests tracking';
COMMENT ON TABLE onboarding_request_dsl IS 'DSL configuration for each onboarding request';
COMMENT ON TABLE onboarding_execution_plan IS 'Compiled execution plans with versioning';
COMMENT ON TABLE onboarding_execution_log IS 'Execution history and audit trail';
