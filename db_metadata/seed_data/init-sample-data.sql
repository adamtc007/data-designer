-- Sample data initialization script for data-designer database
-- Run this after creating the database and schema

-- Insert sample categories
INSERT INTO rule_categories (name, description) VALUES
    ('Risk Assessment', 'Rules for calculating risk scores and assessments'),
    ('Data Validation', 'Rules for validating data formats and completeness'),
    ('KYC Validation', 'Know Your Customer validation rules'),
    ('Compliance', 'Regulatory compliance and due diligence rules'),
    ('Classification', 'Rules for classifying entities and data')
ON CONFLICT (name) DO NOTHING;

-- Insert sample business attributes
INSERT INTO business_attributes (entity_name, attribute_name, data_type, sql_type, rust_type, description) VALUES
    ('Client', 'client_id', 'String', 'VARCHAR(50)', 'String', 'Unique client identifier'),
    ('Client', 'legal_entity_name', 'String', 'VARCHAR(255)', 'String', 'Legal name of the client entity'),
    ('Client', 'lei_code', 'String', 'VARCHAR(20)', 'String', 'Legal Entity Identifier code'),
    ('Client', 'email', 'String', 'VARCHAR(255)', 'String', 'Primary contact email address'),
    ('Client', 'country_code', 'String', 'VARCHAR(2)', 'String', 'ISO country code'),
    ('Client', 'risk_rating', 'Number', 'INTEGER', 'i32', 'Risk rating from 1-10'),
    ('Client', 'pep_status', 'Boolean', 'BOOLEAN', 'bool', 'Politically exposed person status'),
    ('Client', 'aum_usd', 'Number', 'DECIMAL(15,2)', 'f64', 'Assets under management in USD'),
    ('Client', 'swift_code', 'String', 'VARCHAR(11)', 'String', 'SWIFT/BIC code')
ON CONFLICT (entity_name, attribute_name) DO NOTHING;

-- Insert sample derived attributes
INSERT INTO derived_attributes (entity_name, attribute_name, data_type, sql_type, rust_type, description) VALUES
    ('Client', 'risk_score', 'Number', 'DECIMAL(10,2)', 'f64', 'Calculated risk score'),
    ('Client', 'email_valid', 'Boolean', 'BOOLEAN', 'bool', 'Email validation status'),
    ('Client', 'kyc_completion_percentage', 'Number', 'INTEGER', 'i32', 'KYC completion percentage'),
    ('Client', 'enhanced_dd_required', 'Boolean', 'BOOLEAN', 'bool', 'Enhanced due diligence requirement flag'),
    ('Client', 'service_tier', 'String', 'VARCHAR(20)', 'String', 'Client service tier classification')
ON CONFLICT (entity_name, attribute_name) DO NOTHING;

-- Insert sample rules
INSERT INTO rules (rule_id, rule_name, description, category_id, rule_definition, target_attribute_id, status) VALUES
    ('RULE_001',
     'Calculate Risk Score',
     'Calculate overall risk score based on client risk rating, PEP status, and AUM',
     (SELECT id FROM rule_categories WHERE name = 'Risk Assessment'),
     'risk_score = (risk_rating * 2 + pep_status * 3) / aum_usd',
     (SELECT id FROM derived_attributes WHERE entity_name = 'Client' AND attribute_name = 'risk_score'),
     'active'),

    ('RULE_002',
     'Validate Email Format',
     'Validate client email address format',
     (SELECT id FROM rule_categories WHERE name = 'Data Validation'),
     'email_valid = email MATCHES "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"',
     (SELECT id FROM derived_attributes WHERE entity_name = 'Client' AND attribute_name = 'email_valid'),
     'active'),

    ('RULE_003',
     'KYC Completion Status',
     'Determine KYC completion percentage based on required fields',
     (SELECT id FROM rule_categories WHERE name = 'KYC Validation'),
     'kyc_percentage = (HAS(client_id) + HAS(legal_entity_name) + HAS(lei_code) + HAS(email) + HAS(country_code)) * 20',
     (SELECT id FROM derived_attributes WHERE entity_name = 'Client' AND attribute_name = 'kyc_completion_percentage'),
     'active'),

    ('RULE_004',
     'Enhanced Due Diligence Flag',
     'Determine if enhanced due diligence is required',
     (SELECT id FROM rule_categories WHERE name = 'Compliance'),
     'enhanced_dd = risk_rating > 7 OR pep_status = true OR aum_usd > 10000000',
     (SELECT id FROM derived_attributes WHERE entity_name = 'Client' AND attribute_name = 'enhanced_dd_required'),
     'active'),

    ('RULE_005',
     'Client Tier Classification',
     'Classify client into service tiers based on AUM',
     (SELECT id FROM rule_categories WHERE name = 'Classification'),
     'WHEN aum_usd > 10000000 THEN service_tier = "Platinum" WHEN aum_usd > 1000000 THEN service_tier = "Gold" ELSE service_tier = "Silver"',
     (SELECT id FROM derived_attributes WHERE entity_name = 'Client' AND attribute_name = 'service_tier'),
     'active')
ON CONFLICT (rule_id) DO NOTHING;

-- Create rule dependencies (linking rules to their input attributes)
-- RULE_001 depends on risk_rating, pep_status, aum_usd
INSERT INTO rule_dependencies (rule_id, attribute_id, dependency_type) VALUES
    ((SELECT id FROM rules WHERE rule_id = 'RULE_001'),
     (SELECT id FROM business_attributes WHERE entity_name = 'Client' AND attribute_name = 'risk_rating'),
     'input'),
    ((SELECT id FROM rules WHERE rule_id = 'RULE_001'),
     (SELECT id FROM business_attributes WHERE entity_name = 'Client' AND attribute_name = 'pep_status'),
     'input'),
    ((SELECT id FROM rules WHERE rule_id = 'RULE_001'),
     (SELECT id FROM business_attributes WHERE entity_name = 'Client' AND attribute_name = 'aum_usd'),
     'input'),
    -- RULE_002 depends on email
    ((SELECT id FROM rules WHERE rule_id = 'RULE_002'),
     (SELECT id FROM business_attributes WHERE entity_name = 'Client' AND attribute_name = 'email'),
     'input')
ON CONFLICT (rule_id, attribute_id) DO NOTHING;

VACUUM ANALYZE;

SELECT COUNT(*) as rule_count FROM rules;
SELECT COUNT(*) as business_attr_count FROM business_attributes;
SELECT COUNT(*) as derived_attr_count FROM derived_attributes;