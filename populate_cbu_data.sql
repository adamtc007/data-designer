-- Sample data for CBU DSL testing
-- This script populates client_entities and cbu tables with realistic financial services data

-- Clear existing data
DELETE FROM cbu_entity_relationships;
DELETE FROM cbu;
DELETE FROM client_entities;

-- Insert client entities (the entities that can be used in CBUs)
INSERT INTO client_entities (entity_id, entity_name, entity_type, jurisdiction, country_code, lei_code, status, created_at, updated_at) VALUES
-- US Entities
('US001', 'Manhattan Asset Management LLC', 'Investment Manager', 'Delaware', 'US', '549300VPLTI2JI1A8N82', 'active', NOW(), NOW()),
('US002', 'Goldman Sachs Asset Management', 'Investment Manager', 'New York', 'US', '784F5XWPLTWKTBV3E584', 'active', NOW(), NOW()),
('US003', 'BlackRock Institutional Trust', 'Asset Owner', 'Delaware', 'US', '549300WOTC9L6FP6DY29', 'active', NOW(), NOW()),
('US004', 'State Street Global Services', 'Service Provider', 'Massachusetts', 'US', '571474TGEMMWANRLN572', 'active', NOW(), NOW()),
('US005', 'California Public Employees Retirement System', 'Asset Owner', 'California', 'US', 'CAL001PERS2023456789', 'active', NOW(), NOW()),
('US006', 'Vanguard Asset Management', 'Investment Manager', 'Pennsylvania', 'US', '549300X3Q2MCQXP8W764', 'active', NOW(), NOW()),

-- European Entities
('EU001', 'Deutsche Asset Management', 'Investment Manager', 'Germany', 'DE', '529900T8BM49AURSDO55', 'active', NOW(), NOW()),
('EU002', 'BNP Paribas Asset Management', 'Investment Manager', 'France', 'FR', '969500UP76J52A9OXU27', 'active', NOW(), NOW()),
('EU003', 'UBS Asset Management AG', 'Investment Manager', 'Switzerland', 'CH', '549300ZZK73H1MR76N74', 'active', NOW(), NOW()),
('EU004', 'ABP Pension Fund', 'Asset Owner', 'Netherlands', 'NL', 'NL001ABP2023456789XY', 'active', NOW(), NOW()),
('EU005', 'Nordea Asset Management', 'Investment Manager', 'Sweden', 'SE', '549300K6MM4TMZXFGN67', 'active', NOW(), NOW()),

-- APAC Entities
('AP001', 'Nomura Asset Management', 'Investment Manager', 'Japan', 'JP', '353800MLJIGSLQ3JGP81', 'active', NOW(), NOW()),
('AP002', 'China Asset Management Co', 'Investment Manager', 'China', 'CN', '300300S39XTBSNH66F17', 'active', NOW(), NOW()),
('AP003', 'DBS Asset Management', 'Investment Manager', 'Singapore', 'SG', '549300F4WH7V9NCKXX55', 'active', NOW(), NOW()),
('AP004', 'Australia Super Fund', 'Asset Owner', 'Australia', 'AU', 'AU001SUPER2023456789', 'active', NOW(), NOW()),

-- Managing Companies
('MC001', 'Pinnacle Fund Services LLC', 'Managing Company', 'Delaware', 'US', 'US001PINNACLE567890A', 'active', NOW(), NOW()),
('MC002', 'European Fund Administration SA', 'Managing Company', 'Luxembourg', 'LU', 'LU001EFA2023456789XY', 'active', NOW(), NOW()),
('MC003', 'Asia Pacific Fund Services Pte', 'Managing Company', 'Singapore', 'SG', 'SG001APFS23456789XYZ', 'active', NOW(), NOW());

-- Insert sample CBUs
INSERT INTO cbu (cbu_id, cbu_name, description, legal_entity_name, jurisdiction, business_model, status, created_at, updated_at) VALUES
('CBU0000001', 'Growth Equity Fund Alpha', 'A diversified growth-focused investment fund targeting mid-cap US equities', 'Growth Equity Fund Alpha LLC', 'Delaware', 'Investment Fund', 'active', NOW(), NOW()),
('CBU0000002', 'European Pension Fund Beta', 'European pension fund with multi-asset strategy focusing on long-term retirement benefits', 'European Pension Fund Beta Foundation', 'Netherlands', 'Pension Plan', 'active', NOW(), NOW()),
('CBU0000003', 'Asia Pacific Infrastructure Fund', 'Infrastructure investment fund targeting developing markets in Asia Pacific region', 'APAC Infrastructure Fund Ltd', 'Singapore', 'Infrastructure Fund', 'active', NOW(), NOW());

-- Insert CBU entity relationships
INSERT INTO cbu_entity_relationships (cbu_id, entity_id, entity_name, role_name, created_at) VALUES
-- Growth Equity Fund Alpha relationships
('CBU0000001', 'US003', 'BlackRock Institutional Trust', 'Asset Owner', NOW()),
('CBU0000001', 'US002', 'Goldman Sachs Asset Management', 'Investment Manager', NOW()),
('CBU0000001', 'MC001', 'Pinnacle Fund Services LLC', 'Managing Company', NOW()),

-- European Pension Fund Beta relationships
('CBU0000002', 'EU004', 'ABP Pension Fund', 'Asset Owner', NOW()),
('CBU0000002', 'EU001', 'Deutsche Asset Management', 'Investment Manager', NOW()),
('CBU0000002', 'MC002', 'European Fund Administration SA', 'Managing Company', NOW()),

-- Asia Pacific Infrastructure Fund relationships
('CBU0000003', 'AP004', 'Australia Super Fund', 'Asset Owner', NOW()),
('CBU0000003', 'AP003', 'DBS Asset Management', 'Investment Manager', NOW()),
('CBU0000003', 'MC003', 'Asia Pacific Fund Services Pte', 'Managing Company', NOW());

-- Verify the data
SELECT 'Client Entities Count:' as info, COUNT(*) as count FROM client_entities
UNION ALL
SELECT 'CBU Count:' as info, COUNT(*) as count FROM cbu
UNION ALL
SELECT 'CBU Relationships Count:' as info, COUNT(*) as count FROM cbu_entity_relationships;

-- Show sample CBU with relationships
SELECT
    c.cbu_id,
    c.cbu_name,
    c.business_model,
    cer.entity_name,
    cer.role_name
FROM cbu c
JOIN cbu_entity_relationships cer ON c.cbu_id = cer.cbu_id
ORDER BY c.cbu_id, cer.role_name;