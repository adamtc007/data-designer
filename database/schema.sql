-- Data Designer Database Schema with pgvector support
-- Drop and recreate database (be careful in production!)

-- Create database (run this separately as superuser)
-- DROP DATABASE IF EXISTS data_designer;
-- CREATE DATABASE data_designer;

-- Connect to data_designer database before running the rest

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Drop existing tables if they exist
DROP TABLE IF EXISTS rule_dependencies CASCADE;
DROP TABLE IF EXISTS rule_executions CASCADE;
DROP TABLE IF EXISTS rule_versions CASCADE;
DROP TABLE IF EXISTS rules CASCADE;
DROP TABLE IF EXISTS derived_attributes CASCADE;
DROP TABLE IF EXISTS business_attributes CASCADE;
DROP TABLE IF EXISTS attribute_sources CASCADE;
DROP TABLE IF EXISTS data_domains CASCADE;
DROP TABLE IF EXISTS rule_categories CASCADE;

-- Rule Categories
CREATE TABLE rule_categories (
    id SERIAL PRIMARY KEY,
    category_key VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    color VARCHAR(7), -- hex color for UI
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Data Domains (for enums)
CREATE TABLE data_domains (
    id SERIAL PRIMARY KEY,
    domain_name VARCHAR(100) UNIQUE NOT NULL,
    values JSONB NOT NULL, -- Array of possible values
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Attribute Sources
CREATE TABLE attribute_sources (
    id SERIAL PRIMARY KEY,
    source_key VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    trust_level VARCHAR(20) CHECK (trust_level IN ('high', 'medium', 'low')),
    requires_validation BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Business Attributes (source data)
CREATE TABLE business_attributes (
    id SERIAL PRIMARY KEY,
    entity_name VARCHAR(100) NOT NULL, -- e.g., 'Client'
    attribute_name VARCHAR(100) NOT NULL,
    full_path VARCHAR(200) GENERATED ALWAYS AS (entity_name || '.' || attribute_name) STORED,
    data_type VARCHAR(50) NOT NULL, -- String, Number, Boolean, Enum
    sql_type VARCHAR(100), -- VARCHAR(50), DECIMAL(18,2), etc.
    rust_type VARCHAR(100), -- String, i32, bool, etc.
    format_mask VARCHAR(100), -- XXX-999, etc.
    validation_pattern TEXT, -- regex pattern
    domain_id INTEGER REFERENCES data_domains(id),
    source_id INTEGER REFERENCES attribute_sources(id),
    required BOOLEAN DEFAULT FALSE,
    editable BOOLEAN DEFAULT TRUE,
    min_value NUMERIC,
    max_value NUMERIC,
    min_length INTEGER,
    max_length INTEGER,
    description TEXT,
    metadata JSONB, -- Additional flexible metadata
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(entity_name, attribute_name)
);

-- Derived Attributes (calculated via rules)
CREATE TABLE derived_attributes (
    id SERIAL PRIMARY KEY,
    entity_name VARCHAR(100) NOT NULL,
    attribute_name VARCHAR(100) NOT NULL,
    full_path VARCHAR(200) GENERATED ALWAYS AS (entity_name || '.' || attribute_name) STORED,
    data_type VARCHAR(50) NOT NULL,
    sql_type VARCHAR(100),
    rust_type VARCHAR(100),
    domain_id INTEGER REFERENCES data_domains(id),
    description TEXT,
    metadata JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(entity_name, attribute_name)
);

-- Rules table with vector embeddings
CREATE TABLE rules (
    id SERIAL PRIMARY KEY,
    rule_id VARCHAR(50) UNIQUE NOT NULL, -- RULE_001, etc.
    rule_name VARCHAR(200) NOT NULL,
    description TEXT,
    category_id INTEGER REFERENCES rule_categories(id),
    target_attribute_id INTEGER REFERENCES derived_attributes(id),
    rule_definition TEXT NOT NULL, -- The DSL code
    parsed_ast JSONB, -- Parsed abstract syntax tree

    -- Vector embedding for semantic search
    embedding vector(1536), -- OpenAI ada-002 dimension, adjust as needed

    -- Metadata
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'inactive', 'deprecated')),
    version INTEGER DEFAULT 1,
    tags TEXT[],
    performance_metrics JSONB, -- execution time, resource usage, etc.

    -- Audit fields
    created_by VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Search
    search_vector tsvector GENERATED ALWAYS AS (
        setweight(to_tsvector('english', COALESCE(rule_name, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(description, '')), 'B') ||
        setweight(to_tsvector('english', COALESCE(rule_definition, '')), 'C')
    ) STORED
);

-- Rule Dependencies (source attributes for each rule)
CREATE TABLE rule_dependencies (
    id SERIAL PRIMARY KEY,
    rule_id INTEGER REFERENCES rules(id) ON DELETE CASCADE,
    attribute_id INTEGER REFERENCES business_attributes(id),
    dependency_type VARCHAR(20) DEFAULT 'input' CHECK (dependency_type IN ('input', 'lookup', 'reference')),
    UNIQUE(rule_id, attribute_id)
);

-- Rule Version History
CREATE TABLE rule_versions (
    id SERIAL PRIMARY KEY,
    rule_id INTEGER REFERENCES rules(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    rule_definition TEXT NOT NULL,
    change_description TEXT,
    created_by VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(rule_id, version)
);

-- Rule Execution History (for monitoring and debugging)
CREATE TABLE rule_executions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rule_id INTEGER REFERENCES rules(id),
    execution_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    input_data JSONB,
    output_value JSONB,
    execution_duration_ms INTEGER,
    success BOOLEAN,
    error_message TEXT,
    context JSONB -- Additional context about the execution
);

-- Indexes for performance
CREATE INDEX idx_rules_category ON rules(category_id);
CREATE INDEX idx_rules_status ON rules(status);
CREATE INDEX idx_rules_target ON rules(target_attribute_id);
CREATE INDEX idx_rules_search ON rules USING GIN(search_vector);
CREATE INDEX idx_rules_embedding ON rules USING ivfflat(embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX idx_rule_deps_rule ON rule_dependencies(rule_id);
CREATE INDEX idx_rule_deps_attr ON rule_dependencies(attribute_id);
CREATE INDEX idx_executions_rule ON rule_executions(rule_id);
CREATE INDEX idx_executions_time ON rule_executions(execution_time);
CREATE INDEX idx_business_attrs_entity ON business_attributes(entity_name);
CREATE INDEX idx_derived_attrs_entity ON derived_attributes(entity_name);

-- Insert default data
INSERT INTO rule_categories (category_key, name, description, color) VALUES
('risk_assessment', 'Risk Assessment', 'Rules for calculating risk scores and metrics', '#ff6b6b'),
('validation', 'Data Validation', 'Rules for validating data formats and constraints', '#4ecdc4'),
('kyc_validation', 'KYC Validation', 'Rules for KYC completeness and compliance', '#45b7d1'),
('compliance', 'Compliance', 'Rules for regulatory compliance checks', '#96ceb4'),
('classification', 'Classification', 'Rules for categorizing and tiering clients', '#ffeaa7');

INSERT INTO attribute_sources (source_key, name, description, trust_level, requires_validation) VALUES
('client_provided', 'Client Provided', 'Data provided directly by the client', 'medium', true),
('internal_assessment', 'Internal Assessment', 'Data from internal risk assessment', 'high', false),
('system_managed', 'System Managed', 'Data managed by the system', 'high', false),
('screening_result', 'Screening Result', 'Data from external screening services', 'high', false),
('rule_calculated', 'Rule Calculated', 'Data derived from business rules', 'high', false);

INSERT INTO data_domains (domain_name, values, description) VALUES
('RiskLevel', '["LOW", "MEDIUM", "HIGH"]', 'Risk classification levels'),
('KycStatus', '["PENDING", "APPROVED", "REJECTED"]', 'KYC approval status'),
('ServiceTier', '["BRONZE", "SILVER", "GOLD", "PLATINUM"]', 'Client service tier levels');

-- Create a function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
CREATE TRIGGER update_business_attributes_updated_at BEFORE UPDATE ON business_attributes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_derived_attributes_updated_at BEFORE UPDATE ON derived_attributes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rules_updated_at BEFORE UPDATE ON rules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to find similar rules using vector similarity
CREATE OR REPLACE FUNCTION find_similar_rules(
    query_embedding vector(1536),
    match_threshold FLOAT DEFAULT 0.8,
    match_count INT DEFAULT 5
)
RETURNS TABLE(
    rule_id VARCHAR(50),
    rule_name VARCHAR(200),
    similarity FLOAT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        r.rule_id,
        r.rule_name,
        1 - (r.embedding <=> query_embedding) as similarity
    FROM rules r
    WHERE r.embedding IS NOT NULL
    ORDER BY r.embedding <=> query_embedding
    LIMIT match_count;
END;
$$ LANGUAGE plpgsql;

-- Sample data insertion for business attributes
INSERT INTO business_attributes (
    entity_name, attribute_name, data_type, sql_type, rust_type,
    format_mask, validation_pattern, source_id, required, description
) VALUES
('Client', 'client_id', 'String', 'VARCHAR(50)', 'String', 'XXX-999', '^[A-Z]{3}-\d{3,}$', 1, true, 'Unique client identifier'),
('Client', 'legal_entity_name', 'String', 'VARCHAR(255)', 'String', NULL, NULL, 1, true, 'Legal name of the entity'),
('Client', 'lei_code', 'String', 'CHAR(20)', 'String', 'XXXXXXXXXXXXXXXXXXXX', '^[A-Z0-9]{20}$', 1, false, 'Legal Entity Identifier'),
('Client', 'email', 'String', 'VARCHAR(255)', 'String', 'xxx@xxx.xxx', '^[\w.-]+@[\w.-]+\.\w+$', 1, true, 'Primary contact email'),
('Client', 'country_code', 'String', 'CHAR(2)', 'String', NULL, '^[A-Z]{2}$', 1, true, 'ISO country code'),
('Client', 'risk_rating', 'Enum', 'VARCHAR(10)', 'RiskLevel', NULL, NULL, 2, true, 'Risk classification'),
('Client', 'aum_usd', 'Number', 'DECIMAL(18,2)', 'Decimal', '$999,999,999,999.99', NULL, 1, false, 'Assets Under Management in USD'),
('Client', 'kyc_status', 'Enum', 'VARCHAR(10)', 'KycStatus', NULL, NULL, 3, true, 'Current KYC status'),
('Client', 'pep_status', 'Boolean', 'BOOLEAN', 'bool', NULL, NULL, 4, true, 'Politically Exposed Person status');

-- Update risk_rating and kyc_status with domain references
UPDATE business_attributes SET domain_id = (SELECT id FROM data_domains WHERE domain_name = 'RiskLevel')
WHERE attribute_name = 'risk_rating';

UPDATE business_attributes SET domain_id = (SELECT id FROM data_domains WHERE domain_name = 'KycStatus')
WHERE attribute_name = 'kyc_status';