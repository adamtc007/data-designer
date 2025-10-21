-- Migration: Test Data Seeding for DSL Workflow Testing
-- Purpose: Populate database with comprehensive test data to validate DSL workflows,
--          entity relationships, and capability-driven processing

-- Insert sample CBUs for testing
INSERT INTO cbu (cbu_name, entity_name, entity_lei, status, business_unit_type, primary_contact, created_by) VALUES
('Alpha Investment Management', 'Alpha Investments LLC', '549300ABCDEF123456001', 'active', 'Investment Management', 'john.doe@alpha.com', 'system'),
('Beta Pension Fund', 'Beta Pension Solutions Inc', '549300GHIJKL789012002', 'active', 'Pension Fund', 'sarah.smith@beta.com', 'system'),
('Gamma Private Wealth', 'Gamma Wealth Partners', '549300MNOPQR345678003', 'active', 'Private Wealth', 'mike.johnson@gamma.com', 'system'),
('Delta Hedge Fund', 'Delta Capital Management', '549300STUVWX901234004', 'pending', 'Hedge Fund', 'lisa.chen@delta.com', 'system'),
('Epsilon Family Office', 'Epsilon Family Holdings', '549300YZABCD567890005', 'active', 'Family Office', 'david.brown@epsilon.com', 'system')
ON CONFLICT (cbu_name) DO NOTHING;

-- Insert comprehensive product catalog
INSERT INTO products (product_id, product_name, line_of_business, description, status, contract_type, commercial_status, pricing_model, target_market) VALUES
('PROD-CUSTODY-001', 'Institutional Custody Plus', 'Custody Services', 'Comprehensive custody services with enhanced reporting and analytics', 'active', 'Standard Service Agreement', 'Available', 'Asset-based', 'Institutional'),
('PROD-CUSTODY-002', 'Prime Brokerage Suite', 'Prime Brokerage', 'Full prime brokerage services including securities lending and financing', 'active', 'Prime Brokerage Agreement', 'Available', 'Revenue-based', 'Hedge Funds'),
('PROD-ADMIN-001', 'Fund Administration Pro', 'Fund Administration', 'Complete fund administration with NAV calculation and investor reporting', 'active', 'Administration Agreement', 'Available', 'Asset-based', 'Investment Funds'),
('PROD-TRADING-001', 'Execution Services Premium', 'Trading Services', 'Multi-asset execution platform with algorithmic trading capabilities', 'active', 'Execution Agreement', 'Available', 'Transaction-based', 'All Clients'),
('PROD-COMPLIANCE-001', 'Regulatory Compliance Suite', 'Compliance', 'Comprehensive compliance monitoring and reporting solution', 'active', 'Compliance Agreement', 'Available', 'Flat Fee', 'Regulated Entities'),
('PROD-WEALTH-001', 'Private Wealth Management', 'Wealth Management', 'Discretionary portfolio management for high-net-worth individuals', 'active', 'Investment Management Agreement', 'Available', 'Fee-based', 'Private Clients'),
('PROD-PENSION-001', 'Pension Fund Solutions', 'Pension Administration', 'Specialized administration services for pension and retirement funds', 'active', 'Pension Service Agreement', 'Available', 'Participant-based', 'Pension Funds')
ON CONFLICT (product_id) DO NOTHING;

-- Insert service definitions (public lifecycle descriptions)
INSERT INTO services (service_id, service_name, service_category, description, service_type, delivery_model, billable, status) VALUES
('SVC-SAFEKEEPING-001', 'Asset Safekeeping', 'Custody', 'Secure custody and safekeeping of financial instruments', 'Core', 'Standard', true, 'active'),
('SVC-SETTLEMENT-001', 'Trade Settlement', 'Operations', 'Settlement of securities transactions across multiple markets', 'Core', 'Automated', true, 'active'),
('SVC-REPORTING-001', 'Portfolio Reporting', 'Reporting', 'Comprehensive portfolio valuation and performance reporting', 'Value-Added', 'Digital', true, 'active'),
('SVC-COMPLIANCE-001', 'Compliance Monitoring', 'Compliance', 'Real-time monitoring of investment guidelines and regulatory requirements', 'Essential', 'Automated', true, 'active'),
('SVC-PRICING-001', 'Security Pricing', 'Data Services', 'Daily pricing of securities and alternative investments', 'Core', 'Automated', true, 'active'),
('SVC-CASH-001', 'Cash Management', 'Treasury', 'Optimization of cash positions and liquidity management', 'Core', 'Standard', true, 'active'),
('SVC-FX-001', 'Foreign Exchange', 'Trading', 'Foreign exchange execution and hedging services', 'Specialized', 'Real-time', true, 'active'),
('SVC-LENDING-001', 'Securities Lending', 'Revenue Enhancement', 'Securities lending program to generate additional income', 'Optional', 'Program-based', true, 'active'),
('SVC-TAX-001', 'Tax Services', 'Tax', 'Tax reporting, reclaim processing, and withholding optimization', 'Specialized', 'Annual', true, 'active'),
('SVC-TRANSITION-001', 'Portfolio Transition', 'Transition Management', 'Efficient transition of investment portfolios', 'Project-based', 'Managed', true, 'active')
ON CONFLICT (service_id) DO NOTHING;

-- Create product-service mappings
INSERT INTO product_service_mapping (product_id, service_id, mapping_type, priority, created_by)
SELECT p.id, s.id, 'required', 1, 'system'
FROM products p, services s
WHERE (p.product_id = 'PROD-CUSTODY-001' AND s.service_id IN ('SVC-SAFEKEEPING-001', 'SVC-SETTLEMENT-001', 'SVC-REPORTING-001'))
   OR (p.product_id = 'PROD-CUSTODY-002' AND s.service_id IN ('SVC-SAFEKEEPING-001', 'SVC-SETTLEMENT-001', 'SVC-LENDING-001', 'SVC-FX-001'))
   OR (p.product_id = 'PROD-ADMIN-001' AND s.service_id IN ('SVC-PRICING-001', 'SVC-REPORTING-001', 'SVC-TAX-001', 'SVC-CASH-001'))
   OR (p.product_id = 'PROD-TRADING-001' AND s.service_id IN ('SVC-SETTLEMENT-001', 'SVC-FX-001', 'SVC-COMPLIANCE-001'))
   OR (p.product_id = 'PROD-COMPLIANCE-001' AND s.service_id IN ('SVC-COMPLIANCE-001', 'SVC-REPORTING-001'))
   OR (p.product_id = 'PROD-WEALTH-001' AND s.service_id IN ('SVC-SAFEKEEPING-001', 'SVC-REPORTING-001', 'SVC-CASH-001', 'SVC-TAX-001'))
   OR (p.product_id = 'PROD-PENSION-001' AND s.service_id IN ('SVC-SAFEKEEPING-001', 'SVC-REPORTING-001', 'SVC-COMPLIANCE-001', 'SVC-TAX-001'))
ON CONFLICT (product_id, service_id) DO NOTHING;

-- Insert resource sheets (private implementations)
INSERT INTO resource_sheets (resource_id, display_name, description, resource_type, json_data, capabilities, status, visibility) VALUES
('RES-VAULT-NYC-001', 'NYC Physical Vault', 'Physical custody vault in New York', 'Physical Infrastructure',
 '{"location": "New York", "capacity": "unlimited", "security_level": "Grade 5", "operational_hours": "24/7"}',
 '{"safekeeping": true, "segregation": true, "emergency_access": true}', 'active', 'private'),

('RES-STP-ENGINE-001', 'STP Settlement Engine', 'Straight-through processing engine for trade settlement', 'Technology Platform',
 '{"platform": "proprietary", "throughput": "100000_trades_per_day", "latency": "sub_second", "uptime": "99.99%"}',
 '{"trade_settlement": true, "exception_handling": true, "reconciliation": true}', 'active', 'private'),

('RES-REPORT-GEN-001', 'Portfolio Reporting Generator', 'Automated reporting system for client portfolios', 'Software System',
 '{"report_types": ["performance", "holdings", "transactions", "compliance"], "frequency": "daily", "formats": ["PDF", "Excel", "API"]}',
 '{"report_generation": true, "data_aggregation": true, "client_portal": true}', 'active', 'private'),

('RES-COMPLIANCE-MON-001', 'Real-time Compliance Monitor', 'Automated compliance monitoring and alerting system', 'Monitoring System',
 '{"monitoring_types": ["investment_guidelines", "regulatory_limits", "risk_metrics"], "alert_latency": "real_time"}',
 '{"rule_monitoring": true, "breach_detection": true, "workflow_automation": true}', 'active', 'private'),

('RES-PRICING-FEED-001', 'Global Pricing Feed', 'Consolidated pricing data from multiple vendors', 'Data Service',
 '{"vendors": ["Bloomberg", "Refinitiv", "ICE"], "coverage": "global", "update_frequency": "real_time", "historical_depth": "10_years"}',
 '{"price_discovery": true, "validation": true, "analytics": true}', 'active', 'private'),

('RES-CASH-SWEEP-001', 'Automated Cash Sweep', 'Intelligent cash management and optimization system', 'Financial System',
 '{"sweep_frequency": "daily", "optimization_algorithm": "yield_maximization", "currency_support": "multi_currency"}',
 '{"cash_optimization": true, "liquidity_management": true, "yield_enhancement": true}', 'active', 'private'),

('RES-FX-PLATFORM-001', 'FX Trading Platform', 'Multi-bank FX execution platform', 'Trading System',
 '{"execution_venues": ["spot", "forward", "ndf"], "liquidity_providers": 15, "execution_algos": ["TWAP", "VWAP", "Implementation_Shortfall"]}',
 '{"fx_execution": true, "hedging": true, "pre_trade_analytics": true}', 'active', 'private')
ON CONFLICT (resource_id) DO NOTHING;

-- Create service-resource mappings
INSERT INTO service_resource_mapping (service_id, resource_id, usage_type, resource_role, configuration_parameters, created_by)
SELECT s.id, r.resource_id, 'primary', 'core_implementation',
       '{"sla_tier": "tier1", "backup_required": true, "monitoring_level": "enhanced"}', 'system'
FROM services s, resource_sheets r
WHERE (s.service_id = 'SVC-SAFEKEEPING-001' AND r.resource_id = 'RES-VAULT-NYC-001')
   OR (s.service_id = 'SVC-SETTLEMENT-001' AND r.resource_id = 'RES-STP-ENGINE-001')
   OR (s.service_id = 'SVC-REPORTING-001' AND r.resource_id = 'RES-REPORT-GEN-001')
   OR (s.service_id = 'SVC-COMPLIANCE-001' AND r.resource_id = 'RES-COMPLIANCE-MON-001')
   OR (s.service_id = 'SVC-PRICING-001' AND r.resource_id = 'RES-PRICING-FEED-001')
   OR (s.service_id = 'SVC-CASH-001' AND r.resource_id = 'RES-CASH-SWEEP-001')
   OR (s.service_id = 'SVC-FX-001' AND r.resource_id = 'RES-FX-PLATFORM-001')
ON CONFLICT (service_id, resource_id) DO NOTHING;

-- Insert comprehensive resource capabilities for DSL testing
INSERT INTO resource_capabilities (capability_id, capability_name, description, capability_type, required_attributes, execution_timeout_seconds, retry_policy, created_by) VALUES
('CAP-ACCOUNT-SETUP', 'Account Setup', 'Complete client account setup and initialization', 'setup',
 '{"client_id": "string", "cbu_id": "string", "account_type": "string", "jurisdiction": "string", "base_currency": "string"}',
 300, '{"max_retries": 3, "backoff_strategy": "exponential"}', 'system'),

('CAP-KYC-VERIFICATION', 'KYC Verification', 'Know Your Customer verification and compliance checks', 'compliance',
 '{"client_id": "string", "entity_type": "string", "jurisdiction": "string", "risk_profile": "string"}',
 600, '{"max_retries": 2, "backoff_strategy": "linear"}', 'system'),

('CAP-ONBOARD-CUSTODY', 'Custody Onboarding', 'Initialize custody services for new client', 'onboarding',
 '{"account_id": "string", "custody_products": "array", "segregation_model": "string", "reporting_frequency": "string"}',
 900, '{"max_retries": 5, "backoff_strategy": "exponential"}', 'system'),

('CAP-TRADE-FEED-SETUP', 'Trade Feed Setup', 'Configure trade feeds and settlement instructions', 'configuration',
 '{"account_id": "string", "settlement_instructions": "object", "trade_feeds": "array", "counterparties": "array"}',
 450, '{"max_retries": 3, "backoff_strategy": "exponential"}', 'system'),

('CAP-REPORTING-CONFIG', 'Reporting Configuration', 'Set up client reporting preferences and schedules', 'configuration',
 '{"account_id": "string", "report_types": "array", "delivery_method": "string", "frequency": "string", "recipients": "array"}',
 180, '{"max_retries": 2, "backoff_strategy": "linear"}', 'system'),

('CAP-COMPLIANCE-SETUP', 'Compliance Rules Setup', 'Configure investment guidelines and compliance monitoring', 'compliance',
 '{"account_id": "string", "investment_guidelines": "object", "risk_limits": "object", "monitoring_frequency": "string"}',
 360, '{"max_retries": 3, "backoff_strategy": "exponential"}', 'system'),

('CAP-CASH-MANAGEMENT', 'Cash Management Setup', 'Initialize cash management and sweep configurations', 'financial',
 '{"account_id": "string", "sweep_config": "object", "cash_targets": "object", "yield_preferences": "object"}',
 240, '{"max_retries": 3, "backoff_strategy": "exponential"}', 'system'),

('CAP-VALIDATE-SETUP', 'Setup Validation', 'Validate complete account setup and readiness', 'validation',
 '{"account_id": "string", "validation_checklist": "array", "dependencies": "array"}',
 120, '{"max_retries": 2, "backoff_strategy": "linear"}', 'system'),

('CAP-ACTIVATE-SERVICES', 'Service Activation', 'Activate all configured services for client', 'activation',
 '{"account_id": "string", "services_to_activate": "array", "activation_schedule": "object"}',
 600, '{"max_retries": 5, "backoff_strategy": "exponential"}', 'system'),

('CAP-HEALTH-CHECK', 'System Health Check', 'Comprehensive health check of all client services', 'monitoring',
 '{"account_id": "string", "check_types": "array", "threshold_config": "object"}',
 180, '{"max_retries": 1, "backoff_strategy": "none"}', 'system')
ON CONFLICT (capability_id) DO NOTHING;

-- Create enhanced resource templates with comprehensive DSL workflows
INSERT INTO resource_templates (template_id, template_name, description, part_of_product, implements_service, capabilities, metadata, dsl_template, created_by) VALUES
('TMPL-CUSTODY-FULL-001', 'Full Custody Onboarding', 'Complete custody service onboarding workflow', 'PROD-CUSTODY-001', 'SVC-SAFEKEEPING-001',
 '["CAP-ACCOUNT-SETUP", "CAP-KYC-VERIFICATION", "CAP-ONBOARD-CUSTODY", "CAP-REPORTING-CONFIG", "CAP-VALIDATE-SETUP", "CAP-ACTIVATE-SERVICES"]',
 '{"priority": "high", "estimated_duration_hours": 4, "business_criticality": "essential", "rollback_supported": true}',
 'WORKFLOW "FullCustodyOnboarding"
PHASE "PreOnboarding"
    EXECUTE_CAPABILITY "CAP-KYC-VERIFICATION" WITH client_verification
    VALIDATE_RESULT MUST_BE "approved"
PHASE "AccountSetup"
    EXECUTE_CAPABILITY "CAP-ACCOUNT-SETUP" WITH account_config
    EXECUTE_CAPABILITY "CAP-ONBOARD-CUSTODY" WITH custody_config
    EXECUTE_CAPABILITY "CAP-REPORTING-CONFIG" WITH reporting_preferences
PHASE "Validation"
    EXECUTE_CAPABILITY "CAP-VALIDATE-SETUP" WITH validation_criteria
    IF validation_result.status == "failed"
        ROLLBACK_TO "AccountSetup"
    ENDIF
PHASE "Activation"
    EXECUTE_CAPABILITY "CAP-ACTIVATE-SERVICES" WITH service_activation_plan
    EXECUTE_CAPABILITY "CAP-HEALTH-CHECK" WITH health_check_config
    SET_STATUS "operational" FOR account_id', 'system'),

('TMPL-PRIME-BROKERAGE-001', 'Prime Brokerage Onboarding', 'Comprehensive prime brokerage setup with trading capabilities', 'PROD-CUSTODY-002', 'SVC-LENDING-001',
 '["CAP-ACCOUNT-SETUP", "CAP-KYC-VERIFICATION", "CAP-ONBOARD-CUSTODY", "CAP-TRADE-FEED-SETUP", "CAP-CASH-MANAGEMENT", "CAP-COMPLIANCE-SETUP", "CAP-ACTIVATE-SERVICES"]',
 '{"priority": "high", "estimated_duration_hours": 6, "business_criticality": "essential", "complexity": "high"}',
 'WORKFLOW "PrimeBrokerageOnboarding"
PHASE "ClientValidation"
    EXECUTE_CAPABILITY "CAP-KYC-VERIFICATION" WITH enhanced_due_diligence
    VALIDATE_RESULT MUST_BE "approved"
PHASE "CoreSetup"
    EXECUTE_CAPABILITY "CAP-ACCOUNT-SETUP" WITH prime_brokerage_config
    EXECUTE_CAPABILITY "CAP-ONBOARD-CUSTODY" WITH segregated_custody_config
    EXECUTE_CAPABILITY "CAP-TRADE-FEED-SETUP" WITH trading_infrastructure
PHASE "AdvancedServices"
    EXECUTE_CAPABILITY "CAP-CASH-MANAGEMENT" WITH financing_config
    EXECUTE_CAPABILITY "CAP-COMPLIANCE-SETUP" WITH margin_monitoring
PHASE "GoLive"
    EXECUTE_CAPABILITY "CAP-VALIDATE-SETUP" WITH comprehensive_testing
    EXECUTE_CAPABILITY "CAP-ACTIVATE-SERVICES" WITH phased_activation
    EXECUTE_CAPABILITY "CAP-HEALTH-CHECK" WITH continuous_monitoring
    SET_STATUS "trading_ready" FOR account_id', 'system'),

('TMPL-FUND-ADMIN-001', 'Fund Administration Setup', 'Complete fund administration service initialization', 'PROD-ADMIN-001', 'SVC-REPORTING-001',
 '["CAP-ACCOUNT-SETUP", "CAP-KYC-VERIFICATION", "CAP-REPORTING-CONFIG", "CAP-COMPLIANCE-SETUP", "CAP-CASH-MANAGEMENT", "CAP-ACTIVATE-SERVICES"]',
 '{"priority": "medium", "estimated_duration_hours": 3, "business_criticality": "important", "regulatory_impact": "high"}',
 'WORKFLOW "FundAdministrationSetup"
PHASE "FundValidation"
    EXECUTE_CAPABILITY "CAP-KYC-VERIFICATION" WITH fund_entity_verification
PHASE "AdministrationSetup"
    EXECUTE_CAPABILITY "CAP-ACCOUNT-SETUP" WITH fund_admin_config
    EXECUTE_CAPABILITY "CAP-REPORTING-CONFIG" WITH nav_reporting_setup
    EXECUTE_CAPABILITY "CAP-COMPLIANCE-SETUP" WITH regulatory_reporting
PHASE "OperationalSetup"
    EXECUTE_CAPABILITY "CAP-CASH-MANAGEMENT" WITH fund_cash_management
    EXECUTE_CAPABILITY "CAP-VALIDATE-SETUP" WITH operational_readiness
PHASE "ServiceActivation"
    EXECUTE_CAPABILITY "CAP-ACTIVATE-SERVICES" WITH admin_services
    SET_STATUS "administration_active" FOR fund_id', 'system'),

('TMPL-WEALTH-MGMT-001', 'Private Wealth Onboarding', 'High-touch private wealth management setup', 'PROD-WEALTH-001', 'SVC-SAFEKEEPING-001',
 '["CAP-ACCOUNT-SETUP", "CAP-KYC-VERIFICATION", "CAP-ONBOARD-CUSTODY", "CAP-REPORTING-CONFIG", "CAP-CASH-MANAGEMENT", "CAP-ACTIVATE-SERVICES"]',
 '{"priority": "high", "estimated_duration_hours": 2, "business_criticality": "relationship", "white_glove": true}',
 'WORKFLOW "PrivateWealthOnboarding"
PHASE "ClientOnboarding"
    EXECUTE_CAPABILITY "CAP-KYC-VERIFICATION" WITH private_client_verification
    VALIDATE_RESULT MUST_BE "approved"
PHASE "AccountInitialization"
    EXECUTE_CAPABILITY "CAP-ACCOUNT-SETUP" WITH wealth_management_config
    EXECUTE_CAPABILITY "CAP-ONBOARD-CUSTODY" WITH private_custody_setup
PHASE "ServiceConfiguration"
    EXECUTE_CAPABILITY "CAP-REPORTING-CONFIG" WITH personalized_reporting
    EXECUTE_CAPABILITY "CAP-CASH-MANAGEMENT" WITH yield_optimization
PHASE "RelationshipActivation"
    EXECUTE_CAPABILITY "CAP-VALIDATE-SETUP" WITH relationship_readiness
    EXECUTE_CAPABILITY "CAP-ACTIVATE-SERVICES" WITH concierge_activation
    SET_STATUS "relationship_active" FOR client_id', 'system'),

('TMPL-PENSION-FUND-001', 'Pension Fund Onboarding', 'Specialized pension fund administration setup', 'PROD-PENSION-001', 'SVC-COMPLIANCE-001',
 '["CAP-ACCOUNT-SETUP", "CAP-KYC-VERIFICATION", "CAP-ONBOARD-CUSTODY", "CAP-REPORTING-CONFIG", "CAP-COMPLIANCE-SETUP", "CAP-ACTIVATE-SERVICES"]',
 '{"priority": "medium", "estimated_duration_hours": 5, "business_criticality": "fiduciary", "regulatory_complexity": "high"}',
 'WORKFLOW "PensionFundOnboarding"
PHASE "FiduciaryValidation"
    EXECUTE_CAPABILITY "CAP-KYC-VERIFICATION" WITH pension_fund_verification
    VALIDATE_RESULT MUST_BE "fiduciary_approved"
PHASE "PensionSetup"
    EXECUTE_CAPABILITY "CAP-ACCOUNT-SETUP" WITH pension_fund_config
    EXECUTE_CAPABILITY "CAP-ONBOARD-CUSTODY" WITH fiduciary_custody
    EXECUTE_CAPABILITY "CAP-COMPLIANCE-SETUP" WITH pension_regulations
PHASE "ReportingSetup"
    EXECUTE_CAPABILITY "CAP-REPORTING-CONFIG" WITH pension_reporting
    VALIDATE_RESULT MUST_HAVE "actuarial_integration"
PHASE "ServiceLaunch"
    EXECUTE_CAPABILITY "CAP-VALIDATE-SETUP" WITH fiduciary_compliance
    EXECUTE_CAPABILITY "CAP-ACTIVATE-SERVICES" WITH pension_services
    SET_STATUS "pension_operational" FOR pension_fund_id', 'system')
ON CONFLICT (template_id) DO NOTHING;

-- Insert comprehensive onboarding workflows for testing
INSERT INTO onboarding_workflows (workflow_id, cbu_id, product_ids, priority, target_go_live_date, business_requirements, compliance_requirements, execution_plan, workflow_status, created_by) VALUES
('WF-ALPHA-CUSTODY-001', 1, ARRAY[1], 'high', '2024-03-15',
 '{"services_required": ["custody", "reporting", "settlement"], "aum_target": "5B_USD", "asset_classes": ["equity", "fixed_income", "alternatives"]}',
 '{"jurisdictions": ["US", "UK"], "regulations": ["SEC", "FCA"], "reporting_requirements": ["daily_nav", "monthly_performance"]}',
 '{"phases": ["validation", "setup", "testing", "golive"], "estimated_duration_days": 30, "critical_path": ["kyc", "account_setup", "system_integration"]}',
 'in_progress', 'system'),

('WF-BETA-PENSION-001', 2, ARRAY[7], 'medium', '2024-04-01',
 '{"fund_type": "defined_benefit", "participants": 50000, "asset_allocation": {"equity": 60, "fixed_income": 30, "alternatives": 10}}',
 '{"regulations": ["ERISA", "DOL"], "fiduciary_requirements": true, "audit_frequency": "annual"}',
 '{"phases": ["fiduciary_review", "pension_setup", "participant_migration", "activation"], "estimated_duration_days": 45}',
 'pending', 'system'),

('WF-GAMMA-WEALTH-001', 3, ARRAY[6], 'high', '2024-02-28',
 '{"relationship_type": "family_office", "complexity": "high", "custom_reporting": true, "concierge_services": true}',
 '{"privacy_requirements": "enhanced", "regulatory_oversight": ["SEC_RIA"], "suitability_documentation": "detailed"}',
 '{"phases": ["relationship_establishment", "wealth_assessment", "service_customization", "relationship_activation"], "estimated_duration_days": 14}',
 'completed', 'system'),

('WF-DELTA-PRIME-001', 4, ARRAY[2], 'high', '2024-05-01',
 '{"trading_strategy": "multi_asset", "leverage_requirements": "up_to_4x", "funding_currency": "USD", "trading_hours": "24x7"}',
 '{"margin_requirements": "portfolio_margining", "risk_monitoring": "real_time", "stress_testing": "daily"}',
 '{"phases": ["credit_approval", "prime_setup", "trading_infrastructure", "risk_integration", "trading_activation"], "estimated_duration_days": 60}',
 'pending_approval', 'system'),

('WF-EPSILON-MULTI-001', 5, ARRAY[1,3,6], 'medium', '2024-06-15',
 '{"multi_product_setup": true, "consolidated_reporting": true, "relationship_management": "dedicated_team"}',
 '{"consolidated_compliance": true, "cross_product_monitoring": true, "unified_risk_reporting": true}',
 '{"phases": ["consolidated_onboarding", "product_integration", "unified_reporting", "relationship_optimization"], "estimated_duration_days": 90}',
 'planning', 'system')
ON CONFLICT (workflow_id) DO NOTHING;

-- Insert sample onboarding resource tasks for workflow dependency testing
INSERT INTO onboarding_resource_tasks (workflow_id, resource_template_id, task_name, task_description, depends_on_task_ids, required_attributes, task_status, assigned_to, created_by) VALUES
-- Alpha Investment Management custody onboarding tasks
((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-ALPHA-CUSTODY-001'), 'TMPL-CUSTODY-FULL-001', 'Client KYC Verification', 'Complete enhanced KYC for institutional client', '{}',
 '{"client_type": "institutional", "entity_jurisdiction": "Delaware", "regulatory_status": "SEC_registered"}', 'in_progress', 'kyc_team', 'system'),

((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-ALPHA-CUSTODY-001'), 'TMPL-CUSTODY-FULL-001', 'Account Structure Setup', 'Configure multi-entity account structure', '{1}',
 '{"account_structure": "master_feeder", "segregation_level": "client_level", "reporting_currency": "USD"}', 'pending', 'account_setup_team', 'system'),

((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-ALPHA-CUSTODY-001'), 'TMPL-CUSTODY-FULL-001', 'Custody Configuration', 'Set up custody infrastructure and safekeeping', '{2}',
 '{"custody_model": "global_custody", "subcustodian_network": "tier1", "asset_classes": ["equity", "fixed_income", "derivatives"]}', 'pending', 'custody_team', 'system'),

-- Beta Pension Fund onboarding tasks
((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-BETA-PENSION-001'), 'TMPL-PENSION-FUND-001', 'Pension Fund Verification', 'Verify pension fund fiduciary status and compliance', '{}',
 '{"fund_type": "defined_benefit", "fiduciary_status": "plan_trustee", "participant_count": 50000}', 'pending', 'compliance_team', 'system'),

((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-BETA-PENSION-001'), 'TMPL-PENSION-FUND-001', 'Pension Administration Setup', 'Configure pension-specific administration', '{1}',
 '{"administration_model": "full_service", "actuarial_integration": true, "benefit_calculations": "automated"}', 'pending', 'pension_admin_team', 'system'),

-- Gamma Private Wealth onboarding tasks (completed)
((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-GAMMA-WEALTH-001'), 'TMPL-WEALTH-MGMT-001', 'Private Client Onboarding', 'High-touch private client relationship establishment', '{}',
 '{"relationship_type": "family_office", "aum_range": "100M_500M", "service_level": "white_glove"}', 'completed', 'relationship_manager', 'system'),

((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-GAMMA-WEALTH-001'), 'TMPL-WEALTH-MGMT-001', 'Wealth Management Configuration', 'Customize wealth management services', '{1}',
 '{"portfolio_management": "discretionary", "investment_strategy": "custom", "reporting_frequency": "weekly"}', 'completed', 'wealth_team', 'system')
ON CONFLICT DO NOTHING;

-- Insert workflow dependencies for complex dependency testing
INSERT INTO onboarding_workflow_dependencies (workflow_id, depends_on_workflow_id, dependency_type, created_by) VALUES
((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-EPSILON-MULTI-001'),
 (SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-GAMMA-WEALTH-001'), 'reference_implementation', 'system'),
((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-DELTA-PRIME-001'),
 (SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-ALPHA-CUSTODY-001'), 'infrastructure_sharing', 'system')
ON CONFLICT DO NOTHING;

-- Insert workflow approvals for approval workflow testing
INSERT INTO onboarding_workflow_approvals (workflow_id, approval_stage, required_approver_role, approval_criteria, approval_status, created_by) VALUES
((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-ALPHA-CUSTODY-001'), 'business_approval', 'relationship_manager',
 '{"aum_threshold": "1B_USD", "relationship_complexity": "institutional"}', 'approved', 'system'),
((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-ALPHA-CUSTODY-001'), 'risk_approval', 'risk_committee',
 '{"credit_rating": "investment_grade", "operational_risk": "low"}', 'approved', 'system'),
((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-ALPHA-CUSTODY-001'), 'compliance_approval', 'compliance_officer',
 '{"regulatory_clearance": "complete", "sanctions_screening": "clear"}', 'approved', 'system'),

((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-DELTA-PRIME-001'), 'credit_approval', 'credit_committee',
 '{"creditworthiness": "strong", "leverage_limit": "4x", "margin_requirements": "portfolio_margining"}', 'pending', 'system'),
((SELECT id FROM onboarding_workflows WHERE workflow_id = 'WF-DELTA-PRIME-001'), 'operational_approval', 'operations_head',
 '{"infrastructure_readiness": "validated", "risk_systems": "integrated"}', 'pending', 'system')
ON CONFLICT DO NOTHING;

-- Insert resource instances for active workflow testing
INSERT INTO resource_instances (instance_id, onboarding_request_id, template_id, status, instance_data, created_by) VALUES
('INST-ALPHA-CUSTODY-001', 'WF-ALPHA-CUSTODY-001', 'TMPL-CUSTODY-FULL-001', 'executing',
 '{"client_name": "Alpha Investment Management", "account_id": "ALPHA001", "custody_tier": "tier1",
   "business_logic_dsl": "CONFIGURE_SYSTEM \"custody_platform\" WITH {\"client_tier\": \"institutional\", \"aum\": \"5B_USD\"}\nACTIVATE \"safekeeping_service\"\nRUN_HEALTH_CHECK \"custody_readiness\"",
   "current_phase": "AccountSetup", "progress_percentage": 65}', 'system'),

('INST-GAMMA-WEALTH-001', 'WF-GAMMA-WEALTH-001', 'TMPL-WEALTH-MGMT-001', 'completed',
 '{"client_name": "Gamma Private Wealth", "account_id": "GAMMA001", "relationship_tier": "white_glove",
   "business_logic_dsl": "CONFIGURE_SYSTEM \"wealth_platform\" WITH {\"service_level\": \"premium\", \"relationship_model\": \"family_office\"}\nACTIVATE \"concierge_services\"\nSET_STATUS \"relationship_active\"",
   "completion_date": "2024-01-15", "client_satisfaction": "excellent"}', 'system'),

('INST-BETA-PENSION-001', 'WF-BETA-PENSION-001', 'TMPL-PENSION-FUND-001', 'pending',
 '{"fund_name": "Beta Pension Fund", "fund_id": "BETA001", "fund_type": "defined_benefit",
   "business_logic_dsl": "CONFIGURE_SYSTEM \"pension_administration\" WITH {\"fund_type\": \"defined_benefit\", \"participants\": 50000}\nWORKFLOW \"fiduciary_setup\" STEPS [\"compliance_validation\", \"administration_config\", \"reporting_setup\"]",
   "pending_reason": "awaiting_fiduciary_documentation"}', 'system')
ON CONFLICT (instance_id) DO NOTHING;

-- Insert DSL execution logs for execution monitoring and testing
INSERT INTO dsl_execution_logs (instance_id, execution_status, input_data, output_data, log_messages, error_details, execution_time_ms) VALUES
('INST-ALPHA-CUSTODY-001', 'success',
 '{"account_setup_config": {"client_tier": "institutional", "aum": "5B_USD", "base_currency": "USD"}}',
 '{"account_id": "ALPHA001", "custody_config": {"safekeeping_enabled": true, "reporting_configured": true}, "status": "account_ready"}',
 '["Started custody configuration", "Account structure validated", "Safekeeping service configured", "Initial health check passed"]',
 null, 2850),

('INST-GAMMA-WEALTH-001', 'success',
 '{"wealth_config": {"service_level": "premium", "relationship_model": "family_office", "aum": "250M_USD"}}',
 '{"client_id": "GAMMA001", "wealth_services": {"portfolio_management": "active", "concierge": "enabled"}, "status": "relationship_active"}',
 '["Wealth management platform configured", "Premium services activated", "Relationship manager assigned", "Client portal provisioned"]',
 null, 1750),

('INST-BETA-PENSION-001', 'failed',
 '{"pension_config": {"fund_type": "defined_benefit", "participants": 50000}}',
 '{}',
 '["Pension fund validation started", "Fiduciary documentation review initiated", "ERROR: Missing required fiduciary certifications"]',
 'Missing required fiduciary documentation: trustee certification and investment policy statement', 450)
ON CONFLICT DO NOTHING;

-- Create indexes for performance optimization during testing
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_onboarding_workflows_status ON onboarding_workflows(workflow_status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_onboarding_workflows_cbu ON onboarding_workflows(cbu_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_resource_instances_status ON resource_instances(status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_resource_instances_template ON resource_instances(template_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_dsl_execution_logs_status ON dsl_execution_logs(execution_status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_dsl_execution_logs_instance ON dsl_execution_logs(instance_id);

-- Insert sample data validation checks
INSERT INTO data_validation_results (validation_name, validation_status, details, created_at) VALUES
('entity_relationship_integrity', 'passed', 'All product-service-resource mappings are valid and complete', NOW()),
('workflow_dependency_validation', 'passed', 'All workflow dependencies are resolvable and acyclic', NOW()),
('capability_coverage_check', 'passed', 'All required capabilities are defined and mapped to resources', NOW()),
('dsl_syntax_validation', 'passed', 'All DSL templates have valid syntax and executable workflows', NOW()),
('data_consistency_check', 'passed', 'Cross-table data consistency validated across all entities', NOW())
ON CONFLICT (validation_name) DO UPDATE SET
    validation_status = EXCLUDED.validation_status,
    details = EXCLUDED.details,
    created_at = EXCLUDED.created_at;

-- Create a table for data validation results if it doesn't exist
CREATE TABLE IF NOT EXISTS data_validation_results (
    id SERIAL PRIMARY KEY,
    validation_name VARCHAR(255) UNIQUE NOT NULL,
    validation_status VARCHAR(50) NOT NULL,
    details TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Final summary of seeded data
DO $$
BEGIN
    RAISE NOTICE 'Data seeding completed successfully!';
    RAISE NOTICE 'Seeded data summary:';
    RAISE NOTICE '- CBUs: % records', (SELECT COUNT(*) FROM cbu);
    RAISE NOTICE '- Products: % records', (SELECT COUNT(*) FROM products);
    RAISE NOTICE '- Services: % records', (SELECT COUNT(*) FROM services);
    RAISE NOTICE '- Resource Sheets: % records', (SELECT COUNT(*) FROM resource_sheets);
    RAISE NOTICE '- Resource Capabilities: % records', (SELECT COUNT(*) FROM resource_capabilities);
    RAISE NOTICE '- Resource Templates: % records', (SELECT COUNT(*) FROM resource_templates);
    RAISE NOTICE '- Onboarding Workflows: % records', (SELECT COUNT(*) FROM onboarding_workflows);
    RAISE NOTICE '- Resource Instances: % records', (SELECT COUNT(*) FROM resource_instances);
    RAISE NOTICE '- DSL Execution Logs: % records', (SELECT COUNT(*) FROM dsl_execution_logs);
    RAISE NOTICE 'Database is ready for comprehensive DSL workflow testing!';
END $$;