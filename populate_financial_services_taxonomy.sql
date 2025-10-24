-- Populate Financial Services Taxonomy Data
-- Products (commercial sold products) → Services (generic public financial services) → Resources (applications implementing services)

-- Insert sample financial services products (public/generic commercial sold products in contracts)
INSERT INTO products (product_id, product_name, line_of_business, description, status, contract_type, commercial_status, pricing_model, target_market, sales_territory, compliance_requirements, minimum_contract_value, maximum_contract_value, standard_contract_duration, renewable, early_termination_allowed) VALUES

('INST-CUSTODY-PLUS', 'Institutional Custody Plus', 'Custody Services',
 'Comprehensive custody and safekeeping services for institutional investors with enhanced reporting and middle office capabilities',
 'active', 'service_agreement', 'active', 'tiered', 'enterprise', 'global',
 ARRAY['SOC2', 'ISO27001', 'SSAE18', 'AICPA'],
 1000000.00, 50000000.00, 36, true, true),

('FUND-ADMIN-COMPLETE', 'Fund Administration Complete', 'Fund Administration',
 'Full-service fund administration including accounting, reconciliation, reporting, and investor services',
 'active', 'service_agreement', 'active', 'fixed', 'mid_market', 'regional',
 ARRAY['AIFMD', 'UCITS', 'SOX', 'IFRS'],
 250000.00, 10000000.00, 24, true, false),

('TRADE-SETTLEMENT-PRO', 'Trade Settlement Professional', 'Trade Processing',
 'End-to-end trade processing, settlement, and post-trade services with multi-asset class support',
 'active', 'license', 'active', 'usage_based', 'enterprise', 'global',
 ARRAY['MiFID2', 'CSDR', 'T2S', 'SWIFT'],
 500000.00, 25000000.00, 12, true, true),

('MIDDLE-OFFICE-SUITE', 'Middle Office Suite', 'Middle Office',
 'Integrated middle office services including trade order management, compliance monitoring, and risk management',
 'active', 'subscription', 'active', 'subscription', 'enterprise', 'domestic',
 ARRAY['MIFID2', 'EMIR', 'SFTR', 'PRIIPS'],
 100000.00, 5000000.00, 12, true, true)

ON CONFLICT (product_id) DO UPDATE SET
    description = EXCLUDED.description,
    contract_type = EXCLUDED.contract_type,
    commercial_status = EXCLUDED.commercial_status,
    pricing_model = EXCLUDED.pricing_model,
    target_market = EXCLUDED.target_market,
    updated_at = CURRENT_TIMESTAMP;

-- Insert generic public financial services
INSERT INTO services (service_id, service_name, service_category, description, status, service_type, delivery_model, billable, recurring_service, automation_level, customer_facing) VALUES

-- Custody Services
('CUSTODY-CORE', 'Core Custody Services', 'Custody',
 'Basic asset safekeeping, settlement instruction processing, and cash management',
 'active', 'custody', 'managed', true, true, 'semi_automated', true),

('CUSTODY-ENHANCED', 'Enhanced Custody Services', 'Custody',
 'Advanced custody with global reach, multi-currency support, and enhanced reporting',
 'active', 'custody', 'managed', true, true, 'fully_automated', true),

-- Safekeeping Services
('SAFEKEEPING-SECURITIES', 'Securities Safekeeping', 'Safekeeping',
 'Physical and electronic safekeeping of securities with full segregation',
 'active', 'safekeeping', 'managed', true, true, 'fully_automated', false),

('SAFEKEEPING-VALUABLES', 'Valuables Safekeeping', 'Safekeeping',
 'Secure storage and management of physical valuables and documents',
 'active', 'safekeeping', 'managed', true, true, 'manual', false),

-- Reconciliation Services
('RECON-POSITIONS', 'Position Reconciliation', 'Reconciliation',
 'Daily reconciliation of positions across multiple systems and counterparties',
 'active', 'reconciliation', 'self_service', true, true, 'fully_automated', false),

('RECON-CASH', 'Cash Reconciliation', 'Reconciliation',
 'Multi-currency cash reconciliation with automated exception handling',
 'active', 'reconciliation', 'hybrid', true, true, 'fully_automated', false),

-- Fund Accounting Services
('FA-NAV-CALC', 'NAV Calculation', 'Fund Accounting',
 'Daily net asset value calculation with regulatory compliance reporting',
 'active', 'fund_accounting', 'managed', true, true, 'fully_automated', true),

('FA-FINANCIAL-REPORTING', 'Financial Reporting', 'Fund Accounting',
 'Comprehensive financial statements and regulatory reporting for funds',
 'active', 'fund_accounting', 'managed', true, true, 'semi_automated', true),

-- Middle Office Services
('MO-TRADE-SUPPORT', 'Trade Support Services', 'Middle Office',
 'Pre and post-trade support including confirmation, affirmation, and exception management',
 'active', 'middle_office', 'managed', true, true, 'semi_automated', true),

('MO-COMPLIANCE-MONITORING', 'Compliance Monitoring', 'Middle Office',
 'Real-time compliance monitoring and breach management',
 'active', 'middle_office', 'self_service', true, true, 'fully_automated', false),

-- Trade Order Management Services
('TOM-ORDER-ROUTING', 'Order Routing Services', 'Trade Order Management',
 'Intelligent order routing with best execution analytics',
 'active', 'trade_order_management', 'self_service', true, true, 'fully_automated', true),

('TOM-EXECUTION-MANAGEMENT', 'Execution Management', 'Trade Order Management',
 'Trade execution management with real-time monitoring and reporting',
 'active', 'trade_order_management', 'hybrid', true, true, 'fully_automated', true)

ON CONFLICT (service_id) DO UPDATE SET
    description = EXCLUDED.description,
    service_type = EXCLUDED.service_type,
    delivery_model = EXCLUDED.delivery_model,
    updated_at = CURRENT_TIMESTAMP;

-- Insert resources (applications and systems that implement the services)
INSERT INTO resource_objects (dictionary_id, resource_name, description, status, resource_type, criticality_level, operational_status, monitoring_enabled, audit_required) VALUES

-- Get the dictionary ID (assuming we have one)
((SELECT id FROM resource_dictionaries LIMIT 1), 'Custody Application Accounts',
 'Core custody application with client account management and settlement processing',
 'active', 'application_accounts', 'critical', 'active', true, true),

((SELECT id FROM resource_dictionaries LIMIT 1), 'Global Routing Tables',
 'Multi-market routing configuration tables for settlement and custody operations',
 'active', 'routing_tables', 'high', 'active', true, true),

((SELECT id FROM resource_dictionaries LIMIT 1), 'Reconciliation Application',
 'Automated reconciliation engine with exception management and reporting',
 'active', 'reconciliation_app', 'critical', 'active', true, true),

((SELECT id FROM resource_dictionaries LIMIT 1), 'Fund Accounting Application',
 'Comprehensive fund accounting system with NAV calculation and reporting',
 'active', 'fa_app', 'critical', 'active', true, true),

((SELECT id FROM resource_dictionaries LIMIT 1), 'IBOR Application',
 'Investment Book of Records with real-time position management',
 'active', 'ibor_app', 'high', 'active', true, true),

((SELECT id FROM resource_dictionaries LIMIT 1), 'Trading System Interface',
 'Multi-asset trading platform with order management and execution',
 'active', 'trading_system', 'critical', 'active', true, true),

((SELECT id FROM resource_dictionaries LIMIT 1), 'Settlement System Hub',
 'Centralized settlement processing with multiple market connectivity',
 'active', 'settlement_system', 'critical', 'active', true, true),

((SELECT id FROM resource_dictionaries LIMIT 1), 'Cash Management System',
 'Multi-currency cash management with forecasting and optimization',
 'active', 'application_accounts', 'high', 'active', true, true),

((SELECT id FROM resource_dictionaries LIMIT 1), 'Compliance Monitoring Engine',
 'Real-time compliance checking and breach management system',
 'active', 'reconciliation_app', 'critical', 'active', true, true),

((SELECT id FROM resource_dictionaries LIMIT 1), 'Market Data Feed Handler',
 'Real-time market data processing and distribution system',
 'active', 'trading_system', 'high', 'active', true, false)

ON CONFLICT (resource_name) DO UPDATE SET
    description = EXCLUDED.description,
    resource_type = EXCLUDED.resource_type,
    criticality_level = EXCLUDED.criticality_level,
    updated_at = CURRENT_TIMESTAMP;

-- Map Products to Services
INSERT INTO product_service_mappings (product_id, service_id, mapping_type, is_mandatory, customer_configurable, pricing_impact) VALUES

-- Institutional Custody Plus → Services
((SELECT id FROM products WHERE product_id = 'INST-CUSTODY-PLUS'),
 (SELECT id FROM services WHERE service_id = 'CUSTODY-ENHANCED'), 'core', true, false, 0.00),
((SELECT id FROM products WHERE product_id = 'INST-CUSTODY-PLUS'),
 (SELECT id FROM services WHERE service_id = 'SAFEKEEPING-SECURITIES'), 'core', true, false, 0.00),
((SELECT id FROM products WHERE product_id = 'INST-CUSTODY-PLUS'),
 (SELECT id FROM services WHERE service_id = 'RECON-POSITIONS'), 'add_on', false, true, 15000.00),
((SELECT id FROM products WHERE product_id = 'INST-CUSTODY-PLUS'),
 (SELECT id FROM services WHERE service_id = 'MO-TRADE-SUPPORT'), 'premium', false, true, 25000.00),

-- Fund Administration Complete → Services
((SELECT id FROM products WHERE product_id = 'FUND-ADMIN-COMPLETE'),
 (SELECT id FROM services WHERE service_id = 'FA-NAV-CALC'), 'core', true, false, 0.00),
((SELECT id FROM products WHERE product_id = 'FUND-ADMIN-COMPLETE'),
 (SELECT id FROM services WHERE service_id = 'FA-FINANCIAL-REPORTING'), 'core', true, false, 0.00),
((SELECT id FROM products WHERE product_id = 'FUND-ADMIN-COMPLETE'),
 (SELECT id FROM services WHERE service_id = 'RECON-CASH'), 'core', true, false, 0.00),
((SELECT id FROM products WHERE product_id = 'FUND-ADMIN-COMPLETE'),
 (SELECT id FROM services WHERE service_id = 'CUSTODY-CORE'), 'optional', false, true, 10000.00),

-- Trade Settlement Pro → Services
((SELECT id FROM products WHERE product_id = 'TRADE-SETTLEMENT-PRO'),
 (SELECT id FROM services WHERE service_id = 'TOM-ORDER-ROUTING'), 'core', true, false, 0.00),
((SELECT id FROM products WHERE product_id = 'TRADE-SETTLEMENT-PRO'),
 (SELECT id FROM services WHERE service_id = 'TOM-EXECUTION-MANAGEMENT'), 'core', true, false, 0.00),
((SELECT id FROM products WHERE product_id = 'TRADE-SETTLEMENT-PRO'),
 (SELECT id FROM services WHERE service_id = 'RECON-POSITIONS'), 'core', true, false, 0.00),

-- Middle Office Suite → Services
((SELECT id FROM products WHERE product_id = 'MIDDLE-OFFICE-SUITE'),
 (SELECT id FROM services WHERE service_id = 'MO-TRADE-SUPPORT'), 'core', true, false, 0.00),
((SELECT id FROM products WHERE product_id = 'MIDDLE-OFFICE-SUITE'),
 (SELECT id FROM services WHERE service_id = 'MO-COMPLIANCE-MONITORING'), 'core', true, false, 0.00),
((SELECT id FROM products WHERE product_id = 'MIDDLE-OFFICE-SUITE'),
 (SELECT id FROM services WHERE service_id = 'TOM-ORDER-ROUTING'), 'add_on', false, true, 20000.00)

ON CONFLICT (product_id, service_id, mapping_type) DO UPDATE SET
    pricing_impact = EXCLUDED.pricing_impact,
    updated_at = CURRENT_TIMESTAMP;

-- Map Services to Resources
INSERT INTO service_resource_mappings (service_id, resource_id, usage_type, resource_role, dependency_level, cost_allocation_percentage) VALUES

-- Custody Services → Resources
((SELECT id FROM services WHERE service_id = 'CUSTODY-ENHANCED'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Custody Application Accounts'), 'required', 'processor', 1, 40.00),
((SELECT id FROM services WHERE service_id = 'CUSTODY-ENHANCED'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Settlement System Hub'), 'required', 'processor', 1, 30.00),
((SELECT id FROM services WHERE service_id = 'CUSTODY-ENHANCED'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Global Routing Tables'), 'required', 'data_source', 2, 10.00),

-- Reconciliation Services → Resources
((SELECT id FROM services WHERE service_id = 'RECON-POSITIONS'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Reconciliation Application'), 'required', 'processor', 1, 60.00),
((SELECT id FROM services WHERE service_id = 'RECON-POSITIONS'),
 (SELECT id FROM resource_objects WHERE resource_name = 'IBOR Application'), 'required', 'data_source', 1, 25.00),

((SELECT id FROM services WHERE service_id = 'RECON-CASH'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Reconciliation Application'), 'required', 'processor', 1, 50.00),
((SELECT id FROM services WHERE service_id = 'RECON-CASH'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Cash Management System'), 'required', 'data_source', 1, 35.00),

-- Fund Accounting Services → Resources
((SELECT id FROM services WHERE service_id = 'FA-NAV-CALC'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Fund Accounting Application'), 'required', 'processor', 1, 70.00),
((SELECT id FROM services WHERE service_id = 'FA-NAV-CALC'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Market Data Feed Handler'), 'required', 'data_source', 2, 20.00),

((SELECT id FROM services WHERE service_id = 'FA-FINANCIAL-REPORTING'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Fund Accounting Application'), 'required', 'processor', 1, 80.00),

-- Trade Order Management → Resources
((SELECT id FROM services WHERE service_id = 'TOM-ORDER-ROUTING'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Trading System Interface'), 'required', 'processor', 1, 50.00),
((SELECT id FROM services WHERE service_id = 'TOM-ORDER-ROUTING'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Global Routing Tables'), 'required', 'data_source', 1, 25.00),

((SELECT id FROM services WHERE service_id = 'TOM-EXECUTION-MANAGEMENT'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Trading System Interface'), 'required', 'processor', 1, 60.00),
((SELECT id FROM services WHERE service_id = 'TOM-EXECUTION-MANAGEMENT'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Market Data Feed Handler'), 'required', 'data_source', 2, 15.00),

-- Middle Office Services → Resources
((SELECT id FROM services WHERE service_id = 'MO-TRADE-SUPPORT'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Trading System Interface'), 'required', 'processor', 1, 40.00),
((SELECT id FROM services WHERE service_id = 'MO-TRADE-SUPPORT'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Reconciliation Application'), 'required', 'validator', 2, 30.00),

((SELECT id FROM services WHERE service_id = 'MO-COMPLIANCE-MONITORING'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Compliance Monitoring Engine'), 'required', 'processor', 1, 70.00),
((SELECT id FROM services WHERE service_id = 'MO-COMPLIANCE-MONITORING'),
 (SELECT id FROM resource_objects WHERE resource_name = 'Trading System Interface'), 'required', 'data_source', 1, 20.00)

ON CONFLICT (service_id, resource_id, usage_type) DO UPDATE SET
    cost_allocation_percentage = EXCLUDED.cost_allocation_percentage,
    updated_at = CURRENT_TIMESTAMP;

-- Show the taxonomy structure
DO $$
DECLARE
    product_count INTEGER;
    service_count INTEGER;
    resource_count INTEGER;
    mapping_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO product_count FROM products WHERE commercial_status = 'active';
    SELECT COUNT(*) INTO service_count FROM services WHERE status = 'active';
    SELECT COUNT(*) INTO resource_count FROM resource_objects WHERE operational_status = 'active';
    SELECT COUNT(*) INTO mapping_count FROM product_service_mappings;

    RAISE NOTICE '🏦 Financial Services Taxonomy Created:';
    RAISE NOTICE '   📦 Products (Commercial Contracts): %', product_count;
    RAISE NOTICE '   🔧 Services (Generic Financial Services): %', service_count;
    RAISE NOTICE '   💻 Resources (Applications/Systems): %', resource_count;
    RAISE NOTICE '   🔗 Product→Service Mappings: %', mapping_count;
    RAISE NOTICE '';
END $$;

-- Test the taxonomy hierarchy function
SELECT 'Testing taxonomy hierarchy for Institutional Custody Plus:' as test_header;
SELECT * FROM get_product_taxonomy_hierarchy((SELECT id FROM products WHERE product_id = 'INST-CUSTODY-PLUS'));

-- Validate the taxonomy
SELECT 'Taxonomy validation results:' as validation_header;
SELECT * FROM validate_commercial_taxonomy() LIMIT 10;