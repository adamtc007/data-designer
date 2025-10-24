-- Sample data for CBU DSL testing using existing schema
-- This script populates the existing tables with realistic financial services data

-- Clear existing associations and CBU data (keep legal entities if they exist)
DELETE FROM cbu_entity_associations;
DELETE FROM client_business_units;

-- Insert sample legal entities (client entities)
INSERT INTO legal_entities (
    entity_id, lei_code, entity_name, entity_type, entity_subtype,
    incorporation_country, incorporation_jurisdiction, regulatory_status,
    status, created_at, updated_at
) VALUES
-- US Entities
('US001', '549300VPLTI2JI1A8N82', 'Manhattan Asset Management LLC', 'Investment Manager', 'Asset Management Company', 'US', 'Delaware', 'active', 'active', NOW(), NOW()),
('US002', '784F5XWPLTWKTBV3E584', 'Goldman Sachs Asset Management', 'Investment Manager', 'Asset Management Company', 'US', 'New York', 'active', 'active', NOW(), NOW()),
('US003', '549300WOTC9L6FP6DY29', 'BlackRock Institutional Trust', 'Asset Owner', 'Institutional Investor', 'US', 'Delaware', 'active', 'active', NOW(), NOW()),
('US004', '571474TGEMMWANRLN572', 'State Street Global Services', 'Service Provider', 'Fund Administrator', 'US', 'Massachusetts', 'active', 'active', NOW(), NOW()),
('US005', 'CAL001PERS2023456789', 'California Public Employees Retirement System', 'Asset Owner', 'Public Pension Fund', 'US', 'California', 'active', 'active', NOW(), NOW()),
('US006', '549300X3Q2MCQXP8W764', 'Vanguard Asset Management', 'Investment Manager', 'Asset Management Company', 'US', 'Pennsylvania', 'active', 'active', NOW(), NOW()),

-- European Entities
('EU001', '529900T8BM49AURSDO55', 'Deutsche Asset Management', 'Investment Manager', 'Asset Management Company', 'DE', 'Germany', 'active', 'active', NOW(), NOW()),
('EU002', '969500UP76J52A9OXU27', 'BNP Paribas Asset Management', 'Investment Manager', 'Asset Management Company', 'FR', 'France', 'active', 'active', NOW(), NOW()),
('EU003', '549300ZZK73H1MR76N74', 'UBS Asset Management AG', 'Investment Manager', 'Asset Management Company', 'CH', 'Switzerland', 'active', 'active', NOW(), NOW()),
('EU004', 'NL001ABP2023456789XY', 'ABP Pension Fund', 'Asset Owner', 'Pension Fund', 'NL', 'Netherlands', 'active', 'active', NOW(), NOW()),
('EU005', '549300K6MM4TMZXFGN67', 'Nordea Asset Management', 'Investment Manager', 'Asset Management Company', 'SE', 'Sweden', 'active', 'active', NOW(), NOW()),

-- APAC Entities
('AP001', '353800MLJIGSLQ3JGP81', 'Nomura Asset Management', 'Investment Manager', 'Asset Management Company', 'JP', 'Japan', 'active', 'active', NOW(), NOW()),
('AP002', '300300S39XTBSNH66F17', 'China Asset Management Co', 'Investment Manager', 'Asset Management Company', 'CN', 'China', 'active', 'active', NOW(), NOW()),
('AP003', '549300F4WH7V9NCKXX55', 'DBS Asset Management', 'Investment Manager', 'Asset Management Company', 'SG', 'Singapore', 'active', 'active', NOW(), NOW()),
('AP004', 'AU001SUPER2023456789', 'Australia Super Fund', 'Asset Owner', 'Superannuation Fund', 'AU', 'Australia', 'active', 'active', NOW(), NOW()),

-- Managing Companies
('MC001', 'US001PINNACLE567890A', 'Pinnacle Fund Services LLC', 'Service Provider', 'Fund Administrator', 'US', 'Delaware', 'active', 'active', NOW(), NOW()),
('MC002', 'LU001EFA2023456789XY', 'European Fund Administration SA', 'Service Provider', 'Fund Administrator', 'LU', 'Luxembourg', 'active', 'active', NOW(), NOW()),
('MC003', 'SG001APFS23456789XYZ', 'Asia Pacific Fund Services Pte', 'Service Provider', 'Fund Administrator', 'SG', 'Singapore', 'active', 'active', NOW(), NOW())

ON CONFLICT (entity_id) DO UPDATE SET
    entity_name = EXCLUDED.entity_name,
    entity_type = EXCLUDED.entity_type,
    updated_at = NOW();

-- Insert sample CBUs
INSERT INTO client_business_units (
    cbu_id, cbu_name, description, domicile_country,
    regulatory_jurisdiction, business_type, status, created_at, updated_at
) VALUES
('CBU0000001', 'Growth Equity Fund Alpha', 'A diversified growth-focused investment fund targeting mid-cap US equities', 'US', 'Delaware', 'Investment Fund', 'active', NOW(), NOW()),
('CBU0000002', 'European Pension Fund Beta', 'European pension fund with multi-asset strategy focusing on long-term retirement benefits', 'NL', 'Netherlands', 'Pension Plan', 'active', NOW(), NOW()),
('CBU0000003', 'Asia Pacific Infrastructure Fund', 'Infrastructure investment fund targeting developing markets in Asia Pacific region', 'SG', 'Singapore', 'Infrastructure Fund', 'active', NOW(), NOW())

ON CONFLICT (cbu_id) DO UPDATE SET
    cbu_name = EXCLUDED.cbu_name,
    description = EXCLUDED.description,
    updated_at = NOW();

-- Insert CBU entity associations
-- First, we need to get the entity IDs from legal_entities and CBU IDs from client_business_units

-- Growth Equity Fund Alpha associations
INSERT INTO cbu_entity_associations (
    cbu_id, entity_id, association_type, role_in_cbu, active_in_cbu, created_at, updated_at
)
SELECT
    cbu.id as cbu_id,
    le.id as entity_id,
    'ownership' as association_type,
    'Asset Owner' as role_in_cbu,
    true as active_in_cbu,
    NOW() as created_at,
    NOW() as updated_at
FROM client_business_units cbu, legal_entities le
WHERE cbu.cbu_id = 'CBU0000001' AND le.entity_id = 'US003'

UNION ALL

SELECT
    cbu.id as cbu_id,
    le.id as entity_id,
    'management' as association_type,
    'Investment Manager' as role_in_cbu,
    true as active_in_cbu,
    NOW() as created_at,
    NOW() as updated_at
FROM client_business_units cbu, legal_entities le
WHERE cbu.cbu_id = 'CBU0000001' AND le.entity_id = 'US002'

UNION ALL

SELECT
    cbu.id as cbu_id,
    le.id as entity_id,
    'administration' as association_type,
    'Managing Company' as role_in_cbu,
    true as active_in_cbu,
    NOW() as created_at,
    NOW() as updated_at
FROM client_business_units cbu, legal_entities le
WHERE cbu.cbu_id = 'CBU0000001' AND le.entity_id = 'MC001'

UNION ALL

-- European Pension Fund Beta associations
SELECT
    cbu.id as cbu_id,
    le.id as entity_id,
    'ownership' as association_type,
    'Asset Owner' as role_in_cbu,
    true as active_in_cbu,
    NOW() as created_at,
    NOW() as updated_at
FROM client_business_units cbu, legal_entities le
WHERE cbu.cbu_id = 'CBU0000002' AND le.entity_id = 'EU004'

UNION ALL

SELECT
    cbu.id as cbu_id,
    le.id as entity_id,
    'management' as association_type,
    'Investment Manager' as role_in_cbu,
    true as active_in_cbu,
    NOW() as created_at,
    NOW() as updated_at
FROM client_business_units cbu, legal_entities le
WHERE cbu.cbu_id = 'CBU0000002' AND le.entity_id = 'EU001'

UNION ALL

SELECT
    cbu.id as cbu_id,
    le.id as entity_id,
    'administration' as association_type,
    'Managing Company' as role_in_cbu,
    true as active_in_cbu,
    NOW() as created_at,
    NOW() as updated_at
FROM client_business_units cbu, legal_entities le
WHERE cbu.cbu_id = 'CBU0000002' AND le.entity_id = 'MC002'

UNION ALL

-- Asia Pacific Infrastructure Fund associations
SELECT
    cbu.id as cbu_id,
    le.id as entity_id,
    'ownership' as association_type,
    'Asset Owner' as role_in_cbu,
    true as active_in_cbu,
    NOW() as created_at,
    NOW() as updated_at
FROM client_business_units cbu, legal_entities le
WHERE cbu.cbu_id = 'CBU0000003' AND le.entity_id = 'AP004'

UNION ALL

SELECT
    cbu.id as cbu_id,
    le.id as entity_id,
    'management' as association_type,
    'Investment Manager' as role_in_cbu,
    true as active_in_cbu,
    NOW() as created_at,
    NOW() as updated_at
FROM client_business_units cbu, legal_entities le
WHERE cbu.cbu_id = 'CBU0000003' AND le.entity_id = 'AP003'

UNION ALL

SELECT
    cbu.id as cbu_id,
    le.id as entity_id,
    'administration' as association_type,
    'Managing Company' as role_in_cbu,
    true as active_in_cbu,
    NOW() as created_at,
    NOW() as updated_at
FROM client_business_units cbu, legal_entities le
WHERE cbu.cbu_id = 'CBU0000003' AND le.entity_id = 'MC003'

ON CONFLICT (cbu_id, entity_id, association_type) DO UPDATE SET
    role_in_cbu = EXCLUDED.role_in_cbu,
    updated_at = NOW();

-- Verify the data
SELECT 'Legal Entities Count:' as info, COUNT(*) as count FROM legal_entities
UNION ALL
SELECT 'CBU Count:' as info, COUNT(*) as count FROM client_business_units
UNION ALL
SELECT 'CBU Associations Count:' as info, COUNT(*) as count FROM cbu_entity_associations;

-- Show sample CBU with relationships
SELECT
    c.cbu_id,
    c.cbu_name,
    c.business_type,
    le.entity_name,
    cea.role_in_cbu,
    cea.association_type
FROM client_business_units c
JOIN cbu_entity_associations cea ON c.id = cea.cbu_id
JOIN legal_entities le ON cea.entity_id = le.id
WHERE cea.active_in_cbu = true
ORDER BY c.cbu_id, cea.role_in_cbu;