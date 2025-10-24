-- Attribute Classification Schema
-- Distinguishes between real/public attributes and derived/synthetic attributes with EBNF rules

-- Add classification to the main attributes table
ALTER TABLE attribute_objects
ADD COLUMN IF NOT EXISTS attribute_class VARCHAR(20) DEFAULT 'real' CHECK (attribute_class IN ('real', 'derived')),
ADD COLUMN IF NOT EXISTS visibility_scope VARCHAR(20) DEFAULT 'public' CHECK (visibility_scope IN ('public', 'private', 'internal', 'restricted')),
ADD COLUMN IF NOT EXISTS derivation_rule_ebnf TEXT, -- EBNF notation for derivation rules
ADD COLUMN IF NOT EXISTS derivation_dependencies INTEGER[], -- Array of attribute IDs this derives from
ADD COLUMN IF NOT EXISTS derivation_complexity VARCHAR(20) DEFAULT 'simple' CHECK (derivation_complexity IN ('simple', 'moderate', 'complex', 'ai_assisted')),
ADD COLUMN IF NOT EXISTS derivation_frequency VARCHAR(20) DEFAULT 'on_demand' CHECK (derivation_frequency IN ('real_time', 'on_demand', 'batch', 'scheduled')),
ADD COLUMN IF NOT EXISTS materialization_strategy VARCHAR(30) DEFAULT 'computed' CHECK (materialization_strategy IN ('computed', 'cached', 'persisted', 'hybrid')),
ADD COLUMN IF NOT EXISTS source_attributes JSONB DEFAULT '[]', -- Detailed source attribute mapping
ADD COLUMN IF NOT EXISTS transformation_logic JSONB DEFAULT '{}', -- Structured transformation rules
ADD COLUMN IF NOT EXISTS quality_rules JSONB DEFAULT '{}'; -- Data quality and validation rules

-- Create derived attributes metadata table for complex derivations
CREATE TABLE IF NOT EXISTS derived_attribute_rules (
    id SERIAL PRIMARY KEY,
    derived_attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    rule_name VARCHAR(100) NOT NULL,
    rule_description TEXT,
    ebnf_grammar TEXT NOT NULL, -- Full EBNF grammar definition
    execution_order INTEGER DEFAULT 1, -- For multi-step derivations
    rule_type VARCHAR(50) DEFAULT 'transformation', -- 'transformation', 'aggregation', 'validation', 'enrichment'
    input_attributes JSONB NOT NULL, -- Source attributes with their roles
    output_format JSONB DEFAULT '{}', -- Expected output format and type
    error_handling JSONB DEFAULT '{}', -- Error handling strategies
    performance_hints JSONB DEFAULT '{}', -- Optimization hints
    test_cases JSONB DEFAULT '[]', -- Test cases for validation
    version VARCHAR(20) DEFAULT '1.0',
    is_active BOOLEAN DEFAULT true,
    created_by VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- EBNF Grammar templates and patterns
CREATE TABLE IF NOT EXISTS ebnf_grammar_templates (
    id SERIAL PRIMARY KEY,
    template_name VARCHAR(100) UNIQUE NOT NULL,
    template_description TEXT,
    ebnf_pattern TEXT NOT NULL,
    parameter_placeholders JSONB DEFAULT '{}', -- Placeholders in the pattern
    use_cases TEXT[],
    complexity_level VARCHAR(20) DEFAULT 'simple',
    example_usage TEXT,
    documentation_url TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Derivation execution history and audit
CREATE TABLE IF NOT EXISTS derivation_execution_log (
    id SERIAL PRIMARY KEY,
    derived_attribute_id INTEGER REFERENCES attribute_objects(id),
    execution_id UUID DEFAULT gen_random_uuid(),
    input_values JSONB NOT NULL,
    output_value JSONB,
    execution_time_ms INTEGER,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    rule_version VARCHAR(20),
    executed_by VARCHAR(100),
    execution_context JSONB DEFAULT '{}', -- Context information
    execution_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Data lineage tracking for derived attributes
CREATE TABLE IF NOT EXISTS attribute_lineage (
    id SERIAL PRIMARY KEY,
    target_attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    source_attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    lineage_type VARCHAR(50) NOT NULL, -- 'direct_input', 'indirect_input', 'lookup_reference', 'validation_reference'
    transformation_step INTEGER DEFAULT 1,
    contribution_weight DECIMAL(4,3) DEFAULT 1.0, -- How much this source contributes (0.0-1.0)
    lineage_description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(target_attribute_id, source_attribute_id, lineage_type)
);

-- Quality metrics for derived attributes
CREATE TABLE IF NOT EXISTS derived_attribute_quality_metrics (
    id SERIAL PRIMARY KEY,
    attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    metric_date DATE DEFAULT CURRENT_DATE,
    execution_count INTEGER DEFAULT 0,
    success_rate DECIMAL(5,2), -- Percentage 0.00-100.00
    avg_execution_time_ms INTEGER,
    error_count INTEGER DEFAULT 0,
    data_quality_score DECIMAL(4,3), -- 0.000-1.000
    completeness_rate DECIMAL(5,2), -- Percentage of non-null results
    accuracy_score DECIMAL(4,3), -- Based on validation rules
    consistency_score DECIMAL(4,3), -- Consistency across derivations
    metadata JSONB DEFAULT '{}',
    UNIQUE(attribute_id, metric_date)
);

-- Update AttributeObject struct to include all new fields
-- (This comment indicates the Rust struct needs updating)

-- Comprehensive view for attribute classification and derivation
CREATE OR REPLACE VIEW attribute_classification_view AS
SELECT
    ao.id,
    ao.attribute_name,
    ao.attribute_class,
    ao.visibility_scope,
    ao.data_type,
    ao.description,
    ao.extended_description,

    -- Classification details
    CASE
        WHEN ao.attribute_class = 'real' THEN 'Real/Public Attribute'
        WHEN ao.attribute_class = 'derived' THEN 'Derived/Synthetic Attribute'
    END as classification_description,

    -- Derivation information
    ao.derivation_rule_ebnf,
    ao.derivation_dependencies,
    ao.derivation_complexity,
    ao.derivation_frequency,
    ao.materialization_strategy,

    -- Source attribute details
    CASE
        WHEN ao.attribute_class = 'derived' THEN
            (SELECT array_agg(
                jsonb_build_object(
                    'id', src_ao.id,
                    'name', src_ao.attribute_name,
                    'type', src_ao.data_type,
                    'class', src_ao.attribute_class
                )
            )
            FROM unnest(ao.derivation_dependencies) as dep_id
            JOIN attribute_objects src_ao ON src_ao.id = dep_id)
        ELSE NULL
    END as source_attribute_details,

    -- Derivation rules
    array_agg(
        CASE WHEN dar.id IS NOT NULL THEN
            jsonb_build_object(
                'rule_name', dar.rule_name,
                'ebnf_grammar', dar.ebnf_grammar,
                'rule_type', dar.rule_type,
                'execution_order', dar.execution_order,
                'is_active', dar.is_active
            )
        ELSE NULL END
    ) FILTER (WHERE dar.id IS NOT NULL) as derivation_rules,

    -- Quality metrics (latest)
    daqm.success_rate,
    daqm.avg_execution_time_ms,
    daqm.data_quality_score,
    daqm.completeness_rate,

    -- AI and UI metadata
    ao.ai_context,
    ao.semantic_tags,
    ao.ui_component_type,
    ao.ui_layout_config,

    -- Full context for AI
    CONCAT_WS(' | ',
        ao.attribute_name,
        CASE ao.attribute_class
            WHEN 'real' THEN 'Real attribute directly collected from users or systems'
            WHEN 'derived' THEN 'Derived attribute computed from other attributes using: ' || COALESCE(ao.derivation_rule_ebnf, 'undefined rules')
        END,
        ao.description,
        ao.extended_description,
        ao.business_context
    ) as full_ai_context

FROM attribute_objects ao
LEFT JOIN derived_attribute_rules dar ON dar.derived_attribute_id = ao.id AND dar.is_active = true
LEFT JOIN derived_attribute_quality_metrics daqm ON daqm.attribute_id = ao.id
    AND daqm.metric_date = (
        SELECT MAX(metric_date)
        FROM derived_attribute_quality_metrics daqm2
        WHERE daqm2.attribute_id = ao.id
    )
GROUP BY ao.id, ao.attribute_name, ao.attribute_class, ao.visibility_scope, ao.data_type,
         ao.description, ao.extended_description, ao.derivation_rule_ebnf, ao.derivation_dependencies,
         ao.derivation_complexity, ao.derivation_frequency, ao.materialization_strategy,
         ao.ai_context, ao.semantic_tags, ao.ui_component_type, ao.ui_layout_config,
         ao.business_context, daqm.success_rate, daqm.avg_execution_time_ms,
         daqm.data_quality_score, daqm.completeness_rate;

-- Function to validate EBNF grammar syntax
CREATE OR REPLACE FUNCTION validate_ebnf_grammar(ebnf_text TEXT) RETURNS JSONB AS $$
DECLARE
    result JSONB;
    error_count INTEGER := 0;
    warnings TEXT[] := '{}';
BEGIN
    -- Basic EBNF validation checks
    -- Check for balanced parentheses, brackets, and braces
    IF (length(ebnf_text) - length(replace(ebnf_text, '(', ''))) !=
       (length(ebnf_text) - length(replace(ebnf_text, ')', ''))) THEN
        error_count := error_count + 1;
        warnings := array_append(warnings, 'Unbalanced parentheses in EBNF grammar');
    END IF;

    IF (length(ebnf_text) - length(replace(ebnf_text, '[', ''))) !=
       (length(ebnf_text) - length(replace(ebnf_text, ']', ''))) THEN
        error_count := error_count + 1;
        warnings := array_append(warnings, 'Unbalanced square brackets in EBNF grammar');
    END IF;

    IF (length(ebnf_text) - length(replace(ebnf_text, '{', ''))) !=
       (length(ebnf_text) - length(replace(ebnf_text, '}', ''))) THEN
        error_count := error_count + 1;
        warnings := array_append(warnings, 'Unbalanced curly braces in EBNF grammar');
    END IF;

    -- Check for proper rule definitions (contains ::= or =)
    IF ebnf_text !~ '(::=|=)' THEN
        error_count := error_count + 1;
        warnings := array_append(warnings, 'No rule definitions found (missing ::= or =)');
    END IF;

    -- Check for common EBNF syntax issues
    IF ebnf_text ~ '\|\|' THEN
        warnings := array_append(warnings, 'Double pipe (||) found - did you mean single pipe (|)?');
    END IF;

    result := jsonb_build_object(
        'is_valid', error_count = 0,
        'error_count', error_count,
        'warnings', to_jsonb(warnings),
        'complexity_score', CASE
            WHEN length(ebnf_text) < 100 THEN 'simple'
            WHEN length(ebnf_text) < 500 THEN 'moderate'
            ELSE 'complex'
        END
    );

    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Function to execute derivation rule
CREATE OR REPLACE FUNCTION execute_derivation_rule(
    derived_attr_id INTEGER,
    input_context JSONB
) RETURNS JSONB AS $$
DECLARE
    attr_info attribute_objects%ROWTYPE;
    derivation_result JSONB;
    execution_start TIMESTAMP;
    execution_duration INTEGER;
    success BOOLEAN := true;
    error_msg TEXT;
BEGIN
    execution_start := clock_timestamp();

    -- Get attribute information
    SELECT * INTO attr_info
    FROM attribute_objects
    WHERE id = derived_attr_id AND attribute_class = 'derived';

    IF NOT FOUND THEN
        RETURN jsonb_build_object(
            'success', false,
            'error', 'Derived attribute not found or not a derived attribute',
            'result', null
        );
    END IF;

    -- For now, implement basic derivation logic
    -- In a full implementation, this would parse and execute the EBNF rules
    BEGIN
        -- Placeholder for EBNF rule execution
        -- This would be replaced with a proper EBNF parser and executor
        IF attr_info.derivation_rule_ebnf IS NOT NULL THEN
            -- Simple example: if rule contains "CONCAT", concatenate input values
            IF attr_info.derivation_rule_ebnf ILIKE '%CONCAT%' THEN
                derivation_result := jsonb_build_object(
                    'type', 'concatenation',
                    'value', jsonb_array_elements_text(input_context->'values'),
                    'rule_applied', attr_info.derivation_rule_ebnf
                );
            ELSE
                derivation_result := jsonb_build_object(
                    'type', 'computed',
                    'value', input_context,
                    'rule_applied', attr_info.derivation_rule_ebnf
                );
            END IF;
        ELSE
            derivation_result := jsonb_build_object(
                'type', 'passthrough',
                'value', input_context,
                'warning', 'No derivation rule defined'
            );
        END IF;

    EXCEPTION WHEN OTHERS THEN
        success := false;
        error_msg := SQLERRM;
        derivation_result := NULL;
    END;

    execution_duration := EXTRACT(MILLISECONDS FROM clock_timestamp() - execution_start);

    -- Log execution
    INSERT INTO derivation_execution_log (
        derived_attribute_id, input_values, output_value,
        execution_time_ms, success, error_message
    ) VALUES (
        derived_attr_id, input_context, derivation_result,
        execution_duration, success, error_msg
    );

    RETURN jsonb_build_object(
        'success', success,
        'result', derivation_result,
        'execution_time_ms', execution_duration,
        'error', error_msg
    );
END;
$$ LANGUAGE plpgsql;

-- Insert sample EBNF grammar templates
INSERT INTO ebnf_grammar_templates (template_name, template_description, ebnf_pattern, parameter_placeholders, use_cases) VALUES
(
    'simple_concatenation',
    'Concatenate two or more string attributes',
    'result ::= {source_attr} (" " {source_attr})*',
    '{"source_attr": "attribute reference"}',
    ARRAY['Full name from first + last name', 'Address concatenation', 'Description building']
),
(
    'conditional_assignment',
    'Assign value based on condition',
    'result ::= IF {condition} THEN {true_value} ELSE {false_value}',
    '{"condition": "boolean expression", "true_value": "value when true", "false_value": "value when false"}',
    ARRAY['Risk classification', 'Customer tier assignment', 'Status determination']
),
(
    'lookup_transformation',
    'Transform value using lookup table',
    'result ::= LOOKUP({source_attr}, {lookup_table})',
    '{"source_attr": "source attribute", "lookup_table": "reference table"}',
    ARRAY['Country code to country name', 'Risk score to risk level', 'Currency conversion']
),
(
    'arithmetic_calculation',
    'Perform arithmetic operations',
    'result ::= {operand1} {operator} {operand2}',
    '{"operand1": "first operand", "operator": "arithmetic operator", "operand2": "second operand"}',
    ARRAY['Total calculation', 'Percentage computation', 'Score aggregation']
),
(
    'validation_rule',
    'Validate data against business rules',
    'result ::= VALIDATE({source_attr}, {rule_expr})',
    '{"source_attr": "attribute to validate", "rule_expr": "validation expression"}',
    ARRAY['Email format validation', 'Age verification', 'Document completeness check']
),
(
    'aggregation_rule',
    'Aggregate multiple values',
    'result ::= {agg_function}({source_attrs})',
    '{"agg_function": "aggregation function", "source_attrs": "list of attributes"}',
    ARRAY['Total amount calculation', 'Average score', 'Maximum value selection']
);

-- Insert sample derived attributes for KYC/Onboarding
-- This would be done after the real attributes are defined

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_attribute_objects_class ON attribute_objects(attribute_class);
CREATE INDEX IF NOT EXISTS idx_attribute_objects_visibility ON attribute_objects(visibility_scope);
CREATE INDEX IF NOT EXISTS idx_attribute_objects_derivation_deps ON attribute_objects USING GIN(derivation_dependencies);
CREATE INDEX IF NOT EXISTS idx_derived_attribute_rules_derived_attr ON derived_attribute_rules(derived_attribute_id);
CREATE INDEX IF NOT EXISTS idx_derivation_execution_log_attr ON derivation_execution_log(derived_attribute_id);
CREATE INDEX IF NOT EXISTS idx_derivation_execution_log_timestamp ON derivation_execution_log(execution_timestamp);
CREATE INDEX IF NOT EXISTS idx_attribute_lineage_target ON attribute_lineage(target_attribute_id);
CREATE INDEX IF NOT EXISTS idx_attribute_lineage_source ON attribute_lineage(source_attribute_id);

-- Validation trigger for derived attributes
CREATE OR REPLACE FUNCTION validate_derived_attribute() RETURNS TRIGGER AS $$
BEGIN
    -- Validate EBNF grammar if it's a derived attribute
    IF NEW.attribute_class = 'derived' AND NEW.derivation_rule_ebnf IS NOT NULL THEN
        DECLARE
            validation_result JSONB;
        BEGIN
            validation_result := validate_ebnf_grammar(NEW.derivation_rule_ebnf);
            IF NOT (validation_result->>'is_valid')::BOOLEAN THEN
                RAISE EXCEPTION 'Invalid EBNF grammar: %', validation_result->>'warnings';
            END IF;
        END;
    END IF;

    -- Ensure derived attributes have dependencies
    IF NEW.attribute_class = 'derived' AND
       (NEW.derivation_dependencies IS NULL OR array_length(NEW.derivation_dependencies, 1) = 0) THEN
        RAISE WARNING 'Derived attribute % has no dependencies defined', NEW.attribute_name;
    END IF;

    -- Set default materialization strategy based on complexity
    IF NEW.attribute_class = 'derived' AND NEW.materialization_strategy IS NULL THEN
        NEW.materialization_strategy := CASE NEW.derivation_complexity
            WHEN 'simple' THEN 'computed'
            WHEN 'moderate' THEN 'cached'
            WHEN 'complex' THEN 'persisted'
            WHEN 'ai_assisted' THEN 'hybrid'
            ELSE 'computed'
        END;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER validate_derived_attribute_trigger
    BEFORE INSERT OR UPDATE ON attribute_objects
    FOR EACH ROW
    WHEN (NEW.attribute_class = 'derived')
    EXECUTE FUNCTION validate_derived_attribute();

-- Comments for documentation
COMMENT ON COLUMN attribute_objects.attribute_class IS 'Classification: real (directly collected) or derived (computed from other attributes)';
COMMENT ON COLUMN attribute_objects.visibility_scope IS 'Visibility scope: public, private, internal, or restricted';
COMMENT ON COLUMN attribute_objects.derivation_rule_ebnf IS 'EBNF grammar notation for derivation rules (derived attributes only)';
COMMENT ON COLUMN attribute_objects.derivation_dependencies IS 'Array of attribute IDs this derived attribute depends on';
COMMENT ON COLUMN attribute_objects.derivation_complexity IS 'Complexity level of the derivation logic';
COMMENT ON COLUMN attribute_objects.materialization_strategy IS 'How the derived value is computed and stored';
COMMENT ON TABLE derived_attribute_rules IS 'Detailed derivation rules with EBNF grammar for complex derived attributes';
COMMENT ON TABLE ebnf_grammar_templates IS 'Reusable EBNF grammar patterns for common derivation scenarios';
COMMENT ON TABLE derivation_execution_log IS 'Audit log of all derivation rule executions';
COMMENT ON TABLE attribute_lineage IS 'Data lineage tracking for understanding attribute dependencies';
COMMENT ON FUNCTION validate_ebnf_grammar IS 'Validates EBNF grammar syntax for derivation rules';
COMMENT ON FUNCTION execute_derivation_rule IS 'Executes derivation rule for a derived attribute';
COMMENT ON VIEW attribute_classification_view IS 'Comprehensive view of attribute classification and derivation metadata';