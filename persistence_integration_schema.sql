-- Persistence Integration Schema
-- Links attribute definitions to actual value storage systems

-- Create persistence systems registry
CREATE TABLE IF NOT EXISTS persistence_systems (
    id SERIAL PRIMARY KEY,
    system_name VARCHAR(100) UNIQUE NOT NULL,
    system_type VARCHAR(50) NOT NULL, -- 'database', 'api', 'cache', 'file', 'blockchain'
    connection_config JSONB NOT NULL,
    capabilities JSONB DEFAULT '{}', -- read, write, query, aggregate, real_time
    performance_profile JSONB DEFAULT '{}', -- latency, throughput, consistency
    security_config JSONB DEFAULT '{}',
    status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create entity registry for structured data storage
CREATE TABLE IF NOT EXISTS persistence_entities (
    id SERIAL PRIMARY KEY,
    entity_name VARCHAR(100) NOT NULL,
    system_id INTEGER REFERENCES persistence_systems(id),
    entity_type VARCHAR(50) NOT NULL, -- 'table', 'collection', 'endpoint', 'schema'
    entity_config JSONB NOT NULL, -- table name, api endpoint, collection name, etc.
    schema_definition JSONB DEFAULT '{}',
    access_patterns JSONB DEFAULT '{}', -- common query patterns, indexes
    data_retention JSONB DEFAULT '{}',
    versioning_config JSONB DEFAULT '{}',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(system_id, entity_name)
);

-- Enhanced attribute persistence mapping
CREATE TABLE IF NOT EXISTS attribute_persistence_mappings (
    id SERIAL PRIMARY KEY,
    attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    persistence_entity_id INTEGER REFERENCES persistence_entities(id),
    field_mapping JSONB NOT NULL, -- maps attribute to actual field/column
    transformation_rules JSONB DEFAULT '{}', -- data transformation rules
    validation_rules JSONB DEFAULT '{}', -- persistence-specific validation
    access_permissions JSONB DEFAULT '{}',
    caching_config JSONB DEFAULT '{}',
    sync_strategy VARCHAR(50) DEFAULT 'immediate', -- 'immediate', 'batch', 'async', 'event_driven'
    conflict_resolution VARCHAR(50) DEFAULT 'last_write_wins',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(attribute_id, persistence_entity_id)
);

-- Create value change audit trail
CREATE TABLE IF NOT EXISTS attribute_value_audit (
    id SERIAL PRIMARY KEY,
    attribute_id INTEGER REFERENCES attribute_objects(id),
    entity_instance_id VARCHAR(255), -- ID of the actual record being modified
    old_value JSONB,
    new_value JSONB,
    change_type VARCHAR(50), -- 'create', 'update', 'delete', 'bulk_update'
    changed_by VARCHAR(100),
    change_reason TEXT,
    change_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    persistence_entity_id INTEGER REFERENCES persistence_entities(id),
    transaction_id VARCHAR(100), -- for grouping related changes
    metadata JSONB DEFAULT '{}'
);

-- KYC and Onboarding specific domain entities
CREATE TABLE IF NOT EXISTS kyc_onboarding_domains (
    id SERIAL PRIMARY KEY,
    domain_name VARCHAR(100) UNIQUE NOT NULL, -- 'customer_onboarding', 'kyc_verification', 'risk_assessment'
    description TEXT,
    regulatory_framework VARCHAR(100), -- 'KYC', 'AML', 'GDPR', 'PCI_DSS'
    compliance_requirements JSONB DEFAULT '{}',
    data_classification VARCHAR(50), -- 'public', 'internal', 'confidential', 'restricted'
    retention_policy JSONB DEFAULT '{}',
    audit_requirements JSONB DEFAULT '{}',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Link attributes to domain contexts
CREATE TABLE IF NOT EXISTS attribute_domain_mappings (
    id SERIAL PRIMARY KEY,
    attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    domain_id INTEGER REFERENCES kyc_onboarding_domains(id) ON DELETE CASCADE,
    context_role VARCHAR(100), -- 'primary_identifier', 'verification_data', 'risk_factor', 'compliance_check'
    importance_weight DECIMAL(3,2) DEFAULT 1.0,
    compliance_criticality VARCHAR(20) DEFAULT 'medium', -- 'low', 'medium', 'high', 'critical'
    data_sensitivity VARCHAR(20) DEFAULT 'medium',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(attribute_id, domain_id)
);

-- Real-time data flow and synchronization
CREATE TABLE IF NOT EXISTS data_flow_configurations (
    id SERIAL PRIMARY KEY,
    flow_name VARCHAR(100) UNIQUE NOT NULL,
    source_mappings INTEGER[] REFERENCES attribute_persistence_mappings(id),
    target_mappings INTEGER[] REFERENCES attribute_persistence_mappings(id),
    flow_type VARCHAR(50), -- 'real_time', 'batch', 'event_driven', 'scheduled'
    transformation_pipeline JSONB DEFAULT '[]',
    error_handling JSONB DEFAULT '{}',
    monitoring_config JSONB DEFAULT '{}',
    schedule_config JSONB DEFAULT '{}', -- for scheduled flows
    status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Update the original attribute_objects table with persistence links
ALTER TABLE attribute_objects
ADD COLUMN IF NOT EXISTS primary_persistence_entity_id INTEGER REFERENCES persistence_entities(id),
ADD COLUMN IF NOT EXISTS backup_persistence_entities INTEGER[], -- array of entity IDs for redundancy
ADD COLUMN IF NOT EXISTS value_lifecycle JSONB DEFAULT '{}', -- creation, modification, archival, deletion rules
ADD COLUMN IF NOT EXISTS data_governance JSONB DEFAULT '{}', -- ownership, stewardship, classification
ADD COLUMN IF NOT EXISTS compliance_metadata JSONB DEFAULT '{}'; -- regulatory requirements, audit trails

-- Create comprehensive view for attribute value resolution
CREATE OR REPLACE VIEW attribute_value_resolution_view AS
SELECT
    ao.id as attribute_id,
    ao.attribute_name,
    ao.data_type,
    ao.description,

    -- Persistence information
    ps.system_name as primary_system,
    ps.system_type,
    pe.entity_name as primary_entity,
    pe.entity_config,
    apm.field_mapping,
    apm.transformation_rules,
    apm.sync_strategy,

    -- Domain context
    kod.domain_name,
    kod.regulatory_framework,
    adm.context_role,
    adm.compliance_criticality,

    -- AI and UI metadata
    ao.semantic_tags,
    ao.ai_context,
    ao.ui_component_type,
    ao.ui_layout_config,

    -- Comprehensive metadata
    jsonb_build_object(
        'persistence', jsonb_build_object(
            'primary_system', ps.system_name,
            'entity', pe.entity_name,
            'field_mapping', apm.field_mapping,
            'sync_strategy', apm.sync_strategy
        ),
        'domain', jsonb_build_object(
            'name', kod.domain_name,
            'compliance', adm.compliance_criticality,
            'role', adm.context_role
        ),
        'ui', jsonb_build_object(
            'component_type', ao.ui_component_type,
            'layout_config', ao.ui_layout_config,
            'styling', ao.ui_styling
        ),
        'ai', jsonb_build_object(
            'context', ao.ai_context,
            'semantic_tags', ao.semantic_tags,
            'search_keywords', ao.search_keywords
        )
    ) as full_metadata

FROM attribute_objects ao
LEFT JOIN persistence_entities pe ON pe.id = ao.primary_persistence_entity_id
LEFT JOIN persistence_systems ps ON ps.id = pe.system_id
LEFT JOIN attribute_persistence_mappings apm ON apm.attribute_id = ao.id AND apm.persistence_entity_id = pe.id
LEFT JOIN attribute_domain_mappings adm ON adm.attribute_id = ao.id
LEFT JOIN kyc_onboarding_domains kod ON kod.id = adm.domain_id;

-- Insert sample persistence systems for KYC/Onboarding platform
INSERT INTO persistence_systems (system_name, system_type, connection_config, capabilities, performance_profile) VALUES
(
    'customer_database',
    'database',
    '{"host": "localhost", "port": 5432, "database": "customer_data", "schema": "public"}',
    '{"read": true, "write": true, "query": true, "aggregate": true, "real_time": false}',
    '{"avg_latency_ms": 50, "throughput_qps": 1000, "consistency": "strong"}'
),
(
    'kyc_document_store',
    'api',
    '{"base_url": "https://api.kyc-provider.com", "version": "v2", "auth_type": "oauth2"}',
    '{"read": true, "write": true, "query": false, "aggregate": false, "real_time": true}',
    '{"avg_latency_ms": 200, "throughput_qps": 100, "consistency": "eventual"}'
),
(
    'redis_cache',
    'cache',
    '{"host": "localhost", "port": 6379, "database": 0}',
    '{"read": true, "write": true, "query": false, "aggregate": false, "real_time": true}',
    '{"avg_latency_ms": 1, "throughput_qps": 10000, "consistency": "eventual"}'
),
(
    'blockchain_audit',
    'blockchain',
    '{"network": "private", "contract_address": "0x...", "chain_id": 1337}',
    '{"read": true, "write": true, "query": false, "aggregate": false, "real_time": false}',
    '{"avg_latency_ms": 5000, "throughput_qps": 10, "consistency": "immutable"}'
);

-- Insert sample entities
INSERT INTO persistence_entities (entity_name, system_id, entity_type, entity_config, schema_definition) VALUES
(
    'customers',
    1,
    'table',
    '{"table_name": "customers", "primary_key": "customer_id"}',
    '{"customer_id": "uuid", "created_at": "timestamp", "updated_at": "timestamp"}'
),
(
    'kyc_documents',
    2,
    'endpoint',
    '{"endpoint": "/documents", "method": "POST", "id_field": "document_id"}',
    '{"document_id": "string", "customer_id": "string", "document_type": "string", "status": "string"}'
),
(
    'session_data',
    3,
    'keyspace',
    '{"prefix": "onboarding:", "ttl": 3600}',
    '{"session_id": "string", "data": "json", "expires_at": "timestamp"}'
),
(
    'compliance_audit',
    4,
    'contract',
    '{"contract_method": "recordCompliance", "gas_limit": 100000}',
    '{"event_id": "bytes32", "customer_id": "address", "compliance_data": "bytes"}'
);

-- Insert KYC/Onboarding domains
INSERT INTO kyc_onboarding_domains (domain_name, description, regulatory_framework, compliance_requirements) VALUES
(
    'customer_onboarding',
    'Customer registration and initial data collection',
    'KYC',
    '{"required_documents": ["id", "proof_of_address"], "verification_level": "enhanced", "risk_assessment": true}'
),
(
    'kyc_verification',
    'Know Your Customer identity verification process',
    'AML',
    '{"identity_verification": true, "document_verification": true, "biometric_check": false, "ongoing_monitoring": true}'
),
(
    'risk_assessment',
    'Customer risk profiling and scoring',
    'AML',
    '{"risk_scoring": true, "politically_exposed_persons": true, "sanctions_screening": true, "transaction_monitoring": true}'
),
(
    'data_privacy',
    'GDPR and privacy compliance for customer data',
    'GDPR',
    '{"consent_management": true, "data_minimization": true, "right_to_erasure": true, "data_portability": true}'
);

-- Create function to get attribute with full persistence context
CREATE OR REPLACE FUNCTION get_attribute_with_persistence(
    attr_id INTEGER
) RETURNS JSONB AS $$
DECLARE
    result JSONB;
BEGIN
    SELECT to_jsonb(avrv.*) INTO result
    FROM attribute_value_resolution_view avrv
    WHERE avrv.attribute_id = attr_id;

    RETURN COALESCE(result, '{}'::jsonb);
END;
$$ LANGUAGE plpgsql;

-- Create function to resolve where to persist a value
CREATE OR REPLACE FUNCTION resolve_persistence_target(
    attr_id INTEGER,
    operation_type VARCHAR DEFAULT 'write'
) RETURNS JSONB AS $$
DECLARE
    result JSONB;
    primary_target JSONB;
    cache_targets JSONB[];
    audit_targets JSONB[];
BEGIN
    -- Get primary persistence target
    SELECT jsonb_build_object(
        'system', ps.system_name,
        'entity', pe.entity_name,
        'config', pe.entity_config,
        'field_mapping', apm.field_mapping,
        'transformation_rules', apm.transformation_rules
    ) INTO primary_target
    FROM attribute_objects ao
    JOIN persistence_entities pe ON pe.id = ao.primary_persistence_entity_id
    JOIN persistence_systems ps ON ps.id = pe.system_id
    JOIN attribute_persistence_mappings apm ON apm.attribute_id = ao.id AND apm.persistence_entity_id = pe.id
    WHERE ao.id = attr_id;

    -- Get cache targets (for read operations)
    IF operation_type = 'read' THEN
        SELECT array_agg(
            jsonb_build_object(
                'system', ps.system_name,
                'entity', pe.entity_name,
                'config', pe.entity_config,
                'priority', CASE WHEN ps.system_type = 'cache' THEN 1 ELSE 2 END
            )
        ) INTO cache_targets
        FROM attribute_persistence_mappings apm
        JOIN persistence_entities pe ON pe.id = apm.persistence_entity_id
        JOIN persistence_systems ps ON ps.id = pe.system_id
        WHERE apm.attribute_id = attr_id
        AND ps.capabilities->>'read' = 'true';
    END IF;

    -- Get audit targets (for write operations)
    IF operation_type IN ('write', 'update', 'delete') THEN
        SELECT array_agg(
            jsonb_build_object(
                'system', ps.system_name,
                'entity', pe.entity_name,
                'config', pe.entity_config,
                'async', CASE WHEN ps.system_type = 'blockchain' THEN true ELSE false END
            )
        ) INTO audit_targets
        FROM attribute_persistence_mappings apm
        JOIN persistence_entities pe ON pe.id = apm.persistence_entity_id
        JOIN persistence_systems ps ON ps.id = pe.system_id
        WHERE apm.attribute_id = attr_id
        AND ps.system_type IN ('blockchain', 'audit');
    END IF;

    result := jsonb_build_object(
        'primary', primary_target,
        'cache_targets', COALESCE(cache_targets, '[]'::JSONB[]),
        'audit_targets', COALESCE(audit_targets, '[]'::JSONB[]),
        'operation_type', operation_type
    );

    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Create comprehensive indexes
CREATE INDEX IF NOT EXISTS idx_attribute_persistence_mappings_attribute ON attribute_persistence_mappings(attribute_id);
CREATE INDEX IF NOT EXISTS idx_attribute_persistence_mappings_entity ON attribute_persistence_mappings(persistence_entity_id);
CREATE INDEX IF NOT EXISTS idx_attribute_domain_mappings_attribute ON attribute_domain_mappings(attribute_id);
CREATE INDEX IF NOT EXISTS idx_attribute_domain_mappings_domain ON attribute_domain_mappings(domain_id);
CREATE INDEX IF NOT EXISTS idx_attribute_value_audit_attribute ON attribute_value_audit(attribute_id);
CREATE INDEX IF NOT EXISTS idx_attribute_value_audit_entity_instance ON attribute_value_audit(entity_instance_id);
CREATE INDEX IF NOT EXISTS idx_attribute_value_audit_timestamp ON attribute_value_audit(change_timestamp);

-- Comments for documentation
COMMENT ON TABLE persistence_systems IS 'Registry of all systems where attribute values can be stored';
COMMENT ON TABLE persistence_entities IS 'Specific entities (tables, collections, endpoints) within persistence systems';
COMMENT ON TABLE attribute_persistence_mappings IS 'Maps attributes to their actual storage locations with transformation rules';
COMMENT ON TABLE kyc_onboarding_domains IS 'Domain-specific contexts for KYC and onboarding attributes';
COMMENT ON TABLE attribute_domain_mappings IS 'Links attributes to their domain contexts with compliance metadata';
COMMENT ON TABLE data_flow_configurations IS 'Defines how data flows between different persistence systems';
COMMENT ON TABLE attribute_value_audit IS 'Audit trail for all attribute value changes across systems';