-- Create Sample Investment Mandates linked to CBUs
-- Simple implementation to demonstrate CBU → Investment Mandate → Instruments relationship

-- Create sample investment mandates for our existing CBUs
INSERT INTO investment_mandates (
    mandate_id, cbu_id, product_id, mandate_name, mandate_type,
    investment_objective, risk_tolerance, base_currency,
    liquidity_requirement, geographic_focus, mandate_status
) VALUES

-- For CBU-203914 (Global Trade Finance Consortium)
('MAN-TF-GLOBAL-001',
 (SELECT id FROM client_business_units WHERE cbu_id = 'CBU-203914'),
 (SELECT id FROM products WHERE product_id = 'TRADE-SETTLEMENT-PRO'),
 'Global Trade Finance Investment Strategy',
 'absolute_return',
 'Generate stable returns through trade finance instruments while maintaining capital preservation focus',
 'conservative', 'USD', 'quarterly', 'global', 'active'),

-- For CBU-942121 (European Infrastructure Fund)
('MAN-INFRA-EU-001',
 (SELECT id FROM client_business_units WHERE cbu_id = 'CBU-942121'),
 (SELECT id FROM products WHERE product_id = 'FUND-ADMIN-COMPLETE'),
 'European Infrastructure Investment Mandate',
 'benchmark_relative',
 'Long-term capital appreciation through European infrastructure investments with ESG focus',
 'moderate', 'EUR', 'annual', 'regional', 'active'),

-- For CBU-973130 (Cross-Border Payments Network)
('MAN-FINTECH-LIQUID-001',
 (SELECT id FROM client_business_units WHERE cbu_id = 'CBU-973130'),
 (SELECT id FROM products WHERE product_id = 'MIDDLE-OFFICE-SUITE'),
 'Fintech Liquidity Management Strategy',
 'liability_driven',
 'Maintain optimal liquidity for payment processing while generating yield on excess cash',
 'conservative', 'CHF', 'daily', 'global', 'active')

ON CONFLICT (mandate_id) DO UPDATE SET
    mandate_name = EXCLUDED.mandate_name,
    investment_objective = EXCLUDED.investment_objective,
    updated_at = CURRENT_TIMESTAMP;

-- Create sample instrument allocations for these mandates
INSERT INTO mandate_instrument_allocations (
    mandate_id, instrument_id, allocation_type, allocation_percentage,
    maximum_position_size, minimum_credit_rating, active
) VALUES

-- Trade Finance Mandate - Conservative fixed income focus
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-TF-GLOBAL-001'),
 (SELECT id FROM instrument_taxonomy WHERE instrument_code = 'FI_GB'), 'target', 40.00, 5000000.00, 'AA', true),
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-TF-GLOBAL-001'),
 (SELECT id FROM instrument_taxonomy WHERE instrument_code = 'FI_CB'), 'target', 35.00, 3000000.00, 'A', true),
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-TF-GLOBAL-001'),
 (SELECT id FROM instrument_taxonomy WHERE instrument_code = 'CE_MM'), 'target', 25.00, 10000000.00, NULL, true),

-- Infrastructure Mandate - Mix of equity and alternatives
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-INFRA-EU-001'),
 (SELECT id FROM instrument_taxonomy WHERE instrument_code = 'EQ_CS'), 'target', 30.00, 15000000.00, NULL, true),
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-INFRA-EU-001'),
 (SELECT id FROM instrument_taxonomy WHERE instrument_code = 'ALT_RE'), 'target', 50.00, 25000000.00, 'BBB', true),
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-INFRA-EU-001'),
 (SELECT id FROM instrument_taxonomy WHERE instrument_code = 'FI_CB'), 'target', 20.00, 8000000.00, 'A', true),

-- Fintech Liquidity Mandate - High liquidity focus
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-FINTECH-LIQUID-001'),
 (SELECT id FROM instrument_taxonomy WHERE instrument_code = 'CE_MM'), 'target', 60.00, 50000000.00, NULL, true),
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-FINTECH-LIQUID-001'),
 (SELECT id FROM instrument_taxonomy WHERE instrument_code = 'CE_CD'), 'target', 25.00, 20000000.00, 'AA', true),
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-FINTECH-LIQUID-001'),
 (SELECT id FROM instrument_taxonomy WHERE instrument_code = 'FI_GB'), 'target', 15.00, 10000000.00, 'AAA', true)

ON CONFLICT (mandate_id, instrument_id, allocation_type) DO UPDATE SET
    allocation_percentage = EXCLUDED.allocation_percentage,
    maximum_position_size = EXCLUDED.maximum_position_size,
    updated_at = CURRENT_TIMESTAMP;

-- Create instruction format mappings for these mandates
INSERT INTO mandate_instruction_mappings (
    mandate_id, instruction_format_id, usage_context, is_default_format, priority_order
) VALUES

-- Trade Finance Mandate - SWIFT focus for international operations
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-TF-GLOBAL-001'),
 (SELECT id FROM instruction_formats WHERE format_id = 'SWIFT_MT515'), 'settlement', true, 1),
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-TF-GLOBAL-001'),
 (SELECT id FROM instruction_formats WHERE format_id = 'FIX_NEW_ORDER'), 'trade_execution', true, 1),

-- Infrastructure Mandate - ISO20022 for European compliance
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-INFRA-EU-001'),
 (SELECT id FROM instruction_formats WHERE format_id = 'ISO20022_SEMT'), 'settlement', true, 1),
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-INFRA-EU-001'),
 (SELECT id FROM instruction_formats WHERE format_id = 'JSON_PORTFOLIO'), 'reporting', true, 1),

-- Fintech Mandate - Modern JSON/API formats
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-FINTECH-LIQUID-001'),
 (SELECT id FROM instruction_formats WHERE format_id = 'JSON_PORTFOLIO'), 'reporting', true, 1),
((SELECT id FROM investment_mandates WHERE mandate_id = 'MAN-FINTECH-LIQUID-001'),
 (SELECT id FROM instruction_formats WHERE format_id = 'FIX_NEW_ORDER'), 'trade_execution', true, 1)

ON CONFLICT (mandate_id, instruction_format_id, usage_context) DO UPDATE SET
    is_default_format = EXCLUDED.is_default_format,
    priority_order = EXCLUDED.priority_order,
    updated_at = CURRENT_TIMESTAMP;

-- Create a simplified CBU investment view
CREATE OR REPLACE VIEW cbu_investment_mandates_view AS
SELECT
    -- CBU Information
    cbu.cbu_id,
    cbu.cbu_name,
    cbu.business_type,
    cbu.description as cbu_description,

    -- Investment Mandate Information
    im.mandate_id,
    im.mandate_name,
    im.mandate_type,
    im.investment_objective,
    im.risk_tolerance,
    im.base_currency,
    im.liquidity_requirement,
    im.geographic_focus,
    im.mandate_status,

    -- Product Information
    p.product_name,
    p.line_of_business,

    -- Instrument Allocation Summary
    COUNT(DISTINCT mia.instrument_id) as total_instrument_types,
    SUM(mia.allocation_percentage) FILTER (WHERE mia.allocation_type = 'target') as total_target_allocation,
    COUNT(DISTINCT it.instrument_class) as instrument_classes,

    -- Instruction Format Summary
    COUNT(DISTINCT mim.instruction_format_id) as instruction_formats_count,
    string_agg(DISTINCT if_data.message_standard, ', ') as message_standards,

    -- Risk Profile
    string_agg(DISTINCT it.risk_category, ', ') as risk_categories,
    string_agg(DISTINCT it.liquidity_classification, ', ') as liquidity_profile,

    -- Metadata
    im.effective_date,
    im.created_at as mandate_created_at,
    im.updated_at as mandate_updated_at

FROM client_business_units cbu
LEFT JOIN investment_mandates im ON im.cbu_id = cbu.id
LEFT JOIN products p ON p.id = im.product_id
LEFT JOIN mandate_instrument_allocations mia ON mia.mandate_id = im.id
LEFT JOIN instrument_taxonomy it ON it.id = mia.instrument_id
LEFT JOIN mandate_instruction_mappings mim ON mim.mandate_id = im.id
LEFT JOIN instruction_formats if_data ON if_data.id = mim.instruction_format_id
GROUP BY
    cbu.cbu_id, cbu.cbu_name, cbu.business_type, cbu.description,
    im.mandate_id, im.mandate_name, im.mandate_type, im.investment_objective,
    im.risk_tolerance, im.base_currency, im.liquidity_requirement, im.geographic_focus,
    im.mandate_status, im.effective_date, im.created_at, im.updated_at,
    p.product_name, p.line_of_business;

-- Show results
DO $$
DECLARE
    mandate_count INTEGER;
    allocation_count INTEGER;
    format_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO mandate_count FROM investment_mandates WHERE mandate_status = 'active';
    SELECT COUNT(*) INTO allocation_count FROM mandate_instrument_allocations WHERE active = true;
    SELECT COUNT(*) INTO format_count FROM mandate_instruction_mappings WHERE active = true;

    RAISE NOTICE '🎯 Investment Mandates Created:';
    RAISE NOTICE '   📋 Active Mandates: %', mandate_count;
    RAISE NOTICE '   🎪 Instrument Allocations: %', allocation_count;
    RAISE NOTICE '   📄 Instruction Format Mappings: %', format_count;
    RAISE NOTICE '';
    RAISE NOTICE '🔗 CBU → Investment Mandate → Instruments/Formats linkage established';
END $$;

-- Test the view
SELECT 'CBU Investment Mandates Summary:' as header;
SELECT
    cbu_id,
    cbu_name,
    mandate_name,
    mandate_type,
    total_instrument_types,
    total_target_allocation,
    risk_tolerance
FROM cbu_investment_mandates_view
WHERE mandate_id IS NOT NULL
ORDER BY cbu_id;