-- Comprehensive CRUD API Database Schema for DSL Systems
-- Data Designer - Financial Services Business Management Platform

-- Enable PostgreSQL extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- =============================================================================
-- CLIENT ENTITIES TABLE - Foundation for all business relationships
-- =============================================================================
CREATE TABLE IF NOT EXISTS client_entities (
    id SERIAL PRIMARY KEY,
    entity_id VARCHAR(50) UNIQUE NOT NULL,
    entity_name VARCHAR(255) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    jurisdiction VARCHAR(100) NOT NULL DEFAULT 'US',
    country_code CHAR(2) NOT NULL DEFAULT 'US',
    lei_code VARCHAR(20), -- Legal Entity Identifier
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =============================================================================
-- PRODUCTS TABLE - Financial products and services catalog
-- =============================================================================
CREATE TABLE IF NOT EXISTS products (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(50) UNIQUE NOT NULL,
    product_name VARCHAR(255) NOT NULL,
    product_type VARCHAR(100) NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =============================================================================
-- CBU (Client Business Units) TABLE - Core business unit management
-- =============================================================================
CREATE TABLE IF NOT EXISTS cbu (
    id SERIAL PRIMARY KEY,
    cbu_id VARCHAR(50) UNIQUE NOT NULL,
    cbu_name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    business_model VARCHAR(100),
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- CBU Entity Roles Association Table
CREATE TABLE IF NOT EXISTS cbu_entity_roles (
    id SERIAL PRIMARY KEY,
    cbu_id VARCHAR(50) REFERENCES cbu(cbu_id) ON DELETE CASCADE,
    entity_id VARCHAR(50) REFERENCES client_entities(entity_id) ON DELETE CASCADE,
    role VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(cbu_id, entity_id, role)
);

-- =============================================================================
-- OPPORTUNITIES TABLE - Commercial negotiation and revenue tracking
-- =============================================================================
CREATE TABLE IF NOT EXISTS opportunities (
    id SERIAL PRIMARY KEY,
    opportunity_id VARCHAR(50) UNIQUE NOT NULL,
    client_name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'active',
    expected_revenue_annual DECIMAL(15,2),
    probability_percentage INTEGER CHECK (probability_percentage >= 0 AND probability_percentage <= 100),
    negotiation_stage VARCHAR(100),
    commercial_terms TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Opportunity CBU Associations
CREATE TABLE IF NOT EXISTS opportunity_cbu_associations (
    id SERIAL PRIMARY KEY,
    opportunity_id VARCHAR(50) REFERENCES opportunities(opportunity_id) ON DELETE CASCADE,
    cbu_id VARCHAR(50) REFERENCES cbu(cbu_id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(opportunity_id, cbu_id)
);

-- Opportunity Product Associations
CREATE TABLE IF NOT EXISTS opportunity_product_associations (
    id SERIAL PRIMARY KEY,
    opportunity_id VARCHAR(50) REFERENCES opportunities(opportunity_id) ON DELETE CASCADE,
    product_id VARCHAR(50) REFERENCES products(product_id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(opportunity_id, product_id)
);

-- Revenue Streams for Opportunities
CREATE TABLE IF NOT EXISTS opportunity_revenue_streams (
    id SERIAL PRIMARY KEY,
    opportunity_id VARCHAR(50) REFERENCES opportunities(opportunity_id) ON DELETE CASCADE,
    stream_type VARCHAR(100) NOT NULL,
    amount_per_annum DECIMAL(15,2) NOT NULL,
    currency VARCHAR(10) DEFAULT 'USD',
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =============================================================================
-- ONBOARDING REQUESTS TABLE - Deal-driven onboarding management
-- =============================================================================
CREATE TABLE IF NOT EXISTS onboarding_requests (
    id SERIAL PRIMARY KEY,
    deal_id VARCHAR(50) NOT NULL,
    onboarding_description TEXT NOT NULL,
    onboarding_id VARCHAR(50) UNIQUE NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Onboarding Request CBU Associations
CREATE TABLE IF NOT EXISTS onboarding_request_cbu_associations (
    id SERIAL PRIMARY KEY,
    onboarding_id VARCHAR(50) REFERENCES onboarding_requests(onboarding_id) ON DELETE CASCADE,
    cbu_id VARCHAR(50) REFERENCES cbu(cbu_id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(onboarding_id, cbu_id)
);

-- Onboarding Request Product Associations
CREATE TABLE IF NOT EXISTS onboarding_request_product_associations (
    id SERIAL PRIMARY KEY,
    onboarding_id VARCHAR(50) REFERENCES onboarding_requests(onboarding_id) ON DELETE CASCADE,
    product_id VARCHAR(50) REFERENCES products(product_id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(onboarding_id, product_id)
);

-- =============================================================================
-- SUPPORTING BUSINESS ENTITIES
-- =============================================================================

-- Contracts Table
CREATE TABLE IF NOT EXISTS contracts (
    id SERIAL PRIMARY KEY,
    contract_id VARCHAR(50) UNIQUE NOT NULL,
    contract_name VARCHAR(255) NOT NULL,
    contract_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) DEFAULT 'active',
    start_date DATE,
    end_date DATE,
    terms TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- KYC Clearances Table
CREATE TABLE IF NOT EXISTS kyc_clearances (
    id SERIAL PRIMARY KEY,
    kyc_id VARCHAR(50) UNIQUE NOT NULL,
    entity_id VARCHAR(50) REFERENCES client_entities(entity_id),
    clearance_level VARCHAR(100) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    clearance_date DATE,
    expiry_date DATE,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Service Maps Table
CREATE TABLE IF NOT EXISTS service_maps (
    id SERIAL PRIMARY KEY,
    service_map_id VARCHAR(50) UNIQUE NOT NULL,
    service_name VARCHAR(255) NOT NULL,
    service_type VARCHAR(100) NOT NULL,
    delivery_url TEXT,
    configuration JSON,
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =============================================================================
-- DEAL RECORDS TABLE - Master orchestrator for business relationships
-- =============================================================================
CREATE TABLE IF NOT EXISTS deal_records (
    id SERIAL PRIMARY KEY,
    deal_id VARCHAR(50) UNIQUE NOT NULL,
    description TEXT NOT NULL,
    primary_introducing_client VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'active',
    business_value DECIMAL(15,2),
    negotiation_stage VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Universal Deal Resource Associations Table
CREATE TABLE IF NOT EXISTS deal_resource_associations (
    id SERIAL PRIMARY KEY,
    deal_id VARCHAR(50) REFERENCES deal_records(deal_id) ON DELETE CASCADE,
    resource_type VARCHAR(50) NOT NULL, -- CBU, PRODUCT, CONTRACT, KYC, SERVICE_MAP, OPPORTUNITY
    resource_id VARCHAR(50) NOT NULL,
    association_notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(deal_id, resource_type, resource_id)
);

-- =============================================================================
-- BUSINESS INTELLIGENCE AND ANALYTICS VIEWS
-- =============================================================================

-- Deal Summary View with Resource Counts
CREATE OR REPLACE VIEW deal_summary_view AS
SELECT
    dr.deal_id,
    dr.description,
    dr.primary_introducing_client,
    dr.status,
    dr.business_value,
    COUNT(DISTINCT CASE WHEN dra.resource_type = 'CBU' THEN dra.resource_id END) as total_cbus,
    COUNT(DISTINCT CASE WHEN dra.resource_type = 'PRODUCT' THEN dra.resource_id END) as total_products,
    COUNT(DISTINCT CASE WHEN dra.resource_type = 'CONTRACT' THEN dra.resource_id END) as total_contracts,
    COUNT(DISTINCT CASE WHEN dra.resource_type = 'KYC' THEN dra.resource_id END) as total_kyc_clearances,
    COUNT(DISTINCT CASE WHEN dra.resource_type = 'SERVICE_MAP' THEN dra.resource_id END) as total_service_maps,
    COUNT(DISTINCT CASE WHEN dra.resource_type = 'OPPORTUNITY' THEN dra.resource_id END) as total_opportunities,
    COUNT(DISTINCT dra.resource_id) as total_resources,
    dr.created_at,
    dr.updated_at
FROM deal_records dr
LEFT JOIN deal_resource_associations dra ON dr.deal_id = dra.deal_id
GROUP BY dr.deal_id, dr.description, dr.primary_introducing_client, dr.status, dr.business_value, dr.created_at, dr.updated_at;

-- Opportunity Revenue Analysis View
CREATE OR REPLACE VIEW opportunity_revenue_analysis AS
SELECT
    o.opportunity_id,
    o.client_name,
    o.description,
    o.status,
    o.probability_percentage,
    COALESCE(SUM(ors.amount_per_annum), 0) as total_annual_revenue,
    COUNT(DISTINCT oca.cbu_id) as associated_cbus,
    COUNT(DISTINCT opa.product_id) as associated_products,
    COUNT(DISTINCT ors.id) as revenue_streams,
    CASE
        WHEN COALESCE(SUM(ors.amount_per_annum), 0) >= 5000000 THEN 'Premium'
        WHEN COALESCE(SUM(ors.amount_per_annum), 0) >= 1000000 THEN 'Enterprise'
        WHEN COALESCE(SUM(ors.amount_per_annum), 0) >= 250000 THEN 'Professional'
        ELSE 'Standard'
    END as business_tier,
    o.created_at
FROM opportunities o
LEFT JOIN opportunity_revenue_streams ors ON o.opportunity_id = ors.opportunity_id
LEFT JOIN opportunity_cbu_associations oca ON o.opportunity_id = oca.opportunity_id
LEFT JOIN opportunity_product_associations opa ON o.opportunity_id = opa.opportunity_id
GROUP BY o.opportunity_id, o.client_name, o.description, o.status, o.probability_percentage, o.created_at;

-- CBU Business Intelligence View
CREATE TABLE IF NOT EXISTS cbu_business_intelligence AS
SELECT
    c.cbu_id,
    c.cbu_name,
    c.description,
    c.business_model,
    c.status,
    COUNT(DISTINCT cer.entity_id) as total_entities,
    COUNT(DISTINCT CASE WHEN cer.role = 'Asset Owner' THEN cer.entity_id END) as asset_owners,
    COUNT(DISTINCT CASE WHEN cer.role = 'Investment Manager' THEN cer.entity_id END) as investment_managers,
    COUNT(DISTINCT CASE WHEN cer.role = 'Managing Company' THEN cer.entity_id END) as managing_companies,
    COUNT(DISTINCT oca.opportunity_id) as linked_opportunities,
    COUNT(DISTINCT orpa.onboarding_id) as onboarding_requests,
    c.created_at,
    c.updated_at
FROM cbu c
LEFT JOIN cbu_entity_roles cer ON c.cbu_id = cer.cbu_id
LEFT JOIN opportunity_cbu_associations oca ON c.cbu_id = oca.cbu_id
LEFT JOIN onboarding_request_cbu_associations orpa ON c.cbu_id = orpa.cbu_id
GROUP BY c.cbu_id, c.cbu_name, c.description, c.business_model, c.status, c.created_at, c.updated_at;

-- =============================================================================
-- INDEXES FOR PERFORMANCE OPTIMIZATION
-- =============================================================================

-- Entity and relationship indexes
CREATE INDEX IF NOT EXISTS idx_client_entities_entity_id ON client_entities(entity_id);
CREATE INDEX IF NOT EXISTS idx_client_entities_entity_type ON client_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_client_entities_jurisdiction ON client_entities(jurisdiction);
CREATE INDEX IF NOT EXISTS idx_client_entities_country_code ON client_entities(country_code);
CREATE INDEX IF NOT EXISTS idx_cbu_cbu_id ON cbu(cbu_id);
CREATE INDEX IF NOT EXISTS idx_cbu_status ON cbu(status);
CREATE INDEX IF NOT EXISTS idx_products_product_id ON products(product_id);
CREATE INDEX IF NOT EXISTS idx_products_product_type ON products(product_type);

-- Opportunity indexes
CREATE INDEX IF NOT EXISTS idx_opportunities_opportunity_id ON opportunities(opportunity_id);
CREATE INDEX IF NOT EXISTS idx_opportunities_status ON opportunities(status);
CREATE INDEX IF NOT EXISTS idx_opportunities_client_name ON opportunities(client_name);
CREATE INDEX IF NOT EXISTS idx_opportunity_revenue_streams_opportunity_id ON opportunity_revenue_streams(opportunity_id);

-- Deal record indexes
CREATE INDEX IF NOT EXISTS idx_deal_records_deal_id ON deal_records(deal_id);
CREATE INDEX IF NOT EXISTS idx_deal_records_status ON deal_records(status);
CREATE INDEX IF NOT EXISTS idx_deal_resource_associations_deal_id ON deal_resource_associations(deal_id);
CREATE INDEX IF NOT EXISTS idx_deal_resource_associations_resource_type ON deal_resource_associations(resource_type);

-- Onboarding request indexes
CREATE INDEX IF NOT EXISTS idx_onboarding_requests_deal_id ON onboarding_requests(deal_id);
CREATE INDEX IF NOT EXISTS idx_onboarding_requests_onboarding_id ON onboarding_requests(onboarding_id);

-- =============================================================================
-- SAMPLE DATA FOR TESTING AND DEVELOPMENT
-- =============================================================================

-- Sample Client Entities - Global Jurisdictions (US, EU, APAC)
INSERT INTO client_entities (entity_id, entity_name, entity_type, jurisdiction, country_code, lei_code) VALUES
-- US Entities
('US001', 'Manhattan Asset Management LLC', 'Investment Manager', 'Delaware', 'US', '549300VPLTI2JI1A8N82'),
('US002', 'Goldman Sachs Asset Management', 'Investment Manager', 'New York', 'US', '784F5XWPLTWKTBV3E584'),
('US003', 'BlackRock Institutional Trust', 'Asset Owner', 'Delaware', 'US', '549300WOTC9L6FP6DY29'),
('US004', 'State Street Global Services', 'Service Provider', 'Massachusetts', 'US', '571474TGEMMWANRLN572'),
('US005', 'JPMorgan Chase Custody', 'Service Provider', 'Delaware', 'US', '7H6GLXDRUGQFU57RNE97'),
('US006', 'Vanguard Group Holdings', 'Asset Owner', 'Pennsylvania', 'US', '549300JTUDP4YQX98029'),
('US007', 'Fidelity Management Corp', 'Investment Manager', 'Massachusetts', 'US', '549300ZTR8J6RTFZWB31'),

-- EU Entities
('EU001', 'Deutsche Asset Management', 'Investment Manager', 'Germany', 'DE', '529900T8BM49AURSDO55'),
('EU002', 'BNP Paribas Asset Management', 'Investment Manager', 'France', 'FR', '969500UP76J52A9OXU27'),
('EU003', 'UBS Asset Management AG', 'Investment Manager', 'Switzerland', 'CH', '549300ZZK73H1MR76N74'),
('EU004', 'Allianz Global Investors', 'Investment Manager', 'Germany', 'DE', '529900ID4AESFTK13F68'),
('EU005', 'Credit Suisse Asset Management', 'Service Provider', 'Switzerland', 'CH', '549300V7BT7IUKCZ3J20'),
('EU006', 'HSBC Global Asset Management', 'Asset Owner', 'United Kingdom', 'GB', '549300MC6FK0JHIVX093'),
('EU007', 'Societe Generale Securities', 'Service Provider', 'France', 'FR', '969500IG9WDEZ3ZE8J29'),
('EU008', 'ABN AMRO Clearing Bank', 'Service Provider', 'Netherlands', 'NL', '724500X9VHRR6LK92X72'),

-- APAC Entities
('AP001', 'Nomura Asset Management', 'Investment Manager', 'Japan', 'JP', '353800MLJIGSLQ3JGP81'),
('AP002', 'China Asset Management Co', 'Investment Manager', 'China', 'CN', '300300S39XTBSNH66F17'),
('AP003', 'DBS Asset Management', 'Investment Manager', 'Singapore', 'SG', '549300F4WH7V9NCKXX55'),
('AP004', 'ANZ Bank New Zealand', 'Service Provider', 'New Zealand', 'NZ', '549300LJJ8XRXQXF7J76'),
('AP005', 'Mitsubishi UFJ Trust', 'Asset Owner', 'Japan', 'JP', '549300U4YUI5J3T5H493'),
('AP006', 'Korea Investment Partners', 'Investment Manager', 'South Korea', 'KR', '988400OK6BY71LL5H823'),
('AP007', 'Commonwealth Bank Asset Mgmt', 'Service Provider', 'Australia', 'AU', '213800I5D6XDRL88WH45'),
('AP008', 'Standard Chartered Private Bank', 'Asset Owner', 'Hong Kong', 'HK', '549300KF3S2WVZP3CH31'),
('AP009', 'CIMB Investment Bank', 'Service Provider', 'Malaysia', 'MY', '549300P1LJF26Q9NPH44'),
('AP010', 'Bangkok Bank Asset Management', 'Investment Manager', 'Thailand', 'TH', '549300GN5M3NVBF7FX52')

ON CONFLICT (entity_id) DO NOTHING;

-- Sample Products
INSERT INTO products (product_id, product_name, product_type, description) VALUES
('PROD001', 'Custody Services', 'Custody', 'Comprehensive asset custody and safekeeping'),
('PROD002', 'Fund Accounting', 'Accounting', 'Professional fund accounting and reporting'),
('PROD003', 'Investment Management', 'Investment', 'Active portfolio management services'),
('PROD004', 'Risk Analytics', 'Analytics', 'Advanced risk measurement and reporting'),
('PROD005', 'Compliance Monitoring', 'Compliance', 'Regulatory compliance and monitoring')
ON CONFLICT (product_id) DO NOTHING;

-- Sample CBUs
INSERT INTO cbu (cbu_id, cbu_name, description, business_model) VALUES
('CBU001', 'Growth Fund Alpha', 'A diversified growth-focused investment fund', 'Mutual Fund'),
('CBU002', 'Value Equity Beta', 'Value-oriented equity investment strategy', 'Hedge Fund'),
('CBU003', 'Fixed Income Gamma', 'Conservative fixed income portfolio', 'Bond Fund')
ON CONFLICT (cbu_id) DO NOTHING;

-- Sample CBU Entity Roles
INSERT INTO cbu_entity_roles (cbu_id, entity_id, role) VALUES
('CBU001', 'AC001', 'Asset Owner'),
('CBU001', 'BM002', 'Investment Manager'),
('CBU001', 'GS003', 'Managing Company'),
('CBU002', 'DC004', 'Asset Owner'),
('CBU002', 'BM002', 'Investment Manager'),
('CBU003', 'EC005', 'Asset Owner'),
('CBU003', 'GS003', 'Managing Company')
ON CONFLICT (cbu_id, entity_id, role) DO NOTHING;

-- Sample Opportunities
INSERT INTO opportunities (opportunity_id, client_name, description, expected_revenue_annual, probability_percentage, negotiation_stage) VALUES
('OPP001', 'Alpha Capital', 'Multi-product custody and fund accounting deal', 1250000.00, 85, 'Final Negotiation'),
('OPP002', 'Beta Management', 'Investment management platform integration', 750000.00, 70, 'Technical Review'),
('OPP003', 'Gamma Services', 'Comprehensive service delivery expansion', 2000000.00, 60, 'Commercial Discussion')
ON CONFLICT (opportunity_id) DO NOTHING;

-- Sample Opportunity Revenue Streams
INSERT INTO opportunity_revenue_streams (opportunity_id, stream_type, amount_per_annum, description) VALUES
('OPP001', 'Custody', 1000000.00, 'Annual custody fees for asset safekeeping'),
('OPP001', 'Fund Accounting', 250000.00, 'Fund accounting and reporting services'),
('OPP002', 'Investment Management', 750000.00, 'Portfolio management and advisory services'),
('OPP003', 'Service Platform', 2000000.00, 'Comprehensive service delivery platform')
ON CONFLICT DO NOTHING;

-- Sample Contracts
INSERT INTO contracts (contract_id, contract_name, contract_type, status) VALUES
('CONTR001', 'Master Services Agreement Alpha', 'MSA', 'active'),
('CONTR002', 'Investment Management Contract Beta', 'IMA', 'active'),
('CONTR003', 'Custody Services Agreement Gamma', 'CSA', 'draft')
ON CONFLICT (contract_id) DO NOTHING;

-- Sample KYC Clearances
INSERT INTO kyc_clearances (kyc_id, entity_id, clearance_level, status, clearance_date) VALUES
('KYC001', 'AC001', 'Enhanced', 'approved', CURRENT_DATE - INTERVAL '30 days'),
('KYC002', 'BM002', 'Standard', 'approved', CURRENT_DATE - INTERVAL '60 days'),
('KYC003', 'DC004', 'Enhanced', 'pending', NULL)
ON CONFLICT (kyc_id) DO NOTHING;

-- Sample Service Maps
INSERT INTO service_maps (service_map_id, service_name, service_type, delivery_url, status) VALUES
('SM001', 'Custody Portal Alpha', 'Web Portal', 'https://custody.alpha.example.com', 'active'),
('SM002', 'Fund Accounting Dashboard', 'Analytics Dashboard', 'https://accounting.beta.example.com', 'active'),
('SM003', 'Risk Management Console', 'Risk Platform', 'https://risk.gamma.example.com', 'testing')
ON CONFLICT (service_map_id) DO NOTHING;

-- Sample Deal Records
INSERT INTO deal_records (deal_id, description, primary_introducing_client, business_value, status) VALUES
('DEAL001', 'Alpha Bank Multi-Product Onboarding', 'Alpha Corporation', 5000000.00, 'active'),
('DEAL002', 'Beta Investment Management Platform', 'Beta Management', 3000000.00, 'negotiation'),
('DEAL003', 'Gamma Comprehensive Services Deal', 'Gamma Services', 7500000.00, 'proposal')
ON CONFLICT (deal_id) DO NOTHING;

-- =============================================================================
-- COMMENTS AND DOCUMENTATION
-- =============================================================================

COMMENT ON TABLE client_entities IS 'Foundation table for all business entities and relationships';
COMMENT ON TABLE cbu IS 'Client Business Units - core organizational structures for fund management';
COMMENT ON TABLE opportunities IS 'Commercial negotiation entities with revenue projections and business intelligence';
COMMENT ON TABLE deal_records IS 'Master orchestrator linking all business components under negotiated deals';
COMMENT ON TABLE deal_resource_associations IS 'Universal association table for linking any resource type to deals';
COMMENT ON TABLE opportunity_revenue_streams IS 'Revenue modeling for commercial opportunities with annual projections';

COMMENT ON VIEW deal_summary_view IS 'Comprehensive deal analytics with resource counts and business intelligence';
COMMENT ON VIEW opportunity_revenue_analysis IS 'Revenue analysis and business tier classification for opportunities';