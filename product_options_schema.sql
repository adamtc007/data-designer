-- Product Options Schema
-- Adds Product Options layer: Products → Product Options → Services → Resources
-- Example: Custody product → Market Settlement Options (markets X,Y,Z,W,Y,T) → Custody Services → Custody Applications

-- Create product options table
CREATE TABLE IF NOT EXISTS product_options (
    id SERIAL PRIMARY KEY,
    option_id VARCHAR(100) UNIQUE NOT NULL,
    product_id INTEGER REFERENCES products(id) ON DELETE CASCADE,
    option_name VARCHAR(255) NOT NULL,
    option_category VARCHAR(100) NOT NULL, -- 'market_settlement', 'currency_support', 'reporting_options', 'compliance_packages'
    option_type VARCHAR(50) NOT NULL, -- 'required', 'optional', 'premium', 'add_on'

    -- Option Configuration
    option_value JSONB NOT NULL, -- Specific option configuration (e.g., market codes, currency lists)
    display_name VARCHAR(255),
    description TEXT,

    -- Commercial Terms
    pricing_impact DECIMAL(10,2) DEFAULT 0.00, -- Additional cost for this option
    pricing_model VARCHAR(50), -- 'flat_fee', 'percentage', 'per_transaction', 'tiered'
    minimum_commitment DECIMAL(10,2), -- Minimum commitment for this option

    -- Availability
    available_markets TEXT[], -- Which markets this option is available in
    regulatory_approval_required BOOLEAN DEFAULT false,
    compliance_requirements TEXT[],

    -- Dependencies
    prerequisite_options INTEGER[], -- Array of option IDs that must be selected first
    mutually_exclusive_options INTEGER[], -- Options that cannot be selected together

    -- Operational
    implementation_complexity VARCHAR(20) DEFAULT 'medium', -- 'low', 'medium', 'high', 'complex'
    lead_time_days INTEGER, -- Days needed to implement this option
    ongoing_support_required BOOLEAN DEFAULT true,

    -- Status
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'deprecated', 'beta', 'coming_soon')),
    effective_date DATE DEFAULT CURRENT_DATE,
    end_date DATE,

    -- Metadata
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100),
    updated_by VARCHAR(100)
);

-- Create product option to service mappings
CREATE TABLE IF NOT EXISTS product_option_service_mappings (
    id SERIAL PRIMARY KEY,
    product_option_id INTEGER REFERENCES product_options(id) ON DELETE CASCADE,
    service_id INTEGER REFERENCES services(id) ON DELETE CASCADE,
    mapping_relationship VARCHAR(50) NOT NULL, -- 'enables', 'requires', 'enhances', 'configures'

    -- Service Configuration for this Option
    service_configuration JSONB, -- How the service is configured when this option is selected
    sla_modifications JSONB, -- SLA changes specific to this option

    -- Resource Requirements
    additional_resources JSONB, -- Extra resources needed for this option
    resource_scaling_factor DECIMAL(4,2) DEFAULT 1.00, -- How this option affects resource usage

    -- Priority and Ordering
    execution_priority INTEGER DEFAULT 10, -- Order in which services are configured
    dependency_level INTEGER DEFAULT 1, -- How critical this service is for the option

    -- Status
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(product_option_id, service_id, mapping_relationship)
);

-- Enhanced commercial taxonomy view with product options
CREATE OR REPLACE VIEW enhanced_commercial_taxonomy_view AS
SELECT
    -- Product Level
    p.id as product_id,
    p.product_id as product_code,
    p.product_name,
    p.line_of_business,
    p.description as product_description,
    p.contract_type,
    p.commercial_status,
    p.pricing_model as product_pricing_model,
    p.target_market,

    -- Product Options Level
    po.id as option_id,
    po.option_id as option_code,
    po.option_name,
    po.option_category,
    po.option_type,
    po.option_value,
    po.pricing_impact as option_pricing_impact,

    -- Service Level
    s.id as service_id,
    s.service_id as service_code,
    s.service_name,
    s.service_category,
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
    posm.mapping_relationship as option_service_relationship,
    srm.usage_type as resource_usage_type,
    srm.dependency_level as resource_dependency,

    -- Commercial Metrics
    COUNT(DISTINCT cc.id) as active_contracts,
    AVG(cc.contract_value) as avg_contract_value,

    -- Option Metrics
    COUNT(DISTINCT po.id) as total_options,
    COUNT(DISTINCT CASE WHEN po.option_type = 'required' THEN po.id END) as required_options,
    COUNT(DISTINCT CASE WHEN po.option_type = 'premium' THEN po.id END) as premium_options,

    -- Compliance and Risk
    p.compliance_requirements,
    po.regulatory_approval_required,
    r.compliance_classification,
    r.audit_required,

    -- Operational Metrics
    COUNT(DISTINCT CASE WHEN r.operational_status = 'active' THEN r.id END) as active_resources,
    COUNT(DISTINCT CASE WHEN s.service_type = 'custody' THEN s.id END) as custody_services,
    COUNT(DISTINCT CASE WHEN s.service_type = 'reconciliation' THEN s.id END) as reconciliation_services,

    -- Metadata
    p.created_at as product_created_at,
    p.updated_at as product_updated_at

FROM products p
LEFT JOIN product_options po ON po.product_id = p.id
LEFT JOIN product_option_service_mappings posm ON posm.product_option_id = po.id
LEFT JOIN services s ON s.id = posm.service_id
LEFT JOIN product_service_mappings psm ON psm.product_id = p.id AND psm.service_id = s.id
LEFT JOIN service_resource_mappings srm ON srm.service_id = s.id
LEFT JOIN resource_objects r ON r.id = srm.resource_id
LEFT JOIN attribute_objects a ON a.resource_id = r.id
LEFT JOIN commercial_contracts cc ON cc.product_id = p.id AND cc.contract_status = 'active'
GROUP BY
    p.id, p.product_id, p.product_name, p.line_of_business, p.description, p.contract_type,
    p.commercial_status, p.pricing_model, p.target_market, p.compliance_requirements,
    p.created_at, p.updated_at,
    po.id, po.option_id, po.option_name, po.option_category, po.option_type,
    po.option_value, po.pricing_impact, po.regulatory_approval_required,
    s.id, s.service_id, s.service_name, s.service_category, s.description, s.service_type,
    s.delivery_model, s.billable,
    r.id, r.resource_name, r.description, r.resource_type, r.criticality_level,
    r.operational_status, r.compliance_classification, r.audit_required,
    psm.mapping_type, psm.is_mandatory, posm.mapping_relationship,
    srm.usage_type, srm.dependency_level;

-- Function to get complete enhanced taxonomy hierarchy including product options
CREATE OR REPLACE FUNCTION get_enhanced_product_taxonomy_hierarchy(input_product_id INTEGER)
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

    -- Level 2: Product Options
    SELECT
        2 as level,
        'product_option'::VARCHAR as item_type,
        po.id as item_id,
        po.option_name as item_name,
        po.description as item_description,
        p.id as parent_id,
        jsonb_build_object(
            'option_category', po.option_category,
            'option_type', po.option_type,
            'option_value', po.option_value,
            'pricing_impact', po.pricing_impact
        ) as configuration,
        jsonb_build_object(
            'available_markets', po.available_markets,
            'regulatory_approval_required', po.regulatory_approval_required,
            'implementation_complexity', po.implementation_complexity,
            'lead_time_days', po.lead_time_days
        ) as metadata
    FROM products p
    JOIN product_options po ON po.product_id = p.id
    WHERE p.id = input_product_id

    UNION ALL

    -- Level 3: Services (via Product Options)
    SELECT
        3 as level,
        'service'::VARCHAR as item_type,
        s.id as item_id,
        s.service_name as item_name,
        s.description as item_description,
        po.id as parent_id,
        jsonb_build_object(
            'service_type', s.service_type,
            'delivery_model', s.delivery_model,
            'mapping_relationship', posm.mapping_relationship,
            'service_configuration', posm.service_configuration
        ) as configuration,
        jsonb_build_object(
            'billable', s.billable,
            'sla_requirements', s.sla_requirements,
            'automation_level', s.automation_level
        ) as metadata
    FROM products p
    JOIN product_options po ON po.product_id = p.id
    JOIN product_option_service_mappings posm ON posm.product_option_id = po.id
    JOIN services s ON s.id = posm.service_id
    WHERE p.id = input_product_id

    UNION ALL

    -- Level 4: Resources (via Services)
    SELECT
        4 as level,
        'resource'::VARCHAR as item_type,
        r.id as item_id,
        r.resource_name as item_name,
        r.description as item_description,
        s.id as parent_id,
        jsonb_build_object(
            'resource_type', r.resource_type,
            'usage_type', srm.usage_type,
            'dependency_level', srm.dependency_level,
            'cost_allocation_percentage', srm.cost_allocation_percentage
        ) as configuration,
        jsonb_build_object(
            'criticality_level', r.criticality_level,
            'operational_status', r.operational_status,
            'compliance_classification', r.compliance_classification,
            'monitoring_enabled', r.monitoring_enabled
        ) as metadata
    FROM products p
    JOIN product_options po ON po.product_id = p.id
    JOIN product_option_service_mappings posm ON posm.product_option_id = po.id
    JOIN services s ON s.id = posm.service_id
    JOIN service_resource_mappings srm ON srm.service_id = s.id
    JOIN resource_objects r ON r.id = srm.resource_id
    WHERE p.id = input_product_id

    ORDER BY level, item_name;
END;
$$ LANGUAGE plpgsql;

-- Insert sample product options for custody product
INSERT INTO product_options (
    option_id, product_id, option_name, option_category, option_type,
    option_value, display_name, description, pricing_impact, pricing_model,
    available_markets, regulatory_approval_required, lead_time_days
) VALUES

-- Market Settlement Options for Institutional Custody Plus
('CUSTODY-MARKET-US', (SELECT id FROM products WHERE product_id = 'INST-CUSTODY-PLUS'),
 'US Market Settlement', 'market_settlement', 'optional',
 '{"markets": ["NYSE", "NASDAQ", "AMEX"], "settlement_cycle": "T+2", "currencies": ["USD"]}',
 'United States Markets', 'Settlement and custody services for US equity and bond markets',
 25000.00, 'flat_fee', ARRAY['US'], false, 30),

('CUSTODY-MARKET-EU', (SELECT id FROM products WHERE product_id = 'INST-CUSTODY-PLUS'),
 'European Market Settlement', 'market_settlement', 'optional',
 '{"markets": ["XETRA", "LSE", "EURONEXT"], "settlement_cycle": "T+2", "currencies": ["EUR", "GBP"]}',
 'European Markets', 'Settlement and custody services for major European markets',
 35000.00, 'flat_fee', ARRAY['EU', 'GB'], true, 45),

('CUSTODY-MARKET-APAC', (SELECT id FROM products WHERE product_id = 'INST-CUSTODY-PLUS'),
 'Asia Pacific Settlement', 'market_settlement', 'optional',
 '{"markets": ["TSE", "HKEx", "SGX", "ASX"], "settlement_cycle": "T+2", "currencies": ["JPY", "HKD", "SGD", "AUD"]}',
 'Asia Pacific Markets', 'Settlement and custody services for major APAC markets',
 40000.00, 'flat_fee', ARRAY['JP', 'HK', 'SG', 'AU'], true, 60),

-- Currency Support Options
('CUSTODY-MULTI-CCY', (SELECT id FROM products WHERE product_id = 'INST-CUSTODY-PLUS'),
 'Multi-Currency Support', 'currency_support', 'premium',
 '{"currencies": ["USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD"], "fx_services": true, "hedging": true}',
 'Multi-Currency Package', 'Comprehensive multi-currency custody with FX and hedging services',
 50000.00, 'percentage', ARRAY['global'], false, 21),

-- Reporting Options
('CUSTODY-ENHANCED-REPORTING', (SELECT id FROM products WHERE product_id = 'INST-CUSTODY-PLUS'),
 'Enhanced Reporting Suite', 'reporting_options', 'add_on',
 '{"frequency": ["daily", "weekly", "monthly"], "formats": ["PDF", "Excel", "API"], "customization": true}',
 'Enhanced Reporting', 'Advanced reporting and analytics with customizable dashboards',
 15000.00, 'flat_fee', ARRAY['global'], false, 14)

ON CONFLICT (option_id) DO UPDATE SET
    option_name = EXCLUDED.option_name,
    description = EXCLUDED.description,
    pricing_impact = EXCLUDED.pricing_impact,
    updated_at = CURRENT_TIMESTAMP;

-- Map product options to services
INSERT INTO product_option_service_mappings (
    product_option_id, service_id, mapping_relationship,
    service_configuration, execution_priority, dependency_level
) VALUES

-- US Market Settlement requires specific custody and settlement services
((SELECT id FROM product_options WHERE option_id = 'CUSTODY-MARKET-US'),
 (SELECT id FROM services WHERE service_id = 'CUSTODY-ENHANCED'), 'requires',
 '{"market_scope": "US", "settlement_cycle": "T+2", "regulatory_framework": "SEC"}', 1, 1),

-- European Market Settlement
((SELECT id FROM product_options WHERE option_id = 'CUSTODY-MARKET-EU'),
 (SELECT id FROM services WHERE service_id = 'CUSTODY-ENHANCED'), 'requires',
 '{"market_scope": "EU", "settlement_cycle": "T+2", "regulatory_framework": "MiFID2"}', 1, 1),

-- APAC Market Settlement
((SELECT id FROM product_options WHERE option_id = 'CUSTODY-MARKET-APAC'),
 (SELECT id FROM services WHERE service_id = 'CUSTODY-ENHANCED'), 'requires',
 '{"market_scope": "APAC", "settlement_cycle": "T+2", "regulatory_framework": "local"}', 1, 1),

-- Multi-Currency Support enhances multiple services
((SELECT id FROM product_options WHERE option_id = 'CUSTODY-MULTI-CCY'),
 (SELECT id FROM services WHERE service_id = 'CUSTODY-ENHANCED'), 'enhances',
 '{"fx_services": true, "currency_hedging": true, "multi_currency_reporting": true}', 2, 2),

((SELECT id FROM product_options WHERE option_id = 'CUSTODY-MULTI-CCY'),
 (SELECT id FROM services WHERE service_id = 'RECON-CASH'), 'enhances',
 '{"multi_currency": true, "fx_reconciliation": true}', 3, 2),

-- Enhanced Reporting configures reporting services
((SELECT id FROM product_options WHERE option_id = 'CUSTODY-ENHANCED-REPORTING'),
 (SELECT id FROM services WHERE service_id = 'FA-FINANCIAL-REPORTING'), 'configures',
 '{"enhanced_analytics": true, "custom_dashboards": true, "api_access": true}', 4, 3)

ON CONFLICT (product_option_id, service_id, mapping_relationship) DO UPDATE SET
    service_configuration = EXCLUDED.service_configuration,
    updated_at = CURRENT_TIMESTAMP;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_product_options_product ON product_options(product_id);
CREATE INDEX IF NOT EXISTS idx_product_options_category ON product_options(option_category);
CREATE INDEX IF NOT EXISTS idx_product_options_type ON product_options(option_type);
CREATE INDEX IF NOT EXISTS idx_product_options_status ON product_options(status);

CREATE INDEX IF NOT EXISTS idx_product_option_service_mappings_option ON product_option_service_mappings(product_option_id);
CREATE INDEX IF NOT EXISTS idx_product_option_service_mappings_service ON product_option_service_mappings(service_id);
CREATE INDEX IF NOT EXISTS idx_product_option_service_mappings_relationship ON product_option_service_mappings(mapping_relationship);

-- Comments for documentation
COMMENT ON TABLE product_options IS 'Product configuration options such as market settlement choices, currency support, and feature add-ons';
COMMENT ON TABLE product_option_service_mappings IS 'Maps product options to the services they affect, configure, or require';
COMMENT ON VIEW enhanced_commercial_taxonomy_view IS 'Complete view of Products→Options→Services→Resources hierarchy with all commercial metadata';
COMMENT ON FUNCTION get_enhanced_product_taxonomy_hierarchy IS 'Retrieves complete 4-level hierarchical breakdown: Product→Options→Services→Resources';

-- Show the enhanced structure
DO $$
DECLARE
    option_count INTEGER;
    mapping_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO option_count FROM product_options WHERE status = 'active';
    SELECT COUNT(*) INTO mapping_count FROM product_option_service_mappings WHERE is_active = true;

    RAISE NOTICE '🎛️  Product Options Layer Added:';
    RAISE NOTICE '   ⚙️  Product Options: %', option_count;
    RAISE NOTICE '   🔗 Option→Service Mappings: %', mapping_count;
    RAISE NOTICE '';
    RAISE NOTICE '📊 Complete Taxonomy: Products → Options → Services → Resources';
END $$;