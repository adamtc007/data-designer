-- Enhanced Metadata Schema for AI RAG and UI Auto-Layout
-- This extends the existing attribute_objects table with comprehensive metadata

-- Add AI RAG and UI enhancement columns to attribute_objects
ALTER TABLE attribute_objects
ADD COLUMN IF NOT EXISTS semantic_tags JSONB DEFAULT '[]',
ADD COLUMN IF NOT EXISTS ai_context JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS embedding_vector VECTOR(1536), -- OpenAI embedding dimension
ADD COLUMN IF NOT EXISTS search_keywords TEXT[],
ADD COLUMN IF NOT EXISTS ui_component_type VARCHAR(50) DEFAULT 'text-input',
ADD COLUMN IF NOT EXISTS ui_layout_config JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS ui_styling JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS ui_behavior JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS conditional_logic JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS relationship_metadata JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS ai_prompt_templates JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS form_generation_rules JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS accessibility_config JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS responsive_config JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS data_flow_config JSONB DEFAULT '{}';

-- Create indexes for efficient AI and UI queries
CREATE INDEX IF NOT EXISTS idx_attribute_objects_semantic_tags ON attribute_objects USING GIN (semantic_tags);
CREATE INDEX IF NOT EXISTS idx_attribute_objects_ai_context ON attribute_objects USING GIN (ai_context);
CREATE INDEX IF NOT EXISTS idx_attribute_objects_search_keywords ON attribute_objects USING GIN (search_keywords);
CREATE INDEX IF NOT EXISTS idx_attribute_objects_ui_component ON attribute_objects (ui_component_type);
CREATE INDEX IF NOT EXISTS idx_attribute_objects_embedding ON attribute_objects USING ivfflat (embedding_vector vector_cosine_ops);

-- Create AI context and metadata tables for advanced features
CREATE TABLE IF NOT EXISTS ai_metadata_contexts (
    id SERIAL PRIMARY KEY,
    context_name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    prompt_template TEXT,
    example_queries JSONB DEFAULT '[]',
    response_format JSONB DEFAULT '{}',
    model_config JSONB DEFAULT '{}',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ui_component_templates (
    id SERIAL PRIMARY KEY,
    template_name VARCHAR(100) UNIQUE NOT NULL,
    component_type VARCHAR(50) NOT NULL,
    template_config JSONB NOT NULL,
    styling_defaults JSONB DEFAULT '{}',
    behavior_defaults JSONB DEFAULT '{}',
    validation_rules JSONB DEFAULT '{}',
    accessibility_defaults JSONB DEFAULT '{}',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS form_layout_templates (
    id SERIAL PRIMARY KEY,
    template_name VARCHAR(100) UNIQUE NOT NULL,
    layout_type VARCHAR(50) NOT NULL, -- 'grid', 'flex', 'wizard', 'tabs', 'accordion'
    layout_config JSONB NOT NULL,
    responsive_breakpoints JSONB DEFAULT '{}',
    css_framework VARCHAR(50) DEFAULT 'custom',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS attribute_relationships (
    id SERIAL PRIMARY KEY,
    source_attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    target_attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    relationship_type VARCHAR(50) NOT NULL, -- 'depends_on', 'validates_against', 'populates_from', 'triggers'
    relationship_config JSONB DEFAULT '{}',
    strength DECIMAL(3,2) DEFAULT 1.0, -- Relationship strength 0.0-1.0
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(source_attribute_id, target_attribute_id, relationship_type)
);

-- Create semantic search and AI enhancement functions
CREATE OR REPLACE FUNCTION update_embedding_trigger()
RETURNS TRIGGER AS $$
BEGIN
    -- Update search keywords automatically when description or semantic_tags change
    NEW.search_keywords := ARRAY(
        SELECT DISTINCT unnest(
            string_to_array(
                COALESCE(NEW.description, '') || ' ' ||
                COALESCE(NEW.attribute_name, '') || ' ' ||
                COALESCE(array_to_string(ARRAY(SELECT jsonb_array_elements_text(NEW.semantic_tags)), ' '), ''),
                ' '
            )
        )
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_attribute_embedding_trigger
    BEFORE INSERT OR UPDATE ON attribute_objects
    FOR EACH ROW EXECUTE FUNCTION update_embedding_trigger();

-- Insert default AI contexts
INSERT INTO ai_metadata_contexts (context_name, description, prompt_template, example_queries, response_format, model_config) VALUES
(
    'attribute_discovery',
    'Help users discover relevant attributes based on their intent',
    'Based on the user query "{query}", suggest the most relevant attributes from our data dictionary. Consider semantic meaning, use cases, and relationships.',
    '["customer information", "financial calculations", "KYC requirements", "risk assessment"]',
    '{"type": "attribute_list", "include_descriptions": true, "max_results": 10}',
    '{"temperature": 0.3, "max_tokens": 500}'
),
(
    'form_generation',
    'Generate optimal form layouts based on attribute metadata',
    'Create a form layout for the following attributes: {attributes}. Consider user experience, data flow, validation requirements, and accessibility.',
    '["user registration form", "financial application", "KYC documentation"]',
    '{"type": "form_config", "include_layout": true, "include_validation": true}',
    '{"temperature": 0.2, "max_tokens": 1000}'
),
(
    'validation_rules',
    'Generate validation rules and business logic for attributes',
    'Generate appropriate validation rules for attribute "{attribute_name}" of type "{data_type}" with description "{description}".',
    '["email validation", "phone number formats", "financial amounts"]',
    '{"type": "validation_config", "include_regex": true, "include_business_rules": true}',
    '{"temperature": 0.1, "max_tokens": 300}'
);

-- Insert default UI component templates
INSERT INTO ui_component_templates (template_name, component_type, template_config, styling_defaults, behavior_defaults) VALUES
(
    'enhanced_text_input',
    'text-input',
    '{"placeholder_dynamic": true, "autocomplete": true, "validation_live": true}',
    '{"border_radius": "4px", "padding": "8px 12px", "font_size": "14px"}',
    '{"debounce_ms": 300, "trim_whitespace": true, "auto_capitalize": false}'
),
(
    'smart_dropdown',
    'dropdown',
    '{"searchable": true, "multi_select": false, "load_options_async": true}',
    '{"max_height": "200px", "item_padding": "8px", "highlight_color": "#e3f2fd"}',
    '{"search_threshold": 3, "close_on_select": true, "keyboard_navigation": true}'
),
(
    'financial_amount',
    'number-input',
    '{"currency_symbol": true, "thousands_separator": true, "decimal_places": 2}',
    '{"text_align": "right", "font_family": "monospace", "min_width": "120px"}',
    '{"format_on_blur": true, "validate_on_change": true, "max_amount": 999999999}'
),
(
    'date_picker_enhanced',
    'date-input',
    '{"format": "YYYY-MM-DD", "show_calendar": true, "min_date": null, "max_date": null}',
    '{"calendar_theme": "modern", "input_width": "140px"}',
    '{"auto_close": true, "highlight_weekends": true, "disable_past": false}'
),
(
    'wizard_step_container',
    'container',
    '{"show_progress": true, "validate_before_next": true, "save_progress": true}',
    '{"background": "#f9f9f9", "border": "1px solid #e0e0e0", "border_radius": "8px"}',
    '{"auto_save_interval": 30000, "show_step_numbers": true, "allow_step_jumping": false}'
);

-- Insert default form layout templates
INSERT INTO form_layout_templates (template_name, layout_type, layout_config, responsive_breakpoints) VALUES
(
    'two_column_responsive',
    'grid',
    '{"columns": 2, "gap": "20px", "auto_fit": true, "min_column_width": "300px"}',
    '{"mobile": {"columns": 1}, "tablet": {"columns": 2}, "desktop": {"columns": 2}}'
),
(
    'wizard_multi_step',
    'wizard',
    '{"show_progress_bar": true, "allow_back_navigation": true, "confirm_before_exit": true}',
    '{"mobile": {"stack_vertically": true}, "desktop": {"show_sidebar_navigation": true}}'
),
(
    'tabbed_sections',
    'tabs',
    '{"position": "top", "lazy_load": true, "persist_active_tab": true}',
    '{"mobile": {"convert_to_accordion": true}, "desktop": {"show_tab_icons": true}}'
),
(
    'accordion_grouped',
    'accordion',
    '{"allow_multiple_open": false, "animate_transitions": true, "remember_state": true}',
    '{"all_devices": {"smooth_scroll_to_active": true}}'
);

-- Create comprehensive view for AI-enhanced attributes
CREATE OR REPLACE VIEW enhanced_attributes_view AS
SELECT
    ao.*,
    ac.context_name,
    ac.prompt_template,
    uct.template_config as ui_template_config,
    uct.styling_defaults,
    uct.behavior_defaults,
    array_agg(DISTINCT ar_source.relationship_type) FILTER (WHERE ar_source.relationship_type IS NOT NULL) as outgoing_relationships,
    array_agg(DISTINCT ar_target.relationship_type) FILTER (WHERE ar_target.relationship_type IS NOT NULL) as incoming_relationships,
    count(DISTINCT ap.id) as perspective_count
FROM attribute_objects ao
LEFT JOIN ai_metadata_contexts ac ON ac.context_name = 'attribute_discovery'
LEFT JOIN ui_component_templates uct ON uct.component_type = ao.ui_component_type
LEFT JOIN attribute_relationships ar_source ON ar_source.source_attribute_id = ao.id
LEFT JOIN attribute_relationships ar_target ON ar_target.target_attribute_id = ao.id
LEFT JOIN attribute_perspectives ap ON ap.attribute_id = ao.id
GROUP BY ao.id, ac.context_name, ac.prompt_template, uct.template_config, uct.styling_defaults, uct.behavior_defaults;

-- Create function for semantic similarity search
CREATE OR REPLACE FUNCTION find_similar_attributes(
    query_embedding VECTOR(1536),
    similarity_threshold FLOAT DEFAULT 0.7,
    max_results INT DEFAULT 10
) RETURNS TABLE (
    attribute_id INT,
    attribute_name VARCHAR,
    description TEXT,
    similarity_score FLOAT,
    semantic_tags JSONB,
    ui_component_type VARCHAR
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        ao.id,
        ao.attribute_name,
        ao.description,
        1 - (ao.embedding_vector <=> query_embedding) as similarity,
        ao.semantic_tags,
        ao.ui_component_type
    FROM attribute_objects ao
    WHERE ao.embedding_vector IS NOT NULL
        AND 1 - (ao.embedding_vector <=> query_embedding) >= similarity_threshold
    ORDER BY ao.embedding_vector <=> query_embedding
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql;

-- Create function for intelligent form generation
CREATE OR REPLACE FUNCTION generate_form_layout(
    attribute_ids INT[],
    layout_preference VARCHAR DEFAULT 'auto'
) RETURNS JSONB AS $$
DECLARE
    result JSONB;
    attr_count INT;
    layout_type VARCHAR;
BEGIN
    SELECT array_length(attribute_ids, 1) INTO attr_count;

    -- Determine optimal layout based on attribute count and types
    CASE
        WHEN attr_count <= 3 THEN layout_type := 'single_column';
        WHEN attr_count <= 8 THEN layout_type := 'two_column_responsive';
        WHEN attr_count <= 15 THEN layout_type := 'tabbed_sections';
        ELSE layout_type := 'wizard_multi_step';
    END CASE;

    -- Override with preference if provided
    IF layout_preference != 'auto' THEN
        layout_type := layout_preference;
    END IF;

    -- Build the form configuration
    SELECT jsonb_build_object(
        'layout_type', layout_type,
        'attributes', jsonb_agg(
            jsonb_build_object(
                'id', ao.id,
                'name', ao.attribute_name,
                'type', ao.data_type,
                'ui_component', ao.ui_component_type,
                'ui_config', ao.ui_layout_config,
                'validation', jsonb_build_object(
                    'required', ao.is_required,
                    'min_length', ao.min_length,
                    'max_length', ao.max_length,
                    'pattern', ao.validation_pattern
                ),
                'order', ao.ui_display_order,
                'group', ao.ui_group
            ) ORDER BY ao.ui_display_order
        ),
        'template_config', flt.layout_config
    ) INTO result
    FROM attribute_objects ao
    LEFT JOIN form_layout_templates flt ON flt.template_name = layout_type
    WHERE ao.id = ANY(attribute_ids);

    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Create comprehensive metadata update trigger
CREATE OR REPLACE FUNCTION update_comprehensive_metadata()
RETURNS TRIGGER AS $$
BEGIN
    -- Auto-populate semantic tags based on description and attribute name
    IF NEW.semantic_tags = '[]' OR NEW.semantic_tags IS NULL THEN
        NEW.semantic_tags := (
            SELECT jsonb_agg(DISTINCT tag)
            FROM (
                SELECT unnest(string_to_array(
                    lower(COALESCE(NEW.description, '') || ' ' || NEW.attribute_name),
                    ' '
                )) as tag
            ) tags
            WHERE length(tag) > 3
            AND tag NOT IN ('the', 'and', 'for', 'are', 'but', 'not', 'you', 'all', 'can', 'had', 'her', 'was', 'one', 'our', 'out', 'day', 'get', 'has', 'him', 'his', 'how', 'its', 'may', 'new', 'now', 'old', 'see', 'two', 'who', 'boy', 'did', 'has', 'let', 'put', 'say', 'she', 'too', 'use')
        );
    END IF;

    -- Auto-configure UI component type based on data type if not set
    IF NEW.ui_component_type IS NULL OR NEW.ui_component_type = 'text-input' THEN
        NEW.ui_component_type := CASE
            WHEN NEW.data_type = 'Boolean' THEN 'checkbox'
            WHEN NEW.data_type = 'Date' THEN 'date-input'
            WHEN NEW.data_type = 'Number' OR NEW.data_type = 'Decimal' THEN
                CASE WHEN NEW.attribute_name ILIKE '%amount%' OR NEW.attribute_name ILIKE '%price%' OR NEW.attribute_name ILIKE '%cost%'
                THEN 'financial_amount' ELSE 'number-input' END
            WHEN NEW.data_type = 'Enum' OR NEW.allowed_values IS NOT NULL THEN 'smart_dropdown'
            WHEN NEW.max_length > 255 THEN 'textarea'
            ELSE 'enhanced_text_input'
        END;
    END IF;

    -- Set default AI context based on attribute characteristics
    IF NEW.ai_context = '{}' OR NEW.ai_context IS NULL THEN
        NEW.ai_context := jsonb_build_object(
            'searchable', true,
            'context_type', CASE
                WHEN NEW.attribute_name ILIKE '%email%' THEN 'contact'
                WHEN NEW.attribute_name ILIKE '%phone%' THEN 'contact'
                WHEN NEW.attribute_name ILIKE '%address%' THEN 'location'
                WHEN NEW.attribute_name ILIKE '%amount%' OR NEW.data_type = 'Decimal' THEN 'financial'
                WHEN NEW.attribute_name ILIKE '%date%' OR NEW.data_type = 'Date' THEN 'temporal'
                ELSE 'general'
            END,
            'relevance_weight', 1.0
        );
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER comprehensive_metadata_trigger
    BEFORE INSERT OR UPDATE ON attribute_objects
    FOR EACH ROW EXECUTE FUNCTION update_comprehensive_metadata();

-- Comments for documentation
COMMENT ON COLUMN attribute_objects.semantic_tags IS 'Tags for semantic search and AI context understanding';
COMMENT ON COLUMN attribute_objects.ai_context IS 'AI-specific metadata for enhanced search and suggestions';
COMMENT ON COLUMN attribute_objects.embedding_vector IS 'Vector embedding for semantic similarity search';
COMMENT ON COLUMN attribute_objects.ui_component_type IS 'Specific UI component to render this attribute';
COMMENT ON COLUMN attribute_objects.ui_layout_config IS 'Component-specific layout and positioning configuration';
COMMENT ON COLUMN attribute_objects.ui_styling IS 'Custom styling rules for the UI component';
COMMENT ON COLUMN attribute_objects.ui_behavior IS 'Interactive behavior configuration';
COMMENT ON COLUMN attribute_objects.conditional_logic IS 'Rules for show/hide/enable/disable based on other fields';
COMMENT ON COLUMN attribute_objects.relationship_metadata IS 'Metadata about relationships with other attributes';
COMMENT ON COLUMN attribute_objects.ai_prompt_templates IS 'Custom prompt templates for AI interactions';
COMMENT ON COLUMN attribute_objects.form_generation_rules IS 'Rules for automatic form generation';
COMMENT ON COLUMN attribute_objects.accessibility_config IS 'Accessibility settings and ARIA configurations';
COMMENT ON COLUMN attribute_objects.responsive_config IS 'Responsive design configurations for different screen sizes';
COMMENT ON COLUMN attribute_objects.data_flow_config IS 'Configuration for data flow and real-time updates';