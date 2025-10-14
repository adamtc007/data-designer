-- Populate Investment Mandates with CBU Role Linkages
-- Uses existing sophisticated schema with role-based member assignments

-- Insert investment mandates for our CBUs with proper role assignments
INSERT INTO investment_mandates (
    mandate_id, cbu_id,
    asset_owner_name, asset_owner_lei,
    investment_manager_name, investment_manager_lei,
    base_currency, effective_date, expiry_date,
    gross_exposure_pct, net_exposure_pct, leverage_max,
    issuer_concentration_pct, country_concentration_pct, sector_concentration_pct,
    pre_trade_checks_required, maker_checker, stp_required,
    breach_handling, intraday_status, matching_model
) VALUES

-- CBU-203914 (Global Trade Finance Consortium) - Singapore-based
('MANDATE-TF-GLOBAL-2025', 'CBU-203914',
 'Singapore Sovereign Wealth Fund', '529900T8BM49AURSDO61',
 'Asian Trade Capital Management', '529900T8BM49AURSDO62',
 'USD', '2025-01-15', '2027-01-15',
 150.00, 120.00, 2.000,
 25.00, 40.00, 30.00,
 true, true, false,
 'immediate_breach_notification', 'real_time', 'affirmation'),

-- CBU-942121 (European Infrastructure Fund) - Luxembourg-based
('MANDATE-INFRA-EU-2025', 'CBU-942121',
 'European Infrastructure Pension Scheme', '549300T8BM49AURSDO71',
 'Infrastructure Capital Partners SARL', '549300T8BM49AURSDO72',
 'EUR', '2025-02-01', '2030-02-01',
 110.00, 100.00, 1.200,
 15.00, 50.00, 25.00,
 true, true, true,
 'end_of_day_reporting', 'hourly', 'central_matching'),

-- CBU-973130 (Cross-Border Payments Network) - Swiss-based
('MANDATE-FINTECH-CH-2025', 'CBU-973130',
 'SwissPay Digital Holdings AG', '506700T8BM49AURSDO81',
 'CrossBorder Solutions SA', '506700T8BM49AURSDO82',
 'CHF', '2025-03-01', '2026-03-01',
 105.00, 95.00, 1.050,
 10.00, 30.00, 20.00,
 true, false, true,
 'auto_rebalance', 'real_time', 'confirmation')

ON CONFLICT (mandate_id) DO UPDATE SET
    asset_owner_name = EXCLUDED.asset_owner_name,
    investment_manager_name = EXCLUDED.investment_manager_name,
    updated_at = CURRENT_TIMESTAMP;

-- Insert instruments for Trade Finance mandate (conservative, liquid)
INSERT INTO mandate_instruments (
    mandate_id, instrument_family, subtype, cfi_code,
    order_types, time_in_force, settlement_type, settlement_cycle,
    exposure_pct, short_allowed, issuer_max_pct, rating_floor,
    counterparties_whitelist, allocation_model, notes
) VALUES

-- Trade Finance Mandate - Government bonds
('MANDATE-TF-GLOBAL-2025', 'fixed_income', 'government_bond', 'DBVUFR',
 ARRAY['Market', 'Limit'], ARRAY['Day', 'GTC'], 'DVP', 'T+2',
 50.00, false, 15.00, 'AA',
 ARRAY['DBS Bank', 'HSBC Singapore', 'Standard Chartered'], 'pre_trade',
 'Core government bond allocation for capital preservation'),

-- Trade Finance Mandate - Corporate bonds
('MANDATE-TF-GLOBAL-2025', 'fixed_income', 'corporate_bond', 'DBVUFR',
 ARRAY['Market', 'Limit'], ARRAY['Day', 'IOC'], 'DVP', 'T+2',
 30.00, false, 10.00, 'A',
 ARRAY['DBS Bank', 'HSBC Singapore'], 'pre_trade',
 'Investment grade corporate bonds for yield enhancement'),

-- Trade Finance Mandate - Money market
('MANDATE-TF-GLOBAL-2025', 'money_market', 'certificate_deposit', 'MMVUFR',
 ARRAY['Market'], ARRAY['Day'], 'Cash', 'T+0',
 20.00, false, 25.00, 'AAA',
 ARRAY['DBS Bank', 'OCBC Bank', 'UOB'], 'either',
 'Short-term liquidity management and cash parking');

-- Insert instruments for Infrastructure mandate (growth-oriented)
INSERT INTO mandate_instruments (
    mandate_id, instrument_family, subtype, cfi_code,
    order_types, time_in_force, settlement_type, settlement_cycle,
    exposure_pct, short_allowed, issuer_max_pct, rating_floor,
    counterparties_whitelist, allocation_model, notes
) VALUES

-- Infrastructure Mandate - Equity
('MANDATE-INFRA-EU-2025', 'equity', 'common_stock', 'ESVUFR',
 ARRAY['Market', 'Limit', 'TWAP'], ARRAY['Day', 'GTC'], 'DVP', 'T+2',
 40.00, false, 8.00, 'BBB',
 ARRAY['BNP Paribas', 'Deutsche Bank', 'Credit Suisse'], 'post_trade',
 'European infrastructure and utility companies'),

-- Infrastructure Mandate - Infrastructure bonds
('MANDATE-INFRA-EU-2025', 'fixed_income', 'infrastructure_bond', 'DBVUFR',
 ARRAY['Market', 'Limit'], ARRAY['Day', 'GTC'], 'DVP', 'T+2',
 35.00, false, 12.00, 'BBB',
 ARRAY['BNP Paribas', 'KPMG Luxembourg'], 'pre_trade',
 'Infrastructure project financing bonds'),

-- Infrastructure Mandate - Alternative investments (REITs, Infrastructure funds)
('MANDATE-INFRA-EU-2025', 'fund', 'infrastructure_fund', 'MMVUFR',
 ARRAY['Market'], ARRAY['Day'], 'DVP', 'T+3',
 25.00, false, 20.00, 'A',
 ARRAY['European Fund Administration'], 'pre_trade',
 'Direct infrastructure fund investments');

-- Insert instruments for Fintech mandate (high liquidity)
INSERT INTO mandate_instruments (
    mandate_id, instrument_family, subtype, cfi_code,
    order_types, time_in_force, settlement_type, settlement_cycle,
    exposure_pct, short_allowed, issuer_max_pct, rating_floor,
    counterparties_whitelist, allocation_model, notes
) VALUES

-- Fintech Mandate - Money market funds
('MANDATE-FINTECH-CH-2025', 'money_market', 'money_market_fund', 'MMVUFR',
 ARRAY['Market'], ARRAY['Day'], 'Cash', 'T+1',
 60.00, false, 30.00, 'AAA',
 ARRAY['Credit Suisse', 'UBS', 'Swiss Regulatory Compliance'], 'either',
 'High liquidity money market funds for payment processing reserves'),

-- Fintech Mandate - Short-term government bonds
('MANDATE-FINTECH-CH-2025', 'fixed_income', 'government_bond_short', 'DBVUFR',
 ARRAY['Market', 'Limit'], ARRAY['Day'], 'DVP', 'T+2',
 25.00, false, 40.00, 'AAA',
 ARRAY['Credit Suisse', 'UBS'], 'pre_trade',
 'Swiss and German government bonds under 2 years maturity'),

-- Fintech Mandate - FX hedging instruments
('MANDATE-FINTECH-CH-2025', 'fx', 'fx_forward', 'FFVUFR',
 ARRAY['Market'], ARRAY['Day', 'IOC'], 'PvP', 'T+2',
 15.00, true, 50.00, NULL,
 ARRAY['Credit Suisse', 'Blockchain Infrastructure Services'], 'either',
 'Currency hedging for multi-currency payment flows');

-- Create comprehensive view showing CBU → Investment Mandate → Instruments with roles
CREATE OR REPLACE VIEW cbu_investment_mandate_structure AS
SELECT
    -- CBU Information
    cbu.cbu_id,
    cbu.cbu_name,
    cbu.business_type,
    cbu.description as cbu_description,

    -- Investment Mandate
    im.mandate_id,
    im.asset_owner_name,
    im.asset_owner_lei,
    im.investment_manager_name,
    im.investment_manager_lei,
    im.base_currency,
    im.effective_date,
    im.expiry_date,

    -- Risk Parameters
    im.gross_exposure_pct,
    im.net_exposure_pct,
    im.leverage_max,
    im.issuer_concentration_pct,
    im.country_concentration_pct,
    im.sector_concentration_pct,

    -- Instruments Summary
    COUNT(DISTINCT mi.id) as total_instruments,
    COUNT(DISTINCT mi.instrument_family) as instrument_families,
    SUM(mi.exposure_pct) as total_exposure_pct,
    string_agg(DISTINCT mi.instrument_family, ', ' ORDER BY mi.instrument_family) as families,

    -- Trading Controls
    im.pre_trade_checks_required,
    im.maker_checker,
    im.stp_required,
    im.breach_handling,
    im.intraday_status,
    im.matching_model,

    -- Instrument Risk Profile
    AVG(CASE WHEN mi.rating_floor = 'AAA' THEN 1 WHEN mi.rating_floor = 'AA' THEN 2
             WHEN mi.rating_floor = 'A' THEN 3 WHEN mi.rating_floor = 'BBB' THEN 4 ELSE 5 END) as avg_rating_numeric,
    COUNT(CASE WHEN mi.short_allowed THEN 1 END) as instruments_allow_short,

    -- Metadata
    im.created_at as mandate_created_at,
    im.updated_at as mandate_updated_at

FROM client_business_units cbu
LEFT JOIN investment_mandates im ON im.cbu_id = cbu.cbu_id  -- String match on cbu_id
LEFT JOIN mandate_instruments mi ON mi.mandate_id = im.mandate_id
GROUP BY
    cbu.cbu_id, cbu.cbu_name, cbu.business_type, cbu.description,
    im.mandate_id, im.asset_owner_name, im.asset_owner_lei, im.investment_manager_name,
    im.investment_manager_lei, im.base_currency, im.effective_date, im.expiry_date,
    im.gross_exposure_pct, im.net_exposure_pct, im.leverage_max,
    im.issuer_concentration_pct, im.country_concentration_pct, im.sector_concentration_pct,
    im.pre_trade_checks_required, im.maker_checker, im.stp_required,
    im.breach_handling, im.intraday_status, im.matching_model,
    im.created_at, im.updated_at;

-- Show detailed CBU member roles and their investment responsibilities
CREATE OR REPLACE VIEW cbu_member_investment_roles AS
SELECT
    -- CBU Information
    cbu.cbu_id,
    cbu.cbu_name,

    -- Member Information
    cm.entity_name,
    cm.entity_lei,
    cr.role_name,
    cr.role_code,
    cm.has_trading_authority,
    cm.has_settlement_authority,

    -- Investment Mandate Connection
    im.mandate_id,
    CASE
        WHEN cr.role_code = 'ASSET_OWNER' THEN 'Mandate Owner & Capital Provider'
        WHEN cr.role_code = 'INVESTMENT_MANAGER' THEN 'Portfolio Management & Trading'
        WHEN cr.role_code = 'CUSTODIAN' THEN 'Asset Safekeeping & Settlement'
        WHEN cr.role_code = 'ADMINISTRATOR' THEN 'Fund Administration & Reporting'
        WHEN cr.role_code = 'PROCESSOR' THEN 'Payment Processing & Liquidity'
        WHEN cr.role_code = 'COMPLIANCE_OFFICER' THEN 'Compliance Monitoring & Risk'
        ELSE 'Other Investment Role'
    END as investment_responsibility,

    -- Mandate Summary (when this member is primary)
    CASE WHEN im.asset_owner_lei = cm.entity_lei THEN im.base_currency END as mandate_currency,
    CASE WHEN im.asset_owner_lei = cm.entity_lei THEN im.leverage_max END as leverage_limit,
    CASE WHEN im.investment_manager_lei = cm.entity_lei THEN
        CONCAT(im.gross_exposure_pct, '% gross / ', im.net_exposure_pct, '% net exposure')
    END as exposure_limits,

    -- Member Status
    cm.is_primary,
    cm.effective_date as member_effective_date,
    cm.notes as member_notes

FROM client_business_units cbu
JOIN cbu_members cm ON cm.cbu_id = cbu.id
JOIN cbu_roles cr ON cr.id = cm.role_id
LEFT JOIN investment_mandates im ON im.cbu_id = cbu.cbu_id
    AND (im.asset_owner_lei = cm.entity_lei OR im.investment_manager_lei = cm.entity_lei)
ORDER BY cbu.cbu_id, cm.is_primary DESC, cr.role_code;

-- Summary report
DO $$
DECLARE
    mandate_count INTEGER;
    instrument_count INTEGER;
    cbu_with_mandates INTEGER;
BEGIN
    SELECT COUNT(*) INTO mandate_count FROM investment_mandates;
    SELECT COUNT(*) INTO instrument_count FROM mandate_instruments;
    SELECT COUNT(DISTINCT cbu_id) INTO cbu_with_mandates FROM investment_mandates;

    RAISE NOTICE '🎯 Investment Mandate System Populated:';
    RAISE NOTICE '   📋 Total Mandates: %', mandate_count;
    RAISE NOTICE '   🎪 Total Instruments: %', instrument_count;
    RAISE NOTICE '   🏢 CBUs with Mandates: %', cbu_with_mandates;
    RAISE NOTICE '';
    RAISE NOTICE '🔗 Complete Chain: CBU Members (by Role) → Investment Mandates → Instruments → Instruction Formats';
END $$;

-- Show the structure
SELECT 'CBU Investment Mandate Structure:' as header;
SELECT
    cbu_id,
    cbu_name,
    mandate_id,
    asset_owner_name,
    investment_manager_name,
    base_currency,
    total_instruments,
    families as instrument_families,
    total_exposure_pct
FROM cbu_investment_mandate_structure
WHERE mandate_id IS NOT NULL
ORDER BY cbu_id;