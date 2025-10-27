-- ====================================================================
-- Fix CBU Database Schema and Populate Data
-- ====================================================================
-- This script creates the views gRPC expects and populates dummy data

-- ====================================================================
-- PART 1: Create Views that gRPC Queries
-- ====================================================================

-- Drop existing views if they exist
DROP VIEW IF EXISTS cbu_investment_mandate_structure CASCADE;
DROP VIEW IF EXISTS cbu_member_investment_roles CASCADE;

-- View 1: CBU Investment Mandate Structure
-- Maps client_business_units to the format gRPC expects
CREATE OR REPLACE VIEW cbu_investment_mandate_structure AS
SELECT
    cbu.cbu_id,
    cbu.cbu_name,
    cbu.cbu_id AS mandate_id,  -- Using cbu_id as mandate_id for now
    (SELECT entity_name FROM cbu_members WHERE cbu_id = cbu.id AND is_primary = true LIMIT 1) AS asset_owner_name,
    (SELECT cm.entity_name FROM cbu_members cm
     JOIN cbu_roles cr ON cm.role_id = cr.id
     WHERE cm.cbu_id = cbu.id AND cr.role_code = 'INVMGR' LIMIT 1) AS investment_manager_name,
    COALESCE((cbu.metadata->>'base_currency')::varchar, 'USD') AS base_currency,
    COALESCE((cbu.metadata->>'total_instruments')::int, 0) AS total_instruments,
    COALESCE((cbu.metadata->>'families')::varchar, '') AS families,
    COALESCE((cbu.metadata->>'total_exposure_pct')::float, 0.0) AS total_exposure_pct
FROM client_business_units cbu
WHERE cbu.status = 'active';

-- View 2: CBU Member Investment Roles
-- Maps cbu_members with roles to the format gRPC expects
CREATE OR REPLACE VIEW cbu_member_investment_roles AS
SELECT
    cbu.cbu_id,
    cbu.cbu_name,
    cm.entity_name,
    cm.entity_lei,
    cr.role_name,
    cr.role_code,
    COALESCE((cm.metadata->>'investment_responsibility')::varchar, 'Not specified') AS investment_responsibility,
    cbu.cbu_id AS mandate_id,  -- Using cbu_id as mandate_id
    cm.has_trading_authority,
    cm.has_settlement_authority
FROM cbu_members cm
JOIN client_business_units cbu ON cm.cbu_id = cbu.id
JOIN cbu_roles cr ON cm.role_id = cr.id
WHERE cm.is_active = true AND cbu.status = 'active';

-- ====================================================================
-- PART 2: Populate Sample CBU Data
-- ====================================================================

-- Clear existing data (keeping roles!)
DELETE FROM cbu_members;
DELETE FROM client_business_units;

-- Insert sample CBUs (8 total to match CLAUDE.md: "100 entities + 8 CBUs")
INSERT INTO client_business_units (
    cbu_id, cbu_name, description,
    primary_entity_id, primary_lei,
    domicile_country, regulatory_jurisdiction,
    business_type, status, metadata
) VALUES
-- CBU 1: US Growth Fund
('CBU-001', 'US Growth Equity Fund Alpha',
 'Diversified growth-focused investment fund targeting mid-cap US equities with ESG screening',
 'ENT-US-001', '549300VPLTI2JI1A8N82',
 'US', 'SEC', 'Investment Fund', 'active',
 '{"base_currency": "USD", "total_instruments": 45, "families": "equity,derivatives", "total_exposure_pct": 98.5, "aum_millions": 2500, "dsl_version": "1.0"}'::jsonb),

-- CBU 2: European Infrastructure
('CBU-002', 'European Infrastructure Fund Beta',
 'Long-term infrastructure investment fund focusing on renewable energy and digital infrastructure',
 'ENT-EU-001', '529900T8BM49AURSDO55',
 'LU', 'CSSF', 'Infrastructure Fund', 'active',
 '{"base_currency": "EUR", "total_instruments": 28, "families": "bonds,equity", "total_exposure_pct": 95.2, "aum_millions": 1800, "dsl_version": "1.0"}'::jsonb),

-- CBU 3: APAC Trade Finance
('CBU-003', 'Asia-Pacific Trade Finance Consortium',
 'Trade finance and short-term credit facility for Asian supply chain financing',
 'ENT-AP-001', '353800MLJIGSLQ3JGP81',
 'SG', 'MAS', 'Trade Finance', 'active',
 '{"base_currency": "SGD", "total_instruments": 67, "families": "money_market,structured_products", "total_exposure_pct": 102.3, "aum_millions": 3200, "dsl_version": "1.1"}'::jsonb),

-- CBU 4: Global Pension Fund
('CBU-004', 'Global Multi-Asset Pension Scheme',
 'Diversified pension fund with liability-driven investment strategy and dynamic hedging',
 'ENT-UK-001', '213800ZZZZZZZZZZZ001',
 'GB', 'FCA', 'Pension Plan', 'active',
 '{"base_currency": "GBP", "total_instruments": 156, "families": "equity,bonds,alternatives,derivatives", "total_exposure_pct": 97.8, "aum_millions": 8500, "dsl_version": "2.0"}'::jsonb),

-- CBU 5: FinTech Payments
('CBU-005', 'Cross-Border Digital Payments Network',
 'Blockchain-enabled cross-border payment settlement and FX hedging platform',
 'ENT-CH-001', '506700T8BM49AURSDO81',
 'CH', 'FINMA', 'FinTech Platform', 'active',
 '{"base_currency": "CHF", "total_instruments": 23, "families": "fx,crypto,derivatives", "total_exposure_pct": 85.4, "aum_millions": 450, "dsl_version": "1.2"}'::jsonb),

-- CBU 6: Emerging Markets
('CBU-006', 'Emerging Markets Debt Fund',
 'High-yield sovereign and corporate debt from emerging markets with currency overlay',
 'ENT-US-002', '549300X3Q2MCQXP8W764',
 'US', 'SEC', 'Fixed Income Fund', 'active',
 '{"base_currency": "USD", "total_instruments": 89, "families": "bonds,fx,credit_derivatives", "total_exposure_pct": 103.7, "aum_millions": 1950, "dsl_version": "1.0"}'::jsonb),

-- CBU 7: Private Equity
('CBU-007', 'Nordic Private Equity Fund',
 'Growth capital and buyout investments in Nordic technology and healthcare sectors',
 'ENT-SE-001', '549300K6MM4TMZXFGN67',
 'SE', 'FSA', 'Private Equity', 'active',
 '{"base_currency": "SEK", "total_instruments": 12, "families": "equity,structured_notes", "total_exposure_pct": 78.9, "aum_millions": 625, "dsl_version": "1.1"}'::jsonb),

-- CBU 8: Commodity Fund
('CBU-008', 'Global Commodity Opportunities Fund',
 'Tactical commodity trading across energy, metals, and agriculture with systematic strategies',
 'ENT-SG-001', '300300S39XTBSNH66F17',
 'SG', 'MAS', 'Commodity Fund', 'active',
 '{"base_currency": "USD", "total_instruments": 134, "families": "futures,options,swaps", "total_exposure_pct": 112.5, "aum_millions": 1100, "dsl_version": "2.1"}'::jsonb);

-- Get role IDs for members
DO $$
DECLARE
    role_asset_owner INT;
    role_inv_mgr INT;
    role_custodian INT;
    role_admin INT;
    cbu1_id INT;
    cbu2_id INT;
    cbu3_id INT;
    cbu4_id INT;
    cbu5_id INT;
    cbu6_id INT;
    cbu7_id INT;
    cbu8_id INT;
BEGIN
    -- Get role IDs
    SELECT id INTO role_asset_owner FROM cbu_roles WHERE role_code = 'ASTOWN' LIMIT 1;
    SELECT id INTO role_inv_mgr FROM cbu_roles WHERE role_code = 'INVMGR' LIMIT 1;
    SELECT id INTO role_custodian FROM cbu_roles WHERE role_code = 'CUSTOD' LIMIT 1;
    SELECT id INTO role_admin FROM cbu_roles WHERE role_code = 'ADMIN' LIMIT 1;

    -- Get CBU IDs
    SELECT id INTO cbu1_id FROM client_business_units WHERE cbu_id = 'CBU-001';
    SELECT id INTO cbu2_id FROM client_business_units WHERE cbu_id = 'CBU-002';
    SELECT id INTO cbu3_id FROM client_business_units WHERE cbu_id = 'CBU-003';
    SELECT id INTO cbu4_id FROM client_business_units WHERE cbu_id = 'CBU-004';
    SELECT id INTO cbu5_id FROM client_business_units WHERE cbu_id = 'CBU-005';
    SELECT id INTO cbu6_id FROM client_business_units WHERE cbu_id = 'CBU-006';
    SELECT id INTO cbu7_id FROM client_business_units WHERE cbu_id = 'CBU-007';
    SELECT id INTO cbu8_id FROM client_business_units WHERE cbu_id = 'CBU-008';

    -- Insert members for CBU-001 (US Growth Fund)
    INSERT INTO cbu_members (cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary, has_trading_authority, has_settlement_authority, metadata)
    VALUES
    (cbu1_id, role_asset_owner, 'ENT-US-001', 'California Public Employees Retirement System', '549300VPLTI2JI1A8N82', true, false, false, '{"investment_responsibility": "Asset Allocation"}'::jsonb),
    (cbu1_id, role_inv_mgr, 'ENT-US-002', 'BlackRock Institutional Trust', '784F5XWPLTWKTBV3E584', false, true, false, '{"investment_responsibility": "Portfolio Management"}'::jsonb),
    (cbu1_id, role_custodian, 'ENT-US-003', 'State Street Global Services', '571474TGEMMWANRLN572', false, false, true, '{"investment_responsibility": "Asset Servicing"}'::jsonb);

    -- Insert members for CBU-002 (European Infrastructure)
    INSERT INTO cbu_members (cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary, has_trading_authority, has_settlement_authority, metadata)
    VALUES
    (cbu2_id, role_asset_owner, 'ENT-EU-001', 'ABP Pension Fund', '529900T8BM49AURSDO55', true, false, false, '{"investment_responsibility": "Strategic Planning"}'::jsonb),
    (cbu2_id, role_inv_mgr, 'ENT-EU-002', 'Deutsche Asset Management', '969500UP76J52A9OXU27', false, true, false, '{"investment_responsibility": "Infrastructure Selection"}'::jsonb);

    -- Insert members for CBU-003 (APAC Trade Finance)
    INSERT INTO cbu_members (cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary, has_trading_authority, has_settlement_authority, metadata)
    VALUES
    (cbu3_id, role_asset_owner, 'ENT-AP-001', 'Singapore Sovereign Wealth Fund', '353800MLJIGSLQ3JGP81', true, false, false, '{"investment_responsibility": "Mandate Approval"}'::jsonb),
    (cbu3_id, role_inv_mgr, 'ENT-AP-002', 'DBS Asset Management', '549300F4WH7V9NCKXX55', false, true, true, '{"investment_responsibility": "Trade Execution"}'::jsonb);

    -- Insert members for CBU-004 (Global Pension)
    INSERT INTO cbu_members (cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary, has_trading_authority, has_settlement_authority, metadata)
    VALUES
    (cbu4_id, role_asset_owner, 'ENT-UK-001', 'British Pension Protection Fund', '213800ZZZZZZZZZZZ001', true, false, false, '{"investment_responsibility": "Liability Matching"}'::jsonb),
    (cbu4_id, role_inv_mgr, 'ENT-UK-002', 'Schroders Investment Management', '549300ZC3AT2TL8WN979', false, true, false, '{"investment_responsibility": "Multi-Asset Management"}'::jsonb);

    -- Insert members for CBU-005 (FinTech Payments)
    INSERT INTO cbu_members (cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary, has_trading_authority, has_settlement_authority, metadata)
    VALUES
    (cbu5_id, role_asset_owner, 'ENT-CH-001', 'SwissPay Digital Holdings AG', '506700T8BM49AURSDO81', true, true, true, '{"investment_responsibility": "Platform Operations"}'::jsonb);

    -- Insert members for CBU-006 (Emerging Markets Debt)
    INSERT INTO cbu_members (cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary, has_trading_authority, has_settlement_authority, metadata)
    VALUES
    (cbu6_id, role_asset_owner, 'ENT-US-004', 'Vanguard Emerging Markets Trust', '549300X3Q2MCQXP8W764', true, false, false, '{"investment_responsibility": "Credit Analysis"}'::jsonb),
    (cbu6_id, role_inv_mgr, 'ENT-US-005', 'PIMCO LLC', '549300JLQ1R2XPCGJQ67', false, true, false, '{"investment_responsibility": "Fixed Income Trading"}'::jsonb);

    -- Insert members for CBU-007 (Nordic Private Equity)
    INSERT INTO cbu_members (cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary, has_trading_authority, has_settlement_authority, metadata)
    VALUES
    (cbu7_id, role_asset_owner, 'ENT-SE-001', 'Swedish National Pension Funds', '549300K6MM4TMZXFGN67', true, false, false, '{"investment_responsibility": "Deal Sourcing"}'::jsonb),
    (cbu7_id, role_inv_mgr, 'ENT-SE-002', 'Northzone Ventures', '549300NORDIC123456', false, true, false, '{"investment_responsibility": "Portfolio Company Management"}'::jsonb);

    -- Insert members for CBU-008 (Commodity Fund)
    INSERT INTO cbu_members (cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary, has_trading_authority, has_settlement_authority, metadata)
    VALUES
    (cbu8_id, role_asset_owner, 'ENT-SG-001', 'Singapore Commodity Trading Corp', '300300S39XTBSNH66F17', true, true, true, '{"investment_responsibility": "Systematic Trading"}'::jsonb),
    (cbu8_id, role_admin, 'ENT-SG-002', 'Asia Pacific Fund Services', '300300APFUNDS123456', false, false, false, '{"investment_responsibility": "NAV Calculation"}'::jsonb);

END $$;

-- ====================================================================
-- PART 3: Verify Data
-- ====================================================================

-- Show counts
SELECT 'CBUs Created' AS metric, COUNT(*) AS count FROM client_business_units
UNION ALL
SELECT 'Members Created' AS metric, COUNT(*) AS count FROM cbu_members
UNION ALL
SELECT 'Roles Available' AS metric, COUNT(*) AS count FROM cbu_roles;

-- Show view data
SELECT 'VIEW: cbu_investment_mandate_structure' AS info;
SELECT * FROM cbu_investment_mandate_structure LIMIT 3;

SELECT 'VIEW: cbu_member_investment_roles' AS info;
SELECT * FROM cbu_member_investment_roles LIMIT 5;

-- ====================================================================
-- PART 4: DSL Metadata Storage
-- ====================================================================
-- DSL code is stored in the metadata JSONB column of client_business_units
-- Example: metadata->>'dsl_code' or metadata->>'dsl_version'
-- The UI can add DSL by updating: metadata = metadata || '{"dsl_code": "..."}'::jsonb
