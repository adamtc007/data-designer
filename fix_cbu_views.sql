-- Fix CBU Database - Create Missing Tables and Views
-- =====================================================
-- This script creates the views and tables that gRPC expects

-- 1. Create 'cbu' view mapping to client_business_units
DROP VIEW IF EXISTS cbu CASCADE;
CREATE OR REPLACE VIEW cbu AS
SELECT
    id,
    cbu_id,
    cbu_name,
    description,
    -- Map to expected column names for gRPC compatibility
    (SELECT entity_name FROM cbu_members WHERE cbu_id = cbu.id AND is_primary = true LIMIT 1) AS legal_entity_name,
    regulatory_jurisdiction AS jurisdiction,
    business_type AS business_model,
    status,
    primary_entity_id,
    primary_lei,
    domicile_country,
    metadata,
    created_at,
    updated_at
FROM client_business_units cbu;

-- 2. Create 'legal_entities' table
-- First check if it exists
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'legal_entities') THEN
        CREATE TABLE legal_entities (
            id SERIAL PRIMARY KEY,
            entity_id VARCHAR(50) UNIQUE NOT NULL,
            entity_name VARCHAR(255) NOT NULL,
            entity_type VARCHAR(100),
            incorporation_jurisdiction VARCHAR(100),
            incorporation_country VARCHAR(2),
            lei_code VARCHAR(20),
            status VARCHAR(50) DEFAULT 'active',
            metadata JSONB DEFAULT '{}'::jsonb,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- Create indexes
        CREATE INDEX idx_legal_entities_status ON legal_entities(status);
        CREATE INDEX idx_legal_entities_lei ON legal_entities(lei_code);
        CREATE INDEX idx_legal_entities_entity_id ON legal_entities(entity_id);
    END IF;
END $$;

-- 3. Populate legal_entities from cbu_members (extract unique entities)
INSERT INTO legal_entities (entity_id, entity_name, entity_type, lei_code, status)
SELECT DISTINCT
    cm.entity_id,
    cm.entity_name,
    cr.role_name AS entity_type,
    cm.entity_lei AS lei_code,
    CASE WHEN cm.is_active THEN 'active' ELSE 'inactive' END AS status
FROM cbu_members cm
JOIN cbu_roles cr ON cm.role_id = cr.id
ON CONFLICT (entity_id) DO NOTHING;

-- 4. Verify the fix
SELECT 'Tables and Views Created:' AS status;
SELECT COUNT(*) AS cbu_count FROM cbu;
SELECT COUNT(*) AS legal_entities_count FROM legal_entities;
SELECT COUNT(*) AS cbu_investment_mandate_structure_count FROM cbu_investment_mandate_structure;
SELECT COUNT(*) AS cbu_member_investment_roles_count FROM cbu_member_investment_roles;

-- Show sample data
SELECT '--- Sample CBUs ---' AS info;
SELECT cbu_id, cbu_name, status FROM cbu LIMIT 3;

SELECT '--- Sample Legal Entities ---' AS info;
SELECT entity_id, entity_name, entity_type, lei_code FROM legal_entities LIMIT 5;
