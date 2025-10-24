-- Entity and CBU Integration Schema
-- Creates proper entity structure for SPVs, limited companies, partnerships, portfolios, regulatory structures (e.g., SICAV)

-- Create comprehensive entity registry
CREATE TABLE IF NOT EXISTS legal_entities (
    id SERIAL PRIMARY KEY,
    entity_id VARCHAR(50) UNIQUE NOT NULL, -- Global unique identifier
    lei_code VARCHAR(20) UNIQUE, -- Legal Entity Identifier (ISO 17442)
    entity_name VARCHAR(500) NOT NULL,
    entity_type VARCHAR(50) NOT NULL, -- 'corporation', 'partnership', 'spv', 'sicav', 'fund', 'trust', etc.
    entity_subtype VARCHAR(100), -- 'limited_company', 'llp', 'special_purpose_vehicle', 'ucits', etc.
    legal_form VARCHAR(100), -- Specific legal form (e.g., 'LLC', 'S.A.', 'GmbH', 'Ltd', 'SICAV')

    -- Registration and Jurisdiction
    incorporation_country CHAR(2) NOT NULL, -- ISO 3166-1 alpha-2
    incorporation_jurisdiction VARCHAR(100), -- Specific jurisdiction/state
    registration_number VARCHAR(100), -- Company registration number
    registration_authority VARCHAR(200), -- Authority that registered the entity
    incorporation_date DATE,

    -- Regulatory Classification
    regulatory_structure VARCHAR(100), -- 'UCITS', 'AIFM', 'MiFID_entity', etc.
    regulatory_status VARCHAR(50) DEFAULT 'active', -- 'active', 'inactive', 'pending', 'dissolved'
    regulated_entity BOOLEAN DEFAULT false,
    regulatory_authorities TEXT[], -- Array of regulatory bodies

    -- Business Classification
    business_purpose TEXT,
    primary_business_activity VARCHAR(200),
    sic_codes VARCHAR(50)[], -- Standard Industrial Classification codes
    nace_codes VARCHAR(50)[], -- European classification codes

    -- Operational Details
    tax_residence_country CHAR(2),
    tax_identification_number VARCHAR(100),
    vat_number VARCHAR(50),
    operational_currency CHAR(3), -- ISO 4217
    fiscal_year_end DATE,

    -- Contact and Address
    registered_address JSONB, -- {street, city, state, postal_code, country}
    operational_address JSONB,
    mailing_address JSONB,

    -- Key Personnel
    authorized_signatories JSONB[], -- Array of authorized persons
    board_members JSONB[], -- Board/management information

    -- Risk and Compliance
    risk_rating VARCHAR(20) DEFAULT 'medium', -- 'low', 'medium', 'high', 'very_high'
    kyc_status VARCHAR(30) DEFAULT 'pending', -- 'pending', 'in_progress', 'approved', 'rejected'
    sanctions_screening_status VARCHAR(30) DEFAULT 'pending',
    sanctions_screening_date TIMESTAMP,
    sanctions_screening_result VARCHAR(20), -- 'clear', 'match', 'potential_match'

    -- Beneficial Ownership
    has_complex_ownership BOOLEAN DEFAULT false,
    ownership_transparency_score INTEGER CHECK (ownership_transparency_score >= 0 AND ownership_transparency_score <= 100),

    -- Financial Information
    share_capital DECIMAL(20,2),
    share_capital_currency CHAR(3),
    public_company BOOLEAN DEFAULT false,
    listed_exchange VARCHAR(100),

    -- Lifecycle
    status VARCHAR(30) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'dormant', 'dissolved', 'in_liquidation')),
    dissolution_date DATE,

    -- Metadata
    data_sources TEXT[], -- Where entity data comes from
    last_verified_date TIMESTAMP,
    verification_method VARCHAR(100),
    data_quality_score INTEGER CHECK (data_quality_score >= 0 AND data_quality_score <= 100),

    -- Audit
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100),
    updated_by VARCHAR(100)
);

-- Create entity relationship mapping (for complex structures)
CREATE TABLE IF NOT EXISTS entity_relationships (
    id SERIAL PRIMARY KEY,
    parent_entity_id INTEGER REFERENCES legal_entities(id) ON DELETE CASCADE,
    child_entity_id INTEGER REFERENCES legal_entities(id) ON DELETE CASCADE,
    relationship_type VARCHAR(50) NOT NULL, -- 'subsidiary', 'parent', 'affiliate', 'joint_venture', 'portfolio_company'
    ownership_percentage DECIMAL(5,2), -- 0.00 to 100.00
    control_type VARCHAR(50), -- 'direct', 'indirect', 'joint_control', 'no_control'
    effective_date DATE,
    end_date DATE,
    relationship_strength VARCHAR(20) DEFAULT 'medium', -- 'weak', 'medium', 'strong'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(parent_entity_id, child_entity_id, relationship_type)
);

-- Create CBU to Entity mapping (many-to-many relationship)
CREATE TABLE IF NOT EXISTS cbu_entity_associations (
    id SERIAL PRIMARY KEY,
    cbu_id INTEGER REFERENCES client_business_units(id) ON DELETE CASCADE,
    entity_id INTEGER REFERENCES legal_entities(id) ON DELETE CASCADE,
    association_type VARCHAR(50) NOT NULL, -- 'primary', 'subsidiary', 'affiliate', 'service_provider', 'counterparty'
    role_in_cbu VARCHAR(100), -- 'main_entity', 'holding_company', 'operating_company', 'spv', 'fund_entity'
    ownership_stake DECIMAL(5,2), -- Ownership percentage in this relationship
    voting_rights DECIMAL(5,2), -- Voting rights percentage
    control_level VARCHAR(30), -- 'full_control', 'majority_control', 'minority_interest', 'no_control'

    -- Operational Details
    active_in_cbu BOOLEAN DEFAULT true,
    primary_contact BOOLEAN DEFAULT false, -- Is this the primary entity for CBU communications
    service_types TEXT[], -- Types of services this entity provides to/receives from CBU

    -- Risk and Compliance
    risk_contribution_score INTEGER, -- How this entity affects CBU risk
    compliance_responsibility VARCHAR(100), -- What compliance obligations this entity has
    reporting_entity BOOLEAN DEFAULT false, -- Does this entity handle reporting for the CBU

    -- Temporal
    association_date DATE DEFAULT CURRENT_DATE,
    effective_from DATE,
    effective_to DATE,

    -- Audit
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100),
    updated_by VARCHAR(100),

    UNIQUE(cbu_id, entity_id, association_type)
);

-- Create entity attribute mappings to link our enhanced attributes to specific entities
CREATE TABLE IF NOT EXISTS entity_attribute_values (
    id SERIAL PRIMARY KEY,
    entity_id INTEGER REFERENCES legal_entities(id) ON DELETE CASCADE,
    attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    attribute_value JSONB NOT NULL,

    -- Value metadata
    value_source VARCHAR(100), -- 'user_input', 'api', 'document_extraction', 'calculation'
    confidence_score DECIMAL(3,2), -- 0.00 to 1.00
    verification_status VARCHAR(30) DEFAULT 'unverified', -- 'unverified', 'verified', 'disputed', 'outdated'
    verification_method VARCHAR(100),
    verified_by VARCHAR(100),
    verified_at TIMESTAMP,

    -- Temporal validity
    effective_from TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    effective_to TIMESTAMP,
    is_current BOOLEAN DEFAULT true,

    -- Audit trail
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100),
    updated_by VARCHAR(100),

    UNIQUE(entity_id, attribute_id, effective_from)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_legal_entities_lei ON legal_entities(lei_code);
CREATE INDEX IF NOT EXISTS idx_legal_entities_country ON legal_entities(incorporation_country);
CREATE INDEX IF NOT EXISTS idx_legal_entities_type ON legal_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_legal_entities_status ON legal_entities(status);
CREATE INDEX IF NOT EXISTS idx_legal_entities_risk ON legal_entities(risk_rating);
CREATE INDEX IF NOT EXISTS idx_legal_entities_kyc ON legal_entities(kyc_status);
CREATE INDEX IF NOT EXISTS idx_legal_entities_reg_num ON legal_entities(registration_number);

CREATE INDEX IF NOT EXISTS idx_entity_relationships_parent ON entity_relationships(parent_entity_id);
CREATE INDEX IF NOT EXISTS idx_entity_relationships_child ON entity_relationships(child_entity_id);
CREATE INDEX IF NOT EXISTS idx_entity_relationships_type ON entity_relationships(relationship_type);

CREATE INDEX IF NOT EXISTS idx_cbu_entity_cbu ON cbu_entity_associations(cbu_id);
CREATE INDEX IF NOT EXISTS idx_cbu_entity_entity ON cbu_entity_associations(entity_id);
CREATE INDEX IF NOT EXISTS idx_cbu_entity_type ON cbu_entity_associations(association_type);
CREATE INDEX IF NOT EXISTS idx_cbu_entity_active ON cbu_entity_associations(active_in_cbu);

CREATE INDEX IF NOT EXISTS idx_entity_attr_values_entity ON entity_attribute_values(entity_id);
CREATE INDEX IF NOT EXISTS idx_entity_attr_values_attr ON entity_attribute_values(attribute_id);
CREATE INDEX IF NOT EXISTS idx_entity_attr_values_current ON entity_attribute_values(is_current);

-- Create comprehensive view for CBU entity structure
CREATE OR REPLACE VIEW cbu_entity_structure_view AS
SELECT
    cbu.id as cbu_internal_id,
    cbu.cbu_id,
    cbu.cbu_name,
    cbu.description as cbu_description,
    cbu.business_type as cbu_business_type,
    cbu.status as cbu_status,

    -- Primary Entity Information
    pe.entity_id as primary_entity_id,
    pe.entity_name as primary_entity_name,
    pe.entity_type as primary_entity_type,
    pe.legal_form as primary_legal_form,
    pe.incorporation_country as primary_country,
    pe.lei_code as primary_lei,

    -- All Associated Entities
    jsonb_agg(
        DISTINCT jsonb_build_object(
            'entity_id', le.entity_id,
            'entity_name', le.entity_name,
            'entity_type', le.entity_type,
            'legal_form', le.legal_form,
            'association_type', cea.association_type,
            'role_in_cbu', cea.role_in_cbu,
            'ownership_stake', cea.ownership_stake,
            'control_level', cea.control_level,
            'active', cea.active_in_cbu,
            'primary_contact', cea.primary_contact,
            'lei_code', le.lei_code,
            'incorporation_country', le.incorporation_country,
            'risk_rating', le.risk_rating,
            'kyc_status', le.kyc_status
        )
    ) FILTER (WHERE le.id IS NOT NULL) as associated_entities,

    -- Summary Statistics
    COUNT(DISTINCT lea.id) as total_entities,
    COUNT(DISTINCT CASE WHEN cea.active_in_cbu THEN lea.id END) as active_entities,
    COUNT(DISTINCT CASE WHEN le.entity_type = 'spv' THEN lea.id END) as spv_count,
    COUNT(DISTINCT CASE WHEN le.entity_type = 'corporation' THEN lea.id END) as corporation_count,
    COUNT(DISTINCT CASE WHEN le.entity_type = 'partnership' THEN lea.id END) as partnership_count,
    COUNT(DISTINCT CASE WHEN le.regulatory_structure IS NOT NULL THEN lea.id END) as regulated_entities,

    -- Risk Summary
    AVG(le.risk_rating::text::int) FILTER (WHERE le.risk_rating ~ '^[0-9]+$') as avg_entity_risk,
    array_agg(DISTINCT le.incorporation_country) FILTER (WHERE le.incorporation_country IS NOT NULL) as jurisdictions,

    -- Metadata
    cbu.created_at as cbu_created_at,
    cbu.updated_at as cbu_updated_at

FROM client_business_units cbu
LEFT JOIN legal_entities pe ON pe.entity_id = cbu.primary_entity_id
LEFT JOIN cbu_entity_associations cea ON cea.cbu_id = cbu.id
LEFT JOIN legal_entities lea ON lea.id = cea.entity_id
LEFT JOIN legal_entities le ON le.id = cea.entity_id
GROUP BY
    cbu.id, cbu.cbu_id, cbu.cbu_name, cbu.description, cbu.business_type, cbu.status,
    pe.entity_id, pe.entity_name, pe.entity_type, pe.legal_form, pe.incorporation_country, pe.lei_code,
    cbu.created_at, cbu.updated_at;

-- Function to get entity hierarchy for a CBU
CREATE OR REPLACE FUNCTION get_cbu_entity_hierarchy(input_cbu_id VARCHAR)
RETURNS TABLE (
    level INTEGER,
    entity_id VARCHAR,
    entity_name VARCHAR,
    entity_type VARCHAR,
    parent_entity_id VARCHAR,
    relationship_type VARCHAR,
    ownership_percentage DECIMAL,
    path TEXT
) AS $$
WITH RECURSIVE entity_hierarchy AS (
    -- Base case: entities directly associated with CBU
    SELECT
        1 as level,
        le.entity_id,
        le.entity_name,
        le.entity_type,
        NULL::VARCHAR as parent_entity_id,
        cea.association_type as relationship_type,
        cea.ownership_stake as ownership_percentage,
        le.entity_name as path
    FROM cbu_entity_associations cea
    JOIN legal_entities le ON le.id = cea.entity_id
    JOIN client_business_units cbu ON cbu.id = cea.cbu_id
    WHERE cbu.cbu_id = input_cbu_id

    UNION ALL

    -- Recursive case: child entities
    SELECT
        eh.level + 1,
        child.entity_id,
        child.entity_name,
        child.entity_type,
        parent.entity_id as parent_entity_id,
        er.relationship_type,
        er.ownership_percentage,
        eh.path || ' -> ' || child.entity_name as path
    FROM entity_hierarchy eh
    JOIN legal_entities parent ON parent.entity_id = eh.entity_id
    JOIN entity_relationships er ON er.parent_entity_id = parent.id
    JOIN legal_entities child ON child.id = er.child_entity_id
    WHERE eh.level < 10 -- Prevent infinite recursion
)
SELECT * FROM entity_hierarchy
ORDER BY level, entity_name;
$$ LANGUAGE SQL;

-- Insert sample entity types and regulatory structures
INSERT INTO legal_entities (
    entity_id, entity_name, entity_type, entity_subtype, legal_form,
    incorporation_country, regulatory_structure, business_purpose,
    created_by
) VALUES
    ('ENT001', 'Global Investment Holdings Ltd', 'corporation', 'holding_company', 'Limited Company', 'GB', NULL, 'Investment holding company', 'system'),
    ('ENT002', 'Trade Finance SPV Alpha', 'spv', 'special_purpose_vehicle', 'Limited Company', 'LU', NULL, 'Trade finance securitization vehicle', 'system'),
    ('ENT003', 'European Equity Fund SICAV', 'fund', 'ucits_fund', 'SICAV', 'LU', 'UCITS', 'European equity investment fund', 'system'),
    ('ENT004', 'Strategic Partnership LLP', 'partnership', 'limited_liability_partnership', 'LLP', 'GB', NULL, 'Strategic business partnership', 'system'),
    ('ENT005', 'Portfolio Management Services SA', 'corporation', 'service_company', 'S.A.', 'CH', 'MiFID_entity', 'Portfolio management and advisory services', 'system')
ON CONFLICT (entity_id) DO NOTHING;

-- Link sample entities to a CBU
DO $$
DECLARE
    sample_cbu_id INTEGER;
BEGIN
    -- Get or create a sample CBU
    SELECT id INTO sample_cbu_id FROM client_business_units LIMIT 1;

    IF sample_cbu_id IS NOT NULL THEN
        -- Associate entities with the CBU
        INSERT INTO cbu_entity_associations (cbu_id, entity_id, association_type, role_in_cbu, ownership_stake, active_in_cbu)
        SELECT
            sample_cbu_id,
            le.id,
            CASE
                WHEN le.entity_type = 'corporation' AND le.entity_subtype = 'holding_company' THEN 'primary'
                WHEN le.entity_type = 'spv' THEN 'subsidiary'
                WHEN le.entity_type = 'fund' THEN 'affiliate'
                WHEN le.entity_type = 'partnership' THEN 'affiliate'
                ELSE 'subsidiary'
            END,
            CASE
                WHEN le.entity_type = 'corporation' AND le.entity_subtype = 'holding_company' THEN 'main_entity'
                WHEN le.entity_type = 'spv' THEN 'spv'
                WHEN le.entity_type = 'fund' THEN 'fund_entity'
                ELSE 'operating_company'
            END,
            CASE
                WHEN le.entity_type = 'corporation' AND le.entity_subtype = 'holding_company' THEN 100.00
                WHEN le.entity_type = 'spv' THEN 100.00
                WHEN le.entity_type = 'fund' THEN 0.00
                ELSE 50.00
            END,
            true
        FROM legal_entities le
        WHERE le.entity_id IN ('ENT001', 'ENT002', 'ENT003', 'ENT004', 'ENT005')
        ON CONFLICT (cbu_id, entity_id, association_type) DO NOTHING;

        RAISE NOTICE 'Associated sample entities with CBU ID: %', sample_cbu_id;
    ELSE
        RAISE NOTICE 'No CBU found to associate entities with';
    END IF;
END $$;

-- Comments for documentation
COMMENT ON TABLE legal_entities IS 'Comprehensive registry of all legal entities including corporations, SPVs, partnerships, funds, and regulatory structures';
COMMENT ON TABLE entity_relationships IS 'Parent-child and ownership relationships between entities';
COMMENT ON TABLE cbu_entity_associations IS 'Many-to-many mapping between CBUs and their associated entities';
COMMENT ON TABLE entity_attribute_values IS 'Values of enhanced attributes for specific entities with temporal validity';
COMMENT ON VIEW cbu_entity_structure_view IS 'Comprehensive view of CBU structure with all associated entities and metadata';
COMMENT ON FUNCTION get_cbu_entity_hierarchy IS 'Recursive function to retrieve complete entity hierarchy for a CBU';