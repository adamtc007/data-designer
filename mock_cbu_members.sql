-- Mock CBU Members for Testing Multi-User Expansion Functionality
-- Creates realistic KYC/Trade Finance entity members for the empty CBUs

-- First update the CBU names to be more realistic
UPDATE client_business_units SET
    cbu_name = 'Global Trade Finance Consortium',
    description = 'Multi-jurisdiction trade finance and supply chain funding platform',
    business_type = 'Trade Finance',
    domicile_country = 'SG',
    regulatory_jurisdiction = 'Singapore MAS'
WHERE cbu_id = 'CBU-203914';

UPDATE client_business_units SET
    cbu_name = 'European Infrastructure Fund',
    description = 'Infrastructure development and project financing across EU',
    business_type = 'Infrastructure Finance',
    domicile_country = 'LU',
    regulatory_jurisdiction = 'Luxembourg CSSF'
WHERE cbu_id = 'CBU-942121';

UPDATE client_business_units SET
    cbu_name = 'Cross-Border Payments Network',
    description = 'Digital payments and remittance services for emerging markets',
    business_type = 'Payment Services',
    domicile_country = 'CH',
    regulatory_jurisdiction = 'Switzerland FINMA'
WHERE cbu_id = 'CBU-973130';

-- Insert mock members for CBU-203914 (Global Trade Finance Consortium)
INSERT INTO cbu_members (cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary, effective_date, contact_email, has_trading_authority, has_settlement_authority, notes) VALUES

-- Primary Asset Owner
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-203914'),
 (SELECT id FROM cbu_roles WHERE role_code = 'ASSET_OWNER'),
 'TF-SOVEREIGN-001',
 'Singapore Sovereign Wealth Fund',
 '529900T8BM49AURSDO61',
 true,
 '2025-01-15',
 'sovereign.fund@singapore.gov.sg',
 false,
 false,
 'Primary capital provider for trade finance consortium'),

-- Investment Manager
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-203914'),
 (SELECT id FROM cbu_roles WHERE role_code = 'INVESTMENT_MANAGER'),
 'TF-MANAGER-001',
 'Asian Trade Capital Management',
 '529900T8BM49AURSDO62',
 false,
 '2025-01-15',
 'portfolio@asiantradeCapital.com',
 true,
 false,
 'Specialized trade finance investment management'),

-- Custodian
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-203914'),
 (SELECT id FROM cbu_roles WHERE role_code = 'CUSTODIAN'),
 'TF-CUSTODY-001',
 'DBS Institutional Custody',
 '529900T8BM49AURSDO63',
 false,
 '2025-01-15',
 'institutional.custody@dbs.com',
 false,
 true,
 'Regional custody and settlement services'),

-- Administrator
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-203914'),
 (SELECT id FROM cbu_roles WHERE role_code = 'ADMINISTRATOR'),
 'TF-ADMIN-001',
 'Singapore Fund Services Pte Ltd',
 '529900T8BM49AURSDO64',
 false,
 '2025-01-15',
 'fund.admin@sgfundservices.com',
 false,
 false,
 'Fund administration and regulatory reporting');

-- Insert mock members for CBU-942121 (European Infrastructure Fund)
INSERT INTO cbu_members (cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary, effective_date, contact_email, has_trading_authority, has_settlement_authority, notes) VALUES

-- Primary Asset Owner
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-942121'),
 (SELECT id FROM cbu_roles WHERE role_code = 'ASSET_OWNER'),
 'IF-PENSION-001',
 'European Infrastructure Pension Scheme',
 '549300T8BM49AURSDO71',
 true,
 '2025-02-01',
 'investments@eu-pension-infra.eu',
 false,
 false,
 'Multi-national pension fund focused on infrastructure'),

-- Investment Manager
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-942121'),
 (SELECT id FROM cbu_roles WHERE role_code = 'INVESTMENT_MANAGER'),
 'IF-MANAGER-001',
 'Infrastructure Capital Partners SARL',
 '549300T8BM49AURSDO72',
 false,
 '2025-02-01',
 'portfolio@infracp.lu',
 true,
 false,
 'Specialized infrastructure equity and debt management'),

-- Custodian
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-942121'),
 (SELECT id FROM cbu_roles WHERE role_code = 'CUSTODIAN'),
 'IF-CUSTODY-001',
 'BNP Paribas Securities Services Luxembourg',
 '549300T8BM49AURSDO73',
 false,
 '2025-02-01',
 'institutional@bnpparibas.com',
 false,
 true,
 'Global custody with infrastructure expertise'),

-- Administrator
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-942121'),
 (SELECT id FROM cbu_roles WHERE role_code = 'ADMINISTRATOR'),
 'IF-ADMIN-001',
 'European Fund Administration S.A.',
 '549300T8BM49AURSDO74',
 false,
 '2025-02-01',
 'operations@eu-fund-admin.lu',
 false,
 false,
 'CSSF-regulated fund administration services'),

-- Auditor
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-942121'),
 (SELECT id FROM cbu_roles WHERE role_code = 'AUDITOR'),
 'IF-AUDIT-001',
 'KPMG Luxembourg S.A.',
 '549300T8BM49AURSDO75',
 false,
 '2025-02-01',
 'fund.audit@kpmg.lu',
 false,
 false,
 'Independent audit and assurance services');

-- Insert mock members for CBU-973130 (Cross-Border Payments Network)
INSERT INTO cbu_members (cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary, effective_date, contact_email, has_trading_authority, has_settlement_authority, notes) VALUES

-- Primary Asset Owner
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-973130'),
 (SELECT id FROM cbu_roles WHERE role_code = 'ASSET_OWNER'),
 'PY-FINTECH-001',
 'SwissPay Digital Holdings AG',
 '506700T8BM49AURSDO81',
 true,
 '2025-03-01',
 'institutional@swisspay.ch',
 false,
 false,
 'Digital payments technology and infrastructure provider'),

-- Payment Processor
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-973130'),
 (SELECT id FROM cbu_roles WHERE role_code = 'PROCESSOR'),
 'PY-PROCESSOR-001',
 'CrossBorder Solutions SA',
 '506700T8BM49AURSDO82',
 false,
 '2025-03-01',
 'operations@crossborder-solutions.ch',
 true,
 true,
 'Licensed payment processing and settlement'),

-- Bank Partner
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-973130'),
 (SELECT id FROM cbu_roles WHERE role_code = 'BANK_PARTNER'),
 'PY-BANK-001',
 'Credit Suisse International Banking',
 '506700T8BM49AURSDO83',
 false,
 '2025-03-01',
 'correspondent.banking@credit-suisse.com',
 false,
 true,
 'Correspondent banking and liquidity provision'),

-- Compliance Officer
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-973130'),
 (SELECT id FROM cbu_roles WHERE role_code = 'COMPLIANCE_OFFICER'),
 'PY-COMPLIANCE-001',
 'Swiss Regulatory Compliance GmbH',
 '506700T8BM49AURSDO84',
 false,
 '2025-03-01',
 'aml.officer@swiss-regcomp.ch',
 false,
 false,
 'AML/KYC compliance and regulatory monitoring'),

-- Technology Provider
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-973130'),
 (SELECT id FROM cbu_roles WHERE role_code = 'TECHNOLOGY_PROVIDER'),
 'PY-TECH-001',
 'Blockchain Infrastructure Services AG',
 '506700T8BM49AURSDO85',
 false,
 '2025-03-01',
 'platform@blockchain-infra.ch',
 false,
 false,
 'Distributed ledger technology and smart contracts');

-- Create some new role types that might be missing
INSERT INTO cbu_roles (role_code, role_name, description, role_category, display_order, is_active) VALUES
('PROCESSOR', 'Payment Processor', 'Licensed payment processing entity', 'operational', 15, true),
('BANK_PARTNER', 'Banking Partner', 'Correspondent or partner bank', 'operational', 16, true),
('COMPLIANCE_OFFICER', 'Compliance Officer', 'Regulatory compliance and monitoring', 'compliance', 17, true),
('TECHNOLOGY_PROVIDER', 'Technology Provider', 'Technology platform and infrastructure', 'operational', 18, true),
('AUDITOR', 'External Auditor', 'Independent audit and assurance services', 'compliance', 19, true)
ON CONFLICT (role_code) DO NOTHING;

-- Add some varied business relationships to show the complexity
-- CBU-203914 gets some international trade relationships
INSERT INTO cbu_members (cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary, effective_date, contact_email, has_trading_authority, has_settlement_authority, notes) VALUES
((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-203914'),
 (SELECT id FROM cbu_roles WHERE role_code = 'BANK_PARTNER'),
 'TF-CORRESPONDENT-001',
 'HSBC Trade & Receivables Finance',
 '529900T8BM49AURSDO65',
 false,
 '2025-01-15',
 'trade.finance@hsbc.com.sg',
 false,
 true,
 'Trade finance correspondent banking services'),

((SELECT id FROM client_business_units WHERE cbu_id = 'CBU-203914'),
 (SELECT id FROM cbu_roles WHERE role_code = 'TECHNOLOGY_PROVIDER'),
 'TF-PLATFORM-001',
 'TradeFinance.AI Pte Ltd',
 '529900T8BM49AURSDO66',
 false,
 '2025-01-15',
 'platform@tradefinance.ai',
 false,
 false,
 'AI-powered trade finance platform and automation');

-- Summary stats
DO $$
DECLARE
    cbu_count INTEGER;
    member_count INTEGER;
    role_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO cbu_count FROM client_business_units;
    SELECT COUNT(*) INTO member_count FROM cbu_members;
    SELECT COUNT(*) INTO role_count FROM cbu_roles WHERE is_active = true;

    RAISE NOTICE '✅ Mock data created successfully:';
    RAISE NOTICE '   📊 Total CBUs: %', cbu_count;
    RAISE NOTICE '   👥 Total Members: %', member_count;
    RAISE NOTICE '   🎭 Active Roles: %', role_count;
    RAISE NOTICE '';
    RAISE NOTICE '🔍 CBU Breakdown:';
END $$;

-- Show the new structure
SELECT
    cbu.cbu_id,
    cbu.cbu_name,
    COUNT(m.id) as member_count,
    string_agg(DISTINCT r.role_name, ', ' ORDER BY r.role_name) as roles
FROM client_business_units cbu
LEFT JOIN cbu_members m ON m.cbu_id = cbu.id
LEFT JOIN cbu_roles r ON r.id = m.role_id
GROUP BY cbu.id, cbu.cbu_id, cbu.cbu_name
ORDER BY cbu.cbu_id;