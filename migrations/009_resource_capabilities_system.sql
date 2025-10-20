-- Resource Capabilities System Migration
-- Adds resource templates, capabilities, and product-service-resource mappings

-- Resource Templates: Define reusable resource configurations with capabilities
CREATE TABLE resource_templates (
    id SERIAL PRIMARY KEY,
    template_id VARCHAR(50) UNIQUE NOT NULL,
    template_name VARCHAR(200) NOT NULL,
    description TEXT,
    part_of_product VARCHAR(100),  -- Links to product name
    implements_service VARCHAR(100),  -- Links to service name
    resource_type VARCHAR(50) NOT NULL,
    attributes JSONB DEFAULT '[]'::jsonb,  -- Array of attribute definitions
    capabilities JSONB DEFAULT '[]'::jsonb,  -- Array of capability definitions
    dsl_template TEXT,  -- The DSL workflow template
    version VARCHAR(20) DEFAULT '1.0',
    status VARCHAR(20) DEFAULT 'active',
    created_by VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Resource Capabilities: Specific actions that resources can perform
CREATE TABLE resource_capabilities (
    id SERIAL PRIMARY KEY,
    capability_id VARCHAR(50) UNIQUE NOT NULL,
    capability_name VARCHAR(200) NOT NULL,
    description TEXT,
    capability_type VARCHAR(50) NOT NULL, -- setup, configuration, activation, monitoring
    required_attributes JSONB DEFAULT '[]'::jsonb,  -- Array of required attribute names
    optional_attributes JSONB DEFAULT '[]'::jsonb,  -- Array of optional attribute names
    output_attributes JSONB DEFAULT '[]'::jsonb,  -- Array of produced attribute names
    implementation_function VARCHAR(200),  -- Rust function name for implementation
    validation_rules JSONB DEFAULT '{}'::jsonb,  -- Validation rules for attributes
    error_handling JSONB DEFAULT '{}'::jsonb,  -- Error handling configuration
    timeout_seconds INTEGER DEFAULT 300,
    retry_attempts INTEGER DEFAULT 3,
    status VARCHAR(20) DEFAULT 'active',
    created_by VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Product Service Mappings: Links products to the services they require
CREATE TABLE product_service_mappings (
    id SERIAL PRIMARY KEY,
    product_id INTEGER REFERENCES products(id) ON DELETE CASCADE,
    service_id INTEGER REFERENCES services(id) ON DELETE CASCADE,
    service_required BOOLEAN DEFAULT true,
    service_priority INTEGER DEFAULT 1,  -- Order of service implementation
    configuration_overrides JSONB DEFAULT '{}'::jsonb,  -- Product-specific config
    business_rules JSONB DEFAULT '[]'::jsonb,  -- Product-specific business rules
    created_by VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(product_id, service_id)
);

-- Service Resource Mappings: Links services to the resources that implement them
CREATE TABLE service_resource_mappings (
    id SERIAL PRIMARY KEY,
    service_id INTEGER REFERENCES services(id) ON DELETE CASCADE,
    resource_id INTEGER REFERENCES resources(id) ON DELETE CASCADE,
    resource_template_id INTEGER REFERENCES resource_templates(id) ON DELETE SET NULL,
    resource_role VARCHAR(100),  -- primary, secondary, backup, etc.
    resource_priority INTEGER DEFAULT 1,
    configuration_template JSONB DEFAULT '{}'::jsonb,  -- Default configuration
    deployment_requirements JSONB DEFAULT '{}'::jsonb,  -- Hardware, network, etc.
    created_by VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(service_id, resource_id)
);

-- Resource Template Capabilities: Many-to-many mapping of templates to capabilities
CREATE TABLE resource_template_capabilities (
    id SERIAL PRIMARY KEY,
    template_id INTEGER REFERENCES resource_templates(id) ON DELETE CASCADE,
    capability_id INTEGER REFERENCES resource_capabilities(id) ON DELETE CASCADE,
    capability_order INTEGER DEFAULT 1,  -- Execution order within template
    is_required BOOLEAN DEFAULT true,
    configuration_overrides JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(template_id, capability_id)
);

-- Indexes for performance
CREATE INDEX idx_resource_templates_template_id ON resource_templates(template_id);
CREATE INDEX idx_resource_templates_product ON resource_templates(part_of_product);
CREATE INDEX idx_resource_templates_service ON resource_templates(implements_service);
CREATE INDEX idx_resource_capabilities_capability_id ON resource_capabilities(capability_id);
CREATE INDEX idx_resource_capabilities_type ON resource_capabilities(capability_type);
CREATE INDEX idx_product_service_mappings_product ON product_service_mappings(product_id);
CREATE INDEX idx_product_service_mappings_service ON product_service_mappings(service_id);
CREATE INDEX idx_service_resource_mappings_service ON service_resource_mappings(service_id);
CREATE INDEX idx_service_resource_mappings_resource ON service_resource_mappings(resource_id);
CREATE INDEX idx_resource_template_capabilities_template ON resource_template_capabilities(template_id);
CREATE INDEX idx_resource_template_capabilities_capability ON resource_template_capabilities(capability_id);

-- Views for easier querying
CREATE VIEW v_product_service_resource_hierarchy AS
SELECT
    p.product_id,
    p.product_name,
    p.line_of_business,
    s.service_id,
    s.service_name,
    s.service_category,
    psm.service_required,
    psm.service_priority,
    r.resource_id,
    r.resource_name,
    r.resource_type,
    srm.resource_role,
    srm.resource_priority as resource_priority,
    rt.template_id,
    rt.template_name,
    rt.dsl_template
FROM products p
LEFT JOIN product_service_mappings psm ON p.id = psm.product_id
LEFT JOIN services s ON psm.service_id = s.id
LEFT JOIN service_resource_mappings srm ON s.id = srm.service_id
LEFT JOIN resources r ON srm.resource_id = r.id
LEFT JOIN resource_templates rt ON srm.resource_template_id = rt.id
WHERE p.status = 'active' AND (s.status IS NULL OR s.status = 'active') AND (r.status IS NULL OR r.status = 'active')
ORDER BY p.product_name, psm.service_priority, srm.resource_priority;

CREATE VIEW v_resource_template_capabilities AS
SELECT
    rt.template_id,
    rt.template_name,
    rt.part_of_product,
    rt.implements_service,
    rc.capability_id,
    rc.capability_name,
    rc.capability_type,
    rc.required_attributes,
    rc.optional_attributes,
    rc.output_attributes,
    rtc.capability_order,
    rtc.is_required as capability_required,
    rtc.configuration_overrides
FROM resource_templates rt
JOIN resource_template_capabilities rtc ON rt.id = rtc.template_id
JOIN resource_capabilities rc ON rtc.capability_id = rc.id
WHERE rt.status = 'active' AND rc.status = 'active'
ORDER BY rt.template_name, rtc.capability_order;

-- Insert sample fund accounting capabilities
INSERT INTO resource_capabilities (capability_id, capability_name, description, capability_type, required_attributes, output_attributes, implementation_function) VALUES
('account_setup', 'Account Setup', 'Creates the primary fund account structure in the system', 'setup', '["fund_legal_name", "base_currency"]', '[]', 'core_fa_account_setup_impl'),
('trade_feed_setup', 'Trade Feed Setup', 'Configures the data feed for trade capture', 'configuration', '["trade_feed_source_system_id"]', '[]', 'core_fa_trade_feed_setup_impl'),
('nav_calculation_setup', 'NAV Calculation Setup', 'Sets up Net Asset Value calculation parameters', 'configuration', '["pricing_source", "calculation_frequency"]', '[]', 'core_fa_nav_calculation_setup_impl'),
('activate', 'Activate', 'Finalizes the setup and brings the client instance live', 'activation', '[]', '["core_fa_instance_url"]', 'core_fa_activate_impl'),
('health_check', 'Health Check', 'Performs connectivity and system health verification', 'monitoring', '[]', '["health_status", "last_check_time"]', 'core_fa_health_check_impl');

-- Insert sample resource template for Core Fund Accounting
INSERT INTO resource_templates (template_id, template_name, description, part_of_product, implements_service, resource_type, attributes, dsl_template) VALUES
('CoreFAApp_v1', 'Core Fund Accounting Application v1.0', 'The core Fund Accounting application with NAV calculation capabilities', 'Fund Accounting', 'NAV Valuation', 'application',
'[
  {"name": "fund_legal_name", "dataType": "String", "ui": {"label": "Fund Legal Name"}},
  {"name": "base_currency", "dataType": "String", "ui": {"label": "Base Currency"}},
  {"name": "trade_feed_source_system_id", "dataType": "String", "ui": {"label": "Trade Feed Source System ID"}},
  {"name": "pricing_source", "dataType": "String", "ui": {"label": "Pricing Source"}},
  {"name": "calculation_frequency", "dataType": "String", "ui": {"label": "Calculation Frequency"}},
  {"name": "core_fa_instance_url", "dataType": "URL", "ui": {"label": "Instance URL"}}
]',
'WORKFLOW "SetupCoreFA"

# Configure the account structure
CONFIGURE_SYSTEM "account_setup"

# Setup trade data feed
CONFIGURE_SYSTEM "trade_feed_setup"

# Configure NAV calculation
CONFIGURE_SYSTEM "nav_calculation_setup"

# Bring the system live
ACTIVATE

# Verify everything is working
RUN_HEALTH_CHECK "health_check"

SET_STATUS "Active"');

-- Link the template to its capabilities
INSERT INTO resource_template_capabilities (template_id, capability_id, capability_order, is_required) VALUES
((SELECT id FROM resource_templates WHERE template_id = 'CoreFAApp_v1'), (SELECT id FROM resource_capabilities WHERE capability_id = 'account_setup'), 1, true),
((SELECT id FROM resource_templates WHERE template_id = 'CoreFAApp_v1'), (SELECT id FROM resource_capabilities WHERE capability_id = 'trade_feed_setup'), 2, true),
((SELECT id FROM resource_templates WHERE template_id = 'CoreFAApp_v1'), (SELECT id FROM resource_capabilities WHERE capability_id = 'nav_calculation_setup'), 3, true),
((SELECT id FROM resource_templates WHERE template_id = 'CoreFAApp_v1'), (SELECT id FROM resource_capabilities WHERE capability_id = 'activate'), 4, true),
((SELECT id FROM resource_templates WHERE template_id = 'CoreFAApp_v1'), (SELECT id FROM resource_capabilities WHERE capability_id = 'health_check'), 5, false);

-- Add fund accounting grammar extensions
INSERT INTO grammar_extensions (name, type, signature, description, category) VALUES
('CONFIGURE_SYSTEM', 'keyword', 'CONFIGURE_SYSTEM "capability_name"', 'Executes a named capability of the resource', 'fund_accounting'),
('ACTIVATE', 'keyword', 'ACTIVATE', 'Finalizes setup and activates the resource instance', 'fund_accounting'),
('RUN_HEALTH_CHECK', 'keyword', 'RUN_HEALTH_CHECK "check_type"', 'Performs system health verification', 'fund_accounting'),
('SET_STATUS', 'keyword', 'SET_STATUS "status"', 'Sets the operational status of the resource', 'fund_accounting'),
('WORKFLOW', 'keyword', 'WORKFLOW "workflow_name"', 'Defines a named workflow process', 'fund_accounting');

COMMENT ON TABLE resource_templates IS 'Templates defining reusable resource configurations with capabilities';
COMMENT ON TABLE resource_capabilities IS 'Specific actions that resources can perform';
COMMENT ON TABLE product_service_mappings IS 'Links products to the services they require';
COMMENT ON TABLE service_resource_mappings IS 'Links services to the resources that implement them';
COMMENT ON TABLE resource_template_capabilities IS 'Many-to-many mapping of templates to capabilities';