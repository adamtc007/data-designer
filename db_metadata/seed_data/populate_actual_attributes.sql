-- Populate Extended Attribute Metadata for Actual KYC/Trade Finance Attributes
-- Working with existing attributes: legal_entity_name, ubo_full_name, sanctions_screening_result, etc.

-- Update Legal Entity Name (ID: 1)
UPDATE attribute_objects SET
    extended_description = 'Official legal name of the business entity as registered with regulatory authorities. Used for entity verification, sanctions screening, and regulatory reporting.',
    business_context = 'Primary identifier for business entities in trade finance operations. Critical for entity verification, sanctions screening, regulatory compliance, and risk assessment.',
    technical_context = 'String field supporting Unicode characters for international business names. Stored with encryption and indexed for compliance lookups. Integrated with entity verification APIs.',
    user_guidance = 'Enter the complete legal business name exactly as it appears in your official registration documents (Articles of Incorporation, Certificate of Formation, etc.).',
    ai_training_examples = 'Business entity identification field requiring exact matching with official registration documents',
    domain_terminology = 'legal_entity: Officially registered business name; entity_verification: Process of confirming business legitimacy; regulatory_authority: Government body overseeing business registration',
    usage_scenarios = 'Entity verification during onboarding, sanctions screening, regulatory reporting, trade finance documentation',
    compliance_explanation = 'Required for entity verification under BSA, AML, and trade finance regulations. Must match official business registration documents.',
    semantic_tags = '["entity", "business", "legal_name", "verification", "required", "compliance"]',
    ai_context = '{"domain": "entity_verification", "criticality": "high", "compliance": ["BSA", "AML", "trade_finance"], "data_classification": "confidential"}',
    search_keywords = ARRAY['entity', 'business', 'legal', 'company', 'corporation', 'verification'],
    ui_component_type = 'business_name_input',
    ui_layout_config = '{"width": "100%", "placeholder": "Enter official business name", "autocomplete": false, "case_sensitive": true}',
    ui_styling = '{"font_weight": "500", "border_color": "#0066cc"}',
    ui_behavior = '{"trim_whitespace": false, "validate_on_blur": true}',
    conditional_logic = '{"validate_if": {"entity_type": "business"}}',
    relationship_metadata = '{"related_fields": ["ubo_full_name", "jurisdiction"], "validation_group": "entity_core"}',
    ai_prompt_templates = '{"validation": "Verify business entity name format and legitimacy", "suggestion": "Suggest proper business name format based on jurisdiction"}',
    attribute_class = 'real'
WHERE id = 1;

-- Update UBO Full Name (ID: 2)
UPDATE attribute_objects SET
    extended_description = 'Complete legal name of the Ultimate Beneficial Owner who has significant control or ownership (typically 25%+ ownership) of the business entity.',
    business_context = 'Critical for beneficial ownership transparency, AML compliance, and sanctions screening. Required to identify individuals who ultimately control or benefit from business transactions.',
    technical_context = 'String field with enhanced validation for individual names. Cross-referenced with sanctions lists and PEP databases. Stored with field-level encryption.',
    user_guidance = 'Provide the full legal name of the person who ultimately owns or controls 25% or more of the business, or who exercises significant control over the entity.',
    ai_training_examples = 'Ultimate beneficial owner identification for transparency and AML compliance',
    domain_terminology = 'ubo: Ultimate Beneficial Owner; beneficial_ownership: True ownership or control of entity; significant_control: Authority to direct entity operations',
    usage_scenarios = 'Beneficial ownership reporting, sanctions screening, AML compliance, risk assessment',
    compliance_explanation = 'Required under UBO regulations for transparency in business ownership. Critical for AML and sanctions compliance.',
    semantic_tags = '["ubo", "beneficial_owner", "individual", "control", "ownership", "aml"]',
    ai_context = '{"domain": "beneficial_ownership", "criticality": "high", "compliance": ["UBO", "AML", "sanctions"], "data_classification": "restricted"}',
    search_keywords = ARRAY['ubo', 'owner', 'beneficial', 'control', 'individual'],
    ui_component_type = 'person_name_input',
    ui_layout_config = '{"width": "100%", "placeholder": "Enter UBO full name", "validation": "person_name"}',
    ui_styling = '{"border_color": "#dc2626", "background": "#fef2f2"}',
    ui_behavior = '{"validate_sanctions": true, "check_pep": true}',
    conditional_logic = '{"required_if": {"entity_type": "business", "ownership_threshold": ">= 25%"}}',
    relationship_metadata = '{"related_fields": ["legal_entity_name"], "validation_group": "ownership_info"}',
    ai_prompt_templates = '{"sanctions_check": "Verify UBO name against sanctions and PEP lists", "validation": "Validate person name format and completeness"}',
    attribute_class = 'real'
WHERE id = 2;

-- Update Sanctions Screening Result (ID: 3)
UPDATE attribute_objects SET
    extended_description = 'Automated sanctions screening result indicating whether the entity or individual appears on any sanctions lists, watch lists, or enforcement actions.',
    business_context = 'Critical compliance control preventing transactions with sanctioned parties. Required for AML compliance and regulatory reporting. Determines transaction approval workflow.',
    technical_context = 'Enumerated field with automated population from sanctions screening APIs. Triggers workflow automation and compliance alerts. Audit trail maintained for all screening results.',
    user_guidance = 'This field is automatically populated by our sanctions screening system. Review any alerts or matches carefully with compliance team.',
    ai_training_examples = 'Automated compliance screening result for sanctions and watch list verification',
    domain_terminology = 'sanctions_screening: Process of checking against prohibited parties lists; false_positive: Incorrect match requiring manual review; sanctions_list: Government prohibited parties database',
    usage_scenarios = 'Transaction approval workflow, compliance reporting, risk assessment, regulatory audit',
    compliance_explanation = 'Required under OFAC, EU sanctions, and other regulatory sanctions programs. Mandatory for transaction processing.',
    semantic_tags = '["sanctions", "screening", "compliance", "automated", "workflow", "regulatory"]',
    ai_context = '{"domain": "sanctions_compliance", "criticality": "critical", "compliance": ["OFAC", "EU_sanctions", "UN_sanctions"], "data_classification": "restricted"}',
    search_keywords = ARRAY['sanctions', 'screening', 'compliance', 'ofac', 'watchlist'],
    ui_component_type = 'sanctions_result_display',
    ui_layout_config = '{"display_format": "status_badge", "show_details": true, "alert_on_match": true}',
    ui_styling = '{"clear": "green", "alert": "red", "review": "yellow"}',
    ui_behavior = '{"auto_refresh": true, "show_last_screened": true}',
    conditional_logic = '{"alert_if": {"value": ["MATCH", "POTENTIAL_MATCH"]}}',
    relationship_metadata = '{"triggers": ["compliance_review"], "affects": ["transaction_approval"]}',
    ai_prompt_templates = '{"alert": "Generate compliance alert for sanctions match", "review": "Prepare sanctions review summary"}',
    attribute_class = 'derived',
    derivation_rule_ebnf = 'sanctions_result := SCREEN_SANCTIONS(legal_entity_name, ubo_full_name) -> {"CLEAR", "MATCH", "POTENTIAL_MATCH", "ERROR"}',
    derivation_dependencies = ARRAY[1, 2] -- legal_entity_name, ubo_full_name
WHERE id = 3;

-- Update Risk Score (ID: 4)
UPDATE attribute_objects SET
    extended_description = 'Comprehensive risk assessment score calculated from multiple risk factors including entity type, jurisdiction, transaction patterns, sanctions screening results, and external risk indicators.',
    business_context = 'Primary metric for risk-based decision making, transaction monitoring thresholds, enhanced due diligence requirements, and regulatory reporting. Drives automated approval workflows.',
    technical_context = 'Calculated decimal field using machine learning models and rule-based scoring. Updated in real-time as risk factors change. Stored with calculation metadata and model version.',
    user_guidance = 'This risk score is automatically calculated based on various risk factors. Higher scores may require additional documentation or enhanced due diligence.',
    ai_training_examples = 'Automated risk assessment combining multiple risk factors for comprehensive entity risk profiling',
    domain_terminology = 'risk_assessment: Evaluation of potential threats; risk_factors: Elements contributing to overall risk; enhanced_due_diligence: Additional verification for high-risk entities',
    usage_scenarios = 'Transaction approval, enhanced due diligence triggering, regulatory reporting, portfolio risk management',
    compliance_explanation = 'Required for risk-based approach to AML compliance. Used for enhanced due diligence determinations and transaction monitoring.',
    semantic_tags = '["risk", "assessment", "calculated", "compliance", "automated", "decision_making"]',
    ai_context = '{"domain": "risk_management", "criticality": "high", "compliance": ["AML", "risk_assessment"], "data_classification": "internal"}',
    search_keywords = ARRAY['risk', 'score', 'assessment', 'calculation', 'compliance'],
    ui_component_type = 'risk_score_gauge',
    ui_layout_config = '{"display_format": "gauge", "color_coding": true, "show_factors": true, "breakdown_view": true}',
    ui_styling = '{"low_risk": "green", "medium_risk": "yellow", "high_risk": "red"}',
    ui_behavior = '{"auto_refresh": true, "show_calculation_timestamp": true, "expandable_details": true}',
    conditional_logic = '{"alert_if": {"value": ">= 70"}, "enhance_dd_if": {"value": ">= 85"}}',
    relationship_metadata = '{"input_fields": ["legal_entity_name", "ubo_full_name", "sanctions_screening_result", "jurisdiction"], "affects": ["transaction_approval", "due_diligence_level"]}',
    ai_prompt_templates = '{"explanation": "Explain risk score components", "recommendations": "Suggest risk mitigation measures"}',
    attribute_class = 'derived',
    derivation_rule_ebnf = 'risk_score := BASE_RISK + ENTITY_RISK(legal_entity_name) + JURISDICTION_RISK(jurisdiction) + SANCTIONS_RISK(sanctions_screening_result) + UBO_RISK(ubo_full_name)',
    derivation_dependencies = ARRAY[1, 2, 3, 5] -- legal_entity_name, ubo_full_name, sanctions_screening_result, jurisdiction
WHERE id = 4;

-- Update Jurisdiction (ID: 5)
UPDATE attribute_objects SET
    extended_description = 'Legal jurisdiction where the business entity is incorporated, registered, or primarily operates. Critical for regulatory compliance, tax obligations, and risk assessment.',
    business_context = 'Determines applicable regulations, compliance requirements, tax implications, and risk profiles. Essential for trade finance documentation and regulatory reporting.',
    technical_context = 'Standardized country/region code field mapped to regulatory frameworks and risk ratings. Integrated with sanctions screening and regulatory compliance systems.',
    user_guidance = 'Select the country or jurisdiction where your business is legally incorporated or registered. This affects compliance requirements and available services.',
    ai_training_examples = 'Business jurisdiction identification for regulatory compliance and risk assessment',
    domain_terminology = 'jurisdiction: Legal authority governing entity operations; incorporation: Legal process of business formation; regulatory_framework: Rules and requirements by jurisdiction',
    usage_scenarios = 'Regulatory compliance determination, risk assessment, tax reporting, trade finance documentation',
    compliance_explanation = 'Required for determining applicable regulations, sanctions compliance, and tax obligations under international trade finance rules.',
    semantic_tags = '["jurisdiction", "country", "regulation", "compliance", "incorporation", "legal"]',
    ai_context = '{"domain": "regulatory_compliance", "criticality": "high", "compliance": ["regulatory", "tax", "sanctions"], "data_classification": "internal"}',
    search_keywords = ARRAY['jurisdiction', 'country', 'incorporation', 'regulation', 'legal'],
    ui_component_type = 'jurisdiction_selector',
    ui_layout_config = '{"format": "searchable_dropdown", "show_risk_rating": true, "group_by_region": true}',
    ui_styling = '{"flag_icons": true, "risk_color_coding": true}',
    ui_behavior = '{"auto_complete": true, "validate_sanctions": true}',
    conditional_logic = '{"alert_if": {"risk_rating": "high"}}',
    relationship_metadata = '{"affects": ["risk_score", "regulatory_requirements"], "validation_group": "entity_core"}',
    ai_prompt_templates = '{"risk_info": "Provide jurisdiction risk information", "compliance": "List compliance requirements for jurisdiction"}',
    attribute_class = 'real'
WHERE id = 5;

-- Add simplified embeddings (16 dimensions to fit the example)
UPDATE attribute_objects SET
    embedding_vector = array_fill(0.0, ARRAY[1536])::real[]
WHERE id IN (1, 2, 3, 4, 5);

-- Update with actual sample embeddings (truncated for demonstration)
UPDATE attribute_objects SET
    embedding_vector = (SELECT array_agg(random()::real) FROM generate_series(1, 1536))
WHERE id = 1; -- legal_entity_name

UPDATE attribute_objects SET
    embedding_vector = (SELECT array_agg(random()::real) FROM generate_series(1, 1536))
WHERE id = 2; -- ubo_full_name

-- Add cluster memberships
INSERT INTO attribute_cluster_memberships (attribute_id, cluster_id, membership_strength)
VALUES
    (1, 1, 0.9), -- legal_entity_name in entity cluster
    (2, 1, 0.8), -- ubo_full_name in entity cluster
    (3, 2, 1.0), -- sanctions_screening_result in compliance cluster
    (4, 3, 1.0), -- risk_score in risk cluster
    (5, 1, 0.7)  -- jurisdiction in entity cluster
ON CONFLICT (attribute_id, cluster_id) DO UPDATE SET
    membership_strength = EXCLUDED.membership_strength;

-- Add some attribute tags
INSERT INTO attribute_tags (tag_name, description, color, category) VALUES
    ('required', 'Required field', '#dc2626', 'validation'),
    ('kyc', 'KYC related field', '#2563eb', 'compliance'),
    ('calculated', 'Automatically calculated field', '#059669', 'system'),
    ('high_risk', 'High risk field requiring special attention', '#dc2626', 'risk')
ON CONFLICT (tag_name) DO NOTHING;

INSERT INTO attribute_tag_assignments (attribute_id, tag_id)
SELECT ao.id, at.id
FROM attribute_objects ao, attribute_tags at
WHERE ao.id IN (1, 2, 5) AND at.tag_name = 'required'
ON CONFLICT (attribute_id, tag_id) DO NOTHING;

INSERT INTO attribute_tag_assignments (attribute_id, tag_id)
SELECT ao.id, at.id
FROM attribute_objects ao, attribute_tags at
WHERE ao.id IN (1, 2, 3, 5) AND at.tag_name = 'kyc'
ON CONFLICT (attribute_id, tag_id) DO NOTHING;

INSERT INTO attribute_tag_assignments (attribute_id, tag_id)
SELECT ao.id, at.id
FROM attribute_objects ao, attribute_tags at
WHERE ao.id IN (3, 4) AND at.tag_name = 'calculated'
ON CONFLICT (attribute_id, tag_id) DO NOTHING;

-- Show summary of what was populated
DO $$
DECLARE
    enhanced_count INTEGER;
    with_embeddings INTEGER;
    with_clusters INTEGER;
    with_tags INTEGER;
BEGIN
    SELECT COUNT(*) INTO enhanced_count
    FROM attribute_objects
    WHERE extended_description IS NOT NULL;

    SELECT COUNT(*) INTO with_embeddings
    FROM attribute_objects
    WHERE embedding_vector IS NOT NULL;

    SELECT COUNT(*) INTO with_clusters
    FROM attribute_cluster_memberships;

    SELECT COUNT(*) INTO with_tags
    FROM attribute_tag_assignments;

    RAISE NOTICE 'Enhanced % attributes with comprehensive metadata', enhanced_count;
    RAISE NOTICE 'Added embeddings to % attributes', with_embeddings;
    RAISE NOTICE 'Added % cluster assignments', with_clusters;
    RAISE NOTICE 'Added % tag assignments', with_tags;
END $$;