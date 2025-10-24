-- Migration 002: Configuration-Driven UI Schema
-- Adds support for multi-layered resource configuration with perspective-based rendering

-- Resource dictionaries (collections of resources)
CREATE TABLE resource_dictionaries (
    id SERIAL PRIMARY KEY,
    dictionary_name VARCHAR(100) UNIQUE NOT NULL,
    version VARCHAR(20) NOT NULL DEFAULT '1.0',
    description TEXT,
    author VARCHAR(100),
    creation_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'deprecated')),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Resource objects (forms/interfaces within a dictionary)
CREATE TABLE resource_objects (
    id SERIAL PRIMARY KEY,
    dictionary_id INTEGER REFERENCES resource_dictionaries(id) ON DELETE CASCADE,
    resource_name VARCHAR(100) NOT NULL,
    description TEXT,
    version VARCHAR(20) NOT NULL DEFAULT '1.0',
    category VARCHAR(50),
    owner_team VARCHAR(100),
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'deprecated')),

    -- UI configuration
    ui_layout VARCHAR(30) NOT NULL DEFAULT 'vertical-stack'
        CHECK (ui_layout IN ('wizard', 'tabs', 'vertical-stack', 'horizontal-grid', 'accordion')),
    group_order TEXT[], -- Array of group names in display order

    -- UI navigation settings
    navigation_config JSONB DEFAULT '{}'::jsonb,

    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(dictionary_id, resource_name)
);

-- Attribute objects (fields within resources)
CREATE TABLE attribute_objects (
    id SERIAL PRIMARY KEY,
    resource_id INTEGER REFERENCES resource_objects(id) ON DELETE CASCADE,
    attribute_name VARCHAR(100) NOT NULL,
    data_type VARCHAR(30) NOT NULL
        CHECK (data_type IN ('String', 'Number', 'Boolean', 'Date', 'Decimal', 'List', 'Enum')),
    description TEXT,

    -- Constraints
    is_required BOOLEAN DEFAULT false,
    min_length INTEGER,
    max_length INTEGER,
    min_value NUMERIC,
    max_value NUMERIC,
    allowed_values JSONB, -- For enum types
    validation_pattern TEXT,

    -- Persistence information
    persistence_system VARCHAR(100),
    persistence_entity VARCHAR(100),
    persistence_identifier VARCHAR(100),

    -- UI configuration
    ui_group VARCHAR(100),
    ui_display_order INTEGER DEFAULT 0,
    ui_render_hint VARCHAR(30) DEFAULT 'text-input',
    ui_label VARCHAR(200),
    ui_help_text TEXT,

    -- Wizard-specific UI settings
    wizard_step INTEGER,
    wizard_step_title VARCHAR(200),
    wizard_next_button TEXT,
    wizard_previous_button TEXT,
    wizard_description TEXT,

    -- Generation examples for AI
    generation_examples JSONB DEFAULT '[]'::jsonb,

    -- Rules DSL for computed attributes
    rules_dsl TEXT,

    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(resource_id, attribute_name)
);

-- Attribute perspectives (different views of the same attribute)
CREATE TABLE attribute_perspectives (
    id SERIAL PRIMARY KEY,
    attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    perspective_name VARCHAR(50) NOT NULL,
    description TEXT,

    -- Overridden UI configuration for this perspective
    ui_group VARCHAR(100),
    ui_label VARCHAR(200),
    ui_help_text TEXT,

    -- Perspective-specific generation examples
    generation_examples JSONB DEFAULT '[]'::jsonb,

    UNIQUE(attribute_id, perspective_name)
);

-- UI layout groups (for organizing fields within resources)
CREATE TABLE ui_layout_groups (
    id SERIAL PRIMARY KEY,
    resource_id INTEGER REFERENCES resource_objects(id) ON DELETE CASCADE,
    group_name VARCHAR(100) NOT NULL,
    display_order INTEGER DEFAULT 0,

    -- Group styling and behavior
    is_collapsible BOOLEAN DEFAULT false,
    initially_collapsed BOOLEAN DEFAULT false,
    group_description TEXT,

    UNIQUE(resource_id, group_name)
);

-- Indexes for performance
CREATE INDEX idx_resource_dictionaries_name ON resource_dictionaries(dictionary_name);
CREATE INDEX idx_resource_objects_dictionary ON resource_objects(dictionary_id);
CREATE INDEX idx_resource_objects_name ON resource_objects(resource_name);
CREATE INDEX idx_resource_objects_category ON resource_objects(category);
CREATE INDEX idx_attribute_objects_resource ON attribute_objects(resource_id);
CREATE INDEX idx_attribute_objects_name ON attribute_objects(attribute_name);
CREATE INDEX idx_attribute_objects_group ON attribute_objects(ui_group);
CREATE INDEX idx_attribute_perspectives_attr ON attribute_perspectives(attribute_id);
CREATE INDEX idx_attribute_perspectives_name ON attribute_perspectives(perspective_name);
CREATE INDEX idx_ui_layout_groups_resource ON ui_layout_groups(resource_id);

-- Update triggers for timestamp fields
CREATE TRIGGER update_resource_dictionaries_updated_at
    BEFORE UPDATE ON resource_dictionaries
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_resource_objects_updated_at
    BEFORE UPDATE ON resource_objects
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_attribute_objects_updated_at
    BEFORE UPDATE ON attribute_objects
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert sample data based on the configuration-driven schema
INSERT INTO resource_dictionaries (dictionary_name, version, description, author) VALUES
('Financial Services Data Dictionary', '2.0', 'Comprehensive data dictionary for financial services applications', 'Data Architecture Team');

-- Get the dictionary ID for sample data
INSERT INTO resource_objects (
    dictionary_id, resource_name, description, version, ui_layout,
    group_order, category, owner_team, status
) VALUES
((SELECT id FROM resource_dictionaries WHERE dictionary_name = 'Financial Services Data Dictionary'),
 'ClientOnboardingKYC',
 'Manages the data attributes required for the full Know Your Customer due diligence process.',
 '2.0',
 'wizard',
 ARRAY['Client Entity Details', 'Beneficial Owner Details', 'Sanctions Screening Results', 'Risk Assessment'],
 'Compliance',
 'KYC Team',
 'active'),

((SELECT id FROM resource_dictionaries WHERE dictionary_name = 'Financial Services Data Dictionary'),
 'TradeSettlementSystem',
 'Configuration for trade settlement and clearing system integration.',
 '2.0',
 'tabs',
 ARRAY['Trade Details', 'Settlement Instructions', 'Counterparty Information', 'Regulatory Reporting'],
 'Trading',
 'Trading Operations',
 'active');

-- Insert sample attribute objects for ClientOnboardingKYC
INSERT INTO attribute_objects (
    resource_id, attribute_name, data_type, description, is_required,
    min_length, max_length, persistence_system, persistence_entity, persistence_identifier,
    ui_group, ui_display_order, ui_render_hint, ui_label, ui_help_text,
    wizard_step, wizard_step_title, wizard_next_button, wizard_description,
    generation_examples
) VALUES
-- Legal Entity Name
((SELECT id FROM resource_objects WHERE resource_name = 'ClientOnboardingKYC'),
 'legal_entity_name', 'String',
 'The official registered name of the legal entity as it appears in corporate registry documents.',
 true, 2, 200, 'EntityMasterDB', 'legal_entities', 'entity_name',
 'Client Entity Details', 1, 'text-input', 'Legal Entity Name',
 'Enter the exact name as registered with corporate authorities',
 1, 'Entity Information', 'Proceed to UBO Details',
 'Provide basic information about the legal entity',
 '[{"prompt": "Check if the entity name contains restricted words", "response": "RULE \"Check-Entity-Name\" IF legal_entity_name CONTAINS ''Bank'' AND NOT licensed_as_bank THEN ALERT ''Entity name suggests banking activity but no banking license''"}]'::jsonb),

-- UBO Full Name
((SELECT id FROM resource_objects WHERE resource_name = 'ClientOnboardingKYC'),
 'ubo_full_name', 'String',
 'The full legal name of the Ultimate Beneficial Owner, as defined by AML regulations.',
 true, 2, 100, 'EntityMasterDB', 'related_parties', 'full_name',
 'Beneficial Owner Details', 1, 'text-input', 'UBO Full Name', null,
 2, 'Ultimate Beneficial Owner', 'Proceed to Screening',
 null,
 '[{"prompt": "Check if the beneficial owner''s name matches ''John Smith''.", "response": "RULE \"Match-UBO-Name\" IF ubo_full_name == ''John Smith'' THEN SET match_found = true"}]'::jsonb),

-- Sanctions Screening Result
((SELECT id FROM resource_objects WHERE resource_name = 'ClientOnboardingKYC'),
 'sanctions_screening_result', 'Enum',
 'The result of sanctions screening performed against global watchlists.',
 true, null, null, 'ComplianceDB', 'screening_results', 'result_status',
 'Sanctions Screening Results', 1, 'select', 'Screening Result', null,
 3, 'Compliance Screening', 'Proceed to Risk Assessment',
 null,
 '[{"prompt": "Create a rule to handle confirmed sanctions matches", "response": "RULE \"Sanctions-Match\" IF sanctions_screening_result == ''CONFIRMED_MATCH'' THEN BLOCK_ONBOARDING AND ALERT ''Sanctions match detected''"}]'::jsonb),

-- Risk Score
((SELECT id FROM resource_objects WHERE resource_name = 'ClientOnboardingKYC'),
 'risk_score', 'Number',
 'Computed risk score based on multiple risk factors, ranging from 1 (low risk) to 100 (high risk).',
 true, null, null, 'RiskDB', 'risk_assessments', 'computed_score',
 'Risk Assessment', 1, 'number-input', 'Risk Score',
 'Automatically calculated based on risk factors',
 4, 'Risk Assessment', 'Complete Onboarding',
 null,
 '[{"prompt": "Create a rule for high-risk clients that require additional due diligence", "response": "RULE \"High-Risk-EDD\" IF risk_score > 70 THEN REQUIRE enhanced_due_diligence AND NOTIFY compliance_team"}]'::jsonb),

-- Jurisdiction
((SELECT id FROM resource_objects WHERE resource_name = 'ClientOnboardingKYC'),
 'jurisdiction', 'Enum',
 'The regulatory jurisdiction under which the entity operates.',
 true, null, null, 'EntityMasterDB', 'legal_entities', 'jurisdiction_code',
 'Client Entity Details', 2, 'select', 'Jurisdiction', null,
 1, 'Entity Information', null,
 null,
 '[{"prompt": "Apply jurisdiction-specific KYC requirements", "response": "RULE \"Jurisdiction-KYC\" IF jurisdiction == ''EU'' THEN REQUIRE gdpr_consent AND mifid_classification"}]'::jsonb);

-- Set allowed values for enum fields
UPDATE attribute_objects
SET allowed_values = '["CLEAR", "POTENTIAL_MATCH", "CONFIRMED_MATCH", "PENDING_REVIEW"]'::jsonb
WHERE attribute_name = 'sanctions_screening_result';

UPDATE attribute_objects
SET allowed_values = '["US", "UK", "EU", "APAC", "OTHER"]'::jsonb
WHERE attribute_name = 'jurisdiction';

UPDATE attribute_objects
SET min_value = 1, max_value = 100
WHERE attribute_name = 'risk_score';

UPDATE attribute_objects
SET rules_dsl = 'DERIVE risk_score FROM country_risk_rating * 0.3 + business_risk_rating * 0.4 + ubo_risk_rating * 0.3'
WHERE attribute_name = 'risk_score';

-- Insert sample perspectives for attributes
INSERT INTO attribute_perspectives (attribute_id, perspective_name, description, ui_group, ui_label, ui_help_text, generation_examples) VALUES
-- Legal entity name - KYC perspective
((SELECT id FROM attribute_objects WHERE attribute_name = 'legal_entity_name'),
 'KYC',
 'The legal name used for regulatory compliance and customer identification. Must match incorporation documents exactly for AML verification.',
 'KYC Due Diligence', 'Official Legal Name', 'Must match incorporation certificate exactly',
 '[{"prompt": "Verify the entity name matches the incorporation documents", "response": "VERIFY Entity ''legal_entity_name'' MATCHES Document ''incorporation_certificate'' Field ''entity_name''"}]'::jsonb),

-- Legal entity name - Fund Accounting perspective
((SELECT id FROM attribute_objects WHERE attribute_name = 'legal_entity_name'),
 'FundAccounting',
 'The entity name used for fund accounting statements and investor reporting. Used in NAV calculations and shareholder communications.',
 'Fund Entity Details', 'Fund Entity Name', null,
 '[]'::jsonb),

-- UBO name - KYC perspective
((SELECT id FROM attribute_objects WHERE attribute_name = 'ubo_full_name'),
 'KYC',
 'The name of the individual who ultimately owns or controls the client entity. Must be screened against sanctions and PEP lists for AML compliance.',
 'KYC Due Diligence', 'Ultimate Beneficial Owner Name', null,
 '[{"prompt": "We need to screen the beneficial owner''s name against the sanctions list.", "response": "SCREEN Entity ''ubo_full_name'' AGAINST Source ''SanctionsList'' RETURN screening_result"}]'::jsonb),

-- UBO name - Fund Accounting perspective
((SELECT id FROM attribute_objects WHERE attribute_name = 'ubo_full_name'),
 'FundAccounting',
 'The registered shareholder''s name, used for calculating capital distributions and generating shareholder statements.',
 'Shareholder Capital Accounts', 'Shareholder Name', null,
 '[{"prompt": "Calculate the dividend payment for this shareholder based on their holdings.", "response": "DERIVE dividend_payment USING CalculateDividend WITH ubo_full_name, share_count"}]'::jsonb),

-- Jurisdiction - KYC perspective
((SELECT id FROM attribute_objects WHERE attribute_name = 'jurisdiction'),
 'KYC',
 'Primary regulatory jurisdiction for compliance and reporting requirements.',
 null, null, null,
 '[{"prompt": "Apply jurisdiction-specific KYC requirements", "response": "RULE \"Jurisdiction-KYC\" IF jurisdiction == ''EU'' THEN REQUIRE gdpr_consent AND mifid_classification"}]'::jsonb),

-- Jurisdiction - Tax Reporting perspective
((SELECT id FROM attribute_objects WHERE attribute_name = 'jurisdiction'),
 'TaxReporting',
 'Tax jurisdiction for CRS and FATCA reporting purposes.',
 null, 'Tax Jurisdiction', null,
 '[]'::jsonb);

-- Insert sample Trade Settlement attributes
INSERT INTO attribute_objects (
    resource_id, attribute_name, data_type, description, is_required,
    min_value, persistence_system, persistence_entity, persistence_identifier,
    ui_group, ui_display_order, ui_render_hint, ui_label
) VALUES
-- Trade Amount
((SELECT id FROM resource_objects WHERE resource_name = 'TradeSettlementSystem'),
 'trade_amount', 'Decimal',
 'The notional amount of the trade in base currency.',
 true, 0.01, 'TradingSystem', 'trades', 'notional_amount',
 'Trade Details', 1, 'number-input', 'Trade Amount'),

-- Settlement Date
((SELECT id FROM resource_objects WHERE resource_name = 'TradeSettlementSystem'),
 'settlement_date', 'Date',
 'The date when the trade is scheduled to settle.',
 true, null, 'TradingSystem', 'trades', 'settlement_date',
 'Settlement Instructions', 1, 'date-picker', 'Settlement Date'),

-- Counterparty Name
((SELECT id FROM resource_objects WHERE resource_name = 'TradeSettlementSystem'),
 'counterparty_name', 'String',
 'The name of the trading counterparty.',
 true, null, 'TradingSystem', 'counterparties', 'party_name',
 'Counterparty Information', 1, 'text-input', 'Counterparty Name');

UPDATE attribute_objects
SET max_length = 100
WHERE attribute_name = 'counterparty_name';

-- Insert UI layout groups
INSERT INTO ui_layout_groups (resource_id, group_name, display_order) VALUES
-- KYC groups
((SELECT id FROM resource_objects WHERE resource_name = 'ClientOnboardingKYC'), 'Client Entity Details', 1),
((SELECT id FROM resource_objects WHERE resource_name = 'ClientOnboardingKYC'), 'Beneficial Owner Details', 2),
((SELECT id FROM resource_objects WHERE resource_name = 'ClientOnboardingKYC'), 'Sanctions Screening Results', 3),
((SELECT id FROM resource_objects WHERE resource_name = 'ClientOnboardingKYC'), 'Risk Assessment', 4),

-- Trade Settlement groups
((SELECT id FROM resource_objects WHERE resource_name = 'TradeSettlementSystem'), 'Trade Details', 1),
((SELECT id FROM resource_objects WHERE resource_name = 'TradeSettlementSystem'), 'Settlement Instructions', 2),
((SELECT id FROM resource_objects WHERE resource_name = 'TradeSettlementSystem'), 'Counterparty Information', 3),
((SELECT id FROM resource_objects WHERE resource_name = 'TradeSettlementSystem'), 'Regulatory Reporting', 4);