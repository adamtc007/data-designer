-- Populate Extended Attribute Metadata for KYC/Onboarding Domain
-- This script populates comprehensive AI RAG, UI auto-layout, and persistence metadata

-- First, let's check what attributes we have
DO $$
DECLARE
    attr_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO attr_count FROM attribute_objects;
    RAISE NOTICE 'Found % attributes to enhance', attr_count;
END $$;

-- Update Customer Identity Attributes
UPDATE attribute_objects SET
    extended_description = 'Complete legal name of the customer as it appears on official government-issued identification documents. This is the primary identifier used for KYC verification and must match exactly with provided documentation.',
    business_context = 'KYC verification requires exact name matching between application and identity documents. Critical for AML compliance, sanctions screening, and identity fraud prevention.',
    technical_context = 'String field with Unicode support for international characters. Stored in encrypted format with field-level encryption. Indexed for fast lookup but not for wildcard searches due to privacy requirements.',
    user_guidance = 'Enter your full legal name exactly as it appears on your government ID (passport, driver license, or national ID card). Include all middle names and avoid nicknames.',
    ai_summary = 'Primary identity field requiring exact matching with official documents for regulatory compliance',
    domain_terminology = '{"legal_name": "Official name on government documents", "given_name": "First name", "family_name": "Last name", "middle_names": "Additional names between first and last"}',
    contextual_examples = '{"valid": ["John Michael Smith", "María Elena García-López", "李小明"], "invalid": ["J. Smith", "Mike (John) Smith", "john smith"]}',
    llm_prompt_context = 'Customer identity verification: legal name matching for KYC compliance',
    semantic_tags = '["identity", "kyc", "legal_name", "verification", "required", "pii"]',
    ai_context = '{"domain": "identity_verification", "criticality": "high", "compliance": ["KYC", "AML"], "data_classification": "confidential"}',
    search_keywords = ARRAY['name', 'identity', 'legal', 'kyc', 'verification'],
    ui_component_type = 'enhanced_text_input',
    ui_layout_config = '{"width": "100%", "placeholder": "Enter full legal name", "autocomplete": false, "case_sensitive": true}',
    ui_styling = '{"font_weight": "500", "border_color": "#2563eb", "focus_ring": true}',
    ui_behavior = '{"trim_whitespace": false, "auto_uppercase": false, "spell_check": false}',
    conditional_logic = '{"show_if": {"document_type": {"not_empty": true}}, "validate_if": {"step": "identity_verification"}}',
    relationship_metadata = '{"related_fields": ["document_number", "date_of_birth"], "validation_group": "identity_core"}',
    ai_prompt_templates = '{"validation": "Verify that this name could reasonably match: {document_type}", "suggestion": "Based on the document type {document_type}, suggest proper name format"}',
    form_generation_rules = '{"label_position": "top", "required_indicator": "*", "help_text_position": "below"}',
    accessibility_config = '{"aria_label": "Full legal name", "screen_reader_text": "Enter your complete legal name as shown on identification documents"}',
    responsive_config = '{"mobile": {"font_size": "16px"}, "desktop": {"font_size": "14px"}}',
    data_flow_config = '{"source": "user_input", "destinations": ["kyc_service", "identity_database"], "encryption": "field_level"}',

    -- AI LLM Integration Fields
    similarity_threshold = 0.85,

    -- Attribute Classification
    attribute_class = 'real',

    -- Filtering and Clustering
    filter_tags = ARRAY['identity', 'required', 'kyc', 'personal'],
    visibility_rules = '{"roles": ["customer", "agent", "compliance"], "contexts": ["onboarding", "verification"]}'

WHERE attribute_name = 'full_name';

-- Update Date of Birth
UPDATE attribute_objects SET
    extended_description = 'Customer date of birth used for age verification, identity confirmation, and risk assessment. Must be consistent across all provided documents and applications.',
    business_context = 'Critical for age verification (18+ requirements), identity confirmation against documents, risk profiling (age-based risk models), and sanctions screening with DOB matching.',
    technical_context = 'Date field stored in ISO 8601 format (YYYY-MM-DD). Validated against business rules for minimum age (18+) and maximum age (120). Used in composite indexes for fast identity lookup.',
    user_guidance = 'Enter your date of birth exactly as shown on your identification documents. This must match the date on your ID for verification purposes.',
    ai_summary = 'Age verification and identity confirmation field with regulatory compliance requirements',
    domain_terminology = '{"dob": "Date of birth", "age_verification": "Confirming customer meets minimum age requirements", "identity_matching": "Verifying DOB matches identification documents"}',
    contextual_examples = '{"valid": ["1990-05-15", "1985-12-31"], "invalid": ["15/05/1990", "May 15, 1990", "1990"], "edge_cases": ["leap_year: 1988-02-29", "min_age: 2006-01-01"]}',
    llm_prompt_context = 'Age and identity verification: DOB validation for regulatory compliance',
    semantic_tags = '["identity", "age", "verification", "kyc", "required", "sensitive"]',
    ai_context = '{"domain": "identity_verification", "criticality": "high", "compliance": ["age_verification", "KYC"], "data_classification": "confidential"}',
    search_keywords = ARRAY['birth', 'age', 'dob', 'date', 'identity'],
    ui_component_type = 'date_picker',
    ui_layout_config = '{"date_format": "yyyy-mm-dd", "min_date": "1900-01-01", "max_date": "today_minus_18_years", "calendar_type": "dropdown"}',
    ui_styling = '{"width": "200px", "border_color": "#2563eb"}',
    ui_behavior = '{"auto_validate": true, "clear_invalid": false}',
    conditional_logic = '{"validate_if": {"full_name": {"not_empty": true}}}',
    relationship_metadata = '{"related_fields": ["full_name", "document_number"], "validation_group": "identity_core"}',
    ai_prompt_templates = '{"validation": "Check if DOB {value} is consistent with document type {document_type} and customer profile"}',
    attribute_class = 'real',
    filter_tags = ARRAY['identity', 'required', 'kyc', 'personal', 'date']

WHERE attribute_name = 'date_of_birth';

-- Update Email Address with comprehensive metadata
UPDATE attribute_objects SET
    extended_description = 'Primary communication email address for the customer account. Used for notifications, alerts, document delivery, and account recovery. Must be valid and accessible by the customer.',
    business_context = 'Primary communication channel for customer notifications, regulatory communications, account alerts, and document delivery. Critical for customer experience and regulatory notification requirements.',
    technical_context = 'Email field with RFC 5322 validation. Indexed for fast lookup and uniqueness constraints within tenant. Supports internationalized domain names (IDN). Rate-limited for communication.',
    user_guidance = 'Provide a valid email address that you regularly check. We will send important account information and documents to this address.',
    ai_summary = 'Primary communication channel with validation and deliverability requirements',
    domain_terminology = '{"primary_email": "Main contact email", "deliverability": "Ability to successfully deliver emails", "verified_email": "Email confirmed through verification process"}',
    contextual_examples = '{"valid": ["john.smith@example.com", "maría@empresa.es", "user+tag@domain.co.uk"], "invalid": ["plainaddress", "@domain.com", "user@", "spaces in@email.com"]}',
    llm_prompt_context = 'Customer communication: email validation and deliverability for business notifications',
    semantic_tags = '["communication", "contact", "required", "validation", "deliverability"]',
    ai_context = '{"domain": "customer_communication", "criticality": "high", "compliance": ["notification_requirements"], "data_classification": "internal"}',
    search_keywords = ARRAY['email', 'contact', 'communication', 'notification'],
    ui_component_type = 'email_input',
    ui_layout_config = '{"width": "100%", "placeholder": "Enter your email address", "autocomplete": "email", "validation": "real_time"}',
    ui_styling = '{"icon": "envelope", "border_color": "#10b981"}',
    ui_behavior = '{"trim_whitespace": true, "lowercase": true, "validate_on_blur": true}',
    conditional_logic = '{"required_if": {"communication_preference": "email"}}',
    relationship_metadata = '{"related_fields": ["phone_number"], "validation_group": "contact_info", "alternatives": ["phone_number"]}',
    ai_prompt_templates = '{"validation": "Check email deliverability and format for {value}", "suggestion": "Suggest corrections for malformed email: {value}"}',
    attribute_class = 'real',
    filter_tags = ARRAY['contact', 'required', 'communication', 'validation']

WHERE attribute_name = 'email';

-- Update Phone Number
UPDATE attribute_objects SET
    extended_description = 'Customer primary phone number for SMS notifications, two-factor authentication, and urgent communications. Should be a number the customer can receive calls and SMS on.',
    business_context = 'Secondary communication channel for urgent notifications, SMS alerts, and two-factor authentication. Required for enhanced security and customer verification processes.',
    technical_context = 'Phone number in E.164 international format (+[country_code][number]). Validated for format and potentially for deliverability. Used for SMS and voice communications.',
    user_guidance = 'Enter your primary phone number including country code. We may send SMS verifications and important alerts to this number.',
    ai_summary = 'Secondary communication channel for SMS and voice communications with 2FA capability',
    domain_terminology = '{"e164_format": "International phone format +1234567890", "2fa": "Two-factor authentication", "sms_capable": "Can receive text messages"}',
    contextual_examples = '{"valid": ["+1234567890", "+447700900123", "+33123456789"], "invalid": ["123-456-7890", "01234567890", "+1 (234) 567-8900"]}',
    llm_prompt_context = 'Customer communication: phone number validation for SMS and voice communications',
    semantic_tags = '["communication", "contact", "2fa", "sms", "verification"]',
    ai_context = '{"domain": "customer_communication", "criticality": "medium", "compliance": ["2fa_requirements"], "data_classification": "internal"}',
    search_keywords = ARRAY['phone', 'mobile', 'sms', '2fa', 'contact'],
    ui_component_type = 'phone_input',
    ui_layout_config = '{"country_selector": true, "format": "international", "placeholder": "+1 (555) 123-4567"}',
    ui_styling = '{"icon": "phone", "width": "250px"}',
    ui_behavior = '{"auto_format": true, "validate_country": true}',
    conditional_logic = '{"required_if": {"two_factor_enabled": true}}',
    relationship_metadata = '{"related_fields": ["email"], "validation_group": "contact_info"}',
    attribute_class = 'real',
    filter_tags = ARRAY['contact', 'communication', '2fa', 'optional']

WHERE attribute_name = 'phone_number';

-- Update Address with rich metadata
UPDATE attribute_objects SET
    extended_description = 'Complete residential address used for identity verification, document delivery, and regulatory compliance. Must match address on provided proof of address documents.',
    business_context = 'Required for KYC verification, document delivery, tax reporting, and regulatory compliance. Used for address verification against proof of address documents and sanctions screening.',
    technical_context = 'Structured address data stored in normalized format. Geocoded for location verification. Validated against postal services APIs. Encrypted storage with audit trail.',
    user_guidance = 'Provide your current residential address exactly as it appears on your proof of address document (utility bill, bank statement, etc.).',
    ai_summary = 'Residential address for verification, compliance, and document delivery purposes',
    domain_terminology = '{"proof_of_address": "Document showing current residence", "geocoding": "Converting address to coordinates", "address_verification": "Confirming address validity and customer residence"}',
    contextual_examples = '{"valid": ["123 Main St, Apt 4B, New York, NY 10001, USA"], "components": {"street": "123 Main St", "unit": "Apt 4B", "city": "New York", "state": "NY", "postal": "10001", "country": "USA"}}',
    llm_prompt_context = 'Address verification: residential address validation for KYC and document delivery',
    semantic_tags = '["address", "residence", "kyc", "verification", "delivery", "required"]',
    ai_context = '{"domain": "identity_verification", "criticality": "high", "compliance": ["KYC", "address_verification"], "data_classification": "confidential"}',
    search_keywords = ARRAY['address', 'residence', 'location', 'delivery', 'verification'],
    ui_component_type = 'address_input',
    ui_layout_config = '{"components": ["street", "unit", "city", "state", "postal", "country"], "autocomplete": true, "validation": "real_time"}',
    ui_styling = '{"layout": "stacked", "spacing": "compact"}',
    ui_behavior = '{"geocode_validation": true, "auto_complete": true}',
    conditional_logic = '{"validate_if": {"document_type": "proof_of_address"}}',
    relationship_metadata = '{"related_fields": ["country_of_residence"], "validation_group": "location_info"}',
    attribute_class = 'real',
    filter_tags = ARRAY['address', 'required', 'kyc', 'verification', 'location']

WHERE attribute_name = 'address';

-- Add a derived attribute example - Risk Score
INSERT INTO attribute_objects (
    resource_id, attribute_name, data_type, description, is_required,
    ui_group, ui_display_order, ui_label, ui_help_text,
    generation_examples,

    -- Extended metadata
    extended_description, business_context, technical_context, user_guidance, ai_summary,
    domain_terminology, contextual_examples, llm_prompt_context,
    semantic_tags, ai_context, search_keywords,
    ui_component_type, ui_layout_config, ui_styling, ui_behavior,
    conditional_logic, relationship_metadata, ai_prompt_templates,

    -- Classification and derivation
    attribute_class, derivation_rule, ebnf_grammar, derivation_dependencies,

    -- Filtering and clustering
    filter_tags, visibility_rules,

    created_at, updated_at
) VALUES (
    1, -- Assuming resource_id 1 exists
    'risk_score',
    'decimal',
    'Calculated risk score based on customer profile and behavior',
    false,
    'risk_assessment',
    100,
    'Risk Score',
    'Automatically calculated risk assessment score (0-100)',
    '{"examples": [25.5, 67.2, 89.1], "typical_range": "0-100"}',

    -- Extended metadata
    'Comprehensive risk assessment score calculated from multiple data points including customer demographics, transaction patterns, document verification results, and external risk factors. Used for risk-based decision making and ongoing monitoring.',

    'Primary metric for risk-based customer segmentation, automated decision making, transaction monitoring thresholds, and regulatory reporting. Critical for AML compliance and operational risk management.',

    'Calculated field using machine learning models and rule-based scoring. Updated in real-time as new data becomes available. Stored with calculation timestamp and model version for auditability.',

    'This score is automatically calculated based on your profile and activity. Higher scores may require additional verification or documentation.',

    'Automated risk assessment metric for compliance and operational decision making',

    '{"risk_score": "Numerical assessment of customer risk level", "risk_factors": "Elements contributing to risk calculation", "dynamic_scoring": "Score updates as new information becomes available"}',

    '{"calculation_inputs": ["age", "income", "transaction_volume", "document_verification_status"], "score_ranges": {"low": "0-30", "medium": "31-70", "high": "71-100"}}',

    'Risk assessment: automated scoring for compliance and operational risk management',

    '["risk", "score", "assessment", "calculated", "compliance", "aml", "automated"]',

    '{"domain": "risk_management", "criticality": "high", "compliance": ["AML", "risk_assessment"], "data_classification": "internal"}',

    ARRAY['risk', 'score', 'assessment', 'calculated', 'aml'],

    'risk_score_display',

    '{"display_format": "progress_bar", "color_coding": {"low": "green", "medium": "yellow", "high": "red"}, "show_breakdown": true}',

    '{"color_scheme": "risk_gradient", "size": "large"}',

    '{"auto_refresh": true, "show_calculation_date": true}',

    '{"show_if": {"customer_status": "active"}}',

    '{"input_fields": ["age", "income", "transaction_volume"], "calculation_group": "risk_metrics"}',

    '{"explanation": "Explain risk score calculation for {customer_id}", "breakdown": "Provide risk factor breakdown for score {value}"}',

    -- Classification and derivation
    'derived',
    'CALCULATE_RISK_SCORE(age, income, transaction_volume, document_verification_status)',
    'risk_score := BASE_SCORE + AGE_FACTOR(age) + INCOME_FACTOR(income) + TRANSACTION_FACTOR(transaction_volume) + VERIFICATION_FACTOR(document_verification_status)',
    ARRAY['age', 'income', 'transaction_volume', 'document_verification_status'],

    -- Filtering and clustering
    ARRAY['risk', 'calculated', 'compliance', 'internal'],
    '{"roles": ["agent", "compliance", "management"], "contexts": ["risk_review", "onboarding_complete"]}',

    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
) ON CONFLICT (resource_id, attribute_name) DO UPDATE SET
    extended_description = EXCLUDED.extended_description,
    business_context = EXCLUDED.business_context,
    technical_context = EXCLUDED.technical_context,
    user_guidance = EXCLUDED.user_guidance,
    ai_summary = EXCLUDED.ai_summary,
    updated_at = CURRENT_TIMESTAMP;

-- Update Document Type with comprehensive metadata
UPDATE attribute_objects SET
    extended_description = 'Type of identity document provided by the customer for verification purposes. Determines the verification workflow and requirements for identity confirmation.',
    business_context = 'Drives the identity verification process flow, document validation requirements, and risk assessment. Different document types have different verification procedures and reliability scores.',
    technical_context = 'Enumerated field with predefined document types. Triggers workflow automation for document processing and verification. Mapped to verification service APIs.',
    user_guidance = 'Select the type of government-issued identification document you will be providing for verification.',
    ai_summary = 'Document type selector that determines verification workflow and requirements',
    domain_terminology = '{"government_issued": "Official documents from government agencies", "primary_id": "Main identification documents like passports", "secondary_id": "Supporting documents like utility bills"}',
    contextual_examples = '{"primary": ["passport", "drivers_license", "national_id"], "secondary": ["utility_bill", "bank_statement", "tax_document"]}',
    llm_prompt_context = 'Document verification: identity document type selection for KYC workflow',
    semantic_tags = '["document", "identity", "verification", "kyc", "required", "workflow"]',
    ai_context = '{"domain": "document_verification", "criticality": "high", "compliance": ["KYC"], "data_classification": "internal"}',
    search_keywords = ARRAY['document', 'id', 'verification', 'identity', 'kyc'],
    ui_component_type = 'document_type_selector',
    ui_layout_config = '{"display": "grid", "columns": 2, "show_icons": true, "categorize": true}',
    ui_styling = '{"card_style": true, "hover_effects": true}',
    ui_behavior = '{"single_select": true, "trigger_workflow": true}',
    conditional_logic = '{"show_upload_if": {"value": {"not_empty": true}}}',
    relationship_metadata = '{"triggers": ["document_upload", "verification_workflow"], "validation_group": "document_verification"}',
    attribute_class = 'real',
    filter_tags = ARRAY['document', 'required', 'kyc', 'workflow']

WHERE attribute_name = 'document_type';

-- Add some AI embeddings (simulated - in real implementation these would be generated by an embedding model)
UPDATE attribute_objects SET
    semantic_embedding = ARRAY[0.1, 0.3, 0.8, 0.2, 0.7, 0.4, 0.9, 0.1, 0.6, 0.5, 0.3, 0.8, 0.2, 0.7, 0.4, 0.9],
    context_embedding = ARRAY[0.2, 0.4, 0.7, 0.3, 0.8, 0.5, 0.8, 0.2, 0.7, 0.6, 0.4, 0.9, 0.3, 0.8, 0.5, 0.8]
WHERE attribute_name IN ('full_name', 'date_of_birth', 'email', 'phone_number', 'address');

-- Add derived attribute embeddings (more similar to each other)
UPDATE attribute_objects SET
    semantic_embedding = ARRAY[0.8, 0.7, 0.9, 0.6, 0.8, 0.7, 0.9, 0.8, 0.6, 0.7, 0.8, 0.9, 0.7, 0.8, 0.6, 0.9],
    context_embedding = ARRAY[0.9, 0.8, 0.8, 0.7, 0.9, 0.8, 0.8, 0.9, 0.7, 0.8, 0.9, 0.8, 0.8, 0.9, 0.7, 0.8]
WHERE attribute_name = 'risk_score';

-- Add cluster assignments
UPDATE attribute_objects SET
    cluster_assignments = ARRAY[1, 3] -- Identity cluster and Required cluster
WHERE attribute_name IN ('full_name', 'date_of_birth');

UPDATE attribute_objects SET
    cluster_assignments = ARRAY[2, 4] -- Contact cluster and Communication cluster
WHERE attribute_name IN ('email', 'phone_number');

UPDATE attribute_objects SET
    cluster_assignments = ARRAY[1, 3] -- Identity cluster and Required cluster
WHERE attribute_name = 'address';

UPDATE attribute_objects SET
    cluster_assignments = ARRAY[5] -- Risk cluster
WHERE attribute_name = 'risk_score';

-- Update access control and visibility rules
UPDATE attribute_objects SET
    access_control = '{"read": ["customer", "agent", "compliance"], "write": ["customer"], "admin": ["compliance_manager"]}',
    visibility_rules = '{"customer_view": true, "agent_view": true, "compliance_view": true}'
WHERE attribute_name IN ('full_name', 'date_of_birth', 'email', 'phone_number', 'address');

UPDATE attribute_objects SET
    access_control = '{"read": ["agent", "compliance", "management"], "write": ["system"], "admin": ["risk_manager"]}',
    visibility_rules = '{"customer_view": false, "agent_view": true, "compliance_view": true}'
WHERE attribute_name = 'risk_score';

-- Add more sophisticated conditional logic for some fields
UPDATE attribute_objects SET
    conditional_logic = jsonb_build_object(
        'show_if', jsonb_build_object(
            'document_type', jsonb_build_object('in', jsonb_build_array('passport', 'drivers_license', 'national_id'))
        ),
        'validate_if', jsonb_build_object(
            'verification_step', 'identity_check'
        ),
        'required_if', jsonb_build_object(
            'customer_type', 'individual'
        )
    )
WHERE attribute_name = 'full_name';

-- Add form generation rules for better UI layout
UPDATE attribute_objects SET
    form_generation_rules = jsonb_build_object(
        'group_layout', 'card',
        'label_position', 'top',
        'help_text_style', 'tooltip',
        'validation_display', 'inline',
        'error_position', 'below'
    )
WHERE attribute_name IN ('full_name', 'date_of_birth', 'email', 'phone_number', 'address');

-- Update responsive configurations for mobile optimization
UPDATE attribute_objects SET
    responsive_config = jsonb_build_object(
        'mobile', jsonb_build_object(
            'font_size', '16px',
            'input_height', '44px',
            'margin', '8px'
        ),
        'tablet', jsonb_build_object(
            'font_size', '14px',
            'input_height', '40px',
            'margin', '6px'
        ),
        'desktop', jsonb_build_object(
            'font_size', '14px',
            'input_height', '36px',
            'margin', '4px'
        )
    )
WHERE attribute_name IN ('full_name', 'date_of_birth', 'email', 'phone_number', 'address');

-- Create some sample persistence mappings
INSERT INTO persistence_systems (system_name, system_type, connection_config, capabilities, performance_profile) VALUES
('customer_database', 'database', '{"host": "localhost", "port": 5432, "database": "customer_data"}', '{"read": true, "write": true, "query": true}', '{"latency_ms": 50, "throughput_qps": 1000}'),
('document_storage', 'api', '{"base_url": "https://docs-api.example.com", "version": "v1"}', '{"read": true, "write": true}', '{"latency_ms": 200, "throughput_qps": 100}'),
('cache_layer', 'cache', '{"host": "localhost", "port": 6379}', '{"read": true, "write": true, "real_time": true}', '{"latency_ms": 1, "throughput_qps": 10000}')
ON CONFLICT (system_name) DO NOTHING;

INSERT INTO persistence_entities (entity_name, system_id, entity_type, entity_config) VALUES
('customers', 1, 'table', '{"table_name": "customers", "primary_key": "customer_id"}'),
('documents', 2, 'endpoint', '{"endpoint": "/documents", "method": "POST"}'),
('session_data', 3, 'keyspace', '{"prefix": "customer:", "ttl": 3600}')
ON CONFLICT (system_id, entity_name) DO NOTHING;

-- Link attributes to persistence entities
INSERT INTO attribute_persistence_mappings (attribute_id, persistence_entity_id, field_mapping, transformation_rules)
SELECT
    ao.id,
    1, -- customers table
    jsonb_build_object('column_name', ao.attribute_name, 'data_type', ao.data_type),
    jsonb_build_object('encryption', CASE WHEN ao.attribute_name IN ('full_name', 'date_of_birth', 'address') THEN true ELSE false END)
FROM attribute_objects ao
WHERE ao.attribute_name IN ('full_name', 'date_of_birth', 'email', 'phone_number', 'address', 'risk_score')
ON CONFLICT (attribute_id, persistence_entity_id) DO NOTHING;

-- Show summary of what was populated
DO $$
DECLARE
    enhanced_count INTEGER;
    with_embeddings INTEGER;
    with_clusters INTEGER;
BEGIN
    SELECT COUNT(*) INTO enhanced_count
    FROM attribute_objects
    WHERE extended_description IS NOT NULL;

    SELECT COUNT(*) INTO with_embeddings
    FROM attribute_objects
    WHERE semantic_embedding IS NOT NULL;

    SELECT COUNT(*) INTO with_clusters
    FROM attribute_objects
    WHERE cluster_assignments IS NOT NULL;

    RAISE NOTICE 'Enhanced % attributes with comprehensive metadata', enhanced_count;
    RAISE NOTICE 'Added embeddings to % attributes', with_embeddings;
    RAISE NOTICE 'Added cluster assignments to % attributes', with_clusters;
END $$;