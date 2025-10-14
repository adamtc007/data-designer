-- AI LLM Enhancement Schema
-- Provides extensive text descriptions and semantic context for AI understanding

-- Enhanced text descriptions for AI LLM context
ALTER TABLE attribute_objects
ADD COLUMN IF NOT EXISTS extended_description TEXT, -- Comprehensive human-readable description
ADD COLUMN IF NOT EXISTS business_context TEXT, -- Business purpose and usage context
ADD COLUMN IF NOT EXISTS technical_context TEXT, -- Technical implementation details
ADD COLUMN IF NOT EXISTS user_guidance TEXT, -- User-facing help and guidance
ADD COLUMN IF NOT EXISTS ai_training_examples TEXT, -- Examples for AI training and context
ADD COLUMN IF NOT EXISTS domain_terminology TEXT, -- Domain-specific terms and definitions
ADD COLUMN IF NOT EXISTS related_concepts TEXT[], -- Array of related concepts and synonyms
ADD COLUMN IF NOT EXISTS usage_scenarios TEXT, -- Common usage scenarios and patterns
ADD COLUMN IF NOT EXISTS data_lineage_description TEXT, -- Where data comes from and goes to
ADD COLUMN IF NOT EXISTS compliance_explanation TEXT, -- Why this field is required for compliance
ADD COLUMN IF NOT EXISTS error_scenarios TEXT, -- Common error scenarios and solutions
ADD COLUMN IF NOT EXISTS integration_notes TEXT; -- Notes about system integrations

-- Create comprehensive AI context table for rich descriptions
CREATE TABLE IF NOT EXISTS ai_attribute_contexts (
    id SERIAL PRIMARY KEY,
    attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    context_type VARCHAR(50) NOT NULL, -- 'business', 'technical', 'regulatory', 'user_experience'
    context_title VARCHAR(200) NOT NULL,
    detailed_description TEXT NOT NULL,
    examples JSONB DEFAULT '[]',
    keywords TEXT[],
    related_attributes INTEGER[], -- Related attribute IDs
    confidence_score DECIMAL(3,2) DEFAULT 1.0,
    source VARCHAR(100), -- 'manual', 'ai_generated', 'imported', 'inferred'
    last_validated TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- AI-specific semantic understanding tables
CREATE TABLE IF NOT EXISTS attribute_semantic_relationships (
    id SERIAL PRIMARY KEY,
    source_attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    target_attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    relationship_type VARCHAR(50) NOT NULL, -- 'synonym', 'related', 'prerequisite', 'derived_from', 'validates'
    relationship_description TEXT NOT NULL,
    semantic_similarity DECIMAL(4,3), -- 0.000-1.000 similarity score
    context_specific BOOLEAN DEFAULT false,
    domain_context VARCHAR(100),
    ai_confidence DECIMAL(3,2) DEFAULT 1.0,
    human_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- AI prompt templates and context for different scenarios
CREATE TABLE IF NOT EXISTS ai_prompt_contexts (
    id SERIAL PRIMARY KEY,
    context_name VARCHAR(100) UNIQUE NOT NULL,
    scenario_description TEXT NOT NULL,
    base_prompt_template TEXT NOT NULL,
    attribute_inclusion_rules JSONB DEFAULT '{}', -- Rules for which attributes to include
    response_format_template JSONB DEFAULT '{}',
    few_shot_examples JSONB DEFAULT '[]',
    model_parameters JSONB DEFAULT '{}', -- temperature, max_tokens, etc.
    success_criteria TEXT,
    common_failure_modes TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Rich examples and training data for AI
CREATE TABLE IF NOT EXISTS ai_training_examples (
    id SERIAL PRIMARY KEY,
    attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    example_type VARCHAR(50) NOT NULL, -- 'valid_input', 'invalid_input', 'edge_case', 'transformation'
    input_example JSONB NOT NULL,
    expected_output JSONB,
    explanation TEXT NOT NULL,
    difficulty_level VARCHAR(20) DEFAULT 'medium', -- 'easy', 'medium', 'hard', 'expert'
    tags TEXT[],
    ai_model_accuracy DECIMAL(4,3), -- How well AI handles this example
    human_annotation TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Comprehensive attribute documentation for AI understanding
CREATE TABLE IF NOT EXISTS attribute_documentation (
    id SERIAL PRIMARY KEY,
    attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    documentation_type VARCHAR(50) NOT NULL, -- 'overview', 'technical_spec', 'business_rules', 'user_guide'
    title VARCHAR(200) NOT NULL,
    content TEXT NOT NULL,
    content_format VARCHAR(20) DEFAULT 'markdown', -- 'markdown', 'html', 'plain_text'
    tags TEXT[],
    target_audience VARCHAR(50), -- 'developers', 'business_users', 'compliance', 'ai_systems'
    complexity_level VARCHAR(20) DEFAULT 'intermediate',
    last_reviewed TIMESTAMP,
    reviewer VARCHAR(100),
    version VARCHAR(20) DEFAULT '1.0',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- AI-enhanced glossary and terminology
CREATE TABLE IF NOT EXISTS domain_terminology_glossary (
    id SERIAL PRIMARY KEY,
    term VARCHAR(200) UNIQUE NOT NULL,
    definition TEXT NOT NULL,
    domain_context VARCHAR(100), -- 'kyc', 'onboarding', 'compliance', 'general'
    synonyms TEXT[],
    related_terms TEXT[],
    usage_examples TEXT[],
    regulatory_significance TEXT,
    common_misconceptions TEXT,
    ai_context_notes TEXT, -- Notes specifically for AI understanding
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Link terms to attributes for semantic understanding
CREATE TABLE IF NOT EXISTS attribute_terminology_links (
    id SERIAL PRIMARY KEY,
    attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    term_id INTEGER REFERENCES domain_terminology_glossary(id) ON DELETE CASCADE,
    relationship_type VARCHAR(50) DEFAULT 'related', -- 'defined_by', 'related', 'synonym', 'example_of'
    importance_weight DECIMAL(3,2) DEFAULT 1.0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(attribute_id, term_id, relationship_type)
);

-- Create comprehensive AI context view
CREATE OR REPLACE VIEW ai_enhanced_attribute_view AS
SELECT
    ao.id,
    ao.attribute_name,
    ao.data_type,
    ao.description,

    -- Rich text content for AI
    ao.extended_description,
    ao.business_context,
    ao.technical_context,
    ao.user_guidance,
    ao.ai_training_examples,
    ao.domain_terminology,
    ao.usage_scenarios,
    ao.data_lineage_description,
    ao.compliance_explanation,
    ao.error_scenarios,
    ao.integration_notes,

    -- Aggregated AI contexts
    array_agg(DISTINCT aac.detailed_description) FILTER (WHERE aac.detailed_description IS NOT NULL) as ai_contexts,
    array_agg(DISTINCT aac.context_type) FILTER (WHERE aac.context_type IS NOT NULL) as context_types,

    -- Related attributes for semantic understanding
    array_agg(DISTINCT asr_out.target_attribute_id) FILTER (WHERE asr_out.target_attribute_id IS NOT NULL) as related_attributes,
    array_agg(DISTINCT asr_out.relationship_type) FILTER (WHERE asr_out.relationship_type IS NOT NULL) as relationship_types,

    -- Documentation content
    string_agg(DISTINCT ad.content, ' ') FILTER (WHERE ad.content IS NOT NULL) as full_documentation,
    array_agg(DISTINCT ad.documentation_type) FILTER (WHERE ad.documentation_type IS NOT NULL) as doc_types,

    -- Terminology and glossary
    array_agg(DISTINCT dtg.term) FILTER (WHERE dtg.term IS NOT NULL) as related_terms,
    array_agg(DISTINCT dtg.definition) FILTER (WHERE dtg.definition IS NOT NULL) as term_definitions,

    -- Training examples
    count(DISTINCT ate.id) as training_examples_count,
    array_agg(DISTINCT ate.example_type) FILTER (WHERE ate.example_type IS NOT NULL) as example_types,

    -- Full searchable text for AI (concatenated descriptions)
    CONCAT_WS(' | ',
        ao.attribute_name,
        ao.description,
        ao.extended_description,
        ao.business_context,
        ao.technical_context,
        ao.user_guidance,
        ao.usage_scenarios,
        ao.compliance_explanation
    ) as full_ai_context_text,

    -- Semantic tags and keywords
    ao.semantic_tags,
    ao.search_keywords,
    array_agg(DISTINCT unnest(aac.keywords)) FILTER (WHERE aac.keywords IS NOT NULL) as ai_keywords

FROM attribute_objects ao
LEFT JOIN ai_attribute_contexts aac ON aac.attribute_id = ao.id
LEFT JOIN attribute_semantic_relationships asr_out ON asr_out.source_attribute_id = ao.id
LEFT JOIN attribute_documentation ad ON ad.attribute_id = ao.id
LEFT JOIN attribute_terminology_links atl ON atl.attribute_id = ao.id
LEFT JOIN domain_terminology_glossary dtg ON dtg.id = atl.term_id
LEFT JOIN ai_training_examples ate ON ate.attribute_id = ao.id
GROUP BY ao.id, ao.attribute_name, ao.data_type, ao.description, ao.extended_description,
         ao.business_context, ao.technical_context, ao.user_guidance, ao.ai_training_examples,
         ao.domain_terminology, ao.usage_scenarios, ao.data_lineage_description,
         ao.compliance_explanation, ao.error_scenarios, ao.integration_notes,
         ao.semantic_tags, ao.search_keywords;

-- Function to generate comprehensive AI prompt context for an attribute
CREATE OR REPLACE FUNCTION generate_ai_attribute_context(
    attr_id INTEGER,
    context_type VARCHAR DEFAULT 'comprehensive'
) RETURNS TEXT AS $$
DECLARE
    attribute_info ai_enhanced_attribute_view%ROWTYPE;
    context_text TEXT;
    related_info TEXT;
    examples_text TEXT;
BEGIN
    -- Get comprehensive attribute information
    SELECT * INTO attribute_info
    FROM ai_enhanced_attribute_view
    WHERE id = attr_id;

    IF NOT FOUND THEN
        RETURN 'Attribute not found';
    END IF;

    -- Build comprehensive context
    context_text := format('
ATTRIBUTE: %s
TYPE: %s

DESCRIPTION:
%s

EXTENDED DESCRIPTION:
%s

BUSINESS CONTEXT:
%s

TECHNICAL CONTEXT:
%s

USER GUIDANCE:
%s

USAGE SCENARIOS:
%s

COMPLIANCE EXPLANATION:
%s

DATA LINEAGE:
%s

ERROR SCENARIOS:
%s

INTEGRATION NOTES:
%s
',
        attribute_info.attribute_name,
        attribute_info.data_type,
        COALESCE(attribute_info.description, 'No description provided'),
        COALESCE(attribute_info.extended_description, 'No extended description provided'),
        COALESCE(attribute_info.business_context, 'No business context provided'),
        COALESCE(attribute_info.technical_context, 'No technical context provided'),
        COALESCE(attribute_info.user_guidance, 'No user guidance provided'),
        COALESCE(attribute_info.usage_scenarios, 'No usage scenarios provided'),
        COALESCE(attribute_info.compliance_explanation, 'No compliance explanation provided'),
        COALESCE(attribute_info.data_lineage_description, 'No data lineage provided'),
        COALESCE(attribute_info.error_scenarios, 'No error scenarios provided'),
        COALESCE(attribute_info.integration_notes, 'No integration notes provided')
    );

    -- Add related attributes information
    IF array_length(attribute_info.related_attributes, 1) > 0 THEN
        SELECT string_agg(
            format('%s (%s): %s',
                related_ao.attribute_name,
                asr.relationship_type,
                COALESCE(related_ao.description, 'No description')
            ), E'\n'
        ) INTO related_info
        FROM unnest(attribute_info.related_attributes) as related_id
        JOIN attribute_objects related_ao ON related_ao.id = related_id
        JOIN attribute_semantic_relationships asr ON asr.target_attribute_id = related_id AND asr.source_attribute_id = attr_id;

        context_text := context_text || format('

RELATED ATTRIBUTES:
%s', COALESCE(related_info, 'No related attributes'));
    END IF;

    -- Add terminology definitions
    IF array_length(attribute_info.related_terms, 1) > 0 THEN
        context_text := context_text || format('

DOMAIN TERMINOLOGY:
%s', array_to_string(
            ARRAY(
                SELECT format('%s: %s', term, def)
                FROM unnest(attribute_info.related_terms, attribute_info.term_definitions) AS t(term, def)
            ), E'\n'
        ));
    END IF;

    -- Add training examples if available
    IF attribute_info.training_examples_count > 0 THEN
        SELECT string_agg(
            format('Example (%s): %s - %s',
                ate.example_type,
                ate.input_example::text,
                ate.explanation
            ), E'\n'
        ) INTO examples_text
        FROM ai_training_examples ate
        WHERE ate.attribute_id = attr_id
        LIMIT 5; -- Limit to prevent overly long context

        context_text := context_text || format('

EXAMPLES:
%s', COALESCE(examples_text, 'No examples available'));
    END IF;

    RETURN context_text;
END;
$$ LANGUAGE plpgsql;

-- Function to search attributes by semantic similarity (for AI)
CREATE OR REPLACE FUNCTION ai_semantic_attribute_search(
    search_query TEXT,
    max_results INTEGER DEFAULT 10
) RETURNS TABLE (
    attribute_id INTEGER,
    attribute_name VARCHAR,
    relevance_score NUMERIC,
    context_summary TEXT,
    related_terms TEXT[]
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        aeav.id,
        aeav.attribute_name,
        -- Simple text similarity scoring (can be enhanced with vector embeddings)
        GREATEST(
            similarity(search_query, aeav.full_ai_context_text),
            similarity(search_query, aeav.attribute_name),
            CASE WHEN aeav.search_keywords && string_to_array(lower(search_query), ' ') THEN 0.8 ELSE 0 END
        ) as relevance,
        LEFT(aeav.full_ai_context_text, 200) || '...' as summary,
        aeav.related_terms
    FROM ai_enhanced_attribute_view aeav
    WHERE aeav.full_ai_context_text ILIKE '%' || search_query || '%'
       OR aeav.attribute_name ILIKE '%' || search_query || '%'
       OR aeav.search_keywords && string_to_array(lower(search_query), ' ')
    ORDER BY relevance DESC
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql;

-- Insert sample AI contexts for KYC/Onboarding attributes
INSERT INTO ai_attribute_contexts (attribute_id, context_type, context_title, detailed_description, examples, keywords) VALUES
(
    (SELECT id FROM attribute_objects WHERE attribute_name = 'customer_email' LIMIT 1),
    'business',
    'Primary Customer Communication Channel',
    'The customer email address serves as the primary digital communication channel for all customer interactions, notifications, and compliance communications. This field is critical for KYC verification processes, account recovery, and regulatory notifications. The email must be validated and verified to ensure deliverability and compliance with anti-fraud measures.',
    '["user@example.com", "business.contact@company.co.uk", "customer+tag@domain.org"]',
    ARRAY['email', 'communication', 'verification', 'contact', 'primary', 'digital']
),
(
    (SELECT id FROM attribute_objects WHERE attribute_name = 'customer_email' LIMIT 1),
    'technical',
    'Email Validation and Storage Specifications',
    'Email addresses must conform to RFC 5322 standards with additional business rules: maximum length 254 characters, must contain @ symbol, domain validation required, disposable email domains blocked, corporate email domains preferred for business accounts. Stored in normalized lowercase format with original case preserved in metadata.',
    '["Valid: user@domain.com", "Invalid: user@.com", "Blocked: user@tempmail.org"]',
    ARRAY['RFC5322', 'validation', 'normalization', 'domain', 'regex', 'storage']
),
(
    (SELECT id FROM attribute_objects WHERE attribute_name = 'customer_email' LIMIT 1),
    'regulatory',
    'GDPR and Privacy Compliance Requirements',
    'Email addresses are considered personal data under GDPR and require explicit consent for processing. Must implement right to rectification, erasure, and data portability. Email communications must include unsubscribe mechanisms. Retention period limited to business necessity plus regulatory requirements (typically 7 years for financial services).',
    '["Consent management", "Data subject requests", "Retention policies"]',
    ARRAY['GDPR', 'privacy', 'consent', 'personal_data', 'retention', 'compliance']
);

-- Insert sample terminology
INSERT INTO domain_terminology_glossary (term, definition, domain_context, synonyms, usage_examples, regulatory_significance, ai_context_notes) VALUES
(
    'KYC',
    'Know Your Customer - A process of identity verification and due diligence that financial institutions must perform to verify the identity of their clients',
    'compliance',
    ARRAY['Customer Due Diligence', 'CDD', 'Identity Verification'],
    ARRAY['KYC documents required for account opening', 'Enhanced KYC for high-risk customers', 'Ongoing KYC monitoring'],
    'Mandatory under Anti-Money Laundering regulations in most jurisdictions',
    'When AI encounters KYC, understand it relates to identity verification, document collection, risk assessment, and regulatory compliance. Always emphasize security and privacy.'
),
(
    'PEP',
    'Politically Exposed Person - An individual who is or has been entrusted with a prominent public function, including their family members and close associates',
    'kyc',
    ARRAY['Politically Exposed Person', 'Public Official'],
    ARRAY['PEP screening required for all customers', 'Enhanced due diligence for PEP customers', 'PEP status monitoring'],
    'Requires enhanced due diligence under AML regulations',
    'PEP status indicates higher risk requiring additional scrutiny, documentation, and approval levels. AI should flag PEP-related queries as requiring enhanced compliance measures.'
),
(
    'Sanctions Screening',
    'The process of checking customers, transactions, and business partners against government and international sanctions lists',
    'compliance',
    ARRAY['Watchlist Screening', 'Sanctions Check', 'OFAC Screening'],
    ARRAY['Real-time sanctions screening', 'Batch sanctions screening', 'Sanctions alert investigation'],
    'Required under economic sanctions regulations (OFAC, EU, UN)',
    'Critical compliance process that can block transactions or relationships. AI should treat sanctions-related topics with high priority and emphasize immediate action requirements.'
);

-- Create indexes for AI performance
CREATE INDEX IF NOT EXISTS idx_ai_attribute_contexts_attribute ON ai_attribute_contexts(attribute_id);
CREATE INDEX IF NOT EXISTS idx_ai_attribute_contexts_type ON ai_attribute_contexts(context_type);
CREATE INDEX IF NOT EXISTS idx_ai_attribute_contexts_keywords ON ai_attribute_contexts USING GIN(keywords);
CREATE INDEX IF NOT EXISTS idx_attribute_semantic_relationships_source ON attribute_semantic_relationships(source_attribute_id);
CREATE INDEX IF NOT EXISTS idx_attribute_semantic_relationships_target ON attribute_semantic_relationships(target_attribute_id);
CREATE INDEX IF NOT EXISTS idx_ai_training_examples_attribute ON ai_training_examples(attribute_id);
CREATE INDEX IF NOT EXISTS idx_ai_training_examples_type ON ai_training_examples(example_type);
CREATE INDEX IF NOT EXISTS idx_attribute_documentation_attribute ON attribute_documentation(attribute_id);
CREATE INDEX IF NOT EXISTS idx_domain_terminology_glossary_term ON domain_terminology_glossary USING GIN(to_tsvector('english', term || ' ' || definition));
CREATE INDEX IF NOT EXISTS idx_ai_enhanced_view_full_text ON attribute_objects USING GIN(to_tsvector('english',
    COALESCE(attribute_name, '') || ' ' ||
    COALESCE(description, '') || ' ' ||
    COALESCE(extended_description, '') || ' ' ||
    COALESCE(business_context, '') || ' ' ||
    COALESCE(technical_context, '')
));

-- Comments for AI understanding
COMMENT ON COLUMN attribute_objects.extended_description IS 'Comprehensive description for AI LLM context and understanding';
COMMENT ON COLUMN attribute_objects.business_context IS 'Business purpose and usage context for AI decision making';
COMMENT ON COLUMN attribute_objects.technical_context IS 'Technical implementation details for AI code generation';
COMMENT ON COLUMN attribute_objects.user_guidance IS 'User-facing help text for AI user assistance';
COMMENT ON COLUMN attribute_objects.ai_training_examples IS 'Examples specifically for AI training and context';
COMMENT ON COLUMN attribute_objects.domain_terminology IS 'Domain-specific terminology for AI semantic understanding';
COMMENT ON COLUMN attribute_objects.usage_scenarios IS 'Common usage patterns for AI recommendation engines';
COMMENT ON TABLE ai_attribute_contexts IS 'Rich contextual descriptions for AI LLM understanding and reasoning';
COMMENT ON TABLE ai_training_examples IS 'Training examples and edge cases for AI model fine-tuning';
COMMENT ON TABLE domain_terminology_glossary IS 'Comprehensive glossary for AI semantic understanding of domain terms';
COMMENT ON FUNCTION generate_ai_attribute_context IS 'Generates comprehensive context text for AI LLM prompts';
COMMENT ON FUNCTION ai_semantic_attribute_search IS 'Semantic search function optimized for AI attribute discovery';