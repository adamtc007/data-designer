-- Migration 005: Add Missing Tables for gRPC Server
-- Creates tables that grpc-server code queries but don't exist in database

-- ===== PRODUCT OPTIONS TABLE =====
-- Service configuration options from onboarding library

CREATE TABLE IF NOT EXISTS product_options (
    id SERIAL PRIMARY KEY,
    option_id VARCHAR(100) UNIQUE NOT NULL,
    product_id VARCHAR(100) NOT NULL, -- References products.product_id (string, not integer FK)
    option_name VARCHAR(255) NOT NULL,
    option_category VARCHAR(100), -- e.g., 'instruction_method', 'market_access'
    option_type VARCHAR(50) NOT NULL CHECK (option_type IN ('select', 'multiselect', 'text', 'number', 'boolean')),
    option_value JSONB, -- Current value or configuration
    display_name VARCHAR(255),
    description TEXT,
    pricing_impact DECIMAL(10,2), -- Impact on pricing if this option is selected
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'deprecated')),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_product_options_product ON product_options(product_id);
CREATE INDEX idx_product_options_status ON product_options(status);
CREATE INDEX idx_product_options_category ON product_options(option_category);

-- ===== RESOURCE TEMPLATES TABLE =====
-- Template definitions for instantiating resources

CREATE TABLE IF NOT EXISTS resource_templates (
    id SERIAL PRIMARY KEY,
    template_id VARCHAR(100) UNIQUE NOT NULL,
    template_name VARCHAR(255) NOT NULL,
    template_type VARCHAR(50), -- 'form', 'wizard', 'configuration', etc.
    description TEXT,
    version VARCHAR(20) DEFAULT '1.0',

    -- Template schema and structure
    schema_definition JSONB NOT NULL, -- Full JSON schema for this template
    ui_layout VARCHAR(30) DEFAULT 'vertical-stack'
        CHECK (ui_layout IN ('wizard', 'tabs', 'vertical-stack', 'horizontal-grid', 'accordion')),
    initial_data JSONB DEFAULT '{}'::jsonb, -- Default values

    -- DSL and rules
    validation_rules TEXT, -- DSL validation rules
    generation_examples JSONB DEFAULT '[]'::jsonb, -- AI generation examples

    -- Metadata
    category VARCHAR(100),
    tags TEXT[],
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'deprecated', 'draft')),

    created_by VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_resource_templates_type ON resource_templates(template_type);
CREATE INDEX idx_resource_templates_category ON resource_templates(category);
CREATE INDEX idx_resource_templates_status ON resource_templates(status);

-- ===== RESOURCE INSTANCES TABLE =====
-- Instantiated resources from templates (runtime data)

CREATE TABLE IF NOT EXISTS resource_instances (
    id SERIAL PRIMARY KEY,
    instance_id VARCHAR(100) UNIQUE NOT NULL,
    template_id VARCHAR(100) NOT NULL, -- References resource_templates.template_id
    onboarding_request_id VARCHAR(100), -- Link to onboarding workflow

    -- Instance state
    status VARCHAR(20) DEFAULT 'draft'
        CHECK (status IN ('draft', 'in_progress', 'pending_approval', 'approved', 'active', 'inactive', 'error')),
    instance_data JSONB NOT NULL, -- Current instance data (form values, etc.)
    validation_results JSONB, -- Results of validation rules

    -- Tracking
    created_by VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    activated_at TIMESTAMPTZ,

    -- Audit trail
    version_number INTEGER DEFAULT 1,
    change_history JSONB DEFAULT '[]'::jsonb
);

CREATE INDEX idx_resource_instances_template ON resource_instances(template_id);
CREATE INDEX idx_resource_instances_onboarding ON resource_instances(onboarding_request_id);
CREATE INDEX idx_resource_instances_status ON resource_instances(status);
CREATE INDEX idx_resource_instances_created_by ON resource_instances(created_by);

-- ===== RESOURCE SHEETS TABLE =====
-- Template metadata and attribute definitions (similar to resource_objects but for runtime)

CREATE TABLE IF NOT EXISTS resource_sheets (
    id SERIAL PRIMARY KEY,
    resource_id VARCHAR(100) UNIQUE NOT NULL,
    resource_name VARCHAR(255) NOT NULL,
    description TEXT,
    version VARCHAR(20) DEFAULT '1.0',

    -- Sheet definition
    attribute_definitions JSONB NOT NULL, -- Array of attribute schemas
    ui_configuration JSONB, -- UI layout and rendering hints

    -- Integration
    persistence_mapping JSONB, -- Maps attributes to database tables/fields
    validation_schema JSONB, -- JSON schema for validation

    category VARCHAR(100),
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'deprecated')),

    created_by VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_resource_sheets_category ON resource_sheets(category);
CREATE INDEX idx_resource_sheets_status ON resource_sheets(status);

-- ===== DSL EXECUTION LOGS TABLE =====
-- Execution history for DSL scripts

CREATE TABLE IF NOT EXISTS dsl_execution_logs (
    id SERIAL PRIMARY KEY,
    execution_id UUID DEFAULT gen_random_uuid(),
    instance_id VARCHAR(100), -- Links to resource_instances or other entities

    -- Execution details
    execution_type VARCHAR(50), -- 'validation', 'transformation', 'rule_evaluation', etc.
    dsl_script TEXT NOT NULL,
    execution_status VARCHAR(20) NOT NULL
        CHECK (execution_status IN ('success', 'error', 'warning', 'partial')),

    -- Input/Output
    input_data JSONB,
    output_data JSONB,

    -- Logging
    log_messages JSONB DEFAULT '[]'::jsonb, -- Array of log entries
    error_details TEXT,
    stack_trace TEXT,

    -- Performance
    execution_time_ms INTEGER,

    -- Context
    executed_by VARCHAR(100),
    executed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    context_metadata JSONB -- Additional context
);

CREATE INDEX idx_dsl_logs_instance ON dsl_execution_logs(instance_id);
CREATE INDEX idx_dsl_logs_status ON dsl_execution_logs(execution_status);
CREATE INDEX idx_dsl_logs_type ON dsl_execution_logs(execution_type);
CREATE INDEX idx_dsl_logs_executed_at ON dsl_execution_logs(executed_at);

-- ===== CBU INVESTMENT MANDATE STRUCTURE =====
-- NOTE: cbu_investment_mandate_structure already exists as a VIEW
-- It maps from client_business_units + cbu_members + metadata
-- No table creation needed - view is sufficient

-- ===== CBU MEMBER INVESTMENT ROLES =====
-- NOTE: cbu_member_investment_roles already exists as a VIEW
-- It maps from cbu_members + cbu_roles + client_business_units
-- No table creation needed - view is sufficient

-- ===== UPDATE TRIGGERS =====

CREATE TRIGGER update_product_options_updated_at
    BEFORE UPDATE ON product_options
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_resource_templates_updated_at
    BEFORE UPDATE ON resource_templates
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_resource_instances_updated_at
    BEFORE UPDATE ON resource_instances
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_resource_sheets_updated_at
    BEFORE UPDATE ON resource_sheets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ===== SAMPLE DATA =====

-- Add sample product options
INSERT INTO product_options (option_id, product_id, option_name, option_category, option_type, display_name, description, status) VALUES
('OPT-001', 'CUST-001', 'instruction_method', 'trade_capture', 'select', 'Trade Instruction Method', 'How will you instruct trades?', 'active'),
('OPT-002', 'CUST-001', 'reporting_frequency', 'reporting', 'select', 'Reporting Frequency', 'How often do you require reports?', 'active'),
('OPT-003', 'FUND-001', 'nav_calculation_frequency', 'fund_accounting', 'select', 'NAV Calculation Frequency', 'How often should NAV be calculated?', 'active');

-- Add sample resource template
INSERT INTO resource_templates (template_id, template_name, template_type, description, schema_definition, ui_layout, category, status) VALUES
('TPL-KYC-001', 'Client KYC Form', 'form', 'Standard KYC data collection form',
 '{"fields": [{"name": "legal_name", "type": "string", "required": true}, {"name": "jurisdiction", "type": "enum", "values": ["US", "UK", "EU"]}]}'::jsonb,
 'wizard', 'Compliance', 'active');

COMMENT ON TABLE product_options IS 'Service configuration options for products (from onboarding library)';
COMMENT ON TABLE resource_templates IS 'Template definitions for instantiating resources';
COMMENT ON TABLE resource_instances IS 'Runtime instances of resource templates';
COMMENT ON TABLE resource_sheets IS 'Template metadata and attribute definitions';
COMMENT ON TABLE dsl_execution_logs IS 'Execution history for DSL scripts';
