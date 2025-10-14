-- Commercial Taxonomy Management Schema
-- Implements Products→Services→Resources hierarchy for commercial contracts and sales

-- Enhanced Products table for commercial sold products in contracts
DROP TABLE IF EXISTS product_service_mappings CASCADE;
DROP TABLE IF EXISTS service_resource_mappings CASCADE;
DROP VIEW IF EXISTS commercial_taxonomy_view CASCADE;

-- Update products table to reflect commercial contract context
ALTER TABLE products ADD COLUMN IF NOT EXISTS contract_type VARCHAR(100); -- 'sale', 'license', 'subscription', 'service_agreement'
ALTER TABLE products ADD COLUMN IF NOT EXISTS commercial_status VARCHAR(50) DEFAULT 'active'; -- 'active', 'discontinued', 'development', 'sunset'
ALTER TABLE products ADD COLUMN IF NOT EXISTS pricing_model VARCHAR(50); -- 'fixed', 'tiered', 'usage_based', 'subscription'
ALTER TABLE products ADD COLUMN IF NOT EXISTS target_market VARCHAR(100); -- 'enterprise', 'mid_market', 'small_business', 'retail'
ALTER TABLE products ADD COLUMN IF NOT EXISTS sales_territory VARCHAR(100); -- 'global', 'regional', 'domestic'
ALTER TABLE products ADD COLUMN IF NOT EXISTS compliance_requirements TEXT[]; -- Array of compliance standards
ALTER TABLE products ADD COLUMN IF NOT EXISTS contract_terms JSONB; -- Standard contract terms and conditions
ALTER TABLE products ADD COLUMN IF NOT EXISTS minimum_contract_value DECIMAL(15,2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS maximum_contract_value DECIMAL(15,2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS standard_contract_duration INTEGER; -- months
ALTER TABLE products ADD COLUMN IF NOT EXISTS renewable BOOLEAN DEFAULT true;
ALTER TABLE products ADD COLUMN IF NOT EXISTS early_termination_allowed BOOLEAN DEFAULT false;

COMMENT ON COLUMN products.contract_type IS 'Type of commercial contract for this product';
COMMENT ON COLUMN products.commercial_status IS 'Current commercial availability status';
COMMENT ON COLUMN products.pricing_model IS 'How this product is priced and sold';
COMMENT ON COLUMN products.contract_terms IS 'Standard contract terms, warranties, and conditions';

-- Enhanced Services table for generic public financial services
ALTER TABLE services ADD COLUMN IF NOT EXISTS service_type VARCHAR(100); -- 'custody', 'safekeeping', 'reconciliation', 'fund_accounting', 'middle_office', 'trade_order_management'
ALTER TABLE services ADD COLUMN IF NOT EXISTS delivery_model VARCHAR(50); -- 'self_service', 'managed', 'hybrid', 'outsourced'
ALTER TABLE services ADD COLUMN IF NOT EXISTS sla_requirements JSONB; -- Service level agreements
ALTER TABLE services ADD COLUMN IF NOT EXISTS billable BOOLEAN DEFAULT true;
ALTER TABLE services ADD COLUMN IF NOT EXISTS recurring_service BOOLEAN DEFAULT false;
ALTER TABLE services ADD COLUMN IF NOT EXISTS service_dependencies TEXT[]; -- Other services this depends on
ALTER TABLE services ADD COLUMN IF NOT EXISTS skill_requirements TEXT[]; -- Required skills/certifications
ALTER TABLE services ADD COLUMN IF NOT EXISTS automation_level VARCHAR(50); -- 'manual', 'semi_automated', 'fully_automated'
ALTER TABLE services ADD COLUMN IF NOT EXISTS customer_facing BOOLEAN DEFAULT true;

COMMENT ON COLUMN services.service_type IS 'Type of generic public financial service (custody, safekeeping, reconciliation, fund_accounting, middle_office, trade_order_management)';
COMMENT ON COLUMN services.delivery_model IS 'How this financial service is delivered to customers';
COMMENT ON COLUMN services.sla_requirements IS 'Service level agreement requirements and metrics';

-- Enhanced Resources table for applications and systems that implement financial services
ALTER TABLE resource_objects ADD COLUMN IF NOT EXISTS resource_type VARCHAR(100); -- 'application_accounts', 'routing_tables', 'reconciliation_app', 'fa_app', 'ibor_app', 'trading_system', 'settlement_system'
ALTER TABLE resource_objects ADD COLUMN IF NOT EXISTS criticality_level VARCHAR(50) DEFAULT 'medium'; -- 'low', 'medium', 'high', 'critical'
ALTER TABLE resource_objects ADD COLUMN IF NOT EXISTS operational_status VARCHAR(50) DEFAULT 'active'; -- 'active', 'maintenance', 'deprecated', 'retired'
ALTER TABLE resource_objects ADD COLUMN IF NOT EXISTS access_restrictions JSONB; -- Access control and permissions
ALTER TABLE resource_objects ADD COLUMN IF NOT EXISTS compliance_classification VARCHAR(100); -- Data classification for compliance
ALTER TABLE resource_objects ADD COLUMN IF NOT EXISTS data_retention_policy JSONB; -- Retention rules and policies
ALTER TABLE resource_objects ADD COLUMN IF NOT EXISTS backup_requirements JSONB; -- Backup and recovery requirements
ALTER TABLE resource_objects ADD COLUMN IF NOT EXISTS monitoring_enabled BOOLEAN DEFAULT true;
ALTER TABLE resource_objects ADD COLUMN IF NOT EXISTS audit_required BOOLEAN DEFAULT false;

-- Create Product-Service mapping table
CREATE TABLE IF NOT EXISTS product_service_mappings (
    id SERIAL PRIMARY KEY,
    product_id INTEGER REFERENCES products(id) ON DELETE CASCADE,
    service_id INTEGER REFERENCES services(id) ON DELETE CASCADE,
    mapping_type VARCHAR(50) NOT NULL, -- 'core', 'optional', 'add_on', 'premium'
    inclusion_criteria JSONB, -- When this service is included
    pricing_impact DECIMAL(10,2), -- How this affects product pricing
    delivery_sequence INTEGER, -- Order of service delivery
    is_mandatory BOOLEAN DEFAULT false,
    customer_configurable BOOLEAN DEFAULT true,
    effective_date DATE DEFAULT CURRENT_DATE,
    end_date DATE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(product_id, service_id, mapping_type)
);

-- Create Service-Resource mapping table
CREATE TABLE IF NOT EXISTS service_resource_mappings (
    id SERIAL PRIMARY KEY,
    service_id INTEGER REFERENCES services(id) ON DELETE CASCADE,
    resource_id INTEGER REFERENCES resource_objects(id) ON DELETE CASCADE,
    usage_type VARCHAR(50) NOT NULL, -- 'required', 'optional', 'conditional', 'fallback'
    resource_role VARCHAR(100), -- 'data_source', 'processor', 'validator', 'output', 'audit_trail'
    configuration_parameters JSONB, -- How the resource is configured for this service
    performance_requirements JSONB, -- SLA requirements for this resource in this service
    usage_limits JSONB, -- Rate limits, quotas, etc.
    cost_allocation_percentage DECIMAL(5,2), -- How much of service cost is attributed to this resource
    dependency_level INTEGER DEFAULT 1, -- 1=critical, 2=important, 3=nice_to_have
    failover_resource_id INTEGER REFERENCES resource_objects(id),
    monitoring_thresholds JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(service_id, resource_id, usage_type)
);

-- Create commercial contracts tracking table
CREATE TABLE IF NOT EXISTS commercial_contracts (
    id SERIAL PRIMARY KEY,
    contract_id VARCHAR(100) UNIQUE NOT NULL,
    customer_entity_id VARCHAR(100), -- Links to legal_entities
    product_id INTEGER REFERENCES products(id),
    contract_type VARCHAR(50),
    contract_status VARCHAR(50) DEFAULT 'active', -- 'draft', 'active', 'expired', 'terminated', 'renewed'

    -- Commercial Terms
    contract_value DECIMAL(15,2),
    currency CHAR(3),
    payment_terms VARCHAR(100),
    billing_frequency VARCHAR(50), -- 'monthly', 'quarterly', 'annually', 'one_time'

    -- Dates
    contract_start_date DATE,
    contract_end_date DATE,
    renewal_date DATE,
    termination_notice_period INTEGER, -- days

    -- Service Configuration
    included_services JSONB, -- Services included in this contract
    service_configurations JSONB, -- Specific configurations per service
    sla_commitments JSONB, -- Service level commitments

    -- Legal
    governing_law VARCHAR(100),
    dispute_resolution VARCHAR(100),
    liability_cap DECIMAL(15,2),

    -- Metadata
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100),
    updated_by VARCHAR(100)
);

-- Create comprehensive taxonomy view
CREATE OR REPLACE VIEW commercial_taxonomy_view AS
SELECT
    -- Product Level
    p.id as product_id,
    p.product_name,
    p.description as product_description,
    p.contract_type,
    p.commercial_status,
    p.pricing_model,
    p.target_market,

    -- Service Level
    s.id as service_id,
    s.service_name,
    s.description as service_description,
    s.service_type,
    s.delivery_model,
    s.billable as service_billable,

    -- Resource Level
    r.id as resource_id,
    r.resource_name,
    r.description as resource_description,
    r.resource_type,
    r.criticality_level,
    r.operational_status,

    -- Attribute Level
    COUNT(DISTINCT a.id) as attribute_count,
    array_agg(DISTINCT a.attribute_name ORDER BY a.attribute_name) FILTER (WHERE a.id IS NOT NULL) as attributes,

    -- Mapping Information
    psm.mapping_type as service_inclusion_type,
    psm.is_mandatory as service_mandatory,
    srm.usage_type as resource_usage_type,
    srm.dependency_level as resource_dependency,

    -- Commercial Metrics
    COUNT(DISTINCT cc.id) as active_contracts,
    AVG(cc.contract_value) as avg_contract_value,

    -- Compliance and Risk
    p.compliance_requirements,
    r.compliance_classification,
    r.audit_required,

    -- Operational Metrics
    COUNT(DISTINCT CASE WHEN r.operational_status = 'active' THEN r.id END) as active_resources,
    COUNT(DISTINCT CASE WHEN s.service_type = 'core' THEN s.id END) as core_services,

    -- Metadata
    p.created_at as product_created_at,
    p.updated_at as product_updated_at

FROM products p
LEFT JOIN product_service_mappings psm ON psm.product_id = p.id
LEFT JOIN services s ON s.id = psm.service_id
LEFT JOIN service_resource_mappings srm ON srm.service_id = s.id
LEFT JOIN resource_objects r ON r.id = srm.resource_id
LEFT JOIN attribute_objects a ON a.resource_id = r.id
LEFT JOIN commercial_contracts cc ON cc.product_id = p.id AND cc.contract_status = 'active'
GROUP BY
    p.id, p.product_name, p.description, p.contract_type, p.commercial_status,
    p.pricing_model, p.target_market, p.compliance_requirements, p.created_at, p.updated_at,
    s.id, s.service_name, s.description, s.service_type, s.delivery_model, s.billable,
    r.id, r.resource_name, r.description, r.resource_type, r.criticality_level,
    r.operational_status, r.compliance_classification, r.audit_required,
    psm.mapping_type, psm.is_mandatory, srm.usage_type, srm.dependency_level;

-- Function to get complete taxonomy hierarchy for a product
CREATE OR REPLACE FUNCTION get_product_taxonomy_hierarchy(input_product_id INTEGER)
RETURNS TABLE (
    level INTEGER,
    item_type VARCHAR,
    item_id INTEGER,
    item_name VARCHAR,
    item_description TEXT,
    parent_id INTEGER,
    configuration JSONB,
    metadata JSONB
) AS $$
BEGIN
    RETURN QUERY
    WITH RECURSIVE taxonomy_tree AS (
        -- Level 1: Product
        SELECT
            1 as level,
            'product'::VARCHAR as item_type,
            p.id as item_id,
            p.product_name as item_name,
            p.description as item_description,
            NULL::INTEGER as parent_id,
            jsonb_build_object(
                'contract_type', p.contract_type,
                'commercial_status', p.commercial_status,
                'pricing_model', p.pricing_model
            ) as configuration,
            jsonb_build_object(
                'target_market', p.target_market,
                'compliance_requirements', p.compliance_requirements
            ) as metadata
        FROM products p
        WHERE p.id = input_product_id

        UNION ALL

        -- Level 2: Services
        SELECT
            2 as level,
            'service'::VARCHAR as item_type,
            s.id as item_id,
            s.service_name as item_name,
            s.description as item_description,
            tt.item_id as parent_id,
            jsonb_build_object(
                'service_type', s.service_type,
                'delivery_model', s.delivery_model,
                'mapping_type', psm.mapping_type,
                'is_mandatory', psm.is_mandatory
            ) as configuration,
            jsonb_build_object(
                'billable', s.billable,
                'sla_requirements', s.sla_requirements
            ) as metadata
        FROM taxonomy_tree tt
        JOIN product_service_mappings psm ON psm.product_id = tt.item_id
        JOIN services s ON s.id = psm.service_id
        WHERE tt.level = 1

        UNION ALL

        -- Level 3: Resources
        SELECT
            3 as level,
            'resource'::VARCHAR as item_type,
            r.id as item_id,
            r.resource_name as item_name,
            r.description as item_description,
            tt.item_id as parent_id,
            jsonb_build_object(
                'resource_type', r.resource_type,
                'usage_type', srm.usage_type,
                'dependency_level', srm.dependency_level
            ) as configuration,
            jsonb_build_object(
                'criticality_level', r.criticality_level,
                'operational_status', r.operational_status,
                'compliance_classification', r.compliance_classification
            ) as metadata
        FROM taxonomy_tree tt
        JOIN service_resource_mappings srm ON srm.service_id = tt.item_id
        JOIN resource_objects r ON r.id = srm.resource_id
        WHERE tt.level = 2
    )
    SELECT * FROM taxonomy_tree
    ORDER BY level, item_name;
END;
$$ LANGUAGE plpgsql;

-- Function to validate taxonomy integrity
CREATE OR REPLACE FUNCTION validate_commercial_taxonomy()
RETURNS TABLE (
    validation_rule VARCHAR,
    status VARCHAR,
    message TEXT,
    affected_items JSONB
) AS $$
BEGIN
    -- Rule 1: Every product should have at least one core service
    RETURN QUERY
    SELECT
        'product_has_core_service'::VARCHAR,
        CASE WHEN core_service_count > 0 THEN 'PASS' ELSE 'FAIL' END,
        CASE WHEN core_service_count > 0
            THEN format('Product has %s core services', core_service_count)
            ELSE 'Product must have at least one core service'
        END,
        jsonb_build_object('product_id', p.id, 'core_services', core_service_count)
    FROM (
        SELECT
            p.id,
            COUNT(CASE WHEN s.service_type = 'core' THEN 1 END) as core_service_count
        FROM products p
        LEFT JOIN product_service_mappings psm ON psm.product_id = p.id
        LEFT JOIN services s ON s.id = psm.service_id
        GROUP BY p.id
    ) p;

    -- Rule 2: Every service should have at least one required resource
    RETURN QUERY
    SELECT
        'service_has_required_resource'::VARCHAR,
        CASE WHEN required_resource_count > 0 THEN 'PASS' ELSE 'WARN' END,
        CASE WHEN required_resource_count > 0
            THEN format('Service has %s required resources', required_resource_count)
            ELSE 'Service should have at least one required resource'
        END,
        jsonb_build_object('service_id', s.id, 'required_resources', required_resource_count)
    FROM (
        SELECT
            s.id,
            COUNT(CASE WHEN srm.usage_type = 'required' THEN 1 END) as required_resource_count
        FROM services s
        LEFT JOIN service_resource_mappings srm ON srm.service_id = s.id
        GROUP BY s.id
    ) s;

    -- Rule 3: Critical resources should have failover configured
    RETURN QUERY
    SELECT
        'critical_resource_failover'::VARCHAR,
        CASE WHEN failover_resource_id IS NOT NULL THEN 'PASS' ELSE 'WARN' END,
        CASE WHEN failover_resource_id IS NOT NULL
            THEN 'Critical resource has failover configured'
            ELSE format('Critical resource "%s" should have failover configured', r.resource_name)
        END,
        jsonb_build_object('resource_id', r.id, 'resource_name', r.resource_name)
    FROM resource_objects r
    JOIN service_resource_mappings srm ON srm.resource_id = r.id
    WHERE r.criticality_level = 'critical' AND srm.dependency_level = 1;
END;
$$ LANGUAGE plpgsql;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_product_service_mappings_product ON product_service_mappings(product_id);
CREATE INDEX IF NOT EXISTS idx_product_service_mappings_service ON product_service_mappings(service_id);
CREATE INDEX IF NOT EXISTS idx_product_service_mappings_type ON product_service_mappings(mapping_type);

CREATE INDEX IF NOT EXISTS idx_service_resource_mappings_service ON service_resource_mappings(service_id);
CREATE INDEX IF NOT EXISTS idx_service_resource_mappings_resource ON service_resource_mappings(resource_id);
CREATE INDEX IF NOT EXISTS idx_service_resource_mappings_usage ON service_resource_mappings(usage_type);
CREATE INDEX IF NOT EXISTS idx_service_resource_mappings_dependency ON service_resource_mappings(dependency_level);

CREATE INDEX IF NOT EXISTS idx_commercial_contracts_product ON commercial_contracts(product_id);
CREATE INDEX IF NOT EXISTS idx_commercial_contracts_status ON commercial_contracts(contract_status);
CREATE INDEX IF NOT EXISTS idx_commercial_contracts_customer ON commercial_contracts(customer_entity_id);

CREATE INDEX IF NOT EXISTS idx_products_commercial_status ON products(commercial_status);
CREATE INDEX IF NOT EXISTS idx_products_contract_type ON products(contract_type);
CREATE INDEX IF NOT EXISTS idx_services_service_type ON services(service_type);
CREATE INDEX IF NOT EXISTS idx_resources_criticality ON resource_objects(criticality_level);

-- Comments for documentation
COMMENT ON TABLE product_service_mappings IS 'Maps commercial products to their constituent services with inclusion criteria';
COMMENT ON TABLE service_resource_mappings IS 'Maps services to required resources with usage patterns and dependencies';
COMMENT ON TABLE commercial_contracts IS 'Tracks commercial contracts for sold products with terms and service configurations';
COMMENT ON VIEW commercial_taxonomy_view IS 'Comprehensive view of Products→Services→Resources hierarchy with commercial context';
COMMENT ON FUNCTION get_product_taxonomy_hierarchy IS 'Retrieves complete hierarchical breakdown of a commercial product';
COMMENT ON FUNCTION validate_commercial_taxonomy IS 'Validates integrity and completeness of commercial taxonomy structure';