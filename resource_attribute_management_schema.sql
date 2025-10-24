-- Resource Attribute Set Management System
-- Provides comprehensive tooling for maintaining attribute sets within resources

-- Create resource attribute set templates for common patterns
CREATE TABLE IF NOT EXISTS resource_attribute_templates (
    id SERIAL PRIMARY KEY,
    template_name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    category VARCHAR(50), -- 'kyc', 'trade_finance', 'risk_management', 'compliance'
    attributes_config JSONB NOT NULL, -- Template configuration for attributes
    ui_layout_template JSONB DEFAULT '{}',
    validation_rules JSONB DEFAULT '{}',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create attribute set versioning for change management
CREATE TABLE IF NOT EXISTS resource_attribute_versions (
    id SERIAL PRIMARY KEY,
    resource_id INTEGER REFERENCES resource_objects(id) ON DELETE CASCADE,
    version_number VARCHAR(20) NOT NULL,
    change_description TEXT,
    attributes_snapshot JSONB NOT NULL, -- Full attribute configuration at this version
    schema_changes JSONB DEFAULT '{}', -- What changed from previous version
    created_by VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT false,
    UNIQUE(resource_id, version_number)
);

-- Create attribute set validation rules
CREATE TABLE IF NOT EXISTS resource_validation_rules (
    id SERIAL PRIMARY KEY,
    resource_id INTEGER REFERENCES resource_objects(id) ON DELETE CASCADE,
    rule_name VARCHAR(100) NOT NULL,
    rule_type VARCHAR(50) NOT NULL, -- 'required_attributes', 'dependency', 'consistency', 'completeness'
    rule_config JSONB NOT NULL,
    is_active BOOLEAN DEFAULT true,
    severity VARCHAR(20) DEFAULT 'error', -- 'warning', 'error', 'critical'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(resource_id, rule_name)
);

-- Create attribute set dependencies mapping
CREATE TABLE IF NOT EXISTS resource_attribute_dependencies (
    id SERIAL PRIMARY KEY,
    source_resource_id INTEGER REFERENCES resource_objects(id) ON DELETE CASCADE,
    target_resource_id INTEGER REFERENCES resource_objects(id) ON DELETE CASCADE,
    dependency_type VARCHAR(50) NOT NULL, -- 'uses', 'extends', 'references', 'inherits'
    dependency_config JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(source_resource_id, target_resource_id, dependency_type)
);

-- Enhanced resource management view
CREATE OR REPLACE VIEW resource_management_view AS
SELECT
    rd.dictionary_name,
    rd.version as dictionary_version,
    ro.id as resource_id,
    ro.resource_name,
    ro.description,
    ro.version as resource_version,
    ro.category,
    ro.status,
    ro.ui_layout,

    -- Attribute statistics
    COUNT(ao.id) as total_attributes,
    COUNT(CASE WHEN ao.is_required THEN 1 END) as required_attributes,
    COUNT(CASE WHEN ao.attribute_class = 'derived' THEN 1 END) as derived_attributes,
    COUNT(CASE WHEN ao.attribute_class = 'real' THEN 1 END) as real_attributes,

    -- Data completeness metrics
    COUNT(CASE WHEN ao.extended_description IS NOT NULL THEN 1 END) as documented_attributes,
    ROUND(
        COUNT(CASE WHEN ao.extended_description IS NOT NULL THEN 1 END)::DECIMAL /
        GREATEST(COUNT(ao.id), 1) * 100, 2
    ) as documentation_percentage,

    -- UI configuration completeness
    COUNT(CASE WHEN ao.ui_component_type IS NOT NULL THEN 1 END) as ui_configured_attributes,
    ROUND(
        COUNT(CASE WHEN ao.ui_component_type IS NOT NULL THEN 1 END)::DECIMAL /
        GREATEST(COUNT(ao.id), 1) * 100, 2
    ) as ui_configuration_percentage,

    -- AI enhancement completeness
    COUNT(CASE WHEN ao.semantic_tags IS NOT NULL AND ao.semantic_tags != '[]'::jsonb THEN 1 END) as ai_enhanced_attributes,
    ROUND(
        COUNT(CASE WHEN ao.semantic_tags IS NOT NULL AND ao.semantic_tags != '[]'::jsonb THEN 1 END)::DECIMAL /
        GREATEST(COUNT(ao.id), 1) * 100, 2
    ) as ai_enhancement_percentage,

    -- Validation completeness
    COUNT(CASE WHEN ao.validation_pattern IS NOT NULL OR ao.allowed_values IS NOT NULL THEN 1 END) as validated_attributes,

    -- Group organization
    array_agg(DISTINCT ao.ui_group ORDER BY ao.ui_group) FILTER (WHERE ao.ui_group IS NOT NULL) as ui_groups,

    -- Latest changes
    ro.updated_at as last_modified,
    MAX(ao.updated_at) as last_attribute_change

FROM resource_dictionaries rd
JOIN resource_objects ro ON ro.dictionary_id = rd.id
LEFT JOIN attribute_objects ao ON ao.resource_id = ro.id
GROUP BY
    rd.id, rd.dictionary_name, rd.version,
    ro.id, ro.resource_name, ro.description, ro.version, ro.category, ro.status, ro.ui_layout, ro.updated_at;

-- Function to validate resource attribute set integrity
CREATE OR REPLACE FUNCTION validate_resource_attribute_set(
    p_resource_id INTEGER
) RETURNS TABLE (
    rule_name TEXT,
    rule_type TEXT,
    severity TEXT,
    status TEXT,
    message TEXT,
    details JSONB
) AS $$
DECLARE
    resource_rec RECORD;
    attr_count INTEGER;
    required_count INTEGER;
    documented_count INTEGER;
BEGIN
    -- Get resource information
    SELECT * INTO resource_rec FROM resource_objects WHERE id = p_resource_id;

    IF NOT FOUND THEN
        RETURN QUERY SELECT
            'resource_exists'::TEXT,
            'critical'::TEXT,
            'critical'::TEXT,
            'FAIL'::TEXT,
            'Resource not found'::TEXT,
            jsonb_build_object('resource_id', p_resource_id);
        RETURN;
    END IF;

    -- Get attribute counts
    SELECT
        COUNT(*),
        COUNT(CASE WHEN is_required THEN 1 END),
        COUNT(CASE WHEN extended_description IS NOT NULL THEN 1 END)
    INTO attr_count, required_count, documented_count
    FROM attribute_objects
    WHERE resource_id = p_resource_id;

    -- Rule 1: Resource must have at least one attribute
    RETURN QUERY SELECT
        'minimum_attributes'::TEXT,
        'completeness'::TEXT,
        'error'::TEXT,
        CASE WHEN attr_count > 0 THEN 'PASS' ELSE 'FAIL' END,
        CASE WHEN attr_count > 0
            THEN format('Resource has %s attributes', attr_count)
            ELSE 'Resource must have at least one attribute'
        END,
        jsonb_build_object('attribute_count', attr_count);

    -- Rule 2: Resource should have at least one required attribute
    RETURN QUERY SELECT
        'required_attributes'::TEXT,
        'consistency'::TEXT,
        'warning'::TEXT,
        CASE WHEN required_count > 0 THEN 'PASS' ELSE 'WARN' END,
        CASE WHEN required_count > 0
            THEN format('Resource has %s required attributes', required_count)
            ELSE 'Resource should have at least one required attribute'
        END,
        jsonb_build_object('required_count', required_count, 'total_count', attr_count);

    -- Rule 3: At least 80% of attributes should be documented
    RETURN QUERY SELECT
        'documentation_completeness'::TEXT,
        'completeness'::TEXT,
        'warning'::TEXT,
        CASE WHEN documented_count::DECIMAL / GREATEST(attr_count, 1) >= 0.8 THEN 'PASS' ELSE 'WARN' END,
        format('Documentation completeness: %s%% (%s/%s attributes)',
            ROUND(documented_count::DECIMAL / GREATEST(attr_count, 1) * 100, 1),
            documented_count,
            attr_count),
        jsonb_build_object(
            'documented_count', documented_count,
            'total_count', attr_count,
            'percentage', ROUND(documented_count::DECIMAL / GREATEST(attr_count, 1) * 100, 2)
        );

    -- Rule 4: Check for duplicate attribute names
    FOR rule_name, message IN
        SELECT 'duplicate_attribute_names', 'Duplicate attribute name: ' || attribute_name
        FROM attribute_objects
        WHERE resource_id = p_resource_id
        GROUP BY attribute_name
        HAVING COUNT(*) > 1
    LOOP
        RETURN QUERY SELECT
            rule_name::TEXT,
            'consistency'::TEXT,
            'error'::TEXT,
            'FAIL'::TEXT,
            message::TEXT,
            '{}'::JSONB;
    END LOOP;

    -- Rule 5: UI layout validation
    RETURN QUERY SELECT
        'ui_layout_consistency'::TEXT,
        'consistency'::TEXT,
        'warning'::TEXT,
        CASE WHEN resource_rec.ui_layout IN ('wizard', 'tabs', 'vertical-stack') THEN 'PASS' ELSE 'WARN' END,
        CASE WHEN resource_rec.ui_layout IN ('wizard', 'tabs', 'vertical-stack')
            THEN 'UI layout is properly configured'
            ELSE format('UI layout "%s" may need review', resource_rec.ui_layout)
        END,
        jsonb_build_object('ui_layout', resource_rec.ui_layout);

END;
$$ LANGUAGE plpgsql;

-- Function to create attribute set snapshot for versioning
CREATE OR REPLACE FUNCTION create_attribute_set_snapshot(
    p_resource_id INTEGER,
    p_version VARCHAR(20),
    p_description TEXT DEFAULT NULL,
    p_created_by VARCHAR(100) DEFAULT NULL
) RETURNS BOOLEAN AS $$
DECLARE
    attributes_snapshot JSONB;
    schema_changes JSONB := '{}';
    prev_version RECORD;
BEGIN
    -- Create snapshot of current attribute configuration
    SELECT jsonb_agg(
        jsonb_build_object(
            'id', ao.id,
            'attribute_name', ao.attribute_name,
            'data_type', ao.data_type,
            'description', ao.description,
            'is_required', ao.is_required,
            'ui_group', ao.ui_group,
            'ui_display_order', ao.ui_display_order,
            'validation_pattern', ao.validation_pattern,
            'allowed_values', ao.allowed_values,
            'ui_component_type', ao.ui_component_type,
            'semantic_tags', ao.semantic_tags,
            'attribute_class', ao.attribute_class,
            'created_at', ao.created_at,
            'updated_at', ao.updated_at
        ) ORDER BY ao.ui_display_order, ao.attribute_name
    ) INTO attributes_snapshot
    FROM attribute_objects ao
    WHERE ao.resource_id = p_resource_id;

    -- Get previous version for comparison
    SELECT * INTO prev_version
    FROM resource_attribute_versions
    WHERE resource_id = p_resource_id
    ORDER BY created_at DESC
    LIMIT 1;

    -- Calculate changes from previous version
    IF prev_version.id IS NOT NULL THEN
        -- Basic change detection (could be enhanced)
        schema_changes := jsonb_build_object(
            'previous_version', prev_version.version_number,
            'attribute_count_change',
                jsonb_array_length(attributes_snapshot) - jsonb_array_length(prev_version.attributes_snapshot),
            'snapshot_date', CURRENT_TIMESTAMP
        );
    END IF;

    -- Deactivate previous versions
    UPDATE resource_attribute_versions
    SET is_active = false
    WHERE resource_id = p_resource_id;

    -- Insert new version
    INSERT INTO resource_attribute_versions (
        resource_id, version_number, change_description,
        attributes_snapshot, schema_changes, created_by, is_active
    ) VALUES (
        p_resource_id, p_version, p_description,
        attributes_snapshot, schema_changes, p_created_by, true
    );

    RETURN true;
END;
$$ LANGUAGE plpgsql;

-- Function to copy attributes from one resource to another
CREATE OR REPLACE FUNCTION copy_attribute_set(
    p_source_resource_id INTEGER,
    p_target_resource_id INTEGER,
    p_copy_mode VARCHAR(20) DEFAULT 'merge' -- 'merge', 'replace', 'append'
) RETURNS INTEGER AS $$
DECLARE
    copied_count INTEGER := 0;
    attr_record RECORD;
BEGIN
    -- Validate source and target resources exist
    IF NOT EXISTS (SELECT 1 FROM resource_objects WHERE id = p_source_resource_id) THEN
        RAISE EXCEPTION 'Source resource % does not exist', p_source_resource_id;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM resource_objects WHERE id = p_target_resource_id) THEN
        RAISE EXCEPTION 'Target resource % does not exist', p_target_resource_id;
    END IF;

    -- Handle replace mode
    IF p_copy_mode = 'replace' THEN
        DELETE FROM attribute_objects WHERE resource_id = p_target_resource_id;
    END IF;

    -- Copy attributes
    FOR attr_record IN
        SELECT * FROM attribute_objects
        WHERE resource_id = p_source_resource_id
        ORDER BY ui_display_order, attribute_name
    LOOP
        -- Skip if merge mode and attribute already exists
        IF p_copy_mode = 'merge' AND EXISTS (
            SELECT 1 FROM attribute_objects
            WHERE resource_id = p_target_resource_id
            AND attribute_name = attr_record.attribute_name
        ) THEN
            CONTINUE;
        END IF;

        -- Insert copied attribute
        INSERT INTO attribute_objects (
            resource_id, attribute_name, data_type, description, is_required,
            min_length, max_length, min_value, max_value, allowed_values,
            validation_pattern, ui_group, ui_display_order, ui_render_hint,
            ui_label, ui_help_text, generation_examples, rules_dsl,
            semantic_tags, ai_context, search_keywords, ui_component_type,
            ui_layout_config, ui_styling, ui_behavior, conditional_logic,
            relationship_metadata, ai_prompt_templates, form_generation_rules,
            accessibility_config, responsive_config, data_flow_config
        )
        SELECT
            p_target_resource_id, attribute_name, data_type, description, is_required,
            min_length, max_length, min_value, max_value, allowed_values,
            validation_pattern, ui_group, ui_display_order, ui_render_hint,
            ui_label, ui_help_text, generation_examples, rules_dsl,
            semantic_tags, ai_context, search_keywords, ui_component_type,
            ui_layout_config, ui_styling, ui_behavior, conditional_logic,
            relationship_metadata, ai_prompt_templates, form_generation_rules,
            accessibility_config, responsive_config, data_flow_config
        FROM attribute_objects
        WHERE id = attr_record.id;

        copied_count := copied_count + 1;
    END LOOP;

    RETURN copied_count;
END;
$$ LANGUAGE plpgsql;

-- Insert some sample validation rules
INSERT INTO resource_validation_rules (resource_id, rule_name, rule_type, rule_config, severity) VALUES
(1, 'kyc_minimum_required_fields', 'required_attributes',
 '{"required_fields": ["legal_entity_name", "ubo_full_name", "jurisdiction"]}', 'error'),
(1, 'sanctions_screening_dependency', 'dependency',
 '{"if_field": "legal_entity_name", "then_required": ["sanctions_screening_result"]}', 'warning'),
(2, 'trade_settlement_required_fields', 'required_attributes',
 '{"required_fields": ["counterparty_name", "trade_amount", "settlement_date"]}', 'error');

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_resource_attribute_templates_category ON resource_attribute_templates(category);
CREATE INDEX IF NOT EXISTS idx_resource_attribute_versions_resource ON resource_attribute_versions(resource_id);
CREATE INDEX IF NOT EXISTS idx_resource_attribute_versions_active ON resource_attribute_versions(is_active);
CREATE INDEX IF NOT EXISTS idx_resource_validation_rules_resource ON resource_validation_rules(resource_id);
CREATE INDEX IF NOT EXISTS idx_resource_validation_rules_active ON resource_validation_rules(is_active);

-- Comments for documentation
COMMENT ON TABLE resource_attribute_templates IS 'Templates for common attribute set patterns';
COMMENT ON TABLE resource_attribute_versions IS 'Version control for resource attribute sets';
COMMENT ON TABLE resource_validation_rules IS 'Validation rules for resource attribute set integrity';
COMMENT ON TABLE resource_attribute_dependencies IS 'Dependencies between resource attribute sets';
COMMENT ON VIEW resource_management_view IS 'Comprehensive view of resource attribute set management metrics';
COMMENT ON FUNCTION validate_resource_attribute_set IS 'Validates integrity and completeness of resource attribute sets';
COMMENT ON FUNCTION create_attribute_set_snapshot IS 'Creates versioned snapshot of resource attribute configuration';
COMMENT ON FUNCTION copy_attribute_set IS 'Copies attribute sets between resources with different merge strategies';