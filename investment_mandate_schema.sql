-- Investment Mandate Sub-Schema
-- Extends taxonomy with investment mandates specifying instruments, volumes, and instruction formats
-- Links to Products/Services to define what investments can be made and how

-- Create instrument taxonomy table (industry standard classifications)
CREATE TABLE IF NOT EXISTS instrument_taxonomy (
    id SERIAL PRIMARY KEY,
    instrument_code VARCHAR(50) UNIQUE NOT NULL, -- ISO standard codes where applicable
    instrument_name VARCHAR(255) NOT NULL,
    instrument_class VARCHAR(100) NOT NULL, -- 'equity', 'fixed_income', 'derivative', 'alternative', 'cash_equivalent', 'fx'
    instrument_subclass VARCHAR(100), -- 'government_bond', 'corporate_bond', 'common_stock', 'preferred_stock', 'option', 'future', 'swap'

    -- Industry Standard Classifications
    asset_class VARCHAR(100), -- Broad categorization
    cfi_code VARCHAR(6), -- Classification of Financial Instruments (ISO 10962)
    isin_pattern VARCHAR(50), -- ISIN pattern for this instrument type
    fisn_code VARCHAR(50), -- Financial Instrument Short Name (ISO 18774)

    -- Market Classifications
    market_sector VARCHAR(100), -- 'equity', 'government', 'corporate', 'municipal', 'supranational'
    geography VARCHAR(100), -- 'domestic', 'international', 'emerging_markets', 'developed_markets'
    currency_denomination VARCHAR(10), -- 'single_currency', 'multi_currency', 'currency_hedged'

    -- Risk Classifications
    risk_category VARCHAR(50) DEFAULT 'medium', -- 'low', 'medium', 'high', 'complex'
    liquidity_classification VARCHAR(50), -- 'high_liquidity', 'medium_liquidity', 'low_liquidity', 'illiquid'
    credit_rating_required BOOLEAN DEFAULT false,

    -- Regulatory Classifications
    regulatory_category VARCHAR(100), -- 'mifid_complex', 'mifid_non_complex', 'ucits_eligible', 'aif_only'
    professional_investor_only BOOLEAN DEFAULT false,
    regulatory_capital_treatment VARCHAR(100),

    -- Trading Characteristics
    typical_lot_size DECIMAL(20,2),
    minimum_denomination DECIMAL(20,2),
    settlement_cycle VARCHAR(10), -- 'T+0', 'T+1', 'T+2', 'T+3', 'custom'

    -- Metadata
    active BOOLEAN DEFAULT true,
    effective_date DATE DEFAULT CURRENT_DATE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create investment mandate table
CREATE TABLE IF NOT EXISTS investment_mandates (
    id SERIAL PRIMARY KEY,
    mandate_id VARCHAR(100) UNIQUE NOT NULL,
    mandate_name VARCHAR(255) NOT NULL,
    mandate_type VARCHAR(100) NOT NULL, -- 'absolute_return', 'benchmark_relative', 'liability_driven', 'target_date', 'balanced'

    -- Mandate Ownership
    cbu_id INTEGER REFERENCES client_business_units(id) ON DELETE CASCADE, -- Primary link to CBU
    product_id INTEGER REFERENCES products(id) ON DELETE CASCADE,
    fund_entity_id VARCHAR(100), -- Links to legal_entities if this is fund-specific
    client_entity_id VARCHAR(100), -- Client this mandate serves

    -- Investment Objectives
    investment_objective TEXT,
    benchmark_index VARCHAR(100),
    target_return_annual DECIMAL(6,3), -- Percentage
    risk_tolerance VARCHAR(50), -- 'conservative', 'moderate', 'aggressive', 'speculative'
    investment_horizon_years INTEGER,

    -- Geographic and Currency Constraints
    geographic_focus VARCHAR(100), -- 'global', 'regional', 'domestic', 'emerging', 'developed'
    base_currency CHAR(3), -- ISO 4217
    currency_hedging_policy VARCHAR(100), -- 'fully_hedged', 'partially_hedged', 'unhedged', 'opportunistic'
    foreign_exchange_limit DECIMAL(5,2), -- Percentage

    -- ESG and Sustainability
    esg_policy VARCHAR(100), -- 'none', 'exclusions', 'integration', 'best_in_class', 'impact'
    sustainability_requirements JSONB,
    exclusion_criteria TEXT[],

    -- Liquidity Requirements
    liquidity_requirement VARCHAR(100), -- 'daily', 'weekly', 'monthly', 'quarterly', 'annual'
    minimum_liquidity_percentage DECIMAL(5,2),
    redemption_notice_period INTEGER, -- days

    -- Risk Management
    maximum_portfolio_volatility DECIMAL(6,3),
    value_at_risk_limit DECIMAL(10,2),
    concentration_limits JSONB, -- Various concentration rules

    -- Operational
    rebalancing_frequency VARCHAR(50), -- 'daily', 'weekly', 'monthly', 'quarterly', 'annual', 'tactical'
    reporting_frequency VARCHAR(50),
    benchmark_deviation_tolerance DECIMAL(5,2),

    -- Status and Lifecycle
    mandate_status VARCHAR(50) DEFAULT 'active' CHECK (mandate_status IN ('draft', 'active', 'suspended', 'terminated')),
    effective_date DATE,
    termination_date DATE,
    last_review_date DATE,
    next_review_date DATE,

    -- Metadata
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100),
    updated_by VARCHAR(100)
);

-- Create instrument allocation constraints within mandates
CREATE TABLE IF NOT EXISTS mandate_instrument_allocations (
    id SERIAL PRIMARY KEY,
    mandate_id INTEGER REFERENCES investment_mandates(id) ON DELETE CASCADE,
    instrument_id INTEGER REFERENCES instrument_taxonomy(id) ON DELETE CASCADE,
    allocation_type VARCHAR(50) NOT NULL, -- 'target', 'minimum', 'maximum', 'benchmark', 'prohibited'

    -- Allocation Constraints
    allocation_percentage DECIMAL(5,2), -- Target/Min/Max percentage
    allocation_amount DECIMAL(20,2), -- Absolute amount if applicable

    -- Volume Constraints
    minimum_position_size DECIMAL(20,2),
    maximum_position_size DECIMAL(20,2),
    maximum_daily_volume DECIMAL(20,2),
    maximum_single_issuer_exposure DECIMAL(5,2), -- Percentage

    -- Trading Constraints
    maximum_transaction_size DECIMAL(20,2),
    minimum_lot_multiple DECIMAL(20,2),
    allowed_venues TEXT[], -- Exchanges/venues where this can be traded

    -- Timing Constraints
    trading_hours_restriction VARCHAR(100),
    settlement_period_preference VARCHAR(20),
    trade_execution_time_limit INTEGER, -- minutes

    -- Quality Constraints
    minimum_credit_rating VARCHAR(10), -- For fixed income
    maximum_duration DECIMAL(6,3), -- For fixed income
    minimum_market_cap DECIMAL(15,2), -- For equities

    -- Approval Requirements
    requires_committee_approval BOOLEAN DEFAULT false,
    requires_client_approval BOOLEAN DEFAULT false,
    approval_threshold DECIMAL(20,2),

    -- Status
    active BOOLEAN DEFAULT true,
    effective_date DATE DEFAULT CURRENT_DATE,
    expiry_date DATE,

    -- Metadata
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(mandate_id, instrument_id, allocation_type)
);

-- Create instruction format standards table
CREATE TABLE IF NOT EXISTS instruction_formats (
    id SERIAL PRIMARY KEY,
    format_id VARCHAR(100) UNIQUE NOT NULL,
    format_name VARCHAR(255) NOT NULL,
    format_category VARCHAR(100), -- 'trade_instruction', 'settlement_instruction', 'corporate_action', 'cash_instruction'

    -- Format Specifications
    message_standard VARCHAR(100), -- 'FIX', 'SWIFT', 'ISO20022', 'proprietary', 'csv', 'xml', 'json'
    message_type VARCHAR(100), -- Specific message type within standard
    format_version VARCHAR(50),
    schema_definition JSONB, -- JSON schema or structure definition

    -- Content Requirements
    required_fields TEXT[] NOT NULL,
    optional_fields TEXT[],
    validation_rules JSONB,
    field_formats JSONB, -- Specific format requirements for each field

    -- Processing Characteristics
    processing_priority VARCHAR(50) DEFAULT 'normal', -- 'urgent', 'high', 'normal', 'low'
    expected_processing_time_minutes INTEGER,
    retry_policy JSONB,
    error_handling_procedure TEXT,

    -- Security and Compliance
    encryption_required BOOLEAN DEFAULT false,
    digital_signature_required BOOLEAN DEFAULT false,
    authorization_level_required VARCHAR(50),
    audit_trail_required BOOLEAN DEFAULT true,

    -- Technical Details
    max_message_size_kb INTEGER,
    character_encoding VARCHAR(50) DEFAULT 'UTF-8',
    timestamp_format VARCHAR(100),
    decimal_precision INTEGER DEFAULT 2,

    -- Delivery Methods
    supported_delivery_methods TEXT[], -- 'api', 'sftp', 'email', 'portal', 'message_queue'
    acknowledgment_required BOOLEAN DEFAULT true,

    -- Usage Context
    applicable_markets TEXT[],
    applicable_asset_classes TEXT[],
    regulatory_compliance TEXT[],

    -- Status
    status VARCHAR(50) DEFAULT 'active' CHECK (status IN ('draft', 'active', 'deprecated', 'retired')),
    effective_date DATE DEFAULT CURRENT_DATE,
    retirement_date DATE,

    -- Metadata
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100),
    updated_by VARCHAR(100)
);

-- Create mandate instruction format mappings
CREATE TABLE IF NOT EXISTS mandate_instruction_mappings (
    id SERIAL PRIMARY KEY,
    mandate_id INTEGER REFERENCES investment_mandates(id) ON DELETE CASCADE,
    instruction_format_id INTEGER REFERENCES instruction_formats(id) ON DELETE CASCADE,
    usage_context VARCHAR(100) NOT NULL, -- 'trade_execution', 'settlement', 'reporting', 'corporate_actions', 'cash_management'

    -- Format Configuration
    is_default_format BOOLEAN DEFAULT false,
    priority_order INTEGER DEFAULT 10,
    configuration_parameters JSONB,
    custom_field_mappings JSONB,

    -- Volume and Timing
    applicable_volume_threshold DECIMAL(20,2), -- Use this format above this threshold
    applicable_time_periods VARCHAR(100), -- When this format is used

    -- Status
    active BOOLEAN DEFAULT true,
    effective_date DATE DEFAULT CURRENT_DATE,

    -- Metadata
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(mandate_id, instruction_format_id, usage_context)
);

-- Enhanced view including investment mandates
CREATE OR REPLACE VIEW complete_investment_taxonomy_view AS
SELECT
    -- CBU Level
    cbu.id as cbu_internal_id,
    cbu.cbu_id,
    cbu.cbu_name,
    cbu.business_type,

    -- Product Level
    p.id as product_id,
    p.product_id as product_code,
    p.product_name,
    p.line_of_business,

    -- Investment Mandate Level
    im.id as mandate_id,
    im.mandate_id as mandate_code,
    im.mandate_name,
    im.mandate_type,
    im.investment_objective,
    im.risk_tolerance,
    im.base_currency,
    im.liquidity_requirement,

    -- Instrument Allocations
    it.id as instrument_id,
    it.instrument_code,
    it.instrument_name,
    it.instrument_class,
    it.asset_class,
    mia.allocation_type,
    mia.allocation_percentage,
    mia.maximum_position_size,

    -- Instruction Formats
    if_data.id as instruction_format_id,
    if_data.format_id as format_code,
    if_data.format_name,
    if_data.message_standard,
    mim.usage_context,

    -- Product Options
    po.option_name,
    po.option_category,

    -- Services
    s.service_name,
    s.service_type,

    -- Resources
    r.resource_name,
    r.resource_type,

    -- Summary Metrics
    COUNT(DISTINCT it.id) as total_instruments,
    COUNT(DISTINCT if_data.id) as total_instruction_formats,
    SUM(mia.allocation_percentage) FILTER (WHERE mia.allocation_type = 'target') as total_target_allocation,

    -- Metadata
    im.mandate_status,
    im.effective_date as mandate_effective_date,
    p.created_at as product_created_at

FROM client_business_units cbu
LEFT JOIN investment_mandates im ON im.cbu_id = cbu.id
LEFT JOIN products p ON p.id = im.product_id
LEFT JOIN mandate_instrument_allocations mia ON mia.mandate_id = im.id
LEFT JOIN instrument_taxonomy it ON it.id = mia.instrument_id
LEFT JOIN mandate_instruction_mappings mim ON mim.mandate_id = im.id
LEFT JOIN instruction_formats if_data ON if_data.id = mim.instruction_format_id
LEFT JOIN product_options po ON po.product_id = p.id
LEFT JOIN product_option_service_mappings posm ON posm.product_option_id = po.id
LEFT JOIN services s ON s.id = posm.service_id
LEFT JOIN service_resource_mappings srm ON srm.service_id = s.id
LEFT JOIN resource_objects r ON r.id = srm.resource_id
GROUP BY
    cbu.id, cbu.cbu_id, cbu.cbu_name, cbu.business_type,
    p.id, p.product_id, p.product_name, p.line_of_business, p.created_at,
    im.id, im.mandate_id, im.mandate_name, im.mandate_type, im.investment_objective,
    im.risk_tolerance, im.base_currency, im.liquidity_requirement, im.mandate_status, im.effective_date,
    it.id, it.instrument_code, it.instrument_name, it.instrument_class, it.asset_class,
    mia.allocation_type, mia.allocation_percentage, mia.maximum_position_size,
    if_data.id, if_data.format_id, if_data.format_name, if_data.message_standard,
    mim.usage_context, po.option_name, po.option_category,
    s.service_name, s.service_type, r.resource_name, r.resource_type;

-- Insert industry standard instrument taxonomy
INSERT INTO instrument_taxonomy (
    instrument_code, instrument_name, instrument_class, instrument_subclass,
    asset_class, cfi_code, market_sector, risk_category, liquidity_classification,
    regulatory_category, settlement_cycle, typical_lot_size
) VALUES

-- Equity Instruments
('EQ_CS', 'Common Stock', 'equity', 'common_stock', 'Equity', 'ESVUFR', 'equity', 'medium', 'high_liquidity', 'mifid_non_complex', 'T+2', 100.00),
('EQ_PS', 'Preferred Stock', 'equity', 'preferred_stock', 'Equity', 'EPVUFR', 'equity', 'medium', 'medium_liquidity', 'mifid_non_complex', 'T+2', 100.00),
('EQ_ETF', 'Exchange Traded Fund', 'equity', 'etf', 'Equity', 'EUVUFR', 'equity', 'medium', 'high_liquidity', 'ucits_eligible', 'T+2', 1.00),

-- Fixed Income Instruments
('FI_GB', 'Government Bond', 'fixed_income', 'government_bond', 'Fixed Income', 'DBVUFR', 'government', 'low', 'high_liquidity', 'mifid_non_complex', 'T+2', 1000.00),
('FI_CB', 'Corporate Bond', 'fixed_income', 'corporate_bond', 'Fixed Income', 'DBVUFR', 'corporate', 'medium', 'medium_liquidity', 'mifid_non_complex', 'T+2', 1000.00),
('FI_MB', 'Municipal Bond', 'fixed_income', 'municipal_bond', 'Fixed Income', 'DBVUFR', 'municipal', 'low', 'medium_liquidity', 'mifid_non_complex', 'T+2', 1000.00),

-- Derivative Instruments
('DV_OPT', 'Option Contract', 'derivative', 'option', 'Derivatives', 'OPVUFR', 'equity', 'high', 'medium_liquidity', 'mifid_complex', 'T+1', 1.00),
('DV_FUT', 'Future Contract', 'derivative', 'future', 'Derivatives', 'FFVUFR', 'equity', 'high', 'high_liquidity', 'mifid_complex', 'T+1', 1.00),
('DV_SWP', 'Swap Contract', 'derivative', 'swap', 'Derivatives', 'SPVUFR', 'fixed_income', 'high', 'low_liquidity', 'mifid_complex', 'T+2', 1000000.00),

-- Alternative Investments
('ALT_RE', 'Real Estate Investment', 'alternative', 'real_estate', 'Alternatives', 'MMVUFR', 'real_estate', 'medium', 'low_liquidity', 'aif_only', 'T+30', 50000.00),
('ALT_COM', 'Commodity', 'alternative', 'commodity', 'Alternatives', 'TCVUFR', 'commodity', 'high', 'medium_liquidity', 'mifid_complex', 'T+2', 1000.00),

-- Cash Equivalents
('CE_MM', 'Money Market Fund', 'cash_equivalent', 'money_market', 'Cash', 'MMVUFR', 'money_market', 'low', 'high_liquidity', 'ucits_eligible', 'T+1', 1.00),
('CE_CD', 'Certificate of Deposit', 'cash_equivalent', 'certificate_deposit', 'Cash', 'DBVUFR', 'money_market', 'low', 'low_liquidity', 'mifid_non_complex', 'T+0', 10000.00)

ON CONFLICT (instrument_code) DO UPDATE SET
    instrument_name = EXCLUDED.instrument_name,
    updated_at = CURRENT_TIMESTAMP;

-- Insert standard instruction formats
INSERT INTO instruction_formats (
    format_id, format_name, format_category, message_standard, message_type,
    required_fields, processing_priority, supported_delivery_methods,
    applicable_asset_classes, regulatory_compliance
) VALUES

('FIX_NEW_ORDER', 'FIX New Order Single', 'trade_instruction', 'FIX', 'NewOrderSingle',
 ARRAY['ClOrdID', 'Symbol', 'Side', 'OrderQty', 'OrdType'], 'high',
 ARRAY['api', 'message_queue'], ARRAY['equity', 'fixed_income'], ARRAY['MiFID2']),

('SWIFT_MT515', 'SWIFT Client Confirmation', 'settlement_instruction', 'SWIFT', 'MT515',
 ARRAY['TransactionReference', 'AccountIdentification', 'SettlementDetails'], 'normal',
 ARRAY['swift_network'], ARRAY['equity', 'fixed_income'], ARRAY['CSDR']),

('ISO20022_SEMT', 'ISO20022 Statement', 'settlement_instruction', 'ISO20022', 'semt.002.001.07',
 ARRAY['MessageId', 'AccountOwner', 'StatementPeriod'], 'normal',
 ARRAY['api', 'sftp'], ARRAY['equity', 'fixed_income', 'derivatives'], ARRAY['MiFID2', 'CSDR']),

('CSV_TRADE_LIST', 'CSV Trade List', 'trade_instruction', 'CSV', 'TradeList',
 ARRAY['TradeDate', 'Symbol', 'Quantity', 'Price', 'Side'], 'low',
 ARRAY['sftp', 'email'], ARRAY['equity'], ARRAY[]),

('JSON_PORTFOLIO', 'JSON Portfolio Report', 'reporting', 'JSON', 'PortfolioReport',
 ARRAY['portfolioId', 'reportDate', 'positions'], 'normal',
 ARRAY['api', 'portal'], ARRAY['equity', 'fixed_income'], ARRAY['MiFID2'])

ON CONFLICT (format_id) DO UPDATE SET
    format_name = EXCLUDED.format_name,
    updated_at = CURRENT_TIMESTAMP;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_instrument_taxonomy_class ON instrument_taxonomy(instrument_class);
CREATE INDEX IF NOT EXISTS idx_instrument_taxonomy_asset_class ON instrument_taxonomy(asset_class);
CREATE INDEX IF NOT EXISTS idx_instrument_taxonomy_risk ON instrument_taxonomy(risk_category);

CREATE INDEX IF NOT EXISTS idx_investment_mandates_cbu ON investment_mandates(cbu_id);
CREATE INDEX IF NOT EXISTS idx_investment_mandates_product ON investment_mandates(product_id);
CREATE INDEX IF NOT EXISTS idx_investment_mandates_type ON investment_mandates(mandate_type);
CREATE INDEX IF NOT EXISTS idx_investment_mandates_status ON investment_mandates(mandate_status);

CREATE INDEX IF NOT EXISTS idx_mandate_instrument_allocations_mandate ON mandate_instrument_allocations(mandate_id);
CREATE INDEX IF NOT EXISTS idx_mandate_instrument_allocations_instrument ON mandate_instrument_allocations(instrument_id);

CREATE INDEX IF NOT EXISTS idx_instruction_formats_category ON instruction_formats(format_category);
CREATE INDEX IF NOT EXISTS idx_instruction_formats_standard ON instruction_formats(message_standard);

CREATE INDEX IF NOT EXISTS idx_mandate_instruction_mappings_mandate ON mandate_instruction_mappings(mandate_id);
CREATE INDEX IF NOT EXISTS idx_mandate_instruction_mappings_format ON mandate_instruction_mappings(instruction_format_id);

-- Comments
COMMENT ON TABLE instrument_taxonomy IS 'Industry standard instrument classifications with regulatory and trading characteristics';
COMMENT ON TABLE investment_mandates IS 'Investment mandates defining objectives, constraints, and policies for portfolio management';
COMMENT ON TABLE mandate_instrument_allocations IS 'Instrument allocation constraints and volume limits within investment mandates';
COMMENT ON TABLE instruction_formats IS 'Standard message formats for trading, settlement, and reporting instructions';
COMMENT ON TABLE mandate_instruction_mappings IS 'Maps investment mandates to their applicable instruction formats';
COMMENT ON VIEW complete_investment_taxonomy_view IS 'Comprehensive view of Products→Mandates→Instruments/Formats→Options→Services→Resources';

-- Summary
DO $$
DECLARE
    instrument_count INTEGER;
    format_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO instrument_count FROM instrument_taxonomy WHERE active = true;
    SELECT COUNT(*) INTO format_count FROM instruction_formats WHERE status = 'active';

    RAISE NOTICE '📊 Investment Mandate Sub-Schema Created:';
    RAISE NOTICE '   🎯 Instrument Taxonomy Entries: %', instrument_count;
    RAISE NOTICE '   📋 Instruction Formats: %', format_count;
    RAISE NOTICE '';
    RAISE NOTICE '🏗️  Complete Taxonomy: Products → Options → Mandates → Instruments/Formats → Services → Resources';
END $$;