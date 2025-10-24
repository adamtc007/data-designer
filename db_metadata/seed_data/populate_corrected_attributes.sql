-- Corrected Populate Extended Attribute Metadata for KYC/Onboarding Domain
-- Matching actual database schema

-- Update Customer Identity Attributes
UPDATE attribute_objects SET
    extended_description = 'Complete legal name of the customer as it appears on official government-issued identification documents. This is the primary identifier used for KYC verification and must match exactly with provided documentation.',
    business_context = 'KYC verification requires exact name matching between application and identity documents. Critical for AML compliance, sanctions screening, and identity fraud prevention.',
    technical_context = 'String field with Unicode support for international characters. Stored in encrypted format with field-level encryption. Indexed for fast lookup but not for wildcard searches due to privacy requirements.',
    user_guidance = 'Enter your full legal name exactly as it appears on your government ID (passport, driver license, or national ID card). Include all middle names and avoid nicknames.',
    ai_training_examples = 'Primary identity field requiring exact matching with official documents for regulatory compliance',
    domain_terminology = 'legal_name: Official name on government documents; given_name: First name; family_name: Last name; middle_names: Additional names between first and last',
    usage_scenarios = 'Identity verification during customer onboarding, sanctions screening, document matching',
    compliance_explanation = 'Required for KYC compliance under BSA and AML regulations. Must match government-issued identification documents.',
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
    attribute_class = 'real'
WHERE attribute_name = 'full_name';

-- Update Date of Birth
UPDATE attribute_objects SET
    extended_description = 'Customer date of birth used for age verification, identity confirmation, and risk assessment. Must be consistent across all provided documents and applications.',
    business_context = 'Critical for age verification (18+ requirements), identity confirmation against documents, risk profiling (age-based risk models), and sanctions screening with DOB matching.',
    technical_context = 'Date field stored in ISO 8601 format (YYYY-MM-DD). Validated against business rules for minimum age (18+) and maximum age (120). Used in composite indexes for fast identity lookup.',
    user_guidance = 'Enter your date of birth exactly as shown on your identification documents. This must match the date on your ID for verification purposes.',
    ai_training_examples = 'Age verification and identity confirmation field with regulatory compliance requirements',
    domain_terminology = 'dob: Date of birth; age_verification: Confirming customer meets minimum age requirements; identity_matching: Verifying DOB matches identification documents',
    usage_scenarios = 'Age verification during onboarding, identity document matching, risk assessment calculations',
    compliance_explanation = 'Required for age verification under banking regulations. Must be 18+ for account opening.',
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
    attribute_class = 'real'
WHERE attribute_name = 'date_of_birth';

-- Update Email Address
UPDATE attribute_objects SET
    extended_description = 'Primary communication email address for the customer account. Used for notifications, alerts, document delivery, and account recovery. Must be valid and accessible by the customer.',
    business_context = 'Primary communication channel for customer notifications, regulatory communications, account alerts, and document delivery. Critical for customer experience and regulatory notification requirements.',
    technical_context = 'Email field with RFC 5322 validation. Indexed for fast lookup and uniqueness constraints within tenant. Supports internationalized domain names (IDN). Rate-limited for communication.',
    user_guidance = 'Provide a valid email address that you regularly check. We will send important account information and documents to this address.',
    ai_training_examples = 'Primary communication channel with validation and deliverability requirements',
    domain_terminology = 'primary_email: Main contact email; deliverability: Ability to successfully deliver emails; verified_email: Email confirmed through verification process',
    usage_scenarios = 'Customer communications, document delivery, account recovery, regulatory notifications',
    compliance_explanation = 'Required for regulatory communications and customer notifications under banking regulations.',
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
    attribute_class = 'real'
WHERE attribute_name = 'email';

-- Update Phone Number
UPDATE attribute_objects SET
    extended_description = 'Customer primary phone number for SMS notifications, two-factor authentication, and urgent communications. Should be a number the customer can receive calls and SMS on.',
    business_context = 'Secondary communication channel for urgent notifications, SMS alerts, and two-factor authentication. Required for enhanced security and customer verification processes.',
    technical_context = 'Phone number in E.164 international format (+[country_code][number]). Validated for format and potentially for deliverability. Used for SMS and voice communications.',
    user_guidance = 'Enter your primary phone number including country code. We may send SMS verifications and important alerts to this number.',
    ai_training_examples = 'Secondary communication channel for SMS and voice communications with 2FA capability',
    domain_terminology = 'e164_format: International phone format +1234567890; 2fa: Two-factor authentication; sms_capable: Can receive text messages',
    usage_scenarios = 'Two-factor authentication, SMS notifications, urgent customer communications',
    compliance_explanation = 'Required for enhanced authentication under security regulations. Used for identity verification.',
    semantic_tags = '["communication", "contact", "2fa", "sms", "verification"]',
    ai_context = '{"domain": "customer_communication", "criticality": "medium", "compliance": ["2fa_requirements"], "data_classification": "internal"}',
    search_keywords = ARRAY['phone', 'mobile', 'sms', '2fa', 'contact'],
    ui_component_type = 'phone_input',
    ui_layout_config = '{"country_selector": true, "format": "international", "placeholder": "+1 (555) 123-4567"}',
    ui_styling = '{"icon": "phone", "width": "250px"}',
    ui_behavior = '{"auto_format": true, "validate_country": true}',
    conditional_logic = '{"required_if": {"two_factor_enabled": true}}',
    relationship_metadata = '{"related_fields": ["email"], "validation_group": "contact_info"}',
    attribute_class = 'real'
WHERE attribute_name = 'phone_number';

-- Update Address
UPDATE attribute_objects SET
    extended_description = 'Complete residential address used for identity verification, document delivery, and regulatory compliance. Must match address on provided proof of address documents.',
    business_context = 'Required for KYC verification, document delivery, tax reporting, and regulatory compliance. Used for address verification against proof of address documents and sanctions screening.',
    technical_context = 'Structured address data stored in normalized format. Geocoded for location verification. Validated against postal services APIs. Encrypted storage with audit trail.',
    user_guidance = 'Provide your current residential address exactly as it appears on your proof of address document (utility bill, bank statement, etc.).',
    ai_training_examples = 'Residential address for verification, compliance, and document delivery purposes',
    domain_terminology = 'proof_of_address: Document showing current residence; geocoding: Converting address to coordinates; address_verification: Confirming address validity and customer residence',
    usage_scenarios = 'Address verification, document delivery, tax reporting, sanctions screening',
    compliance_explanation = 'Required for KYC compliance and address verification under banking regulations.',
    semantic_tags = '["address", "residence", "kyc", "verification", "delivery", "required"]',
    ai_context = '{"domain": "identity_verification", "criticality": "high", "compliance": ["KYC", "address_verification"], "data_classification": "confidential"}',
    search_keywords = ARRAY['address', 'residence', 'location', 'delivery', 'verification'],
    ui_component_type = 'address_input',
    ui_layout_config = '{"components": ["street", "unit", "city", "state", "postal", "country"], "autocomplete": true, "validation": "real_time"}',
    ui_styling = '{"layout": "stacked", "spacing": "compact"}',
    ui_behavior = '{"geocode_validation": true, "auto_complete": true}',
    conditional_logic = '{"validate_if": {"document_type": "proof_of_address"}}',
    relationship_metadata = '{"related_fields": ["country_of_residence"], "validation_group": "location_info"}',
    attribute_class = 'real'
WHERE attribute_name = 'address';

-- Add a derived attribute example - Risk Score
INSERT INTO attribute_objects (
    resource_id, attribute_name, data_type, description, is_required,
    ui_group, ui_display_order, ui_label, ui_help_text,
    generation_examples,
    extended_description, business_context, technical_context, user_guidance, ai_training_examples,
    domain_terminology, usage_scenarios, compliance_explanation,
    semantic_tags, ai_context, search_keywords,
    ui_component_type, ui_layout_config, ui_styling, ui_behavior,
    conditional_logic, relationship_metadata, ai_prompt_templates,
    attribute_class, derivation_rule_ebnf, derivation_dependencies,
    created_at, updated_at
) VALUES (
    1, -- Assuming resource_id 1 exists
    'risk_score',
    'Decimal',
    'Calculated risk score based on customer profile and behavior',
    false,
    'risk_assessment',
    100,
    'Risk Score',
    'Automatically calculated risk assessment score (0-100)',
    '{"examples": [25.5, 67.2, 89.1], "typical_range": "0-100"}',
    'Comprehensive risk assessment score calculated from multiple data points including customer demographics, transaction patterns, document verification results, and external risk factors. Used for risk-based decision making and ongoing monitoring.',
    'Primary metric for risk-based customer segmentation, automated decision making, transaction monitoring thresholds, and regulatory reporting. Critical for AML compliance and operational risk management.',
    'Calculated field using machine learning models and rule-based scoring. Updated in real-time as new data becomes available. Stored with calculation timestamp and model version for auditability.',
    'This score is automatically calculated based on your profile and activity. Higher scores may require additional verification or documentation.',
    'Automated risk assessment metric for compliance and operational decision making',
    'risk_score: Numerical assessment of customer risk level; risk_factors: Elements contributing to risk calculation; dynamic_scoring: Score updates as new information becomes available',
    'Risk assessment calculations, compliance reporting, operational decision making',
    'Required for AML compliance and risk-based customer management under banking regulations.',
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
    'derived',
    'risk_score := BASE_SCORE + AGE_FACTOR(age) + INCOME_FACTOR(income) + TRANSACTION_FACTOR(transaction_volume) + VERIFICATION_FACTOR(document_verification_status)',
    ARRAY[1, 2, 3, 4], -- Assuming these are IDs of existing attributes
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
) ON CONFLICT (resource_id, attribute_name) DO UPDATE SET
    extended_description = EXCLUDED.extended_description,
    business_context = EXCLUDED.business_context,
    technical_context = EXCLUDED.technical_context,
    user_guidance = EXCLUDED.user_guidance,
    ai_training_examples = EXCLUDED.ai_training_examples,
    updated_at = CURRENT_TIMESTAMP;

-- Update Document Type
UPDATE attribute_objects SET
    extended_description = 'Type of identity document provided by the customer for verification purposes. Determines the verification workflow and requirements for identity confirmation.',
    business_context = 'Drives the identity verification process flow, document validation requirements, and risk assessment. Different document types have different verification procedures and reliability scores.',
    technical_context = 'Enumerated field with predefined document types. Triggers workflow automation for document processing and verification. Mapped to verification service APIs.',
    user_guidance = 'Select the type of government-issued identification document you will be providing for verification.',
    ai_training_examples = 'Document type selector that determines verification workflow and requirements',
    domain_terminology = 'government_issued: Official documents from government agencies; primary_id: Main identification documents like passports; secondary_id: Supporting documents like utility bills',
    usage_scenarios = 'Document verification workflow triggering, identity verification process configuration',
    compliance_explanation = 'Required for KYC document verification under banking regulations.',
    semantic_tags = '["document", "identity", "verification", "kyc", "required", "workflow"]',
    ai_context = '{"domain": "document_verification", "criticality": "high", "compliance": ["KYC"], "data_classification": "internal"}',
    search_keywords = ARRAY['document', 'id', 'verification', 'identity', 'kyc'],
    ui_component_type = 'document_type_selector',
    ui_layout_config = '{"display": "grid", "columns": 2, "show_icons": true, "categorize": true}',
    ui_styling = '{"card_style": true, "hover_effects": true}',
    ui_behavior = '{"single_select": true, "trigger_workflow": true}',
    conditional_logic = '{"show_upload_if": {"value": {"not_empty": true}}}',
    relationship_metadata = '{"triggers": ["document_upload", "verification_workflow"], "validation_group": "document_verification"}',
    attribute_class = 'real'
WHERE attribute_name = 'document_type';

-- Add some AI embeddings (simulated - in real implementation these would be generated by an embedding model)
UPDATE attribute_objects SET
    embedding_vector = ARRAY[0.1, 0.3, 0.8, 0.2, 0.7, 0.4, 0.9, 0.1, 0.6, 0.5, 0.3, 0.8, 0.2, 0.7, 0.4, 0.9]::real[]
WHERE attribute_name IN ('full_name', 'date_of_birth', 'email', 'phone_number', 'address')
AND array_length(ARRAY[0.1, 0.3, 0.8, 0.2, 0.7, 0.4, 0.9, 0.1, 0.6, 0.5, 0.3, 0.8, 0.2, 0.7, 0.4, 0.9]::real[], 1) <= 1536;

-- Add derived attribute embeddings (more similar to each other)
UPDATE attribute_objects SET
    embedding_vector = ARRAY[0.8, 0.7, 0.9, 0.6, 0.8, 0.7, 0.9, 0.8, 0.6, 0.7, 0.8, 0.9, 0.7, 0.8, 0.6, 0.9]::real[]
WHERE attribute_name = 'risk_score'
AND array_length(ARRAY[0.8, 0.7, 0.9, 0.6, 0.8, 0.7, 0.9, 0.8, 0.6, 0.7, 0.8, 0.9, 0.7, 0.8, 0.6, 0.9]::real[], 1) <= 1536;

-- Insert into attribute_cluster_memberships table for clustering
INSERT INTO attribute_cluster_memberships (attribute_id, cluster_id, membership_strength, assigned_at)
SELECT
    ao.id,
    1, -- Identity cluster
    0.9,
    CURRENT_TIMESTAMP
FROM attribute_objects ao
WHERE ao.attribute_name IN ('full_name', 'date_of_birth', 'address')
ON CONFLICT (attribute_id, cluster_id) DO NOTHING;

INSERT INTO attribute_cluster_memberships (attribute_id, cluster_id, membership_strength, assigned_at)
SELECT
    ao.id,
    2, -- Contact cluster
    0.8,
    CURRENT_TIMESTAMP
FROM attribute_objects ao
WHERE ao.attribute_name IN ('email', 'phone_number')
ON CONFLICT (attribute_id, cluster_id) DO NOTHING;

INSERT INTO attribute_cluster_memberships (attribute_id, cluster_id, membership_strength, assigned_at)
SELECT
    ao.id,
    5, -- Risk cluster
    1.0,
    CURRENT_TIMESTAMP
FROM attribute_objects ao
WHERE ao.attribute_name = 'risk_score'
ON CONFLICT (attribute_id, cluster_id) DO NOTHING;

-- Add attribute tags
INSERT INTO attribute_tag_assignments (attribute_id, tag_id, assigned_at)
SELECT
    ao.id,
    1, -- 'required' tag
    CURRENT_TIMESTAMP
FROM attribute_objects ao
WHERE ao.attribute_name IN ('full_name', 'date_of_birth', 'email', 'address')
AND ao.is_required = true
ON CONFLICT (attribute_id, tag_id) DO NOTHING;

INSERT INTO attribute_tag_assignments (attribute_id, tag_id, assigned_at)
SELECT
    ao.id,
    2, -- 'kyc' tag
    CURRENT_TIMESTAMP
FROM attribute_objects ao
WHERE ao.attribute_name IN ('full_name', 'date_of_birth', 'address', 'document_type')
ON CONFLICT (attribute_id, tag_id) DO NOTHING;

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
    WHERE embedding_vector IS NOT NULL;

    SELECT COUNT(*) INTO with_clusters
    FROM attribute_cluster_memberships;

    RAISE NOTICE 'Enhanced % attributes with comprehensive metadata', enhanced_count;
    RAISE NOTICE 'Added embeddings to % attributes', with_embeddings;
    RAISE NOTICE 'Added % cluster assignments', with_clusters;
END $$;