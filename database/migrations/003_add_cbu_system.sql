-- Client Business Unit (CBU) System Migration
-- Creates tables for managing client business units with role-based relationships

-- Enable UUID extension if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Drop existing tables if they exist (for clean migration)
DROP TABLE IF EXISTS cbu_members CASCADE;
DROP TABLE IF EXISTS cbu_roles CASCADE;
DROP TABLE IF EXISTS client_business_units CASCADE;

-- CBU Roles table - defines the role taxonomy
CREATE TABLE cbu_roles (
    id SERIAL PRIMARY KEY,
    role_code VARCHAR(50) UNIQUE NOT NULL,
    role_name VARCHAR(100) NOT NULL,
    description TEXT,
    role_category VARCHAR(50), -- e.g., 'asset_management', 'regulatory', 'operational'
    display_order INTEGER DEFAULT 999,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Client Business Units table - main CBU entity
CREATE TABLE client_business_units (
    id SERIAL PRIMARY KEY,
    cbu_id VARCHAR(100) UNIQUE NOT NULL, -- External identifier
    cbu_name VARCHAR(255) NOT NULL,
    description TEXT,

    -- Primary contact/entity
    primary_entity_id VARCHAR(100), -- References external client system
    primary_lei VARCHAR(20), -- Legal Entity Identifier of primary entity

    -- Business metadata
    domicile_country CHAR(2), -- ISO country code
    regulatory_jurisdiction VARCHAR(50),
    business_type VARCHAR(50), -- e.g., 'asset_manager', 'pension_fund', 'insurance'

    -- Status and lifecycle
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'pending', 'suspended')),
    created_date DATE DEFAULT CURRENT_DATE,
    last_review_date DATE,
    next_review_date DATE,

    -- Audit fields
    created_by VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Metadata
    metadata JSONB, -- For additional flexible data

    CONSTRAINT valid_lei CHECK (primary_lei IS NULL OR primary_lei ~ '^[A-Z0-9]{20}$'),
    CONSTRAINT valid_country CHECK (domicile_country IS NULL OR domicile_country ~ '^[A-Z]{2}$')
);

-- CBU Members table - links entities to CBUs with roles
CREATE TABLE cbu_members (
    id SERIAL PRIMARY KEY,
    cbu_id INTEGER NOT NULL REFERENCES client_business_units(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES cbu_roles(id),

    -- Entity information
    entity_id VARCHAR(100) NOT NULL, -- External client/entity identifier
    entity_name VARCHAR(255) NOT NULL,
    entity_lei VARCHAR(20), -- Legal Entity Identifier

    -- Relationship details
    is_primary BOOLEAN DEFAULT FALSE, -- Is this the primary entity for this role?
    effective_date DATE DEFAULT CURRENT_DATE,
    expiry_date DATE,

    -- Contact and operational details
    contact_email VARCHAR(255),
    contact_phone VARCHAR(50),
    authorized_persons JSONB, -- Array of authorized contact persons

    -- Operational flags
    is_active BOOLEAN DEFAULT TRUE,
    receives_notifications BOOLEAN DEFAULT TRUE,
    has_trading_authority BOOLEAN DEFAULT FALSE,
    has_settlement_authority BOOLEAN DEFAULT FALSE,

    -- Audit fields
    created_by VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Metadata
    notes TEXT,
    metadata JSONB,

    -- Constraints
    UNIQUE(cbu_id, entity_id, role_id), -- An entity can have only one instance of each role per CBU
    CONSTRAINT valid_member_lei CHECK (entity_lei IS NULL OR entity_lei ~ '^[A-Z0-9]{20}$'),
    CONSTRAINT valid_email CHECK (contact_email IS NULL OR contact_email ~ '^[^@]+@[^@]+\.[^@]+$'),
    CONSTRAINT valid_dates CHECK (expiry_date IS NULL OR expiry_date > effective_date)
);

-- Insert default CBU roles
INSERT INTO cbu_roles (role_code, role_name, description, role_category, display_order) VALUES
-- Asset Management Roles
('ASSET_OWNER', 'Asset Owner', 'Ultimate beneficial owner of the assets', 'asset_management', 1),
('INVESTMENT_MANAGER', 'Investment Manager', 'Entity responsible for investment decisions', 'asset_management', 2),
('MANAGEMENT_COMPANY', 'Management Company', 'Legal management entity for funds', 'asset_management', 3),
('LUX_SICAV', 'Luxembourg SICAV', 'Luxembourg investment company with variable capital', 'asset_management', 4),

-- Regulatory and Compliance Roles
('REGULATORY_CONTACT', 'Regulatory Contact', 'Primary regulatory liaison', 'regulatory', 10),
('COMPLIANCE_OFFICER', 'Compliance Officer', 'Compliance oversight contact', 'regulatory', 11),
('AUTHORIZED_REPRESENTATIVE', 'Authorized Representative', 'Legal representative for jurisdiction', 'regulatory', 12),

-- Operational Roles
('CUSTODIAN', 'Custodian', 'Asset custody provider', 'operational', 20),
('PRIME_BROKER', 'Prime Broker', 'Prime brokerage services provider', 'operational', 21),
('ADMINISTRATOR', 'Administrator', 'Fund administration services', 'operational', 22),
('TRANSFER_AGENT', 'Transfer Agent', 'Share registration and transfer services', 'operational', 23),

-- Trading and Settlement
('EXECUTION_BROKER', 'Execution Broker', 'Trade execution services', 'trading', 30),
('SETTLEMENT_AGENT', 'Settlement Agent', 'Trade settlement processing', 'trading', 31),
('CLEARING_MEMBER', 'Clearing Member', 'Clearing and risk management', 'trading', 32),

-- Service Providers
('AUDITOR', 'Auditor', 'External audit services', 'services', 40),
('TAX_ADVISOR', 'Tax Advisor', 'Tax advisory services', 'services', 41),
('LEGAL_COUNSEL', 'Legal Counsel', 'Legal advisory services', 'services', 42);

-- Create indexes for performance
CREATE INDEX idx_cbu_cbu_id ON client_business_units(cbu_id);
CREATE INDEX idx_cbu_status ON client_business_units(status);
CREATE INDEX idx_cbu_primary_lei ON client_business_units(primary_lei);
CREATE INDEX idx_cbu_country ON client_business_units(domicile_country);
CREATE INDEX idx_cbu_business_type ON client_business_units(business_type);
CREATE INDEX idx_cbu_created_date ON client_business_units(created_date);

CREATE INDEX idx_cbu_roles_code ON cbu_roles(role_code);
CREATE INDEX idx_cbu_roles_category ON cbu_roles(role_category);
CREATE INDEX idx_cbu_roles_active ON cbu_roles(is_active);

CREATE INDEX idx_cbu_members_cbu_id ON cbu_members(cbu_id);
CREATE INDEX idx_cbu_members_role_id ON cbu_members(role_id);
CREATE INDEX idx_cbu_members_entity_id ON cbu_members(entity_id);
CREATE INDEX idx_cbu_members_entity_lei ON cbu_members(entity_lei);
CREATE INDEX idx_cbu_members_active ON cbu_members(is_active);
CREATE INDEX idx_cbu_members_primary ON cbu_members(is_primary);
CREATE INDEX idx_cbu_members_effective_date ON cbu_members(effective_date);

-- Create update triggers for updated_at timestamps
CREATE TRIGGER update_cbu_updated_at
    BEFORE UPDATE ON client_business_units
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_cbu_roles_updated_at
    BEFORE UPDATE ON cbu_roles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_cbu_members_updated_at
    BEFORE UPDATE ON cbu_members
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create views for easier querying

-- CBU Summary View
CREATE VIEW v_cbu_summary AS
SELECT
    cbu.id,
    cbu.cbu_id,
    cbu.cbu_name,
    cbu.description,
    cbu.primary_lei,
    cbu.domicile_country,
    cbu.business_type,
    cbu.status,
    cbu.created_date,
    COUNT(DISTINCT cm.id) as member_count,
    COUNT(DISTINCT cm.role_id) as role_count,
    STRING_AGG(DISTINCT cr.role_name, ', ' ORDER BY cr.role_name) as roles,
    cbu.created_at,
    cbu.updated_at
FROM client_business_units cbu
LEFT JOIN cbu_members cm ON cbu.id = cm.cbu_id AND cm.is_active = true
LEFT JOIN cbu_roles cr ON cm.role_id = cr.id
GROUP BY cbu.id;

-- CBU Members Detail View
CREATE VIEW v_cbu_members_detail AS
SELECT
    cm.id,
    cbu.cbu_id,
    cbu.cbu_name,
    cr.role_code,
    cr.role_name,
    cr.role_category,
    cm.entity_id,
    cm.entity_name,
    cm.entity_lei,
    cm.is_primary,
    cm.effective_date,
    cm.expiry_date,
    cm.contact_email,
    cm.is_active,
    cm.has_trading_authority,
    cm.has_settlement_authority,
    cm.notes,
    cm.created_at,
    cm.updated_at
FROM cbu_members cm
JOIN client_business_units cbu ON cm.cbu_id = cbu.id
JOIN cbu_roles cr ON cm.role_id = cr.id
ORDER BY cbu.cbu_name, cr.display_order, cm.entity_name;

-- Role Taxonomy View
CREATE VIEW v_cbu_roles_taxonomy AS
SELECT
    cr.id,
    cr.role_code,
    cr.role_name,
    cr.description,
    cr.role_category,
    cr.display_order,
    cr.is_active,
    COUNT(DISTINCT cm.cbu_id) as usage_count
FROM cbu_roles cr
LEFT JOIN cbu_members cm ON cr.id = cm.role_id AND cm.is_active = true
GROUP BY cr.id, cr.role_code, cr.role_name, cr.description, cr.role_category, cr.display_order, cr.is_active
ORDER BY cr.role_category, cr.display_order;

-- Grant permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON client_business_units TO data_designer_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON cbu_roles TO data_designer_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON cbu_members TO data_designer_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO data_designer_app;
GRANT SELECT ON v_cbu_summary TO data_designer_app;
GRANT SELECT ON v_cbu_members_detail TO data_designer_app;
GRANT SELECT ON v_cbu_roles_taxonomy TO data_designer_app;

-- Comments for documentation
COMMENT ON TABLE client_business_units IS 'Main table for Client Business Units - logical groupings of related entities';
COMMENT ON TABLE cbu_roles IS 'Role taxonomy for CBU members - defines available roles like Asset Owner, Investment Manager, etc.';
COMMENT ON TABLE cbu_members IS 'Links entities to CBUs with specific roles and relationship details';

COMMENT ON COLUMN client_business_units.cbu_id IS 'External identifier for the CBU, used in APIs and other systems';
COMMENT ON COLUMN client_business_units.primary_lei IS 'LEI of the primary/controlling entity in this CBU';
COMMENT ON COLUMN cbu_members.is_primary IS 'Indicates if this is the primary entity for this role within the CBU';
COMMENT ON COLUMN cbu_members.authorized_persons IS 'JSON array of authorized contact persons with names, emails, roles';