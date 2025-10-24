-- Migration 004: Product-Services-Resources Hierarchy for Custody Banking
-- This implements the commercial product model with virtual services and physical resources

-- ===== CORE HIERARCHY TABLES =====

-- Products: Commercial custody banking products sold to financial institutions
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(100) UNIQUE NOT NULL,
    product_name VARCHAR(255) NOT NULL,
    line_of_business VARCHAR(100) NOT NULL, -- Custody, Fund Accounting, Transfer Agency
    description TEXT,
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'deprecated')),
    pricing_model VARCHAR(50), -- per-transaction, per-asset, subscription, etc.
    target_market VARCHAR(100), -- institutional, retail, pension funds, etc.
    regulatory_requirements JSONB, -- compliance, reporting requirements
    sla_commitments JSONB, -- service level agreements
    created_by VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Services: Virtual/logical services that collectively define products
CREATE TABLE services (
    id SERIAL PRIMARY KEY,
    service_id VARCHAR(100) UNIQUE NOT NULL,
    service_name VARCHAR(255) NOT NULL,
    service_category VARCHAR(100), -- Operational, Reporting, Settlement, etc.
    description TEXT,
    is_core_service BOOLEAN DEFAULT false, -- required for all products vs optional
    configuration_schema JSONB, -- JSON schema for service-specific configuration
    dependencies TEXT[], -- array of service_ids this service depends on
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'development')),
    created_by VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Resources: Physical implementors (applications, teams) of logical services
CREATE TABLE resources (
    id SERIAL PRIMARY KEY,
    resource_id VARCHAR(100) UNIQUE NOT NULL,
    resource_name VARCHAR(255) NOT NULL,
    resource_type VARCHAR(50) NOT NULL, -- Application, Team, System, Infrastructure
    description TEXT,
    location VARCHAR(100), -- data center, office location, cloud region
    capacity_limits JSONB, -- throughput, concurrent users, etc.
    operational_hours VARCHAR(100), -- 24/7, business hours, scheduled maintenance
    contact_information JSONB, -- primary contacts, escalation procedures
    technical_specifications JSONB, -- versions, configurations, capabilities
    compliance_certifications TEXT[], -- SOC2, ISO27001, etc.
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'maintenance', 'deprecated')),
    created_by VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ===== RELATIONSHIP TABLES =====

-- Product-Service mappings: Which services make up each product
CREATE TABLE product_services (
    id SERIAL PRIMARY KEY,
    product_id INTEGER REFERENCES products(id) ON DELETE CASCADE,
    service_id INTEGER REFERENCES services(id) ON DELETE CASCADE,
    is_required BOOLEAN DEFAULT true, -- required vs optional service for this product
    configuration JSONB, -- product-specific service configuration
    pricing_component DECIMAL(10,2), -- cost contribution of this service to product
    display_order INTEGER, -- ordering for UI display
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(product_id, service_id)
);

-- Service-Resource mappings: Which resources implement each service
CREATE TABLE service_resources (
    id SERIAL PRIMARY KEY,
    service_id INTEGER REFERENCES services(id) ON DELETE CASCADE,
    resource_id INTEGER REFERENCES resources(id) ON DELETE CASCADE,
    resource_role VARCHAR(100), -- Primary, Backup, Load-Balancer, etc.
    configuration JSONB, -- service-specific resource configuration
    priority INTEGER DEFAULT 1, -- execution priority (1 = highest)
    health_check_endpoint VARCHAR(255), -- for monitoring
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(service_id, resource_id)
);

-- Resource dependencies: Inter-resource dependencies within services
CREATE TABLE resource_dependencies (
    id SERIAL PRIMARY KEY,
    dependent_resource_id INTEGER REFERENCES resources(id) ON DELETE CASCADE,
    prerequisite_resource_id INTEGER REFERENCES resources(id) ON DELETE CASCADE,
    dependency_type VARCHAR(50), -- hard, soft, conditional
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CHECK (dependent_resource_id != prerequisite_resource_id),
    UNIQUE(dependent_resource_id, prerequisite_resource_id)
);

-- ===== CBU INTEGRATION TABLES =====

-- CBU Product subscriptions: Which products CBUs are subscribed to
CREATE TABLE cbu_product_subscriptions (
    id SERIAL PRIMARY KEY,
    cbu_id INTEGER REFERENCES client_business_units(id) ON DELETE CASCADE,
    product_id INTEGER REFERENCES products(id) ON DELETE CASCADE,
    subscription_status VARCHAR(20) DEFAULT 'pending'
        CHECK (subscription_status IN ('pending', 'active', 'suspended', 'terminated')),
    subscription_date TIMESTAMPTZ,
    activation_date TIMESTAMPTZ,
    termination_date TIMESTAMPTZ,
    billing_arrangement JSONB, -- pricing, payment terms, discounts
    contract_reference VARCHAR(100),
    primary_contact_role_id INTEGER REFERENCES cbu_roles(id), -- main contact for this product
    created_by VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(cbu_id, product_id)
);

-- Role-based service access: How different CBU roles interact with services
CREATE TABLE role_service_access (
    id SERIAL PRIMARY KEY,
    cbu_role_id INTEGER REFERENCES cbu_roles(id) ON DELETE CASCADE,
    service_id INTEGER REFERENCES services(id) ON DELETE CASCADE,
    access_type VARCHAR(50), -- Full, ReadOnly, Restricted, NoAccess
    interaction_mode VARCHAR(50), -- Direct, Portal, API, Manual
    business_justification TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(cbu_role_id, service_id)
);

-- ===== ONBOARDING WORKFLOW TABLES =====

-- Onboarding requests: Workflow for CBU product onboarding
CREATE TABLE onboarding_requests (
    id SERIAL PRIMARY KEY,
    request_id VARCHAR(100) UNIQUE NOT NULL,
    cbu_id INTEGER REFERENCES client_business_units(id) ON DELETE CASCADE,
    product_id INTEGER REFERENCES products(id) ON DELETE CASCADE,
    request_status VARCHAR(20) DEFAULT 'draft'
        CHECK (request_status IN ('draft', 'submitted', 'under_review', 'approved', 'in_progress', 'completed', 'rejected', 'cancelled')),
    priority VARCHAR(20) DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    target_go_live_date DATE,
    business_requirements JSONB, -- specific requirements for this onboarding
    compliance_requirements JSONB, -- regulatory, audit requirements
    requested_by VARCHAR(100),
    assigned_to VARCHAR(100), -- onboarding manager
    approval_chain JSONB, -- approval workflow and status
    estimated_duration_days INTEGER,
    actual_duration_days INTEGER,
    created_by VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Onboarding tasks: Individual tasks within an onboarding request
CREATE TABLE onboarding_tasks (
    id SERIAL PRIMARY KEY,
    onboarding_request_id INTEGER REFERENCES onboarding_requests(id) ON DELETE CASCADE,
    task_id VARCHAR(100) NOT NULL,
    resource_id INTEGER REFERENCES resources(id), -- resource that needs to be configured
    task_type VARCHAR(50), -- Configuration, Setup, Testing, Training, Documentation
    task_name VARCHAR(255) NOT NULL,
    description TEXT,
    task_status VARCHAR(20) DEFAULT 'pending'
        CHECK (task_status IN ('pending', 'assigned', 'in_progress', 'blocked', 'completed', 'skipped')),
    assigned_to VARCHAR(100), -- team or individual responsible
    dependencies TEXT[], -- array of task_ids this task depends on
    estimated_hours DECIMAL(5,1),
    actual_hours DECIMAL(5,1),
    due_date DATE,
    completion_date DATE,
    blocking_issues TEXT,
    completion_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(onboarding_request_id, task_id)
);

-- CBU Service Resource mapping: Track which resources are configured for each CBU
CREATE TABLE cbu_service_resources (
    id SERIAL PRIMARY KEY,
    cbu_id INTEGER REFERENCES client_business_units(id) ON DELETE CASCADE,
    service_id INTEGER REFERENCES services(id) ON DELETE CASCADE,
    resource_id INTEGER REFERENCES resources(id) ON DELETE CASCADE,
    configuration_status VARCHAR(20) DEFAULT 'not_configured'
        CHECK (configuration_status IN ('not_configured', 'in_progress', 'configured', 'testing', 'active', 'inactive', 'error')),
    configuration_details JSONB, -- specific configuration for this CBU-service-resource
    go_live_date DATE,
    last_health_check TIMESTAMPTZ,
    health_status VARCHAR(20) DEFAULT 'unknown'
        CHECK (health_status IN ('healthy', 'warning', 'error', 'unknown')),
    responsible_team VARCHAR(100),
    onboarding_request_id INTEGER REFERENCES onboarding_requests(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(cbu_id, service_id, resource_id)
);

-- ===== INDEXES FOR PERFORMANCE =====

-- Product hierarchy indexes
CREATE INDEX idx_products_line_of_business ON products(line_of_business);
CREATE INDEX idx_products_status ON products(status);
CREATE INDEX idx_services_category ON services(service_category);
CREATE INDEX idx_services_status ON services(status);
CREATE INDEX idx_resources_type ON resources(resource_type);
CREATE INDEX idx_resources_status ON resources(status);

-- Relationship indexes
CREATE INDEX idx_product_services_product ON product_services(product_id);
CREATE INDEX idx_product_services_service ON product_services(service_id);
CREATE INDEX idx_service_resources_service ON service_resources(service_id);
CREATE INDEX idx_service_resources_resource ON service_resources(resource_id);

-- CBU integration indexes
CREATE INDEX idx_cbu_subscriptions_cbu ON cbu_product_subscriptions(cbu_id);
CREATE INDEX idx_cbu_subscriptions_product ON cbu_product_subscriptions(product_id);
CREATE INDEX idx_cbu_subscriptions_status ON cbu_product_subscriptions(subscription_status);

-- Onboarding workflow indexes
CREATE INDEX idx_onboarding_cbu ON onboarding_requests(cbu_id);
CREATE INDEX idx_onboarding_product ON onboarding_requests(product_id);
CREATE INDEX idx_onboarding_status ON onboarding_requests(request_status);
CREATE INDEX idx_onboarding_tasks_request ON onboarding_tasks(onboarding_request_id);
CREATE INDEX idx_onboarding_tasks_resource ON onboarding_tasks(resource_id);
CREATE INDEX idx_cbu_service_resources_cbu ON cbu_service_resources(cbu_id);

-- ===== SAMPLE DATA =====

-- Insert sample Lines of Business and Products
INSERT INTO products (product_id, product_name, line_of_business, description, target_market, pricing_model) VALUES
('CUST-001', 'Institutional Custody Plus', 'Custody', 'Full-service custody solution for institutional investors', 'Pension Funds, Insurance Companies', 'per-asset'),
('CUST-002', 'Prime Brokerage Services', 'Custody', 'Prime brokerage with securities lending and financing', 'Hedge Funds, Asset Managers', 'per-transaction'),
('FUND-001', 'Fund Accounting Pro', 'Fund Accounting', 'Comprehensive fund accounting and reporting', 'Mutual Funds, ETFs', 'subscription'),
('FUND-002', 'Alternative Investment Accounting', 'Fund Accounting', 'Specialized accounting for alternative investments', 'Private Equity, Real Estate Funds', 'per-fund'),
('TRANS-001', 'Transfer Agency Standard', 'Transfer Agency', 'Shareholder record keeping and processing', 'Mutual Fund Companies', 'per-account');

-- Insert sample Services
INSERT INTO services (service_id, service_name, service_category, description, is_core_service) VALUES
('SERV-RECON', 'Reconciliation', 'Operational', 'Daily reconciliation of positions and cash', true),
('SERV-TRADE', 'Trade and Instruction Capture', 'Operational', 'Trade capture, validation, and settlement instruction processing', true),
('SERV-NAV', 'NAV Dissemination', 'Reporting', 'Net Asset Value calculation and distribution', true),
('SERV-SAFE', 'Safe Keeping', 'Custody', 'Physical and book-entry asset safekeeping', true),
('SERV-FUND-ACC', 'Fund Accounting', 'Accounting', 'Complete fund accounting and financial reporting', false),
('SERV-PERF', 'Performance Measurement', 'Reporting', 'Investment performance calculation and attribution', false),
('SERV-COMP', 'Compliance Monitoring', 'Risk', 'Real-time compliance and risk monitoring', false),
('SERV-CLIENT-REP', 'Client Reporting', 'Reporting', 'Customized client reporting and analytics', false);

-- Insert sample Resources
INSERT INTO resources (resource_id, resource_name, resource_type, description, location) VALUES
('RES-EAGLE', 'Eagle PACE', 'Application', 'Core accounting and reporting system', 'Primary Data Center'),
('RES-SWIFT', 'SWIFT Network', 'System', 'Global financial messaging network', 'Multiple Locations'),
('RES-PORTIA', 'Portia Trading System', 'Application', 'Trade order management and execution', 'Trading Floor'),
('RES-RECON-TEAM', 'Reconciliation Team', 'Team', 'Operational team managing daily reconciliations', 'Operations Center'),
('RES-CUSTODY-TEAM', 'Custody Operations Team', 'Team', 'Team managing physical asset custody', 'Vault Operations'),
('RES-DTCC', 'DTCC Settlement', 'System', 'Depository Trust & Clearing Corporation connection', 'Settlement Network'),
('RES-EUROCLEAR', 'Euroclear System', 'System', 'European settlement and custody system', 'European Network'),
('RES-BLOOMBERG', 'Bloomberg Terminal', 'Application', 'Market data and analytics platform', 'Trading Floor');

-- Link Products to Services
INSERT INTO product_services (product_id, service_id, is_required, display_order) VALUES
-- Institutional Custody Plus
((SELECT id FROM products WHERE product_id = 'CUST-001'), (SELECT id FROM services WHERE service_id = 'SERV-SAFE'), true, 1),
((SELECT id FROM products WHERE product_id = 'CUST-001'), (SELECT id FROM services WHERE service_id = 'SERV-RECON'), true, 2),
((SELECT id FROM products WHERE product_id = 'CUST-001'), (SELECT id FROM services WHERE service_id = 'SERV-CLIENT-REP'), true, 3),
((SELECT id FROM products WHERE product_id = 'CUST-001'), (SELECT id FROM services WHERE service_id = 'SERV-COMP'), false, 4),

-- Fund Accounting Pro
((SELECT id FROM products WHERE product_id = 'FUND-001'), (SELECT id FROM services WHERE service_id = 'SERV-FUND-ACC'), true, 1),
((SELECT id FROM products WHERE product_id = 'FUND-001'), (SELECT id FROM services WHERE service_id = 'SERV-NAV'), true, 2),
((SELECT id FROM products WHERE product_id = 'FUND-001'), (SELECT id FROM services WHERE service_id = 'SERV-RECON'), true, 3),
((SELECT id FROM products WHERE product_id = 'FUND-001'), (SELECT id FROM services WHERE service_id = 'SERV-PERF'), false, 4);

-- Link Services to Resources
INSERT INTO service_resources (service_id, resource_id, resource_role, priority) VALUES
-- Reconciliation Service
((SELECT id FROM services WHERE service_id = 'SERV-RECON'), (SELECT id FROM resources WHERE resource_id = 'RES-EAGLE'), 'Primary', 1),
((SELECT id FROM services WHERE service_id = 'SERV-RECON'), (SELECT id FROM resources WHERE resource_id = 'RES-RECON-TEAM'), 'Operational', 2),

-- Safe Keeping Service
((SELECT id FROM services WHERE service_id = 'SERV-SAFE'), (SELECT id FROM resources WHERE resource_id = 'RES-DTCC'), 'Primary', 1),
((SELECT id FROM services WHERE service_id = 'SERV-SAFE'), (SELECT id FROM resources WHERE resource_id = 'RES-EUROCLEAR'), 'Secondary', 2),
((SELECT id FROM services WHERE service_id = 'SERV-SAFE'), (SELECT id FROM resources WHERE resource_id = 'RES-CUSTODY-TEAM'), 'Operational', 3),

-- Trade Capture Service
((SELECT id FROM services WHERE service_id = 'SERV-TRADE'), (SELECT id FROM resources WHERE resource_id = 'RES-PORTIA'), 'Primary', 1),
((SELECT id FROM services WHERE service_id = 'SERV-TRADE'), (SELECT id FROM resources WHERE resource_id = 'RES-SWIFT'), 'Messaging', 2);

-- Set up role-based service access patterns
INSERT INTO role_service_access (cbu_role_id, service_id, access_type, interaction_mode, business_justification) VALUES
-- Investment Manager role access
((SELECT id FROM cbu_roles WHERE role_code = 'INV_MGR'), (SELECT id FROM services WHERE service_id = 'SERV-TRADE'), 'Full', 'Portal', 'Investment managers need full access to trade capture and execution'),
((SELECT id FROM cbu_roles WHERE role_code = 'INV_MGR'), (SELECT id FROM services WHERE service_id = 'SERV-PERF'), 'ReadOnly', 'Portal', 'Investment managers monitor performance but do not modify calculations'),

-- Asset Owner role access
((SELECT id FROM cbu_roles WHERE role_code = 'ASSET_OWNER'), (SELECT id FROM services WHERE service_id = 'SERV-SAFE'), 'ReadOnly', 'Portal', 'Asset owners monitor custody but do not execute operational functions'),
((SELECT id FROM cbu_roles WHERE role_code = 'ASSET_OWNER'), (SELECT id FROM services WHERE service_id = 'SERV-CLIENT-REP'), 'Full', 'Portal', 'Asset owners have full access to their reporting and analytics');

-- ===== TRIGGERS FOR AUDIT TRAIL =====

-- Update timestamp triggers
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_products_updated_at BEFORE UPDATE ON products FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_services_updated_at BEFORE UPDATE ON services FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_resources_updated_at BEFORE UPDATE ON resources FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_cbu_subscriptions_updated_at BEFORE UPDATE ON cbu_product_subscriptions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_onboarding_requests_updated_at BEFORE UPDATE ON onboarding_requests FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_onboarding_tasks_updated_at BEFORE UPDATE ON onboarding_tasks FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_cbu_service_resources_updated_at BEFORE UPDATE ON cbu_service_resources FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ===== VIEWS FOR COMMON QUERIES =====

-- Complete product hierarchy view
CREATE VIEW v_product_hierarchy AS
SELECT
    p.product_id,
    p.product_name,
    p.line_of_business,
    p.status as product_status,
    s.service_id,
    s.service_name,
    s.service_category,
    ps.is_required as service_required,
    r.resource_id,
    r.resource_name,
    r.resource_type,
    sr.resource_role,
    sr.priority as resource_priority
FROM products p
JOIN product_services ps ON p.id = ps.product_id
JOIN services s ON ps.service_id = s.id
JOIN service_resources sr ON s.id = sr.service_id
JOIN resources r ON sr.resource_id = r.id
WHERE p.status = 'active' AND s.status = 'active' AND r.status = 'active'
ORDER BY p.product_name, ps.display_order, sr.priority;

-- CBU product subscription summary
CREATE VIEW v_cbu_product_subscriptions AS
SELECT
    cbu.cbu_id,
    cbu.cbu_name,
    p.product_id,
    p.product_name,
    p.line_of_business,
    cps.subscription_status,
    cps.subscription_date,
    cps.activation_date,
    cr.role_name as primary_contact_role
FROM client_business_units cbu
JOIN cbu_product_subscriptions cps ON cbu.id = cps.cbu_id
JOIN products p ON cps.product_id = p.id
LEFT JOIN cbu_roles cr ON cps.primary_contact_role_id = cr.id
ORDER BY cbu.cbu_name, p.product_name;

-- Onboarding progress view
CREATE VIEW v_onboarding_progress AS
SELECT
    or_main.request_id,
    cbu.cbu_name,
    p.product_name,
    or_main.request_status,
    or_main.target_go_live_date,
    COUNT(ot.id) as total_tasks,
    COUNT(CASE WHEN ot.task_status = 'completed' THEN 1 END) as completed_tasks,
    COUNT(CASE WHEN ot.task_status = 'blocked' THEN 1 END) as blocked_tasks,
    ROUND(
        COUNT(CASE WHEN ot.task_status = 'completed' THEN 1 END)::numeric /
        NULLIF(COUNT(ot.id), 0) * 100, 1
    ) as completion_percentage
FROM onboarding_requests or_main
JOIN client_business_units cbu ON or_main.cbu_id = cbu.id
JOIN products p ON or_main.product_id = p.id
LEFT JOIN onboarding_tasks ot ON or_main.id = ot.onboarding_request_id
GROUP BY or_main.id, or_main.request_id, cbu.cbu_name, p.product_name,
         or_main.request_status, or_main.target_go_live_date
ORDER BY or_main.created_at DESC;

COMMENT ON TABLE products IS 'Commercial custody banking products sold to financial institutions';
COMMENT ON TABLE services IS 'Virtual/logical services that collectively define products';
COMMENT ON TABLE resources IS 'Physical implementors (applications, teams) of logical services';
COMMENT ON TABLE onboarding_requests IS 'Workflow tracking for CBU product onboarding process';
COMMENT ON VIEW v_product_hierarchy IS 'Complete view of product-service-resource relationships';
COMMENT ON VIEW v_onboarding_progress IS 'Real-time onboarding progress tracking with completion metrics';