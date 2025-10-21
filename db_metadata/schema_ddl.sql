--
-- PostgreSQL database dump
--

\restrict 45sLfvTajVPmRdsVA53Za6LGK6Ug5PdHHh0dcEgkGqPHBkwkQ5FTlayh7esnald

-- Dumped from database version 17.6 (Homebrew)
-- Dumped by pg_dump version 17.6 (Homebrew)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';


--
-- Name: vector; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS vector WITH SCHEMA public;


--
-- Name: EXTENSION vector; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION vector IS 'vector data type and ivfflat and hnsw access methods';


--
-- Name: ai_semantic_attribute_search(text, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.ai_semantic_attribute_search(search_query text, max_results integer DEFAULT 10) RETURNS TABLE(attribute_id integer, attribute_name character varying, relevance_score numeric, context_summary text, related_terms text[])
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: FUNCTION ai_semantic_attribute_search(search_query text, max_results integer); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.ai_semantic_attribute_search(search_query text, max_results integer) IS 'Semantic search function optimized for AI attribute discovery';


--
-- Name: compile_rule_to_rust(integer, text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.compile_rule_to_rust(p_rule_id integer, p_dsl_code text) RETURNS text
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_rust_code TEXT;
    v_dependencies TEXT[];
    v_target_attr RECORD;
BEGIN
    -- Get target attribute info
    SELECT da.attribute_name, da.rust_type, da.entity_name
    INTO v_target_attr
    FROM rules r
    JOIN derived_attributes da ON r.target_attribute_id = da.id
    WHERE r.id = p_rule_id;

    -- Get dependencies
    SELECT ARRAY_AGG(ba.attribute_name)
    INTO v_dependencies
    FROM rule_dependencies rd
    JOIN business_attributes ba ON rd.attribute_id = ba.id
    WHERE rd.rule_id = p_rule_id;

    -- Generate Rust function (simplified - actual implementation would use proper parser)
    v_rust_code := format('
pub fn calculate_%s(context: &HashMap<String, Value>) -> Result<%s, String> {
    // Auto-generated from DSL rule
    // Dependencies: %s

    // DSL: %s

    // TODO: Implement actual DSL to Rust transpilation
    // This is a placeholder implementation

    Ok(Default::default())
}',
        lower(v_target_attr.attribute_name),
        v_target_attr.rust_type,
        array_to_string(v_dependencies, ', '),
        p_dsl_code
    );

    RETURN v_rust_code;
END;
$$;


--
-- Name: copy_attribute_set(integer, integer, character varying); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.copy_attribute_set(p_source_resource_id integer, p_target_resource_id integer, p_copy_mode character varying DEFAULT 'merge'::character varying) RETURNS integer
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: FUNCTION copy_attribute_set(p_source_resource_id integer, p_target_resource_id integer, p_copy_mode character varying); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.copy_attribute_set(p_source_resource_id integer, p_target_resource_id integer, p_copy_mode character varying) IS 'Copies attribute sets between resources with different merge strategies';


--
-- Name: create_attribute_set_snapshot(integer, character varying, text, character varying); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.create_attribute_set_snapshot(p_resource_id integer, p_version character varying, p_description text DEFAULT NULL::text, p_created_by character varying DEFAULT NULL::character varying) RETURNS boolean
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: FUNCTION create_attribute_set_snapshot(p_resource_id integer, p_version character varying, p_description text, p_created_by character varying); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.create_attribute_set_snapshot(p_resource_id integer, p_version character varying, p_description text, p_created_by character varying) IS 'Creates versioned snapshot of resource attribute configuration';


--
-- Name: create_semantic_clusters(integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.create_semantic_clusters(cluster_count integer DEFAULT 10, min_cluster_size integer DEFAULT 3) RETURNS jsonb
    LANGUAGE plpgsql
    AS $$
DECLARE
    cluster_result JSONB;
    created_clusters INTEGER := 0;
    total_attributes INTEGER;
BEGIN
    -- Get total number of attributes with embeddings
    SELECT COUNT(*) INTO total_attributes
    FROM attribute_objects
    WHERE embedding_vector IS NOT NULL;

    IF total_attributes < cluster_count * min_cluster_size THEN
        RETURN jsonb_build_object(
            'success', false,
            'message', 'Insufficient attributes for clustering',
            'total_attributes', total_attributes,
            'required_minimum', cluster_count * min_cluster_size
        );
    END IF;

    -- This is a simplified clustering implementation
    -- In a production system, you would use a proper clustering algorithm

    -- Create clusters based on semantic similarity
    -- For now, create clusters based on attribute types and domains
    INSERT INTO attribute_vector_clusters (cluster_name, cluster_description, cluster_type)
    SELECT DISTINCT
        format('%s_%s_cluster',
               COALESCE(kod.domain_name, 'general'),
               ao.data_type),
        format('Cluster for %s attributes in %s domain',
               ao.data_type,
               COALESCE(kod.domain_name, 'general')),
        'semantic'
    FROM attribute_objects ao
    LEFT JOIN attribute_domain_mappings adm ON adm.attribute_id = ao.id
    LEFT JOIN kyc_onboarding_domains kod ON kod.id = adm.domain_id
    WHERE ao.embedding_vector IS NOT NULL
    ON CONFLICT (cluster_name) DO NOTHING;

    GET DIAGNOSTICS created_clusters = ROW_COUNT;

    -- Assign attributes to clusters
    INSERT INTO attribute_cluster_memberships (attribute_id, cluster_id, membership_strength)
    SELECT
        ao.id,
        avc.id,
        0.8 + (random() * 0.2) -- Random membership strength between 0.8 and 1.0
    FROM attribute_objects ao
    LEFT JOIN attribute_domain_mappings adm ON adm.attribute_id = ao.id
    LEFT JOIN kyc_onboarding_domains kod ON kod.id = adm.domain_id
    JOIN attribute_vector_clusters avc ON avc.cluster_name = format('%s_%s_cluster',
                                                                   COALESCE(kod.domain_name, 'general'),
                                                                   ao.data_type)
    WHERE ao.embedding_vector IS NOT NULL
    ON CONFLICT (attribute_id, cluster_id) DO NOTHING;

    -- Update cluster member counts
    UPDATE attribute_vector_clusters
    SET member_count = (
        SELECT COUNT(*)
        FROM attribute_cluster_memberships
        WHERE cluster_id = attribute_vector_clusters.id
    );

    cluster_result := jsonb_build_object(
        'success', true,
        'clusters_created', created_clusters,
        'total_attributes_clustered', total_attributes,
        'cluster_details', (
            SELECT jsonb_agg(
                jsonb_build_object(
                    'cluster_name', cluster_name,
                    'member_count', member_count,
                    'cluster_type', cluster_type
                )
            )
            FROM attribute_vector_clusters
            WHERE member_count > 0
        )
    );

    RETURN cluster_result;
END;
$$;


--
-- Name: FUNCTION create_semantic_clusters(cluster_count integer, min_cluster_size integer); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.create_semantic_clusters(cluster_count integer, min_cluster_size integer) IS 'Creates semantic clusters based on attribute embeddings';


--
-- Name: discover_all_attributes(character varying, boolean); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.discover_all_attributes(p_entity_filter character varying DEFAULT NULL::character varying, p_include_system boolean DEFAULT true) RETURNS TABLE(attribute_type character varying, entity_name character varying, attribute_name character varying, full_path character varying, data_type character varying, sql_type character varying, rust_type character varying, description text, is_nullable boolean, is_derived boolean, derivation_rule text, rule_id integer)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        dd.attribute_type::VARCHAR,
        dd.entity_name::VARCHAR,
        dd.attribute_name::VARCHAR,
        dd.full_path::VARCHAR,
        dd.data_type::VARCHAR,
        dd.sql_type::VARCHAR,
        dd.rust_type::VARCHAR,
        dd.description,
        NOT dd.required as is_nullable,
        dd.attribute_type = 'derived' as is_derived,
        dd.rule_definition as derivation_rule,
        dd.rule_id
    FROM mv_data_dictionary dd
    WHERE (p_entity_filter IS NULL OR dd.entity_name = p_entity_filter)
        AND (p_include_system = true OR dd.attribute_type != 'system')
        AND dd.status = 'active';
END;
$$;


--
-- Name: execute_derivation_rule(integer, jsonb); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.execute_derivation_rule(derived_attr_id integer, input_context jsonb) RETURNS jsonb
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: FUNCTION execute_derivation_rule(derived_attr_id integer, input_context jsonb); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.execute_derivation_rule(derived_attr_id integer, input_context jsonb) IS 'Executes derivation rule for a derived attribute';


--
-- Name: filter_attributes_advanced(jsonb); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.filter_attributes_advanced(filter_criteria jsonb) RETURNS TABLE(attribute_id integer, attribute_name character varying, relevance_score numeric, filter_metadata jsonb, cluster_info jsonb)
    LANGUAGE plpgsql
    AS $_$
DECLARE
    base_query TEXT;
    where_conditions TEXT[] := '{}';
    order_clause TEXT := 'ORDER BY acfv.attribute_name';
    limit_clause TEXT := '';
    vector_query_embedding VECTOR(1536);
    similarity_threshold DECIMAL := 0.7;
BEGIN
    -- Build WHERE conditions based on filter criteria

    -- Basic attribute filters
    IF filter_criteria ? 'attribute_class' THEN
        where_conditions := array_append(where_conditions,
            format('acfv.attribute_class = %L', filter_criteria->>'attribute_class'));
    END IF;

    IF filter_criteria ? 'visibility_scope' THEN
        where_conditions := array_append(where_conditions,
            format('acfv.visibility_scope = %L', filter_criteria->>'visibility_scope'));
    END IF;

    IF filter_criteria ? 'data_type' THEN
        where_conditions := array_append(where_conditions,
            format('acfv.data_type = %L', filter_criteria->>'data_type'));
    END IF;

    -- Domain and compliance filters
    IF filter_criteria ? 'domain_name' THEN
        where_conditions := array_append(where_conditions,
            format('acfv.domain_name = %L', filter_criteria->>'domain_name'));
    END IF;

    IF filter_criteria ? 'compliance_criticality' THEN
        where_conditions := array_append(where_conditions,
            format('acfv.compliance_criticality = %L', filter_criteria->>'compliance_criticality'));
    END IF;

    -- Cluster filters
    IF filter_criteria ? 'clusters' THEN
        where_conditions := array_append(where_conditions,
            format('acfv.cluster_names && %L::TEXT[]', filter_criteria->'clusters'));
    END IF;

    -- Tag filters
    IF filter_criteria ? 'tags' THEN
        where_conditions := array_append(where_conditions,
            format('acfv.tags && %L::TEXT[]', filter_criteria->'tags'));
    END IF;

    -- UI component filters
    IF filter_criteria ? 'ui_component_type' THEN
        where_conditions := array_append(where_conditions,
            format('acfv.ui_component_type = %L', filter_criteria->>'ui_component_type'));
    END IF;

    -- Persistence system filters
    IF filter_criteria ? 'persistence_system' THEN
        where_conditions := array_append(where_conditions,
            format('acfv.persistence_system = %L', filter_criteria->>'persistence_system'));
    END IF;

    -- Text search
    IF filter_criteria ? 'search_text' THEN
        where_conditions := array_append(where_conditions,
            format('acfv.searchable_text ILIKE ''%%%s%%''', filter_criteria->>'search_text'));
    END IF;

    -- Vector similarity search
    IF filter_criteria ? 'embedding_query' THEN
        vector_query_embedding := (filter_criteria->>'embedding_query')::VECTOR(1536);
        similarity_threshold := COALESCE((filter_criteria->>'similarity_threshold')::DECIMAL, 0.7);

        where_conditions := array_append(where_conditions,
            format('acfv.embedding_vector IS NOT NULL AND (1 - (acfv.embedding_vector <=> %L::VECTOR(1536))) >= %s',
                   vector_query_embedding, similarity_threshold));

        order_clause := format('ORDER BY acfv.embedding_vector <=> %L::VECTOR(1536)', vector_query_embedding);
    END IF;

    -- Quality filters (for derived attributes)
    IF filter_criteria ? 'min_quality_score' THEN
        where_conditions := array_append(where_conditions,
            format('acfv.data_quality_score >= %s', filter_criteria->>'min_quality_score'));
    END IF;

    -- Limit and pagination
    IF filter_criteria ? 'limit' THEN
        limit_clause := format('LIMIT %s', (filter_criteria->>'limit')::INTEGER);
    END IF;

    IF filter_criteria ? 'offset' THEN
        limit_clause := limit_clause || format(' OFFSET %s', (filter_criteria->>'offset')::INTEGER);
    END IF;

    -- Build the final query
    base_query := 'SELECT
        acfv.id,
        acfv.attribute_name,
        CASE
            WHEN $1 ? ''embedding_query'' THEN (1 - (acfv.embedding_vector <=> $2::VECTOR(1536)))
            ELSE 1.0
        END as relevance,
        acfv.filter_metadata,
        jsonb_build_object(
            ''clusters'', acfv.cluster_names,
            ''cluster_types'', acfv.cluster_types,
            ''cluster_memberships'', acfv.cluster_memberships
        ) as cluster_info
    FROM attribute_comprehensive_filter_view acfv';

    -- Add WHERE clause if conditions exist
    IF array_length(where_conditions, 1) > 0 THEN
        base_query := base_query || ' WHERE ' || array_to_string(where_conditions, ' AND ');
    END IF;

    -- Add ORDER and LIMIT
    base_query := base_query || ' ' || order_clause || ' ' || limit_clause;

    -- Execute the dynamic query
    RETURN QUERY EXECUTE base_query USING filter_criteria, vector_query_embedding;
END;
$_$;


--
-- Name: FUNCTION filter_attributes_advanced(filter_criteria jsonb); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.filter_attributes_advanced(filter_criteria jsonb) IS 'Advanced multi-criteria filtering with vector similarity support';


--
-- Name: find_similar_attributes(public.vector, double precision, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.find_similar_attributes(query_embedding public.vector, similarity_threshold double precision DEFAULT 0.7, max_results integer DEFAULT 10) RETURNS TABLE(attribute_id integer, attribute_name character varying, description text, similarity_score double precision, semantic_tags jsonb, ui_component_type character varying)
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: generate_form_layout(integer[], character varying); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.generate_form_layout(attribute_ids integer[], layout_preference character varying DEFAULT 'auto'::character varying) RETURNS jsonb
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: get_attribute_with_persistence(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.get_attribute_with_persistence(attr_id integer) RETURNS jsonb
    LANGUAGE plpgsql
    AS $$
DECLARE
    result JSONB;
BEGIN
    SELECT to_jsonb(avrv.*) INTO result
    FROM attribute_value_resolution_view avrv
    WHERE avrv.attribute_id = attr_id;

    RETURN COALESCE(result, '{}'::jsonb);
END;
$$;


--
-- Name: get_enhanced_product_taxonomy_hierarchy(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.get_enhanced_product_taxonomy_hierarchy(input_product_id integer) RETURNS TABLE(level integer, item_type character varying, item_id integer, item_name character varying, item_description text, parent_id integer, configuration jsonb, metadata jsonb)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    -- Level 1: Product
    SELECT
        1 as level,
        'product'::VARCHAR as item_type,
        p.id as item_id,
        p.product_name as item_name,
        p.description as item_description,
        NULL::INTEGER as parent_id,
        jsonb_build_object(
            'contract_type', p.contract_type,
            'commercial_status', p.commercial_status,
            'pricing_model', p.pricing_model
        ) as configuration,
        jsonb_build_object(
            'target_market', p.target_market,
            'compliance_requirements', p.compliance_requirements
        ) as metadata
    FROM products p
    WHERE p.id = input_product_id

    UNION ALL

    -- Level 2: Product Options
    SELECT
        2 as level,
        'product_option'::VARCHAR as item_type,
        po.id as item_id,
        po.option_name as item_name,
        po.description as item_description,
        p.id as parent_id,
        jsonb_build_object(
            'option_category', po.option_category,
            'option_type', po.option_type,
            'option_value', po.option_value,
            'pricing_impact', po.pricing_impact
        ) as configuration,
        jsonb_build_object(
            'available_markets', po.available_markets,
            'regulatory_approval_required', po.regulatory_approval_required,
            'implementation_complexity', po.implementation_complexity,
            'lead_time_days', po.lead_time_days
        ) as metadata
    FROM products p
    JOIN product_options po ON po.product_id = p.id
    WHERE p.id = input_product_id

    UNION ALL

    -- Level 3: Services (via Product Options)
    SELECT
        3 as level,
        'service'::VARCHAR as item_type,
        s.id as item_id,
        s.service_name as item_name,
        s.description as item_description,
        po.id as parent_id,
        jsonb_build_object(
            'service_type', s.service_type,
            'delivery_model', s.delivery_model,
            'mapping_relationship', posm.mapping_relationship,
            'service_configuration', posm.service_configuration
        ) as configuration,
        jsonb_build_object(
            'billable', s.billable,
            'sla_requirements', s.sla_requirements,
            'automation_level', s.automation_level
        ) as metadata
    FROM products p
    JOIN product_options po ON po.product_id = p.id
    JOIN product_option_service_mappings posm ON posm.product_option_id = po.id
    JOIN services s ON s.id = posm.service_id
    WHERE p.id = input_product_id

    UNION ALL

    -- Level 4: Resources (via Services)
    SELECT
        4 as level,
        'resource'::VARCHAR as item_type,
        r.id as item_id,
        r.resource_name as item_name,
        r.description as item_description,
        s.id as parent_id,
        jsonb_build_object(
            'resource_type', r.resource_type,
            'usage_type', srm.usage_type,
            'dependency_level', srm.dependency_level,
            'cost_allocation_percentage', srm.cost_allocation_percentage
        ) as configuration,
        jsonb_build_object(
            'criticality_level', r.criticality_level,
            'operational_status', r.operational_status,
            'compliance_classification', r.compliance_classification,
            'monitoring_enabled', r.monitoring_enabled
        ) as metadata
    FROM products p
    JOIN product_options po ON po.product_id = p.id
    JOIN product_option_service_mappings posm ON posm.product_option_id = po.id
    JOIN services s ON s.id = posm.service_id
    JOIN service_resource_mappings srm ON srm.service_id = s.id
    JOIN resource_objects r ON r.id = srm.resource_id
    WHERE p.id = input_product_id

    ORDER BY level, item_name;
END;
$$;


--
-- Name: FUNCTION get_enhanced_product_taxonomy_hierarchy(input_product_id integer); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.get_enhanced_product_taxonomy_hierarchy(input_product_id integer) IS 'Retrieves complete 4-level hierarchical breakdown: Product→Options→Services→Resources';


--
-- Name: get_filter_suggestions(jsonb); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.get_filter_suggestions(user_context jsonb DEFAULT '{}'::jsonb) RETURNS jsonb
    LANGUAGE plpgsql
    AS $$
DECLARE
    suggestions JSONB;
BEGIN
    -- Generate filter suggestions based on common patterns
    suggestions := jsonb_build_object(
        'quick_filters', jsonb_build_array(
            jsonb_build_object('name', 'Real Attributes Only', 'criteria', '{"attribute_class": "real"}'),
            jsonb_build_object('name', 'Derived Attributes Only', 'criteria', '{"attribute_class": "derived"}'),
            jsonb_build_object('name', 'High Compliance', 'criteria', '{"compliance_criticality": "high"}'),
            jsonb_build_object('name', 'KYC Domain', 'criteria', '{"domain_name": "kyc_verification"}'),
            jsonb_build_object('name', 'Public Visibility', 'criteria', '{"visibility_scope": "public"}')
        ),
        'cluster_filters', (
            SELECT jsonb_agg(
                jsonb_build_object(
                    'cluster_name', cluster_name,
                    'member_count', member_count,
                    'cluster_type', cluster_type,
                    'criteria', jsonb_build_object('clusters', jsonb_build_array(cluster_name))
                )
            )
            FROM attribute_vector_clusters
            WHERE member_count > 0
            ORDER BY member_count DESC
            LIMIT 10
        ),
        'domain_filters', (
            SELECT jsonb_agg(
                jsonb_build_object(
                    'domain_name', domain_name,
                    'regulatory_framework', regulatory_framework,
                    'criteria', jsonb_build_object('domain_name', domain_name)
                )
            )
            FROM kyc_onboarding_domains
        )
    );

    RETURN suggestions;
END;
$$;


--
-- Name: FUNCTION get_filter_suggestions(user_context jsonb); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.get_filter_suggestions(user_context jsonb) IS 'Provides intelligent filter suggestions based on user context';


--
-- Name: get_product_taxonomy_hierarchy(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.get_product_taxonomy_hierarchy(input_product_id integer) RETURNS TABLE(level integer, item_type character varying, item_id integer, item_name character varying, item_description text, parent_id integer, configuration jsonb, metadata jsonb)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    WITH RECURSIVE taxonomy_tree AS (
        -- Level 1: Product
        SELECT
            1 as level,
            'product'::VARCHAR as item_type,
            p.id as item_id,
            p.product_name as item_name,
            p.description as item_description,
            NULL::INTEGER as parent_id,
            jsonb_build_object(
                'contract_type', p.contract_type,
                'commercial_status', p.commercial_status,
                'pricing_model', p.pricing_model
            ) as configuration,
            jsonb_build_object(
                'target_market', p.target_market,
                'compliance_requirements', p.compliance_requirements
            ) as metadata
        FROM products p
        WHERE p.id = input_product_id

        UNION ALL

        -- Level 2: Services
        SELECT
            2 as level,
            'service'::VARCHAR as item_type,
            s.id as item_id,
            s.service_name as item_name,
            s.description as item_description,
            tt.item_id as parent_id,
            jsonb_build_object(
                'service_type', s.service_type,
                'delivery_model', s.delivery_model,
                'mapping_type', psm.mapping_type,
                'is_mandatory', psm.is_mandatory
            ) as configuration,
            jsonb_build_object(
                'billable', s.billable,
                'sla_requirements', s.sla_requirements
            ) as metadata
        FROM taxonomy_tree tt
        JOIN product_service_mappings psm ON psm.product_id = tt.item_id
        JOIN services s ON s.id = psm.service_id
        WHERE tt.level = 1

        UNION ALL

        -- Level 3: Resources
        SELECT
            3 as level,
            'resource'::VARCHAR as item_type,
            r.id as item_id,
            r.resource_name as item_name,
            r.description as item_description,
            tt.item_id as parent_id,
            jsonb_build_object(
                'resource_type', r.resource_type,
                'usage_type', srm.usage_type,
                'dependency_level', srm.dependency_level
            ) as configuration,
            jsonb_build_object(
                'criticality_level', r.criticality_level,
                'operational_status', r.operational_status,
                'compliance_classification', r.compliance_classification
            ) as metadata
        FROM taxonomy_tree tt
        JOIN service_resource_mappings srm ON srm.service_id = tt.item_id
        JOIN resource_objects r ON r.id = srm.resource_id
        WHERE tt.level = 2
    )
    SELECT * FROM taxonomy_tree
    ORDER BY level, item_name;
END;
$$;


--
-- Name: FUNCTION get_product_taxonomy_hierarchy(input_product_id integer); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.get_product_taxonomy_hierarchy(input_product_id integer) IS 'Retrieves complete hierarchical breakdown of a commercial product';


--
-- Name: insert_mandate_with_instruments(jsonb); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.insert_mandate_with_instruments(p_mandate jsonb) RETURNS character varying
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_mandate_id VARCHAR(100);
    v_instrument_id INTEGER;
    v_instrument JSONB;
    v_identifier TEXT;
    v_venue JSONB;
    v_channel JSONB;
    v_event TEXT;
BEGIN
    -- Insert main mandate
    INSERT INTO investment_mandates (
        mandate_id, cbu_id, asset_owner_name, asset_owner_lei,
        investment_manager_name, investment_manager_lei, base_currency,
        effective_date, expiry_date,
        gross_exposure_pct, net_exposure_pct, leverage_max,
        issuer_concentration_pct, country_concentration_pct, sector_concentration_pct,
        duration_min, duration_max, var_limit, dv01_limit,
        pre_trade_checks_required, maker_checker, stp_required, breach_handling,
        intraday_status, end_of_day_blotter, confirmations_required, matching_model
    ) VALUES (
        p_mandate->>'mandate_id',
        p_mandate->>'cbu_id',
        p_mandate->'asset_owner'->>'name',
        p_mandate->'asset_owner'->>'lei',
        p_mandate->'investment_manager'->>'name',
        p_mandate->'investment_manager'->>'lei',
        p_mandate->>'base_currency',
        (p_mandate->>'effective_date')::DATE,
        (p_mandate->>'expiry_date')::DATE,
        (p_mandate->'global_limits'->>'gross_exposure_pct')::NUMERIC,
        (p_mandate->'global_limits'->>'net_exposure_pct')::NUMERIC,
        (p_mandate->'global_limits'->>'leverage_max')::NUMERIC,
        (p_mandate->'global_limits'->>'issuer_concentration_pct')::NUMERIC,
        (p_mandate->'global_limits'->>'country_concentration_pct')::NUMERIC,
        (p_mandate->'global_limits'->>'sector_concentration_pct')::NUMERIC,
        (p_mandate->'global_limits'->'duration_bounds'->>'min')::NUMERIC,
        (p_mandate->'global_limits'->'duration_bounds'->>'max')::NUMERIC,
        (p_mandate->'global_limits'->>'var_limit')::NUMERIC,
        (p_mandate->'global_limits'->>'dv01_limit')::NUMERIC,
        (p_mandate->'controls'->>'pre_trade_checks_required')::BOOLEAN,
        (p_mandate->'controls'->>'maker_checker')::BOOLEAN,
        (p_mandate->'controls'->>'stp_required')::BOOLEAN,
        p_mandate->'controls'->>'breach_handling',
        p_mandate->'reporting'->>'intraday_status',
        (p_mandate->'reporting'->>'end_of_day_blotter')::BOOLEAN,
        (p_mandate->'reporting'->>'confirmations_required')::BOOLEAN,
        p_mandate->'reporting'->>'matching_model'
    ) RETURNING mandate_id INTO v_mandate_id;

    -- Insert benchmarks
    FOR v_identifier IN SELECT jsonb_array_elements_text(p_mandate->'benchmarks')
    LOOP
        INSERT INTO mandate_benchmarks (mandate_id, benchmark_name)
        VALUES (v_mandate_id, v_identifier);
    END LOOP;

    -- Insert instruments
    FOR v_instrument IN SELECT jsonb_array_elements(p_mandate->'instruments')
    LOOP
        INSERT INTO mandate_instruments (
            mandate_id, instrument_family, subtype,
            cfi_code, isda_taxonomy,
            order_types, time_in_force, min_clip, algo_flags_allowed,
            settlement_type, settlement_cycle, place_of_settlement, allow_partials, ssi_reference,
            clearing_required, clearing_house, margin_model, eligible_collateral_schedule,
            min_tenor, max_tenor,
            exposure_pct, short_allowed, issuer_max_pct, rating_floor,
            limit_duration_min, limit_duration_max, dv01_cap,
            allocation_model, notes
        ) VALUES (
            v_mandate_id,
            v_instrument->>'instrument_family',
            v_instrument->>'subtype',
            v_instrument->'classification'->>'cfi_code',
            v_instrument->'classification'->>'isda_taxonomy',
            ARRAY(SELECT jsonb_array_elements_text(v_instrument->'order_capabilities'->'order_types')),
            ARRAY(SELECT jsonb_array_elements_text(v_instrument->'order_capabilities'->'time_in_force')),
            (v_instrument->'order_capabilities'->>'min_clip')::NUMERIC,
            (v_instrument->'order_capabilities'->>'algo_flags_allowed')::BOOLEAN,
            v_instrument->'settlement'->>'type',
            v_instrument->'settlement'->>'cycle',
            v_instrument->'settlement'->>'place_of_settlement',
            (v_instrument->'settlement'->>'allow_partials')::BOOLEAN,
            v_instrument->'settlement'->>'ssi_reference',
            (v_instrument->'otc_terms'->>'clearing_required')::BOOLEAN,
            v_instrument->'otc_terms'->>'clearing_house',
            v_instrument->'otc_terms'->>'margin_model',
            v_instrument->'otc_terms'->>'eligible_collateral_schedule',
            v_instrument->'otc_terms'->'term_bounds'->>'min_tenor',
            v_instrument->'otc_terms'->'term_bounds'->>'max_tenor',
            (v_instrument->'limits'->>'exposure_pct')::NUMERIC,
            (v_instrument->'limits'->>'short_allowed')::BOOLEAN,
            (v_instrument->'limits'->>'issuer_max_pct')::NUMERIC,
            v_instrument->'limits'->>'rating_floor',
            (v_instrument->'limits'->'duration_bounds'->>'min')::NUMERIC,
            (v_instrument->'limits'->'duration_bounds'->>'max')::NUMERIC,
            (v_instrument->'limits'->>'dv01_cap')::NUMERIC,
            v_instrument->>'allocation_model',
            v_instrument->>'notes'
        ) RETURNING id INTO v_instrument_id;

        -- Insert identifiers required
        FOR v_identifier IN SELECT jsonb_array_elements_text(v_instrument->'identifiers_required')
        LOOP
            INSERT INTO mandate_instrument_identifiers (instrument_id, identifier_type)
            VALUES (v_instrument_id, v_identifier);
        END LOOP;

        -- Insert venues
        FOR v_venue IN SELECT jsonb_array_elements(v_instrument->'venues')
        LOOP
            INSERT INTO mandate_instrument_venues (instrument_id, mic, preferred)
            VALUES (v_instrument_id, v_venue->>'mic', (v_venue->>'preferred')::BOOLEAN);
        END LOOP;

        -- Insert instruction channels
        FOR v_channel IN SELECT jsonb_array_elements(v_instrument->'instruction_channels')
        LOOP
            INSERT INTO mandate_instruction_channels (instrument_id, channel, formats, allowed_flows, stp_required)
            VALUES (
                v_instrument_id,
                v_channel->>'channel',
                ARRAY(SELECT jsonb_array_elements_text(v_channel->'formats')),
                ARRAY(SELECT jsonb_array_elements_text(v_channel->'allowed_flows')),
                (v_channel->>'stp_required')::BOOLEAN
            );
        END LOOP;

        -- Insert lifecycle events
        FOR v_event IN SELECT jsonb_array_elements_text(v_instrument->'lifecycle_events')
        LOOP
            INSERT INTO mandate_lifecycle_events (instrument_id, event_type)
            VALUES (v_instrument_id, v_event);
        END LOOP;
    END LOOP;

    RETURN v_mandate_id;
END;
$$;


--
-- Name: mark_rule_for_recompilation(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.mark_rule_for_recompilation() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.rule_definition IS DISTINCT FROM NEW.rule_definition THEN
        NEW.compilation_status = 'outdated';
        NEW.compiled_rust_code = NULL;
        NEW.compiled_wasm_binary = NULL;

        -- Add to compilation queue
        INSERT INTO rule_compilation_queue (rule_id, compilation_type, priority)
        VALUES (NEW.id, 'both', 5)
        ON CONFLICT (rule_id, compilation_type, status)
        DO UPDATE SET priority = LEAST(rule_compilation_queue.priority, 5);
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: refresh_data_dictionary(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.refresh_data_dictionary() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_data_dictionary;
END;
$$;


--
-- Name: resolve_persistence_target(integer, character varying); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.resolve_persistence_target(attr_id integer, operation_type character varying DEFAULT 'write'::character varying) RETURNS jsonb
    LANGUAGE plpgsql
    AS $$
DECLARE
    result JSONB;
    primary_target JSONB;
    cache_targets JSONB[];
    audit_targets JSONB[];
BEGIN
    -- Get primary persistence target
    SELECT jsonb_build_object(
        'system', ps.system_name,
        'entity', pe.entity_name,
        'config', pe.entity_config,
        'field_mapping', apm.field_mapping,
        'transformation_rules', apm.transformation_rules
    ) INTO primary_target
    FROM attribute_objects ao
    JOIN persistence_entities pe ON pe.id = ao.primary_persistence_entity_id
    JOIN persistence_systems ps ON ps.id = pe.system_id
    JOIN attribute_persistence_mappings apm ON apm.attribute_id = ao.id AND apm.persistence_entity_id = pe.id
    WHERE ao.id = attr_id;

    -- Get cache targets (for read operations)
    IF operation_type = 'read' THEN
        SELECT array_agg(
            jsonb_build_object(
                'system', ps.system_name,
                'entity', pe.entity_name,
                'config', pe.entity_config,
                'priority', CASE WHEN ps.system_type = 'cache' THEN 1 ELSE 2 END
            )
        ) INTO cache_targets
        FROM attribute_persistence_mappings apm
        JOIN persistence_entities pe ON pe.id = apm.persistence_entity_id
        JOIN persistence_systems ps ON ps.id = pe.system_id
        WHERE apm.attribute_id = attr_id
        AND ps.capabilities->>'read' = 'true';
    END IF;

    -- Get audit targets (for write operations)
    IF operation_type IN ('write', 'update', 'delete') THEN
        SELECT array_agg(
            jsonb_build_object(
                'system', ps.system_name,
                'entity', pe.entity_name,
                'config', pe.entity_config,
                'async', CASE WHEN ps.system_type = 'blockchain' THEN true ELSE false END
            )
        ) INTO audit_targets
        FROM attribute_persistence_mappings apm
        JOIN persistence_entities pe ON pe.id = apm.persistence_entity_id
        JOIN persistence_systems ps ON ps.id = pe.system_id
        WHERE apm.attribute_id = attr_id
        AND ps.system_type IN ('blockchain', 'audit');
    END IF;

    result := jsonb_build_object(
        'primary', primary_target,
        'cache_targets', COALESCE(cache_targets, '[]'::JSONB[]),
        'audit_targets', COALESCE(audit_targets, '[]'::JSONB[]),
        'operation_type', operation_type
    );

    RETURN result;
END;
$$;


--
-- Name: update_comprehensive_metadata(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_comprehensive_metadata() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: update_embedding_trigger(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_embedding_trigger() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: update_resource_instances_updated_at(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_resource_instances_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$;


--
-- Name: update_updated_at_column(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_updated_at_column() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$;


--
-- Name: update_workflow_completion(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_workflow_completion() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Update completion percentage and current stage
    UPDATE onboarding_workflows SET
        completion_percentage = (
            SELECT ROUND(
                100.0 * COUNT(CASE WHEN task_status = 'completed' THEN 1 END) /
                NULLIF(COUNT(*), 0)
            )::int
            FROM onboarding_resource_tasks
            WHERE workflow_id = NEW.workflow_id
        ),
        current_stage = (
            SELECT CONCAT(rt.template_name, ' - ', rc.capability_name)
            FROM onboarding_resource_tasks ort
            JOIN resource_templates rt ON ort.resource_template_id = rt.id
            JOIN resource_capabilities rc ON ort.capability_id = rc.id
            WHERE ort.workflow_id = NEW.workflow_id
            AND ort.task_status = 'in_progress'
            ORDER BY ort.task_order
            LIMIT 1
        ),
        workflow_status = CASE
            WHEN (SELECT COUNT(*) FROM onboarding_resource_tasks
                  WHERE workflow_id = NEW.workflow_id AND task_status = 'failed') > 0
            THEN 'failed'
            WHEN (SELECT COUNT(*) FROM onboarding_resource_tasks
                  WHERE workflow_id = NEW.workflow_id AND task_status != 'completed') = 0
            THEN 'completed'
            WHEN (SELECT COUNT(*) FROM onboarding_resource_tasks
                  WHERE workflow_id = NEW.workflow_id AND task_status IN ('in_progress', 'completed')) > 0
            THEN 'in_progress'
            ELSE workflow_status
        END,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.workflow_id;

    RETURN NEW;
END;
$$;


--
-- Name: validate_commercial_taxonomy(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.validate_commercial_taxonomy() RETURNS TABLE(validation_rule character varying, status character varying, message text, affected_items jsonb)
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Rule 1: Every product should have at least one core service
    RETURN QUERY
    SELECT
        'product_has_core_service'::VARCHAR,
        CASE WHEN core_service_count > 0 THEN 'PASS' ELSE 'FAIL' END,
        CASE WHEN core_service_count > 0
            THEN format('Product has %s core services', core_service_count)
            ELSE 'Product must have at least one core service'
        END,
        jsonb_build_object('product_id', p.id, 'core_services', core_service_count)
    FROM (
        SELECT
            p.id,
            COUNT(CASE WHEN s.service_type = 'core' THEN 1 END) as core_service_count
        FROM products p
        LEFT JOIN product_service_mappings psm ON psm.product_id = p.id
        LEFT JOIN services s ON s.id = psm.service_id
        GROUP BY p.id
    ) p;

    -- Rule 2: Every service should have at least one required resource
    RETURN QUERY
    SELECT
        'service_has_required_resource'::VARCHAR,
        CASE WHEN required_resource_count > 0 THEN 'PASS' ELSE 'WARN' END,
        CASE WHEN required_resource_count > 0
            THEN format('Service has %s required resources', required_resource_count)
            ELSE 'Service should have at least one required resource'
        END,
        jsonb_build_object('service_id', s.id, 'required_resources', required_resource_count)
    FROM (
        SELECT
            s.id,
            COUNT(CASE WHEN srm.usage_type = 'required' THEN 1 END) as required_resource_count
        FROM services s
        LEFT JOIN service_resource_mappings srm ON srm.service_id = s.id
        GROUP BY s.id
    ) s;

    -- Rule 3: Critical resources should have failover configured
    RETURN QUERY
    SELECT
        'critical_resource_failover'::VARCHAR,
        CASE WHEN failover_resource_id IS NOT NULL THEN 'PASS' ELSE 'WARN' END,
        CASE WHEN failover_resource_id IS NOT NULL
            THEN 'Critical resource has failover configured'
            ELSE format('Critical resource "%s" should have failover configured', r.resource_name)
        END,
        jsonb_build_object('resource_id', r.id, 'resource_name', r.resource_name)
    FROM resource_objects r
    JOIN service_resource_mappings srm ON srm.resource_id = r.id
    WHERE r.criticality_level = 'critical' AND srm.dependency_level = 1;
END;
$$;


--
-- Name: FUNCTION validate_commercial_taxonomy(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.validate_commercial_taxonomy() IS 'Validates integrity and completeness of commercial taxonomy structure';


--
-- Name: validate_derived_attribute(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.validate_derived_attribute() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: validate_ebnf_grammar(text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.validate_ebnf_grammar(ebnf_text text) RETURNS jsonb
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: FUNCTION validate_ebnf_grammar(ebnf_text text); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.validate_ebnf_grammar(ebnf_text text) IS 'Validates EBNF grammar syntax for derivation rules';


--
-- Name: validate_resource_attribute_set(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.validate_resource_attribute_set(p_resource_id integer) RETURNS TABLE(rule_name text, rule_type text, severity text, status text, message text, details jsonb)
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: FUNCTION validate_resource_attribute_set(p_resource_id integer); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.validate_resource_attribute_set(p_resource_id integer) IS 'Validates integrity and completeness of resource attribute sets';


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: ai_attribute_contexts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_attribute_contexts (
    id integer NOT NULL,
    attribute_id integer,
    context_type character varying(50) NOT NULL,
    context_title character varying(200) NOT NULL,
    detailed_description text NOT NULL,
    examples jsonb DEFAULT '[]'::jsonb,
    keywords text[],
    related_attributes integer[],
    confidence_score numeric(3,2) DEFAULT 1.0,
    source character varying(100),
    last_validated timestamp without time zone,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE ai_attribute_contexts; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.ai_attribute_contexts IS 'Rich contextual descriptions for AI LLM understanding and reasoning';


--
-- Name: ai_attribute_contexts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.ai_attribute_contexts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ai_attribute_contexts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.ai_attribute_contexts_id_seq OWNED BY public.ai_attribute_contexts.id;


--
-- Name: ai_metadata_contexts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_metadata_contexts (
    id integer NOT NULL,
    context_name character varying(100) NOT NULL,
    description text,
    prompt_template text,
    example_queries jsonb DEFAULT '[]'::jsonb,
    response_format jsonb DEFAULT '{}'::jsonb,
    model_config jsonb DEFAULT '{}'::jsonb,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: ai_metadata_contexts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.ai_metadata_contexts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ai_metadata_contexts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.ai_metadata_contexts_id_seq OWNED BY public.ai_metadata_contexts.id;


--
-- Name: ai_prompt_contexts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_prompt_contexts (
    id integer NOT NULL,
    context_name character varying(100) NOT NULL,
    scenario_description text NOT NULL,
    base_prompt_template text NOT NULL,
    attribute_inclusion_rules jsonb DEFAULT '{}'::jsonb,
    response_format_template jsonb DEFAULT '{}'::jsonb,
    few_shot_examples jsonb DEFAULT '[]'::jsonb,
    model_parameters jsonb DEFAULT '{}'::jsonb,
    success_criteria text,
    common_failure_modes text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: ai_prompt_contexts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.ai_prompt_contexts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ai_prompt_contexts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.ai_prompt_contexts_id_seq OWNED BY public.ai_prompt_contexts.id;


--
-- Name: ai_training_examples; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_training_examples (
    id integer NOT NULL,
    attribute_id integer,
    example_type character varying(50) NOT NULL,
    input_example jsonb NOT NULL,
    expected_output jsonb,
    explanation text NOT NULL,
    difficulty_level character varying(20) DEFAULT 'medium'::character varying,
    tags text[],
    ai_model_accuracy numeric(4,3),
    human_annotation text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE ai_training_examples; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.ai_training_examples IS 'Training examples and edge cases for AI model fine-tuning';


--
-- Name: ai_training_examples_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.ai_training_examples_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ai_training_examples_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.ai_training_examples_id_seq OWNED BY public.ai_training_examples.id;


--
-- Name: attribute_objects; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_objects (
    id integer NOT NULL,
    resource_id integer,
    attribute_name character varying(100) NOT NULL,
    data_type character varying(30) NOT NULL,
    description text,
    is_required boolean DEFAULT false,
    min_length integer,
    max_length integer,
    min_value numeric,
    max_value numeric,
    allowed_values jsonb,
    validation_pattern text,
    persistence_system character varying(100),
    persistence_entity character varying(100),
    persistence_identifier character varying(100),
    ui_group character varying(100),
    ui_display_order integer DEFAULT 0,
    ui_render_hint character varying(30) DEFAULT 'text-input'::character varying,
    ui_label character varying(200),
    ui_help_text text,
    wizard_step integer,
    wizard_step_title character varying(200),
    wizard_next_button text,
    wizard_previous_button text,
    wizard_description text,
    generation_examples jsonb DEFAULT '[]'::jsonb,
    rules_dsl text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    semantic_tags jsonb DEFAULT '[]'::jsonb,
    ai_context jsonb DEFAULT '{}'::jsonb,
    embedding_vector public.vector(1536),
    search_keywords text[],
    ui_component_type character varying(50) DEFAULT 'text-input'::character varying,
    ui_layout_config jsonb DEFAULT '{}'::jsonb,
    ui_styling jsonb DEFAULT '{}'::jsonb,
    ui_behavior jsonb DEFAULT '{}'::jsonb,
    conditional_logic jsonb DEFAULT '{}'::jsonb,
    relationship_metadata jsonb DEFAULT '{}'::jsonb,
    ai_prompt_templates jsonb DEFAULT '{}'::jsonb,
    form_generation_rules jsonb DEFAULT '{}'::jsonb,
    accessibility_config jsonb DEFAULT '{}'::jsonb,
    responsive_config jsonb DEFAULT '{}'::jsonb,
    data_flow_config jsonb DEFAULT '{}'::jsonb,
    primary_persistence_entity_id integer,
    backup_persistence_entities integer[],
    value_lifecycle jsonb DEFAULT '{}'::jsonb,
    data_governance jsonb DEFAULT '{}'::jsonb,
    compliance_metadata jsonb DEFAULT '{}'::jsonb,
    extended_description text,
    business_context text,
    technical_context text,
    user_guidance text,
    ai_training_examples text,
    domain_terminology text,
    related_concepts text[],
    usage_scenarios text,
    data_lineage_description text,
    compliance_explanation text,
    error_scenarios text,
    integration_notes text,
    attribute_class character varying(20) DEFAULT 'real'::character varying,
    visibility_scope character varying(20) DEFAULT 'public'::character varying,
    derivation_rule_ebnf text,
    derivation_dependencies integer[],
    derivation_complexity character varying(20) DEFAULT 'simple'::character varying,
    derivation_frequency character varying(20) DEFAULT 'on_demand'::character varying,
    materialization_strategy character varying(30) DEFAULT 'computed'::character varying,
    source_attributes jsonb DEFAULT '[]'::jsonb,
    transformation_logic jsonb DEFAULT '{}'::jsonb,
    quality_rules jsonb DEFAULT '{}'::jsonb,
    CONSTRAINT attribute_objects_attribute_class_check CHECK (((attribute_class)::text = ANY ((ARRAY['real'::character varying, 'derived'::character varying])::text[]))),
    CONSTRAINT attribute_objects_data_type_check CHECK (((data_type)::text = ANY ((ARRAY['String'::character varying, 'Number'::character varying, 'Boolean'::character varying, 'Date'::character varying, 'Decimal'::character varying, 'List'::character varying, 'Enum'::character varying])::text[]))),
    CONSTRAINT attribute_objects_derivation_complexity_check CHECK (((derivation_complexity)::text = ANY ((ARRAY['simple'::character varying, 'moderate'::character varying, 'complex'::character varying, 'ai_assisted'::character varying])::text[]))),
    CONSTRAINT attribute_objects_derivation_frequency_check CHECK (((derivation_frequency)::text = ANY ((ARRAY['real_time'::character varying, 'on_demand'::character varying, 'batch'::character varying, 'scheduled'::character varying])::text[]))),
    CONSTRAINT attribute_objects_materialization_strategy_check CHECK (((materialization_strategy)::text = ANY ((ARRAY['computed'::character varying, 'cached'::character varying, 'persisted'::character varying, 'hybrid'::character varying])::text[]))),
    CONSTRAINT attribute_objects_visibility_scope_check CHECK (((visibility_scope)::text = ANY ((ARRAY['public'::character varying, 'private'::character varying, 'internal'::character varying, 'restricted'::character varying])::text[])))
);


--
-- Name: COLUMN attribute_objects.semantic_tags; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.semantic_tags IS 'Tags for semantic search and AI context understanding';


--
-- Name: COLUMN attribute_objects.ai_context; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.ai_context IS 'AI-specific metadata for enhanced search and suggestions';


--
-- Name: COLUMN attribute_objects.embedding_vector; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.embedding_vector IS 'Vector embedding for semantic similarity search';


--
-- Name: COLUMN attribute_objects.ui_component_type; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.ui_component_type IS 'Specific UI component to render this attribute';


--
-- Name: COLUMN attribute_objects.ui_layout_config; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.ui_layout_config IS 'Component-specific layout and positioning configuration';


--
-- Name: COLUMN attribute_objects.ui_styling; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.ui_styling IS 'Custom styling rules for the UI component';


--
-- Name: COLUMN attribute_objects.ui_behavior; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.ui_behavior IS 'Interactive behavior configuration';


--
-- Name: COLUMN attribute_objects.conditional_logic; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.conditional_logic IS 'Rules for show/hide/enable/disable based on other fields';


--
-- Name: COLUMN attribute_objects.relationship_metadata; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.relationship_metadata IS 'Metadata about relationships with other attributes';


--
-- Name: COLUMN attribute_objects.ai_prompt_templates; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.ai_prompt_templates IS 'Custom prompt templates for AI interactions';


--
-- Name: COLUMN attribute_objects.form_generation_rules; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.form_generation_rules IS 'Rules for automatic form generation';


--
-- Name: COLUMN attribute_objects.accessibility_config; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.accessibility_config IS 'Accessibility settings and ARIA configurations';


--
-- Name: COLUMN attribute_objects.responsive_config; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.responsive_config IS 'Responsive design configurations for different screen sizes';


--
-- Name: COLUMN attribute_objects.data_flow_config; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.data_flow_config IS 'Configuration for data flow and real-time updates';


--
-- Name: COLUMN attribute_objects.extended_description; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.extended_description IS 'Comprehensive description for AI LLM context and understanding';


--
-- Name: COLUMN attribute_objects.business_context; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.business_context IS 'Business purpose and usage context for AI decision making';


--
-- Name: COLUMN attribute_objects.technical_context; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.technical_context IS 'Technical implementation details for AI code generation';


--
-- Name: COLUMN attribute_objects.user_guidance; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.user_guidance IS 'User-facing help text for AI user assistance';


--
-- Name: COLUMN attribute_objects.ai_training_examples; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.ai_training_examples IS 'Examples specifically for AI training and context';


--
-- Name: COLUMN attribute_objects.domain_terminology; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.domain_terminology IS 'Domain-specific terminology for AI semantic understanding';


--
-- Name: COLUMN attribute_objects.usage_scenarios; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.usage_scenarios IS 'Common usage patterns for AI recommendation engines';


--
-- Name: COLUMN attribute_objects.attribute_class; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.attribute_class IS 'Classification: real (directly collected) or derived (computed from other attributes)';


--
-- Name: COLUMN attribute_objects.visibility_scope; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.visibility_scope IS 'Visibility scope: public, private, internal, or restricted';


--
-- Name: COLUMN attribute_objects.derivation_rule_ebnf; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.derivation_rule_ebnf IS 'EBNF grammar notation for derivation rules (derived attributes only)';


--
-- Name: COLUMN attribute_objects.derivation_dependencies; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.derivation_dependencies IS 'Array of attribute IDs this derived attribute depends on';


--
-- Name: COLUMN attribute_objects.derivation_complexity; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.derivation_complexity IS 'Complexity level of the derivation logic';


--
-- Name: COLUMN attribute_objects.materialization_strategy; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.attribute_objects.materialization_strategy IS 'How the derived value is computed and stored';


--
-- Name: derived_attribute_quality_metrics; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.derived_attribute_quality_metrics (
    id integer NOT NULL,
    attribute_id integer,
    metric_date date DEFAULT CURRENT_DATE,
    execution_count integer DEFAULT 0,
    success_rate numeric(5,2),
    avg_execution_time_ms integer,
    error_count integer DEFAULT 0,
    data_quality_score numeric(4,3),
    completeness_rate numeric(5,2),
    accuracy_score numeric(4,3),
    consistency_score numeric(4,3),
    metadata jsonb DEFAULT '{}'::jsonb
);


--
-- Name: derived_attribute_rules; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.derived_attribute_rules (
    id integer NOT NULL,
    derived_attribute_id integer,
    rule_name character varying(100) NOT NULL,
    rule_description text,
    ebnf_grammar text NOT NULL,
    execution_order integer DEFAULT 1,
    rule_type character varying(50) DEFAULT 'transformation'::character varying,
    input_attributes jsonb NOT NULL,
    output_format jsonb DEFAULT '{}'::jsonb,
    error_handling jsonb DEFAULT '{}'::jsonb,
    performance_hints jsonb DEFAULT '{}'::jsonb,
    test_cases jsonb DEFAULT '[]'::jsonb,
    version character varying(20) DEFAULT '1.0'::character varying,
    is_active boolean DEFAULT true,
    created_by character varying(100),
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE derived_attribute_rules; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.derived_attribute_rules IS 'Detailed derivation rules with EBNF grammar for complex derived attributes';


--
-- Name: attribute_classification_view; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.attribute_classification_view AS
 SELECT ao.id,
    ao.attribute_name,
    ao.attribute_class,
    ao.visibility_scope,
    ao.data_type,
    ao.description,
    ao.extended_description,
        CASE
            WHEN ((ao.attribute_class)::text = 'real'::text) THEN 'Real/Public Attribute'::text
            WHEN ((ao.attribute_class)::text = 'derived'::text) THEN 'Derived/Synthetic Attribute'::text
            ELSE NULL::text
        END AS classification_description,
    ao.derivation_rule_ebnf,
    ao.derivation_dependencies,
    ao.derivation_complexity,
    ao.derivation_frequency,
    ao.materialization_strategy,
        CASE
            WHEN ((ao.attribute_class)::text = 'derived'::text) THEN ( SELECT array_agg(jsonb_build_object('id', src_ao.id, 'name', src_ao.attribute_name, 'type', src_ao.data_type, 'class', src_ao.attribute_class)) AS array_agg
               FROM (unnest(ao.derivation_dependencies) dep_id(dep_id)
                 JOIN public.attribute_objects src_ao ON ((src_ao.id = dep_id.dep_id))))
            ELSE NULL::jsonb[]
        END AS source_attribute_details,
    array_agg(
        CASE
            WHEN (dar.id IS NOT NULL) THEN jsonb_build_object('rule_name', dar.rule_name, 'ebnf_grammar', dar.ebnf_grammar, 'rule_type', dar.rule_type, 'execution_order', dar.execution_order, 'is_active', dar.is_active)
            ELSE NULL::jsonb
        END) FILTER (WHERE (dar.id IS NOT NULL)) AS derivation_rules,
    daqm.success_rate,
    daqm.avg_execution_time_ms,
    daqm.data_quality_score,
    daqm.completeness_rate,
    ao.ai_context,
    ao.semantic_tags,
    ao.ui_component_type,
    ao.ui_layout_config,
    concat_ws(' | '::text, ao.attribute_name,
        CASE ao.attribute_class
            WHEN 'real'::text THEN 'Real attribute directly collected from users or systems'::text
            WHEN 'derived'::text THEN ('Derived attribute computed from other attributes using: '::text || COALESCE(ao.derivation_rule_ebnf, 'undefined rules'::text))
            ELSE NULL::text
        END, ao.description, ao.extended_description, ao.business_context) AS full_ai_context
   FROM ((public.attribute_objects ao
     LEFT JOIN public.derived_attribute_rules dar ON (((dar.derived_attribute_id = ao.id) AND (dar.is_active = true))))
     LEFT JOIN public.derived_attribute_quality_metrics daqm ON (((daqm.attribute_id = ao.id) AND (daqm.metric_date = ( SELECT max(daqm2.metric_date) AS max
           FROM public.derived_attribute_quality_metrics daqm2
          WHERE (daqm2.attribute_id = ao.id))))))
  GROUP BY ao.id, ao.attribute_name, ao.attribute_class, ao.visibility_scope, ao.data_type, ao.description, ao.extended_description, ao.derivation_rule_ebnf, ao.derivation_dependencies, ao.derivation_complexity, ao.derivation_frequency, ao.materialization_strategy, ao.ai_context, ao.semantic_tags, ao.ui_component_type, ao.ui_layout_config, ao.business_context, daqm.success_rate, daqm.avg_execution_time_ms, daqm.data_quality_score, daqm.completeness_rate;


--
-- Name: VIEW attribute_classification_view; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON VIEW public.attribute_classification_view IS 'Comprehensive view of attribute classification and derivation metadata';


--
-- Name: attribute_cluster_memberships; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_cluster_memberships (
    id integer NOT NULL,
    attribute_id integer,
    cluster_id integer,
    membership_strength numeric(4,3) DEFAULT 1.0,
    distance_to_centroid numeric(10,6),
    membership_type character varying(50) DEFAULT 'primary'::character varying,
    assigned_by character varying(50) DEFAULT 'algorithm'::character varying,
    confidence_score numeric(4,3) DEFAULT 1.0,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE attribute_cluster_memberships; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.attribute_cluster_memberships IS 'Many-to-many relationship between attributes and clusters';


--
-- Name: attribute_cluster_memberships_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_cluster_memberships_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_cluster_memberships_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_cluster_memberships_id_seq OWNED BY public.attribute_cluster_memberships.id;


--
-- Name: attribute_domain_mappings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_domain_mappings (
    id integer NOT NULL,
    attribute_id integer,
    domain_id integer,
    context_role character varying(100),
    importance_weight numeric(3,2) DEFAULT 1.0,
    compliance_criticality character varying(20) DEFAULT 'medium'::character varying,
    data_sensitivity character varying(20) DEFAULT 'medium'::character varying,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE attribute_domain_mappings; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.attribute_domain_mappings IS 'Links attributes to their domain contexts with compliance metadata';


--
-- Name: attribute_tag_assignments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_tag_assignments (
    id integer NOT NULL,
    attribute_id integer,
    tag_id integer,
    assignment_confidence numeric(4,3) DEFAULT 1.0,
    assigned_by character varying(50) DEFAULT 'manual'::character varying,
    assignment_reason text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: attribute_tags; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_tags (
    id integer NOT NULL,
    tag_name character varying(100) NOT NULL,
    tag_description text,
    tag_category character varying(50),
    tag_color character varying(7),
    icon_name character varying(50),
    parent_tag_id integer,
    hierarchy_level integer DEFAULT 1,
    usage_count integer DEFAULT 0,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE attribute_tags; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.attribute_tags IS 'Hierarchical tagging system for attribute categorization';


--
-- Name: attribute_vector_clusters; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_vector_clusters (
    id integer NOT NULL,
    cluster_name character varying(100) NOT NULL,
    cluster_description text,
    cluster_type character varying(50) DEFAULT 'semantic'::character varying,
    cluster_algorithm character varying(50) DEFAULT 'kmeans'::character varying,
    cluster_parameters jsonb DEFAULT '{}'::jsonb,
    centroid_vector public.vector(1536),
    cluster_radius numeric(10,6),
    quality_score numeric(4,3),
    member_count integer DEFAULT 0,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE attribute_vector_clusters; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.attribute_vector_clusters IS 'Semantic and functional clusters of related attributes';


--
-- Name: kyc_onboarding_domains; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.kyc_onboarding_domains (
    id integer NOT NULL,
    domain_name character varying(100) NOT NULL,
    description text,
    regulatory_framework character varying(100),
    compliance_requirements jsonb DEFAULT '{}'::jsonb,
    data_classification character varying(50),
    retention_policy jsonb DEFAULT '{}'::jsonb,
    audit_requirements jsonb DEFAULT '{}'::jsonb,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE kyc_onboarding_domains; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.kyc_onboarding_domains IS 'Domain-specific contexts for KYC and onboarding attributes';


--
-- Name: persistence_entities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.persistence_entities (
    id integer NOT NULL,
    entity_name character varying(100) NOT NULL,
    system_id integer,
    entity_type character varying(50) NOT NULL,
    entity_config jsonb NOT NULL,
    schema_definition jsonb DEFAULT '{}'::jsonb,
    access_patterns jsonb DEFAULT '{}'::jsonb,
    data_retention jsonb DEFAULT '{}'::jsonb,
    versioning_config jsonb DEFAULT '{}'::jsonb,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE persistence_entities; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.persistence_entities IS 'Specific entities (tables, collections, endpoints) within persistence systems';


--
-- Name: persistence_systems; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.persistence_systems (
    id integer NOT NULL,
    system_name character varying(100) NOT NULL,
    system_type character varying(50) NOT NULL,
    connection_config jsonb NOT NULL,
    capabilities jsonb DEFAULT '{}'::jsonb,
    performance_profile jsonb DEFAULT '{}'::jsonb,
    security_config jsonb DEFAULT '{}'::jsonb,
    status character varying(20) DEFAULT 'active'::character varying,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE persistence_systems; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.persistence_systems IS 'Registry of all systems where attribute values can be stored';


--
-- Name: attribute_comprehensive_filter_view; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.attribute_comprehensive_filter_view AS
 SELECT ao.id,
    ao.attribute_name,
    ao.attribute_class,
    ao.visibility_scope,
    ao.data_type,
    ao.description,
    ao.extended_description,
    ao.business_context,
    ao.technical_context,
    ao.ui_component_type,
    ao.ui_group,
    ao.derivation_complexity,
    ao.materialization_strategy,
    kod.domain_name,
    kod.regulatory_framework,
    adm.compliance_criticality,
    adm.data_sensitivity,
    array_agg(DISTINCT avc.cluster_name) FILTER (WHERE (avc.cluster_name IS NOT NULL)) AS cluster_names,
    array_agg(DISTINCT avc.cluster_type) FILTER (WHERE (avc.cluster_type IS NOT NULL)) AS cluster_types,
    array_agg(DISTINCT acm.membership_strength) FILTER (WHERE (acm.membership_strength IS NOT NULL)) AS cluster_memberships,
    array_agg(DISTINCT at.tag_name) FILTER (WHERE (at.tag_name IS NOT NULL)) AS tags,
    array_agg(DISTINCT at.tag_category) FILTER (WHERE (at.tag_category IS NOT NULL)) AS tag_categories,
    ao.semantic_tags,
    ao.search_keywords,
    ao.ai_context,
    daqm.success_rate,
    daqm.data_quality_score,
    daqm.completeness_rate,
    ps.system_name AS persistence_system,
    ps.system_type AS persistence_type,
    ao.embedding_vector,
    concat_ws(' '::text, ao.attribute_name, ao.description, ao.extended_description, ao.business_context, ao.technical_context, string_agg(DISTINCT (at.tag_name)::text, ' '::text), string_agg(DISTINCT (avc.cluster_name)::text, ' '::text), kod.domain_name) AS searchable_text,
    jsonb_build_object('basic_info', jsonb_build_object('id', ao.id, 'name', ao.attribute_name, 'class', ao.attribute_class, 'visibility', ao.visibility_scope, 'type', ao.data_type), 'classification', jsonb_build_object('domain', kod.domain_name, 'compliance', adm.compliance_criticality, 'sensitivity', adm.data_sensitivity, 'derivation_complexity', ao.derivation_complexity), 'clustering', jsonb_build_object('clusters', array_agg(DISTINCT avc.cluster_name) FILTER (WHERE (avc.cluster_name IS NOT NULL)), 'cluster_types', array_agg(DISTINCT avc.cluster_type) FILTER (WHERE (avc.cluster_type IS NOT NULL))), 'ui_metadata', jsonb_build_object('component_type', ao.ui_component_type, 'group', ao.ui_group, 'display_order', ao.ui_display_order), 'persistence', jsonb_build_object('system', ps.system_name, 'type', ps.system_type), 'tags', array_agg(DISTINCT at.tag_name) FILTER (WHERE (at.tag_name IS NOT NULL))) AS filter_metadata
   FROM (((((((((public.attribute_objects ao
     LEFT JOIN public.attribute_domain_mappings adm ON ((adm.attribute_id = ao.id)))
     LEFT JOIN public.kyc_onboarding_domains kod ON ((kod.id = adm.domain_id)))
     LEFT JOIN public.attribute_cluster_memberships acm ON ((acm.attribute_id = ao.id)))
     LEFT JOIN public.attribute_vector_clusters avc ON ((avc.id = acm.cluster_id)))
     LEFT JOIN public.attribute_tag_assignments ata ON ((ata.attribute_id = ao.id)))
     LEFT JOIN public.attribute_tags at ON ((at.id = ata.tag_id)))
     LEFT JOIN public.derived_attribute_quality_metrics daqm ON (((daqm.attribute_id = ao.id) AND (daqm.metric_date = ( SELECT max(derived_attribute_quality_metrics.metric_date) AS max
           FROM public.derived_attribute_quality_metrics
          WHERE (derived_attribute_quality_metrics.attribute_id = ao.id))))))
     LEFT JOIN public.persistence_entities pe ON ((pe.id = ao.primary_persistence_entity_id)))
     LEFT JOIN public.persistence_systems ps ON ((ps.id = pe.system_id)))
  GROUP BY ao.id, ao.attribute_name, ao.attribute_class, ao.visibility_scope, ao.data_type, ao.description, ao.extended_description, ao.business_context, ao.technical_context, ao.ui_component_type, ao.ui_group, ao.ui_display_order, ao.derivation_complexity, ao.materialization_strategy, kod.domain_name, kod.regulatory_framework, adm.compliance_criticality, adm.data_sensitivity, ao.semantic_tags, ao.search_keywords, ao.ai_context, daqm.success_rate, daqm.data_quality_score, daqm.completeness_rate, ps.system_name, ps.system_type, ao.embedding_vector;


--
-- Name: VIEW attribute_comprehensive_filter_view; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON VIEW public.attribute_comprehensive_filter_view IS 'Comprehensive view optimized for filtering and searching attributes';


--
-- Name: attribute_documentation; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_documentation (
    id integer NOT NULL,
    attribute_id integer,
    documentation_type character varying(50) NOT NULL,
    title character varying(200) NOT NULL,
    content text NOT NULL,
    content_format character varying(20) DEFAULT 'markdown'::character varying,
    tags text[],
    target_audience character varying(50),
    complexity_level character varying(20) DEFAULT 'intermediate'::character varying,
    last_reviewed timestamp without time zone,
    reviewer character varying(100),
    version character varying(20) DEFAULT '1.0'::character varying,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: attribute_documentation_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_documentation_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_documentation_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_documentation_id_seq OWNED BY public.attribute_documentation.id;


--
-- Name: attribute_domain_mappings_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_domain_mappings_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_domain_mappings_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_domain_mappings_id_seq OWNED BY public.attribute_domain_mappings.id;


--
-- Name: attribute_filter_configurations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_filter_configurations (
    id integer NOT NULL,
    filter_name character varying(100) NOT NULL,
    filter_description text,
    filter_type character varying(50) NOT NULL,
    filter_config jsonb NOT NULL,
    target_audience character varying(50),
    use_case character varying(100),
    performance_profile jsonb DEFAULT '{}'::jsonb,
    is_active boolean DEFAULT true,
    created_by character varying(100),
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE attribute_filter_configurations; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.attribute_filter_configurations IS 'Predefined filter configurations for different use cases';


--
-- Name: attribute_filter_configurations_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_filter_configurations_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_filter_configurations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_filter_configurations_id_seq OWNED BY public.attribute_filter_configurations.id;


--
-- Name: attribute_lineage; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_lineage (
    id integer NOT NULL,
    target_attribute_id integer,
    source_attribute_id integer,
    lineage_type character varying(50) NOT NULL,
    transformation_step integer DEFAULT 1,
    contribution_weight numeric(4,3) DEFAULT 1.0,
    lineage_description text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE attribute_lineage; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.attribute_lineage IS 'Data lineage tracking for understanding attribute dependencies';


--
-- Name: attribute_lineage_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_lineage_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_lineage_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_lineage_id_seq OWNED BY public.attribute_lineage.id;


--
-- Name: attribute_objects_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_objects_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_objects_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_objects_id_seq OWNED BY public.attribute_objects.id;


--
-- Name: attribute_persistence_mappings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_persistence_mappings (
    id integer NOT NULL,
    attribute_id integer,
    persistence_entity_id integer,
    field_mapping jsonb NOT NULL,
    transformation_rules jsonb DEFAULT '{}'::jsonb,
    validation_rules jsonb DEFAULT '{}'::jsonb,
    access_permissions jsonb DEFAULT '{}'::jsonb,
    caching_config jsonb DEFAULT '{}'::jsonb,
    sync_strategy character varying(50) DEFAULT 'immediate'::character varying,
    conflict_resolution character varying(50) DEFAULT 'last_write_wins'::character varying,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE attribute_persistence_mappings; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.attribute_persistence_mappings IS 'Maps attributes to their actual storage locations with transformation rules';


--
-- Name: attribute_persistence_mappings_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_persistence_mappings_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_persistence_mappings_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_persistence_mappings_id_seq OWNED BY public.attribute_persistence_mappings.id;


--
-- Name: attribute_perspectives; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_perspectives (
    id integer NOT NULL,
    attribute_id integer,
    perspective_name character varying(50) NOT NULL,
    description text,
    ui_group character varying(100),
    ui_label character varying(200),
    ui_help_text text,
    generation_examples jsonb DEFAULT '[]'::jsonb
);


--
-- Name: attribute_perspectives_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_perspectives_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_perspectives_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_perspectives_id_seq OWNED BY public.attribute_perspectives.id;


--
-- Name: attribute_relationships; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_relationships (
    id integer NOT NULL,
    source_attribute_id integer,
    target_attribute_id integer,
    relationship_type character varying(50) NOT NULL,
    relationship_config jsonb DEFAULT '{}'::jsonb,
    strength numeric(3,2) DEFAULT 1.0,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: attribute_relationships_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_relationships_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_relationships_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_relationships_id_seq OWNED BY public.attribute_relationships.id;


--
-- Name: attribute_semantic_relationships; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_semantic_relationships (
    id integer NOT NULL,
    source_attribute_id integer,
    target_attribute_id integer,
    relationship_type character varying(50) NOT NULL,
    relationship_description text NOT NULL,
    semantic_similarity numeric(4,3),
    context_specific boolean DEFAULT false,
    domain_context character varying(100),
    ai_confidence numeric(3,2) DEFAULT 1.0,
    human_verified boolean DEFAULT false,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: attribute_semantic_relationships_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_semantic_relationships_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_semantic_relationships_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_semantic_relationships_id_seq OWNED BY public.attribute_semantic_relationships.id;


--
-- Name: attribute_sources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_sources (
    id integer NOT NULL,
    source_key character varying(50) NOT NULL,
    name character varying(100) NOT NULL,
    description text,
    trust_level character varying(20),
    requires_validation boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT attribute_sources_trust_level_check CHECK (((trust_level)::text = ANY (ARRAY[('high'::character varying)::text, ('medium'::character varying)::text, ('low'::character varying)::text])))
);


--
-- Name: attribute_sources_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_sources_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_sources_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_sources_id_seq OWNED BY public.attribute_sources.id;


--
-- Name: attribute_tag_assignments_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_tag_assignments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_tag_assignments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_tag_assignments_id_seq OWNED BY public.attribute_tag_assignments.id;


--
-- Name: attribute_tags_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_tags_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_tags_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_tags_id_seq OWNED BY public.attribute_tags.id;


--
-- Name: attribute_terminology_links; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_terminology_links (
    id integer NOT NULL,
    attribute_id integer,
    term_id integer,
    relationship_type character varying(50) DEFAULT 'related'::character varying,
    importance_weight numeric(3,2) DEFAULT 1.0,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: attribute_terminology_links_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_terminology_links_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_terminology_links_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_terminology_links_id_seq OWNED BY public.attribute_terminology_links.id;


--
-- Name: attribute_value_audit; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attribute_value_audit (
    id integer NOT NULL,
    attribute_id integer,
    entity_instance_id character varying(255),
    old_value jsonb,
    new_value jsonb,
    change_type character varying(50),
    changed_by character varying(100),
    change_reason text,
    change_timestamp timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    persistence_entity_id integer,
    transaction_id character varying(100),
    metadata jsonb DEFAULT '{}'::jsonb
);


--
-- Name: TABLE attribute_value_audit; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.attribute_value_audit IS 'Audit trail for all attribute value changes across systems';


--
-- Name: attribute_value_audit_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_value_audit_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_value_audit_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_value_audit_id_seq OWNED BY public.attribute_value_audit.id;


--
-- Name: attribute_value_resolution_view; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.attribute_value_resolution_view AS
 SELECT ao.id AS attribute_id,
    ao.attribute_name,
    ao.data_type,
    ao.description,
    ps.system_name AS primary_system,
    ps.system_type,
    pe.entity_name AS primary_entity,
    pe.entity_config,
    apm.field_mapping,
    apm.transformation_rules,
    apm.sync_strategy,
    kod.domain_name,
    kod.regulatory_framework,
    adm.context_role,
    adm.compliance_criticality,
    ao.semantic_tags,
    ao.ai_context,
    ao.ui_component_type,
    ao.ui_layout_config,
    jsonb_build_object('persistence', jsonb_build_object('primary_system', ps.system_name, 'entity', pe.entity_name, 'field_mapping', apm.field_mapping, 'sync_strategy', apm.sync_strategy), 'domain', jsonb_build_object('name', kod.domain_name, 'compliance', adm.compliance_criticality, 'role', adm.context_role), 'ui', jsonb_build_object('component_type', ao.ui_component_type, 'layout_config', ao.ui_layout_config, 'styling', ao.ui_styling), 'ai', jsonb_build_object('context', ao.ai_context, 'semantic_tags', ao.semantic_tags, 'search_keywords', ao.search_keywords)) AS full_metadata
   FROM (((((public.attribute_objects ao
     LEFT JOIN public.persistence_entities pe ON ((pe.id = ao.primary_persistence_entity_id)))
     LEFT JOIN public.persistence_systems ps ON ((ps.id = pe.system_id)))
     LEFT JOIN public.attribute_persistence_mappings apm ON (((apm.attribute_id = ao.id) AND (apm.persistence_entity_id = pe.id))))
     LEFT JOIN public.attribute_domain_mappings adm ON ((adm.attribute_id = ao.id)))
     LEFT JOIN public.kyc_onboarding_domains kod ON ((kod.id = adm.domain_id)));


--
-- Name: attribute_vector_clusters_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.attribute_vector_clusters_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: attribute_vector_clusters_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.attribute_vector_clusters_id_seq OWNED BY public.attribute_vector_clusters.id;


--
-- Name: business_attributes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.business_attributes (
    id integer NOT NULL,
    entity_name character varying(100) NOT NULL,
    attribute_name character varying(100) NOT NULL,
    full_path character varying(200) GENERATED ALWAYS AS ((((entity_name)::text || '.'::text) || (attribute_name)::text)) STORED,
    data_type character varying(50) NOT NULL,
    sql_type character varying(100),
    rust_type character varying(100),
    format_mask character varying(100),
    validation_pattern text,
    domain_id integer,
    source_id integer,
    required boolean DEFAULT false,
    editable boolean DEFAULT true,
    min_value numeric,
    max_value numeric,
    min_length integer,
    max_length integer,
    description text,
    metadata jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    is_active boolean DEFAULT true
);


--
-- Name: business_attributes_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.business_attributes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: business_attributes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.business_attributes_id_seq OWNED BY public.business_attributes.id;


--
-- Name: cbu_entity_associations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.cbu_entity_associations (
    id integer NOT NULL,
    cbu_id integer,
    entity_id integer,
    association_type character varying(50) NOT NULL,
    role_in_cbu character varying(100),
    ownership_stake numeric(5,2),
    voting_rights numeric(5,2),
    control_level character varying(30),
    active_in_cbu boolean DEFAULT true,
    primary_contact boolean DEFAULT false,
    service_types text[],
    risk_contribution_score integer,
    compliance_responsibility character varying(100),
    reporting_entity boolean DEFAULT false,
    association_date date DEFAULT CURRENT_DATE,
    effective_from date,
    effective_to date,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    created_by character varying(100),
    updated_by character varying(100)
);


--
-- Name: TABLE cbu_entity_associations; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.cbu_entity_associations IS 'Many-to-many mapping between CBUs and their associated entities';


--
-- Name: cbu_entity_associations_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.cbu_entity_associations_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: cbu_entity_associations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.cbu_entity_associations_id_seq OWNED BY public.cbu_entity_associations.id;


--
-- Name: client_business_units; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.client_business_units (
    id integer NOT NULL,
    cbu_id character varying(100) NOT NULL,
    cbu_name character varying(255) NOT NULL,
    description text,
    primary_entity_id character varying(100),
    primary_lei character varying(20),
    domicile_country character(2),
    regulatory_jurisdiction character varying(50),
    business_type character varying(50),
    status character varying(20) DEFAULT 'active'::character varying,
    created_date date DEFAULT CURRENT_DATE,
    last_review_date date,
    next_review_date date,
    created_by character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by character varying(100),
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    metadata jsonb,
    CONSTRAINT client_business_units_status_check CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'inactive'::character varying, 'pending'::character varying, 'suspended'::character varying])::text[]))),
    CONSTRAINT valid_country CHECK (((domicile_country IS NULL) OR (domicile_country ~ '^[A-Z]{2}$'::text))),
    CONSTRAINT valid_lei CHECK (((primary_lei IS NULL) OR ((primary_lei)::text ~ '^[A-Z0-9]{20}$'::text)))
);


--
-- Name: TABLE client_business_units; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.client_business_units IS 'Main table for Client Business Units - logical groupings of related entities';


--
-- Name: COLUMN client_business_units.cbu_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.client_business_units.cbu_id IS 'External identifier for the CBU, used in APIs and other systems';


--
-- Name: COLUMN client_business_units.primary_lei; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.client_business_units.primary_lei IS 'LEI of the primary/controlling entity in this CBU';


--
-- Name: legal_entities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.legal_entities (
    id integer NOT NULL,
    entity_id character varying(50) NOT NULL,
    lei_code character varying(20),
    entity_name character varying(500) NOT NULL,
    entity_type character varying(50) NOT NULL,
    entity_subtype character varying(100),
    legal_form character varying(100),
    incorporation_country character(2) NOT NULL,
    incorporation_jurisdiction character varying(100),
    registration_number character varying(100),
    registration_authority character varying(200),
    incorporation_date date,
    regulatory_structure character varying(100),
    regulatory_status character varying(50) DEFAULT 'active'::character varying,
    regulated_entity boolean DEFAULT false,
    regulatory_authorities text[],
    business_purpose text,
    primary_business_activity character varying(200),
    sic_codes character varying(50)[],
    nace_codes character varying(50)[],
    tax_residence_country character(2),
    tax_identification_number character varying(100),
    vat_number character varying(50),
    operational_currency character(3),
    fiscal_year_end date,
    registered_address jsonb,
    operational_address jsonb,
    mailing_address jsonb,
    authorized_signatories jsonb[],
    board_members jsonb[],
    risk_rating character varying(20) DEFAULT 'medium'::character varying,
    kyc_status character varying(30) DEFAULT 'pending'::character varying,
    sanctions_screening_status character varying(30) DEFAULT 'pending'::character varying,
    sanctions_screening_date timestamp without time zone,
    sanctions_screening_result character varying(20),
    has_complex_ownership boolean DEFAULT false,
    ownership_transparency_score integer,
    share_capital numeric(20,2),
    share_capital_currency character(3),
    public_company boolean DEFAULT false,
    listed_exchange character varying(100),
    status character varying(30) DEFAULT 'active'::character varying,
    dissolution_date date,
    data_sources text[],
    last_verified_date timestamp without time zone,
    verification_method character varying(100),
    data_quality_score integer,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    created_by character varying(100),
    updated_by character varying(100),
    CONSTRAINT legal_entities_data_quality_score_check CHECK (((data_quality_score >= 0) AND (data_quality_score <= 100))),
    CONSTRAINT legal_entities_ownership_transparency_score_check CHECK (((ownership_transparency_score >= 0) AND (ownership_transparency_score <= 100))),
    CONSTRAINT legal_entities_status_check CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'inactive'::character varying, 'dormant'::character varying, 'dissolved'::character varying, 'in_liquidation'::character varying])::text[])))
);


--
-- Name: TABLE legal_entities; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.legal_entities IS 'Comprehensive registry of all legal entities including corporations, SPVs, partnerships, funds, and regulatory structures';


--
-- Name: cbu_entity_structure_view; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.cbu_entity_structure_view AS
 SELECT cbu.id AS cbu_internal_id,
    cbu.cbu_id,
    cbu.cbu_name,
    cbu.description AS cbu_description,
    cbu.business_type AS cbu_business_type,
    cbu.status AS cbu_status,
    pe.entity_id AS primary_entity_id,
    pe.entity_name AS primary_entity_name,
    pe.entity_type AS primary_entity_type,
    pe.legal_form AS primary_legal_form,
    pe.incorporation_country AS primary_country,
    pe.lei_code AS primary_lei,
    jsonb_agg(DISTINCT jsonb_build_object('entity_id', le.entity_id, 'entity_name', le.entity_name, 'entity_type', le.entity_type, 'legal_form', le.legal_form, 'association_type', cea.association_type, 'role_in_cbu', cea.role_in_cbu, 'ownership_stake', cea.ownership_stake, 'control_level', cea.control_level, 'active', cea.active_in_cbu, 'primary_contact', cea.primary_contact, 'lei_code', le.lei_code, 'incorporation_country', le.incorporation_country, 'risk_rating', le.risk_rating, 'kyc_status', le.kyc_status)) FILTER (WHERE (le.id IS NOT NULL)) AS associated_entities,
    count(DISTINCT lea.id) AS total_entities,
    count(DISTINCT
        CASE
            WHEN cea.active_in_cbu THEN lea.id
            ELSE NULL::integer
        END) AS active_entities,
    count(DISTINCT
        CASE
            WHEN ((le.entity_type)::text = 'spv'::text) THEN lea.id
            ELSE NULL::integer
        END) AS spv_count,
    count(DISTINCT
        CASE
            WHEN ((le.entity_type)::text = 'corporation'::text) THEN lea.id
            ELSE NULL::integer
        END) AS corporation_count,
    count(DISTINCT
        CASE
            WHEN ((le.entity_type)::text = 'partnership'::text) THEN lea.id
            ELSE NULL::integer
        END) AS partnership_count,
    count(DISTINCT
        CASE
            WHEN (le.regulatory_structure IS NOT NULL) THEN lea.id
            ELSE NULL::integer
        END) AS regulated_entities,
    avg(((le.risk_rating)::text)::integer) FILTER (WHERE ((le.risk_rating)::text ~ '^[0-9]+$'::text)) AS avg_entity_risk,
    array_agg(DISTINCT le.incorporation_country) FILTER (WHERE (le.incorporation_country IS NOT NULL)) AS jurisdictions,
    cbu.created_at AS cbu_created_at,
    cbu.updated_at AS cbu_updated_at
   FROM ((((public.client_business_units cbu
     LEFT JOIN public.legal_entities pe ON (((pe.entity_id)::text = (cbu.primary_entity_id)::text)))
     LEFT JOIN public.cbu_entity_associations cea ON ((cea.cbu_id = cbu.id)))
     LEFT JOIN public.legal_entities lea ON ((lea.id = cea.entity_id)))
     LEFT JOIN public.legal_entities le ON ((le.id = cea.entity_id)))
  GROUP BY cbu.id, cbu.cbu_id, cbu.cbu_name, cbu.description, cbu.business_type, cbu.status, pe.entity_id, pe.entity_name, pe.entity_type, pe.legal_form, pe.incorporation_country, pe.lei_code, cbu.created_at, cbu.updated_at;


--
-- Name: VIEW cbu_entity_structure_view; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON VIEW public.cbu_entity_structure_view IS 'Comprehensive view of CBU structure with all associated entities and metadata';


--
-- Name: investment_mandates; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.investment_mandates (
    mandate_id character varying(100) NOT NULL,
    cbu_id character varying(100),
    asset_owner_name character varying(255) NOT NULL,
    asset_owner_lei character varying(20) NOT NULL,
    investment_manager_name character varying(255) NOT NULL,
    investment_manager_lei character varying(20) NOT NULL,
    base_currency character(3) NOT NULL,
    effective_date date NOT NULL,
    expiry_date date,
    gross_exposure_pct numeric(6,3),
    net_exposure_pct numeric(6,3),
    leverage_max numeric(10,3),
    issuer_concentration_pct numeric(6,3),
    country_concentration_pct numeric(6,3),
    sector_concentration_pct numeric(6,3),
    duration_min numeric(10,3),
    duration_max numeric(10,3),
    var_limit numeric(15,3),
    dv01_limit numeric(15,3),
    pre_trade_checks_required boolean DEFAULT false,
    maker_checker boolean DEFAULT false,
    stp_required boolean DEFAULT false,
    breach_handling character varying(100),
    intraday_status character varying(20),
    end_of_day_blotter boolean DEFAULT false,
    confirmations_required boolean DEFAULT false,
    matching_model character varying(30),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT investment_mandates_base_currency_check CHECK ((base_currency ~ '^[A-Z]{3}$'::text)),
    CONSTRAINT investment_mandates_check CHECK (((expiry_date IS NULL) OR (expiry_date > effective_date))),
    CONSTRAINT investment_mandates_intraday_status_check CHECK (((intraday_status)::text = ANY ((ARRAY['none'::character varying, 'hourly'::character varying, 'real_time'::character varying])::text[]))),
    CONSTRAINT investment_mandates_matching_model_check CHECK (((matching_model)::text = ANY ((ARRAY['affirmation'::character varying, 'confirmation'::character varying, 'central_matching'::character varying])::text[])))
);


--
-- Name: TABLE investment_mandates; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.investment_mandates IS 'Investment mandates defining objectives, constraints, and policies for portfolio management';


--
-- Name: mandate_instruments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mandate_instruments (
    id integer NOT NULL,
    mandate_id character varying(100) NOT NULL,
    instrument_family character varying(30) NOT NULL,
    subtype character varying(100),
    cfi_code character varying(10),
    isda_taxonomy character varying(100),
    order_types text[],
    time_in_force text[],
    min_clip numeric(15,3),
    algo_flags_allowed boolean DEFAULT false,
    settlement_type character varying(10),
    settlement_cycle character varying(20),
    place_of_settlement character varying(100),
    allow_partials boolean DEFAULT false,
    ssi_reference character varying(100),
    clearing_required boolean DEFAULT false,
    clearing_house character varying(100),
    margin_model character varying(20),
    eligible_collateral_schedule character varying(100),
    min_tenor character varying(20),
    max_tenor character varying(20),
    exposure_pct numeric(6,3),
    short_allowed boolean DEFAULT false,
    issuer_max_pct numeric(6,3),
    rating_floor character varying(10),
    limit_duration_min numeric(10,3),
    limit_duration_max numeric(10,3),
    dv01_cap numeric(15,3),
    counterparties_whitelist text[],
    allocation_model character varying(20),
    notes text,
    CONSTRAINT mandate_instruments_allocation_model_check CHECK (((allocation_model)::text = ANY ((ARRAY['pre_trade'::character varying, 'post_trade'::character varying, 'either'::character varying])::text[]))),
    CONSTRAINT mandate_instruments_instrument_family_check CHECK (((instrument_family)::text = ANY ((ARRAY['equity'::character varying, 'fixed_income'::character varying, 'money_market'::character varying, 'fund'::character varying, 'fx'::character varying, 'commodity'::character varying, 'derivative_otc'::character varying, 'derivative_etd'::character varying, 'securities_financing'::character varying, 'cash'::character varying])::text[]))),
    CONSTRAINT mandate_instruments_margin_model_check CHECK (((margin_model)::text = ANY ((ARRAY['Bilateral'::character varying, 'CCP_IM'::character varying, 'SIMM'::character varying])::text[]))),
    CONSTRAINT mandate_instruments_settlement_type_check CHECK (((settlement_type)::text = ANY ((ARRAY['DVP'::character varying, 'RVP'::character varying, 'FOP'::character varying, 'PvP'::character varying, 'Cash'::character varying])::text[])))
);


--
-- Name: TABLE mandate_instruments; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.mandate_instruments IS 'Instrument-specific policies and limits for each mandate';


--
-- Name: cbu_investment_mandate_structure; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.cbu_investment_mandate_structure AS
 SELECT cbu.cbu_id,
    cbu.cbu_name,
    cbu.business_type,
    cbu.description AS cbu_description,
    im.mandate_id,
    im.asset_owner_name,
    im.asset_owner_lei,
    im.investment_manager_name,
    im.investment_manager_lei,
    im.base_currency,
    im.effective_date,
    im.expiry_date,
    im.gross_exposure_pct,
    im.net_exposure_pct,
    im.leverage_max,
    im.issuer_concentration_pct,
    im.country_concentration_pct,
    im.sector_concentration_pct,
    count(DISTINCT mi.id) AS total_instruments,
    count(DISTINCT mi.instrument_family) AS instrument_families,
    sum(mi.exposure_pct) AS total_exposure_pct,
    string_agg(DISTINCT (mi.instrument_family)::text, ', '::text ORDER BY (mi.instrument_family)::text) AS families,
    im.pre_trade_checks_required,
    im.maker_checker,
    im.stp_required,
    im.breach_handling,
    im.intraday_status,
    im.matching_model,
    avg(
        CASE
            WHEN ((mi.rating_floor)::text = 'AAA'::text) THEN 1
            WHEN ((mi.rating_floor)::text = 'AA'::text) THEN 2
            WHEN ((mi.rating_floor)::text = 'A'::text) THEN 3
            WHEN ((mi.rating_floor)::text = 'BBB'::text) THEN 4
            ELSE 5
        END) AS avg_rating_numeric,
    count(
        CASE
            WHEN mi.short_allowed THEN 1
            ELSE NULL::integer
        END) AS instruments_allow_short,
    im.created_at AS mandate_created_at,
    im.updated_at AS mandate_updated_at
   FROM ((public.client_business_units cbu
     LEFT JOIN public.investment_mandates im ON (((im.cbu_id)::text = (cbu.cbu_id)::text)))
     LEFT JOIN public.mandate_instruments mi ON (((mi.mandate_id)::text = (im.mandate_id)::text)))
  GROUP BY cbu.cbu_id, cbu.cbu_name, cbu.business_type, cbu.description, im.mandate_id, im.asset_owner_name, im.asset_owner_lei, im.investment_manager_name, im.investment_manager_lei, im.base_currency, im.effective_date, im.expiry_date, im.gross_exposure_pct, im.net_exposure_pct, im.leverage_max, im.issuer_concentration_pct, im.country_concentration_pct, im.sector_concentration_pct, im.pre_trade_checks_required, im.maker_checker, im.stp_required, im.breach_handling, im.intraday_status, im.matching_model, im.created_at, im.updated_at;


--
-- Name: cbu_members; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.cbu_members (
    id integer NOT NULL,
    cbu_id integer NOT NULL,
    role_id integer NOT NULL,
    entity_id character varying(100) NOT NULL,
    entity_name character varying(255) NOT NULL,
    entity_lei character varying(20),
    is_primary boolean DEFAULT false,
    effective_date date DEFAULT CURRENT_DATE,
    expiry_date date,
    contact_email character varying(255),
    contact_phone character varying(50),
    authorized_persons jsonb,
    is_active boolean DEFAULT true,
    receives_notifications boolean DEFAULT true,
    has_trading_authority boolean DEFAULT false,
    has_settlement_authority boolean DEFAULT false,
    created_by character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by character varying(100),
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    notes text,
    metadata jsonb,
    CONSTRAINT valid_dates CHECK (((expiry_date IS NULL) OR (expiry_date > effective_date))),
    CONSTRAINT valid_email CHECK (((contact_email IS NULL) OR ((contact_email)::text ~ '^[^@]+@[^@]+\.[^@]+$'::text))),
    CONSTRAINT valid_member_lei CHECK (((entity_lei IS NULL) OR ((entity_lei)::text ~ '^[A-Z0-9]{20}$'::text)))
);


--
-- Name: TABLE cbu_members; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.cbu_members IS 'Links entities to CBUs with specific roles and relationship details';


--
-- Name: COLUMN cbu_members.is_primary; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.cbu_members.is_primary IS 'Indicates if this is the primary entity for this role within the CBU';


--
-- Name: COLUMN cbu_members.authorized_persons; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.cbu_members.authorized_persons IS 'JSON array of authorized contact persons with names, emails, roles';


--
-- Name: cbu_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.cbu_roles (
    id integer NOT NULL,
    role_code character varying(50) NOT NULL,
    role_name character varying(100) NOT NULL,
    description text,
    role_category character varying(50),
    display_order integer DEFAULT 999,
    is_active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE cbu_roles; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.cbu_roles IS 'Role taxonomy for CBU members - defines available roles like Asset Owner, Investment Manager, etc.';


--
-- Name: cbu_member_investment_roles; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.cbu_member_investment_roles AS
 SELECT cbu.cbu_id,
    cbu.cbu_name,
    cm.entity_name,
    cm.entity_lei,
    cr.role_name,
    cr.role_code,
    cm.has_trading_authority,
    cm.has_settlement_authority,
    im.mandate_id,
        CASE
            WHEN ((cr.role_code)::text = 'ASSET_OWNER'::text) THEN 'Mandate Owner & Capital Provider'::text
            WHEN ((cr.role_code)::text = 'INVESTMENT_MANAGER'::text) THEN 'Portfolio Management & Trading'::text
            WHEN ((cr.role_code)::text = 'CUSTODIAN'::text) THEN 'Asset Safekeeping & Settlement'::text
            WHEN ((cr.role_code)::text = 'ADMINISTRATOR'::text) THEN 'Fund Administration & Reporting'::text
            WHEN ((cr.role_code)::text = 'PROCESSOR'::text) THEN 'Payment Processing & Liquidity'::text
            WHEN ((cr.role_code)::text = 'COMPLIANCE_OFFICER'::text) THEN 'Compliance Monitoring & Risk'::text
            ELSE 'Other Investment Role'::text
        END AS investment_responsibility,
        CASE
            WHEN ((im.asset_owner_lei)::text = (cm.entity_lei)::text) THEN im.base_currency
            ELSE NULL::bpchar
        END AS mandate_currency,
        CASE
            WHEN ((im.asset_owner_lei)::text = (cm.entity_lei)::text) THEN im.leverage_max
            ELSE NULL::numeric
        END AS leverage_limit,
        CASE
            WHEN ((im.investment_manager_lei)::text = (cm.entity_lei)::text) THEN concat(im.gross_exposure_pct, '% gross / ', im.net_exposure_pct, '% net exposure')
            ELSE NULL::text
        END AS exposure_limits,
    cm.is_primary,
    cm.effective_date AS member_effective_date,
    cm.notes AS member_notes
   FROM (((public.client_business_units cbu
     JOIN public.cbu_members cm ON ((cm.cbu_id = cbu.id)))
     JOIN public.cbu_roles cr ON ((cr.id = cm.role_id)))
     LEFT JOIN public.investment_mandates im ON ((((im.cbu_id)::text = (cbu.cbu_id)::text) AND (((im.asset_owner_lei)::text = (cm.entity_lei)::text) OR ((im.investment_manager_lei)::text = (cm.entity_lei)::text)))))
  ORDER BY cbu.cbu_id, cm.is_primary DESC, cr.role_code;


--
-- Name: cbu_members_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.cbu_members_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: cbu_members_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.cbu_members_id_seq OWNED BY public.cbu_members.id;


--
-- Name: cbu_product_subscriptions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.cbu_product_subscriptions (
    id integer NOT NULL,
    cbu_id integer,
    product_id integer,
    subscription_status character varying(20) DEFAULT 'pending'::character varying,
    subscription_date timestamp with time zone,
    activation_date timestamp with time zone,
    termination_date timestamp with time zone,
    billing_arrangement jsonb,
    contract_reference character varying(100),
    primary_contact_role_id integer,
    created_by character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by character varying(100),
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT cbu_product_subscriptions_subscription_status_check CHECK (((subscription_status)::text = ANY ((ARRAY['pending'::character varying, 'active'::character varying, 'suspended'::character varying, 'terminated'::character varying])::text[])))
);


--
-- Name: cbu_product_subscriptions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.cbu_product_subscriptions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: cbu_product_subscriptions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.cbu_product_subscriptions_id_seq OWNED BY public.cbu_product_subscriptions.id;


--
-- Name: cbu_roles_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.cbu_roles_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: cbu_roles_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.cbu_roles_id_seq OWNED BY public.cbu_roles.id;


--
-- Name: cbu_service_resources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.cbu_service_resources (
    id integer NOT NULL,
    cbu_id integer,
    service_id integer,
    resource_id integer,
    configuration_status character varying(20) DEFAULT 'not_configured'::character varying,
    configuration_details jsonb,
    go_live_date date,
    last_health_check timestamp with time zone,
    health_status character varying(20) DEFAULT 'unknown'::character varying,
    responsible_team character varying(100),
    onboarding_request_id integer,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT cbu_service_resources_configuration_status_check CHECK (((configuration_status)::text = ANY ((ARRAY['not_configured'::character varying, 'in_progress'::character varying, 'configured'::character varying, 'testing'::character varying, 'active'::character varying, 'inactive'::character varying, 'error'::character varying])::text[]))),
    CONSTRAINT cbu_service_resources_health_status_check CHECK (((health_status)::text = ANY ((ARRAY['healthy'::character varying, 'warning'::character varying, 'error'::character varying, 'unknown'::character varying])::text[])))
);


--
-- Name: cbu_service_resources_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.cbu_service_resources_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: cbu_service_resources_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.cbu_service_resources_id_seq OWNED BY public.cbu_service_resources.id;


--
-- Name: client_business_units_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.client_business_units_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: client_business_units_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.client_business_units_id_seq OWNED BY public.client_business_units.id;


--
-- Name: commercial_contracts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.commercial_contracts (
    id integer NOT NULL,
    contract_id character varying(100) NOT NULL,
    customer_entity_id character varying(100),
    product_id integer,
    contract_type character varying(50),
    contract_status character varying(50) DEFAULT 'active'::character varying,
    contract_value numeric(15,2),
    currency character(3),
    payment_terms character varying(100),
    billing_frequency character varying(50),
    contract_start_date date,
    contract_end_date date,
    renewal_date date,
    termination_notice_period integer,
    included_services jsonb,
    service_configurations jsonb,
    sla_commitments jsonb,
    governing_law character varying(100),
    dispute_resolution character varying(100),
    liability_cap numeric(15,2),
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    created_by character varying(100),
    updated_by character varying(100)
);


--
-- Name: TABLE commercial_contracts; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.commercial_contracts IS 'Tracks commercial contracts for sold products with terms and service configurations';


--
-- Name: commercial_contracts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.commercial_contracts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: commercial_contracts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.commercial_contracts_id_seq OWNED BY public.commercial_contracts.id;


--
-- Name: product_service_mappings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.product_service_mappings (
    id integer NOT NULL,
    product_id integer,
    service_id integer,
    mapping_type character varying(50) NOT NULL,
    inclusion_criteria jsonb,
    pricing_impact numeric(10,2),
    delivery_sequence integer,
    is_mandatory boolean DEFAULT false,
    customer_configurable boolean DEFAULT true,
    effective_date date DEFAULT CURRENT_DATE,
    end_date date,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE product_service_mappings; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.product_service_mappings IS 'Links products to the services they require';


--
-- Name: products; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.products (
    id integer NOT NULL,
    product_id character varying(100) NOT NULL,
    product_name character varying(255) NOT NULL,
    line_of_business character varying(100) NOT NULL,
    description text,
    status character varying(20) DEFAULT 'active'::character varying,
    pricing_model character varying(50),
    target_market character varying(100),
    regulatory_requirements jsonb,
    sla_commitments jsonb,
    created_by character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by character varying(100),
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    contract_type character varying(100),
    commercial_status character varying(50) DEFAULT 'active'::character varying,
    sales_territory character varying(100),
    compliance_requirements text[],
    contract_terms jsonb,
    minimum_contract_value numeric(15,2),
    maximum_contract_value numeric(15,2),
    standard_contract_duration integer,
    renewable boolean DEFAULT true,
    early_termination_allowed boolean DEFAULT false,
    CONSTRAINT products_status_check CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'inactive'::character varying, 'deprecated'::character varying])::text[])))
);


--
-- Name: TABLE products; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.products IS 'Commercial custody banking products sold to financial institutions';


--
-- Name: COLUMN products.pricing_model; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.products.pricing_model IS 'How this product is priced and sold';


--
-- Name: COLUMN products.contract_type; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.products.contract_type IS 'Type of commercial contract for this product';


--
-- Name: COLUMN products.commercial_status; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.products.commercial_status IS 'Current commercial availability status';


--
-- Name: COLUMN products.contract_terms; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.products.contract_terms IS 'Standard contract terms, warranties, and conditions';


--
-- Name: resource_objects; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_objects (
    id integer NOT NULL,
    dictionary_id integer,
    resource_name character varying(100) NOT NULL,
    description text,
    version character varying(20) DEFAULT '1.0'::character varying NOT NULL,
    category character varying(50),
    owner_team character varying(100),
    status character varying(20) DEFAULT 'active'::character varying,
    ui_layout character varying(30) DEFAULT 'vertical-stack'::character varying NOT NULL,
    group_order text[],
    navigation_config jsonb DEFAULT '{}'::jsonb,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    resource_type character varying(100),
    criticality_level character varying(50) DEFAULT 'medium'::character varying,
    operational_status character varying(50) DEFAULT 'active'::character varying,
    access_restrictions jsonb,
    compliance_classification character varying(100),
    data_retention_policy jsonb,
    backup_requirements jsonb,
    monitoring_enabled boolean DEFAULT true,
    audit_required boolean DEFAULT false,
    CONSTRAINT resource_objects_status_check CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'inactive'::character varying, 'deprecated'::character varying])::text[]))),
    CONSTRAINT resource_objects_ui_layout_check CHECK (((ui_layout)::text = ANY ((ARRAY['wizard'::character varying, 'tabs'::character varying, 'vertical-stack'::character varying, 'horizontal-grid'::character varying, 'accordion'::character varying])::text[])))
);


--
-- Name: service_resource_mappings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.service_resource_mappings (
    id integer NOT NULL,
    service_id integer,
    resource_id integer,
    usage_type character varying(50) NOT NULL,
    resource_role character varying(100),
    configuration_parameters jsonb,
    performance_requirements jsonb,
    usage_limits jsonb,
    cost_allocation_percentage numeric(5,2),
    dependency_level integer DEFAULT 1,
    failover_resource_id integer,
    monitoring_thresholds jsonb,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE service_resource_mappings; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.service_resource_mappings IS 'Links services to the resources that implement them';


--
-- Name: services; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.services (
    id integer NOT NULL,
    service_id character varying(100) NOT NULL,
    service_name character varying(255) NOT NULL,
    service_category character varying(100),
    description text,
    is_core_service boolean DEFAULT false,
    configuration_schema jsonb,
    dependencies text[],
    status character varying(20) DEFAULT 'active'::character varying,
    created_by character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by character varying(100),
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    service_type character varying(100),
    delivery_model character varying(50),
    sla_requirements jsonb,
    billable boolean DEFAULT true,
    recurring_service boolean DEFAULT false,
    service_dependencies text[],
    skill_requirements text[],
    automation_level character varying(50),
    customer_facing boolean DEFAULT true,
    CONSTRAINT services_status_check CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'inactive'::character varying, 'development'::character varying])::text[])))
);


--
-- Name: TABLE services; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.services IS 'Virtual/logical services that collectively define products';


--
-- Name: COLUMN services.service_type; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.services.service_type IS 'Type of generic public financial service (custody, safekeeping, reconciliation, fund_accounting, middle_office, trade_order_management)';


--
-- Name: COLUMN services.delivery_model; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.services.delivery_model IS 'How this financial service is delivered to customers';


--
-- Name: COLUMN services.sla_requirements; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.services.sla_requirements IS 'Service level agreement requirements and metrics';


--
-- Name: commercial_taxonomy_view; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.commercial_taxonomy_view AS
 SELECT p.id AS product_id,
    p.product_name,
    p.description AS product_description,
    p.contract_type,
    p.commercial_status,
    p.pricing_model,
    p.target_market,
    s.id AS service_id,
    s.service_name,
    s.description AS service_description,
    s.service_type,
    s.delivery_model,
    s.billable AS service_billable,
    r.id AS resource_id,
    r.resource_name,
    r.description AS resource_description,
    r.resource_type,
    r.criticality_level,
    r.operational_status,
    count(DISTINCT a.id) AS attribute_count,
    array_agg(DISTINCT a.attribute_name ORDER BY a.attribute_name) FILTER (WHERE (a.id IS NOT NULL)) AS attributes,
    psm.mapping_type AS service_inclusion_type,
    psm.is_mandatory AS service_mandatory,
    srm.usage_type AS resource_usage_type,
    srm.dependency_level AS resource_dependency,
    count(DISTINCT cc.id) AS active_contracts,
    avg(cc.contract_value) AS avg_contract_value,
    p.compliance_requirements,
    r.compliance_classification,
    r.audit_required,
    count(DISTINCT
        CASE
            WHEN ((r.operational_status)::text = 'active'::text) THEN r.id
            ELSE NULL::integer
        END) AS active_resources,
    count(DISTINCT
        CASE
            WHEN ((s.service_type)::text = 'core'::text) THEN s.id
            ELSE NULL::integer
        END) AS core_services,
    p.created_at AS product_created_at,
    p.updated_at AS product_updated_at
   FROM ((((((public.products p
     LEFT JOIN public.product_service_mappings psm ON ((psm.product_id = p.id)))
     LEFT JOIN public.services s ON ((s.id = psm.service_id)))
     LEFT JOIN public.service_resource_mappings srm ON ((srm.service_id = s.id)))
     LEFT JOIN public.resource_objects r ON ((r.id = srm.resource_id)))
     LEFT JOIN public.attribute_objects a ON ((a.resource_id = r.id)))
     LEFT JOIN public.commercial_contracts cc ON (((cc.product_id = p.id) AND ((cc.contract_status)::text = 'active'::text))))
  GROUP BY p.id, p.product_name, p.description, p.contract_type, p.commercial_status, p.pricing_model, p.target_market, p.compliance_requirements, p.created_at, p.updated_at, s.id, s.service_name, s.description, s.service_type, s.delivery_model, s.billable, r.id, r.resource_name, r.description, r.resource_type, r.criticality_level, r.operational_status, r.compliance_classification, r.audit_required, psm.mapping_type, psm.is_mandatory, srm.usage_type, srm.dependency_level;


--
-- Name: VIEW commercial_taxonomy_view; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON VIEW public.commercial_taxonomy_view IS 'Comprehensive view of Products→Services→Resources hierarchy with commercial context';


--
-- Name: data_domains; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.data_domains (
    id integer NOT NULL,
    domain_name character varying(100) NOT NULL,
    "values" jsonb NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: data_domains_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.data_domains_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: data_domains_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.data_domains_id_seq OWNED BY public.data_domains.id;


--
-- Name: derivation_execution_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.derivation_execution_log (
    id integer NOT NULL,
    derived_attribute_id integer,
    execution_id uuid DEFAULT gen_random_uuid(),
    input_values jsonb NOT NULL,
    output_value jsonb,
    execution_time_ms integer,
    success boolean NOT NULL,
    error_message text,
    rule_version character varying(20),
    executed_by character varying(100),
    execution_context jsonb DEFAULT '{}'::jsonb,
    execution_timestamp timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE derivation_execution_log; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.derivation_execution_log IS 'Audit log of all derivation rule executions';


--
-- Name: derivation_execution_log_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.derivation_execution_log_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: derivation_execution_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.derivation_execution_log_id_seq OWNED BY public.derivation_execution_log.id;


--
-- Name: derived_attribute_quality_metrics_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.derived_attribute_quality_metrics_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: derived_attribute_quality_metrics_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.derived_attribute_quality_metrics_id_seq OWNED BY public.derived_attribute_quality_metrics.id;


--
-- Name: derived_attribute_rules_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.derived_attribute_rules_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: derived_attribute_rules_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.derived_attribute_rules_id_seq OWNED BY public.derived_attribute_rules.id;


--
-- Name: derived_attributes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.derived_attributes (
    id integer NOT NULL,
    entity_name character varying(100) NOT NULL,
    attribute_name character varying(100) NOT NULL,
    full_path character varying(200) GENERATED ALWAYS AS ((((entity_name)::text || '.'::text) || (attribute_name)::text)) STORED,
    data_type character varying(50) NOT NULL,
    sql_type character varying(100),
    rust_type character varying(100),
    domain_id integer,
    description text,
    metadata jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: derived_attributes_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.derived_attributes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: derived_attributes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.derived_attributes_id_seq OWNED BY public.derived_attributes.id;


--
-- Name: domain_terminology_glossary; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.domain_terminology_glossary (
    id integer NOT NULL,
    term character varying(200) NOT NULL,
    definition text NOT NULL,
    domain_context character varying(100),
    synonyms text[],
    related_terms text[],
    usage_examples text[],
    regulatory_significance text,
    common_misconceptions text,
    ai_context_notes text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE domain_terminology_glossary; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.domain_terminology_glossary IS 'Comprehensive glossary for AI semantic understanding of domain terms';


--
-- Name: domain_terminology_glossary_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.domain_terminology_glossary_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: domain_terminology_glossary_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.domain_terminology_glossary_id_seq OWNED BY public.domain_terminology_glossary.id;


--
-- Name: dsl_execution_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.dsl_execution_logs (
    execution_id text DEFAULT (gen_random_uuid())::text NOT NULL,
    instance_id text NOT NULL,
    execution_status text NOT NULL,
    input_data jsonb DEFAULT '{}'::jsonb,
    output_data jsonb DEFAULT '{}'::jsonb,
    log_messages text[],
    error_details text,
    execution_time_ms double precision,
    executed_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT dsl_execution_logs_execution_status_check CHECK ((execution_status = ANY (ARRAY['success'::text, 'failed'::text, 'partial'::text])))
);


--
-- Name: ebnf_grammar_templates; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ebnf_grammar_templates (
    id integer NOT NULL,
    template_name character varying(100) NOT NULL,
    template_description text,
    ebnf_pattern text NOT NULL,
    parameter_placeholders jsonb DEFAULT '{}'::jsonb,
    use_cases text[],
    complexity_level character varying(20) DEFAULT 'simple'::character varying,
    example_usage text,
    documentation_url text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE ebnf_grammar_templates; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.ebnf_grammar_templates IS 'Reusable EBNF grammar patterns for common derivation scenarios';


--
-- Name: ebnf_grammar_templates_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.ebnf_grammar_templates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ebnf_grammar_templates_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.ebnf_grammar_templates_id_seq OWNED BY public.ebnf_grammar_templates.id;


--
-- Name: enhanced_attributes_view; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.enhanced_attributes_view AS
SELECT
    NULL::integer AS id,
    NULL::integer AS resource_id,
    NULL::character varying(100) AS attribute_name,
    NULL::character varying(30) AS data_type,
    NULL::text AS description,
    NULL::boolean AS is_required,
    NULL::integer AS min_length,
    NULL::integer AS max_length,
    NULL::numeric AS min_value,
    NULL::numeric AS max_value,
    NULL::jsonb AS allowed_values,
    NULL::text AS validation_pattern,
    NULL::character varying(100) AS persistence_system,
    NULL::character varying(100) AS persistence_entity,
    NULL::character varying(100) AS persistence_identifier,
    NULL::character varying(100) AS ui_group,
    NULL::integer AS ui_display_order,
    NULL::character varying(30) AS ui_render_hint,
    NULL::character varying(200) AS ui_label,
    NULL::text AS ui_help_text,
    NULL::integer AS wizard_step,
    NULL::character varying(200) AS wizard_step_title,
    NULL::text AS wizard_next_button,
    NULL::text AS wizard_previous_button,
    NULL::text AS wizard_description,
    NULL::jsonb AS generation_examples,
    NULL::text AS rules_dsl,
    NULL::timestamp without time zone AS created_at,
    NULL::timestamp without time zone AS updated_at,
    NULL::jsonb AS semantic_tags,
    NULL::jsonb AS ai_context,
    NULL::public.vector(1536) AS embedding_vector,
    NULL::text[] AS search_keywords,
    NULL::character varying(50) AS ui_component_type,
    NULL::jsonb AS ui_layout_config,
    NULL::jsonb AS ui_styling,
    NULL::jsonb AS ui_behavior,
    NULL::jsonb AS conditional_logic,
    NULL::jsonb AS relationship_metadata,
    NULL::jsonb AS ai_prompt_templates,
    NULL::jsonb AS form_generation_rules,
    NULL::jsonb AS accessibility_config,
    NULL::jsonb AS responsive_config,
    NULL::jsonb AS data_flow_config,
    NULL::character varying(100) AS context_name,
    NULL::text AS prompt_template,
    NULL::jsonb AS ui_template_config,
    NULL::jsonb AS styling_defaults,
    NULL::jsonb AS behavior_defaults,
    NULL::character varying[] AS outgoing_relationships,
    NULL::character varying[] AS incoming_relationships,
    NULL::bigint AS perspective_count;


--
-- Name: product_option_service_mappings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.product_option_service_mappings (
    id integer NOT NULL,
    product_option_id integer,
    service_id integer,
    mapping_relationship character varying(50) NOT NULL,
    service_configuration jsonb,
    sla_modifications jsonb,
    additional_resources jsonb,
    resource_scaling_factor numeric(4,2) DEFAULT 1.00,
    execution_priority integer DEFAULT 10,
    dependency_level integer DEFAULT 1,
    is_active boolean DEFAULT true,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE product_option_service_mappings; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.product_option_service_mappings IS 'Maps product options to the services they affect, configure, or require';


--
-- Name: product_options; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.product_options (
    id integer NOT NULL,
    option_id character varying(100) NOT NULL,
    product_id integer,
    option_name character varying(255) NOT NULL,
    option_category character varying(100) NOT NULL,
    option_type character varying(50) NOT NULL,
    option_value jsonb NOT NULL,
    display_name character varying(255),
    description text,
    pricing_impact numeric(10,2) DEFAULT 0.00,
    pricing_model character varying(50),
    minimum_commitment numeric(10,2),
    available_markets text[],
    regulatory_approval_required boolean DEFAULT false,
    compliance_requirements text[],
    prerequisite_options integer[],
    mutually_exclusive_options integer[],
    implementation_complexity character varying(20) DEFAULT 'medium'::character varying,
    lead_time_days integer,
    ongoing_support_required boolean DEFAULT true,
    status character varying(20) DEFAULT 'active'::character varying,
    effective_date date DEFAULT CURRENT_DATE,
    end_date date,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    created_by character varying(100),
    updated_by character varying(100),
    CONSTRAINT product_options_status_check CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'deprecated'::character varying, 'beta'::character varying, 'coming_soon'::character varying])::text[])))
);


--
-- Name: TABLE product_options; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.product_options IS 'Product configuration options such as market settlement choices, currency support, and feature add-ons';


--
-- Name: enhanced_commercial_taxonomy_view; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.enhanced_commercial_taxonomy_view AS
 SELECT p.id AS product_id,
    p.product_id AS product_code,
    p.product_name,
    p.line_of_business,
    p.description AS product_description,
    p.contract_type,
    p.commercial_status,
    p.pricing_model AS product_pricing_model,
    p.target_market,
    po.id AS option_id,
    po.option_id AS option_code,
    po.option_name,
    po.option_category,
    po.option_type,
    po.option_value,
    po.pricing_impact AS option_pricing_impact,
    s.id AS service_id,
    s.service_id AS service_code,
    s.service_name,
    s.service_category,
    s.description AS service_description,
    s.service_type,
    s.delivery_model,
    s.billable AS service_billable,
    r.id AS resource_id,
    r.resource_name,
    r.description AS resource_description,
    r.resource_type,
    r.criticality_level,
    r.operational_status,
    count(DISTINCT a.id) AS attribute_count,
    array_agg(DISTINCT a.attribute_name ORDER BY a.attribute_name) FILTER (WHERE (a.id IS NOT NULL)) AS attributes,
    psm.mapping_type AS service_inclusion_type,
    psm.is_mandatory AS service_mandatory,
    posm.mapping_relationship AS option_service_relationship,
    srm.usage_type AS resource_usage_type,
    srm.dependency_level AS resource_dependency,
    count(DISTINCT cc.id) AS active_contracts,
    avg(cc.contract_value) AS avg_contract_value,
    count(DISTINCT po.id) AS total_options,
    count(DISTINCT
        CASE
            WHEN ((po.option_type)::text = 'required'::text) THEN po.id
            ELSE NULL::integer
        END) AS required_options,
    count(DISTINCT
        CASE
            WHEN ((po.option_type)::text = 'premium'::text) THEN po.id
            ELSE NULL::integer
        END) AS premium_options,
    p.compliance_requirements,
    po.regulatory_approval_required,
    r.compliance_classification,
    r.audit_required,
    count(DISTINCT
        CASE
            WHEN ((r.operational_status)::text = 'active'::text) THEN r.id
            ELSE NULL::integer
        END) AS active_resources,
    count(DISTINCT
        CASE
            WHEN ((s.service_type)::text = 'custody'::text) THEN s.id
            ELSE NULL::integer
        END) AS custody_services,
    count(DISTINCT
        CASE
            WHEN ((s.service_type)::text = 'reconciliation'::text) THEN s.id
            ELSE NULL::integer
        END) AS reconciliation_services,
    p.created_at AS product_created_at,
    p.updated_at AS product_updated_at
   FROM ((((((((public.products p
     LEFT JOIN public.product_options po ON ((po.product_id = p.id)))
     LEFT JOIN public.product_option_service_mappings posm ON ((posm.product_option_id = po.id)))
     LEFT JOIN public.services s ON ((s.id = posm.service_id)))
     LEFT JOIN public.product_service_mappings psm ON (((psm.product_id = p.id) AND (psm.service_id = s.id))))
     LEFT JOIN public.service_resource_mappings srm ON ((srm.service_id = s.id)))
     LEFT JOIN public.resource_objects r ON ((r.id = srm.resource_id)))
     LEFT JOIN public.attribute_objects a ON ((a.resource_id = r.id)))
     LEFT JOIN public.commercial_contracts cc ON (((cc.product_id = p.id) AND ((cc.contract_status)::text = 'active'::text))))
  GROUP BY p.id, p.product_id, p.product_name, p.line_of_business, p.description, p.contract_type, p.commercial_status, p.pricing_model, p.target_market, p.compliance_requirements, p.created_at, p.updated_at, po.id, po.option_id, po.option_name, po.option_category, po.option_type, po.option_value, po.pricing_impact, po.regulatory_approval_required, s.id, s.service_id, s.service_name, s.service_category, s.description, s.service_type, s.delivery_model, s.billable, r.id, r.resource_name, r.description, r.resource_type, r.criticality_level, r.operational_status, r.compliance_classification, r.audit_required, psm.mapping_type, psm.is_mandatory, posm.mapping_relationship, srm.usage_type, srm.dependency_level;


--
-- Name: VIEW enhanced_commercial_taxonomy_view; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON VIEW public.enhanced_commercial_taxonomy_view IS 'Complete view of Products→Options→Services→Resources hierarchy with all commercial metadata';


--
-- Name: entity_attribute_values; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.entity_attribute_values (
    id integer NOT NULL,
    entity_id integer,
    attribute_id integer,
    attribute_value jsonb NOT NULL,
    value_source character varying(100),
    confidence_score numeric(3,2),
    verification_status character varying(30) DEFAULT 'unverified'::character varying,
    verification_method character varying(100),
    verified_by character varying(100),
    verified_at timestamp without time zone,
    effective_from timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    effective_to timestamp without time zone,
    is_current boolean DEFAULT true,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    created_by character varying(100),
    updated_by character varying(100)
);


--
-- Name: TABLE entity_attribute_values; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.entity_attribute_values IS 'Values of enhanced attributes for specific entities with temporal validity';


--
-- Name: entity_attribute_values_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.entity_attribute_values_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: entity_attribute_values_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.entity_attribute_values_id_seq OWNED BY public.entity_attribute_values.id;


--
-- Name: entity_relationships; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.entity_relationships (
    id integer NOT NULL,
    parent_entity_id integer,
    child_entity_id integer,
    relationship_type character varying(50) NOT NULL,
    ownership_percentage numeric(5,2),
    control_type character varying(50),
    effective_date date,
    end_date date,
    relationship_strength character varying(20) DEFAULT 'medium'::character varying,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE entity_relationships; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.entity_relationships IS 'Parent-child and ownership relationships between entities';


--
-- Name: entity_relationships_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.entity_relationships_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: entity_relationships_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.entity_relationships_id_seq OWNED BY public.entity_relationships.id;


--
-- Name: form_layout_templates; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.form_layout_templates (
    id integer NOT NULL,
    template_name character varying(100) NOT NULL,
    layout_type character varying(50) NOT NULL,
    layout_config jsonb NOT NULL,
    responsive_breakpoints jsonb DEFAULT '{}'::jsonb,
    css_framework character varying(50) DEFAULT 'custom'::character varying,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: form_layout_templates_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.form_layout_templates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: form_layout_templates_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.form_layout_templates_id_seq OWNED BY public.form_layout_templates.id;


--
-- Name: grammar_extensions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.grammar_extensions (
    id integer NOT NULL,
    name character varying(100) NOT NULL,
    type character varying(20) NOT NULL,
    signature character varying(200),
    description text,
    category character varying(50),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: grammar_extensions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.grammar_extensions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: grammar_extensions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.grammar_extensions_id_seq OWNED BY public.grammar_extensions.id;


--
-- Name: grammar_metadata; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.grammar_metadata (
    id integer NOT NULL,
    version character varying(20) DEFAULT '1.0'::character varying NOT NULL,
    description text,
    author character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    is_active boolean DEFAULT true
);


--
-- Name: grammar_metadata_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.grammar_metadata_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: grammar_metadata_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.grammar_metadata_id_seq OWNED BY public.grammar_metadata.id;


--
-- Name: grammar_rules; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.grammar_rules (
    id integer NOT NULL,
    name character varying(100) NOT NULL,
    definition text NOT NULL,
    rule_type character varying(20) DEFAULT 'normal'::character varying NOT NULL,
    description text,
    category character varying(50),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: grammar_rules_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.grammar_rules_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: grammar_rules_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.grammar_rules_id_seq OWNED BY public.grammar_rules.id;


--
-- Name: instruction_formats; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.instruction_formats (
    id integer NOT NULL,
    format_id character varying(100) NOT NULL,
    format_name character varying(255) NOT NULL,
    format_category character varying(100),
    message_standard character varying(100),
    message_type character varying(100),
    format_version character varying(50),
    schema_definition jsonb,
    required_fields text[] NOT NULL,
    optional_fields text[],
    validation_rules jsonb,
    field_formats jsonb,
    processing_priority character varying(50) DEFAULT 'normal'::character varying,
    expected_processing_time_minutes integer,
    retry_policy jsonb,
    error_handling_procedure text,
    encryption_required boolean DEFAULT false,
    digital_signature_required boolean DEFAULT false,
    authorization_level_required character varying(50),
    audit_trail_required boolean DEFAULT true,
    max_message_size_kb integer,
    character_encoding character varying(50) DEFAULT 'UTF-8'::character varying,
    timestamp_format character varying(100),
    decimal_precision integer DEFAULT 2,
    supported_delivery_methods text[],
    acknowledgment_required boolean DEFAULT true,
    applicable_markets text[],
    applicable_asset_classes text[],
    regulatory_compliance text[],
    status character varying(50) DEFAULT 'active'::character varying,
    effective_date date DEFAULT CURRENT_DATE,
    retirement_date date,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    created_by character varying(100),
    updated_by character varying(100),
    CONSTRAINT instruction_formats_status_check CHECK (((status)::text = ANY ((ARRAY['draft'::character varying, 'active'::character varying, 'deprecated'::character varying, 'retired'::character varying])::text[])))
);


--
-- Name: TABLE instruction_formats; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.instruction_formats IS 'Standard message formats for trading, settlement, and reporting instructions';


--
-- Name: instruction_formats_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.instruction_formats_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: instruction_formats_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.instruction_formats_id_seq OWNED BY public.instruction_formats.id;


--
-- Name: instrument_taxonomy; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.instrument_taxonomy (
    id integer NOT NULL,
    instrument_code character varying(50) NOT NULL,
    instrument_name character varying(255) NOT NULL,
    instrument_class character varying(100) NOT NULL,
    instrument_subclass character varying(100),
    asset_class character varying(100),
    cfi_code character varying(6),
    isin_pattern character varying(50),
    fisn_code character varying(50),
    market_sector character varying(100),
    geography character varying(100),
    currency_denomination character varying(10),
    risk_category character varying(50) DEFAULT 'medium'::character varying,
    liquidity_classification character varying(50),
    credit_rating_required boolean DEFAULT false,
    regulatory_category character varying(100),
    professional_investor_only boolean DEFAULT false,
    regulatory_capital_treatment character varying(100),
    typical_lot_size numeric(20,2),
    minimum_denomination numeric(20,2),
    settlement_cycle character varying(10),
    active boolean DEFAULT true,
    effective_date date DEFAULT CURRENT_DATE,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE instrument_taxonomy; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.instrument_taxonomy IS 'Industry standard instrument classifications with regulatory and trading characteristics';


--
-- Name: instrument_taxonomy_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.instrument_taxonomy_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: instrument_taxonomy_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.instrument_taxonomy_id_seq OWNED BY public.instrument_taxonomy.id;


--
-- Name: kyc_onboarding_domains_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.kyc_onboarding_domains_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: kyc_onboarding_domains_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.kyc_onboarding_domains_id_seq OWNED BY public.kyc_onboarding_domains.id;


--
-- Name: legal_entities_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.legal_entities_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: legal_entities_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.legal_entities_id_seq OWNED BY public.legal_entities.id;


--
-- Name: mandate_benchmarks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mandate_benchmarks (
    id integer NOT NULL,
    mandate_id character varying(100) NOT NULL,
    benchmark_name character varying(255) NOT NULL
);


--
-- Name: TABLE mandate_benchmarks; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.mandate_benchmarks IS 'Performance benchmarks associated with each mandate';


--
-- Name: mandate_benchmarks_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.mandate_benchmarks_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: mandate_benchmarks_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.mandate_benchmarks_id_seq OWNED BY public.mandate_benchmarks.id;


--
-- Name: mandate_instruction_channels; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mandate_instruction_channels (
    id integer NOT NULL,
    instrument_id integer NOT NULL,
    channel character varying(30) NOT NULL,
    formats text[],
    allowed_flows text[],
    stp_required boolean DEFAULT false,
    CONSTRAINT mandate_instruction_channels_channel_check CHECK (((channel)::text = ANY ((ARRAY['FIX'::character varying, 'SWIFT_MT'::character varying, 'ISO20022_XML'::character varying, 'FpML'::character varying, 'Portal'::character varying, 'CSV_SFTP'::character varying, 'PhoneRecorded'::character varying])::text[])))
);


--
-- Name: TABLE mandate_instruction_channels; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.mandate_instruction_channels IS 'Communication channels and formats for each instrument';


--
-- Name: mandate_instruction_channels_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.mandate_instruction_channels_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: mandate_instruction_channels_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.mandate_instruction_channels_id_seq OWNED BY public.mandate_instruction_channels.id;


--
-- Name: mandate_instrument_identifiers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mandate_instrument_identifiers (
    id integer NOT NULL,
    instrument_id integer NOT NULL,
    identifier_type character varying(20) NOT NULL,
    CONSTRAINT mandate_instrument_identifiers_identifier_type_check CHECK (((identifier_type)::text = ANY ((ARRAY['ISIN'::character varying, 'FISN'::character varying, 'UPI'::character varying, 'UTI'::character varying, 'RIC'::character varying, 'BloombergFIGI'::character varying, 'CUSIP'::character varying, 'SEDOL'::character varying])::text[])))
);


--
-- Name: TABLE mandate_instrument_identifiers; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.mandate_instrument_identifiers IS 'Required identifier types for each instrument in a mandate';


--
-- Name: mandate_instrument_identifiers_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.mandate_instrument_identifiers_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: mandate_instrument_identifiers_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.mandate_instrument_identifiers_id_seq OWNED BY public.mandate_instrument_identifiers.id;


--
-- Name: mandate_instrument_venues; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mandate_instrument_venues (
    id integer NOT NULL,
    instrument_id integer NOT NULL,
    mic character varying(10) NOT NULL,
    preferred boolean DEFAULT false
);


--
-- Name: TABLE mandate_instrument_venues; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.mandate_instrument_venues IS 'Approved trading venues for each instrument';


--
-- Name: mandate_instrument_venues_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.mandate_instrument_venues_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: mandate_instrument_venues_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.mandate_instrument_venues_id_seq OWNED BY public.mandate_instrument_venues.id;


--
-- Name: mandate_instruments_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.mandate_instruments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: mandate_instruments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.mandate_instruments_id_seq OWNED BY public.mandate_instruments.id;


--
-- Name: mandate_lifecycle_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mandate_lifecycle_events (
    id integer NOT NULL,
    instrument_id integer NOT NULL,
    event_type character varying(30) NOT NULL,
    CONSTRAINT mandate_lifecycle_events_event_type_check CHECK (((event_type)::text = ANY ((ARRAY['corporate_actions'::character varying, 'option_exercise'::character varying, 'roll'::character varying, 'expiry'::character varying, 'recall'::character varying])::text[])))
);


--
-- Name: TABLE mandate_lifecycle_events; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.mandate_lifecycle_events IS 'Lifecycle events applicable to each instrument';


--
-- Name: mandate_lifecycle_events_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.mandate_lifecycle_events_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: mandate_lifecycle_events_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.mandate_lifecycle_events_id_seq OWNED BY public.mandate_lifecycle_events.id;


--
-- Name: rules; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rules (
    id integer NOT NULL,
    rule_id character varying(50) NOT NULL,
    rule_name character varying(200) NOT NULL,
    description text,
    category_id integer,
    target_attribute_id integer,
    rule_definition text NOT NULL,
    parsed_ast jsonb,
    status character varying(20) DEFAULT 'draft'::character varying,
    version integer DEFAULT 1,
    tags text[],
    performance_metrics jsonb,
    embedding_data jsonb,
    created_by character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by character varying(100),
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    search_vector tsvector GENERATED ALWAYS AS (((setweight(to_tsvector('english'::regconfig, (COALESCE(rule_name, ''::character varying))::text), 'A'::"char") || setweight(to_tsvector('english'::regconfig, COALESCE(description, ''::text)), 'B'::"char")) || setweight(to_tsvector('english'::regconfig, COALESCE(rule_definition, ''::text)), 'C'::"char"))) STORED,
    embedding public.vector(1536),
    compiled_rust_code text,
    compiled_wasm_binary bytea,
    compilation_status character varying(20) DEFAULT 'pending'::character varying,
    compilation_error text,
    compilation_timestamp timestamp with time zone,
    compiler_version character varying(50),
    execution_count bigint DEFAULT 0,
    avg_execution_time_ms numeric(10,3),
    CONSTRAINT rules_compilation_status_check CHECK (((compilation_status)::text = ANY ((ARRAY['pending'::character varying, 'compiling'::character varying, 'success'::character varying, 'failed'::character varying, 'outdated'::character varying])::text[]))),
    CONSTRAINT rules_status_check CHECK (((status)::text = ANY (ARRAY[('draft'::character varying)::text, ('active'::character varying)::text, ('inactive'::character varying)::text, ('deprecated'::character varying)::text])))
);


--
-- Name: mv_data_dictionary; Type: MATERIALIZED VIEW; Schema: public; Owner: -
--

CREATE MATERIALIZED VIEW public.mv_data_dictionary AS
 WITH all_attributes AS (
         SELECT 'business'::text AS attribute_type,
            business_attributes.entity_name,
            business_attributes.attribute_name,
            (((business_attributes.entity_name)::text || '.'::text) || (business_attributes.attribute_name)::text) AS full_path,
            business_attributes.data_type,
            business_attributes.sql_type,
            business_attributes.rust_type,
            business_attributes.description,
            business_attributes.required,
            business_attributes.validation_pattern,
            NULL::text AS rule_definition,
            NULL::integer AS rule_id,
            'active'::character varying AS status
           FROM public.business_attributes
          WHERE (business_attributes.is_active = true)
        UNION ALL
         SELECT 'derived'::text AS attribute_type,
            da.entity_name,
            da.attribute_name,
            (((da.entity_name)::text || '.'::text) || (da.attribute_name)::text) AS full_path,
            da.data_type,
            da.sql_type,
            da.rust_type,
            da.description,
            false AS required,
            NULL::text AS validation_pattern,
            r.rule_definition,
            r.id AS rule_id,
            COALESCE(r.status, 'draft'::character varying) AS status
           FROM (public.derived_attributes da
             LEFT JOIN public.rules r ON ((r.target_attribute_id = da.id)))
        UNION ALL
         SELECT DISTINCT 'system'::text AS attribute_type,
            c.table_name AS entity_name,
            c.column_name AS attribute_name,
            (((c.table_name)::text || '.'::text) || (c.column_name)::text) AS full_path,
            c.data_type,
            c.data_type AS sql_type,
                CASE
                    WHEN ((c.data_type)::text ~~ '%int%'::text) THEN 'i32'::text
                    WHEN (((c.data_type)::text ~~ '%numeric%'::text) OR ((c.data_type)::text ~~ '%decimal%'::text)) THEN 'f64'::text
                    WHEN ((c.data_type)::text = 'boolean'::text) THEN 'bool'::text
                    WHEN ((c.data_type)::text ~~ '%json%'::text) THEN 'JsonValue'::text
                    WHEN ((c.data_type)::text ~~ 'vector%'::text) THEN 'Vec<f32>'::text
                    WHEN ((c.data_type)::text ~~ '%timestamp%'::text) THEN 'DateTime'::text
                    WHEN ((c.data_type)::text = 'date'::text) THEN 'Date'::text
                    WHEN ((c.data_type)::text ~~ 'text%'::text) THEN 'String'::text
                    WHEN ((c.data_type)::text ~~ 'character%'::text) THEN 'String'::text
                    ELSE 'String'::text
                END AS rust_type,
            obj_description(pgc.oid, 'pg_class'::name) AS description,
            ((c.is_nullable)::text = 'NO'::text) AS required,
            NULL::text AS validation_pattern,
            NULL::text AS rule_definition,
            NULL::integer AS rule_id,
            'active'::text AS status
           FROM (information_schema.columns c
             LEFT JOIN pg_class pgc ON ((pgc.relname = (c.table_name)::name)))
          WHERE (((c.table_schema)::name = 'public'::name) AND ((c.table_name)::name = ANY (ARRAY['investment_mandates'::name, 'mandate_instruments'::name, 'mandate_benchmarks'::name, 'mandate_instrument_venues'::name, 'mandate_instruction_channels'::name, 'mandate_instrument_identifiers'::name, 'mandate_lifecycle_events'::name])))
        )
 SELECT attribute_type,
    entity_name,
    attribute_name,
    full_path,
    data_type,
    sql_type,
    rust_type,
    description,
    required,
    validation_pattern,
    rule_definition,
    rule_id,
    status
   FROM all_attributes
  ORDER BY entity_name, attribute_type, attribute_name
  WITH NO DATA;


--
-- Name: onboarding_approvals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.onboarding_approvals (
    id integer NOT NULL,
    workflow_id integer,
    approval_stage character varying(50) NOT NULL,
    approver_role character varying(100) NOT NULL,
    approver_user character varying(100),
    approval_status character varying(20) DEFAULT 'pending'::character varying,
    approval_notes text,
    approved_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT approval_status_check CHECK (((approval_status)::text = ANY ((ARRAY['pending'::character varying, 'approved'::character varying, 'rejected'::character varying, 'delegated'::character varying])::text[])))
);


--
-- Name: TABLE onboarding_approvals; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.onboarding_approvals IS 'Approval workflow tracking for onboarding processes';


--
-- Name: onboarding_approvals_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.onboarding_approvals_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: onboarding_approvals_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.onboarding_approvals_id_seq OWNED BY public.onboarding_approvals.id;


--
-- Name: onboarding_dependencies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.onboarding_dependencies (
    id integer NOT NULL,
    workflow_id integer,
    source_task_id integer,
    target_task_id integer,
    dependency_type character varying(20) DEFAULT 'blocking'::character varying,
    dependency_condition text,
    is_satisfied boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT dependency_type_check CHECK (((dependency_type)::text = ANY ((ARRAY['blocking'::character varying, 'informational'::character varying, 'conditional'::character varying])::text[]))),
    CONSTRAINT no_self_dependency CHECK ((source_task_id <> target_task_id))
);


--
-- Name: TABLE onboarding_dependencies; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.onboarding_dependencies IS 'Task dependency management for onboarding workflows';


--
-- Name: onboarding_dependencies_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.onboarding_dependencies_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: onboarding_dependencies_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.onboarding_dependencies_id_seq OWNED BY public.onboarding_dependencies.id;


--
-- Name: onboarding_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.onboarding_requests (
    id integer NOT NULL,
    request_id character varying(100) NOT NULL,
    cbu_id integer,
    product_id integer,
    request_status character varying(20) DEFAULT 'draft'::character varying,
    priority character varying(20) DEFAULT 'medium'::character varying,
    target_go_live_date date,
    business_requirements jsonb,
    compliance_requirements jsonb,
    requested_by character varying(100),
    assigned_to character varying(100),
    approval_chain jsonb,
    estimated_duration_days integer,
    actual_duration_days integer,
    created_by character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by character varying(100),
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT onboarding_requests_priority_check CHECK (((priority)::text = ANY ((ARRAY['low'::character varying, 'medium'::character varying, 'high'::character varying, 'urgent'::character varying])::text[]))),
    CONSTRAINT onboarding_requests_request_status_check CHECK (((request_status)::text = ANY ((ARRAY['draft'::character varying, 'submitted'::character varying, 'under_review'::character varying, 'approved'::character varying, 'in_progress'::character varying, 'completed'::character varying, 'rejected'::character varying, 'cancelled'::character varying])::text[])))
);


--
-- Name: TABLE onboarding_requests; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.onboarding_requests IS 'Workflow tracking for CBU product onboarding process';


--
-- Name: onboarding_requests_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.onboarding_requests_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: onboarding_requests_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.onboarding_requests_id_seq OWNED BY public.onboarding_requests.id;


--
-- Name: onboarding_resource_tasks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.onboarding_resource_tasks (
    id integer NOT NULL,
    workflow_id integer,
    resource_template_id integer,
    capability_id integer,
    task_order integer NOT NULL,
    task_status character varying(20) DEFAULT 'pending'::character varying,
    input_attributes jsonb DEFAULT '{}'::jsonb,
    output_attributes jsonb,
    validation_results jsonb,
    execution_log jsonb DEFAULT '[]'::jsonb,
    assigned_to character varying(100),
    started_at timestamp with time zone,
    completed_at timestamp with time zone,
    estimated_hours real,
    actual_hours real,
    blocking_issues text,
    retry_count integer DEFAULT 0,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT task_status_check CHECK (((task_status)::text = ANY ((ARRAY['pending'::character varying, 'in_progress'::character varying, 'completed'::character varying, 'failed'::character varying, 'blocked'::character varying])::text[])))
);


--
-- Name: TABLE onboarding_resource_tasks; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.onboarding_resource_tasks IS 'Individual capability execution tasks within an onboarding workflow';


--
-- Name: onboarding_resource_tasks_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.onboarding_resource_tasks_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: onboarding_resource_tasks_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.onboarding_resource_tasks_id_seq OWNED BY public.onboarding_resource_tasks.id;


--
-- Name: onboarding_tasks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.onboarding_tasks (
    id integer NOT NULL,
    onboarding_request_id integer,
    task_id character varying(100) NOT NULL,
    resource_id integer,
    task_type character varying(50),
    task_name character varying(255) NOT NULL,
    description text,
    task_status character varying(20) DEFAULT 'pending'::character varying,
    assigned_to character varying(100),
    dependencies text[],
    estimated_hours numeric(5,1),
    actual_hours numeric(5,1),
    due_date date,
    completion_date date,
    blocking_issues text,
    completion_notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT onboarding_tasks_task_status_check CHECK (((task_status)::text = ANY ((ARRAY['pending'::character varying, 'assigned'::character varying, 'in_progress'::character varying, 'blocked'::character varying, 'completed'::character varying, 'skipped'::character varying])::text[])))
);


--
-- Name: onboarding_tasks_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.onboarding_tasks_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: onboarding_tasks_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.onboarding_tasks_id_seq OWNED BY public.onboarding_tasks.id;


--
-- Name: onboarding_workflows; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.onboarding_workflows (
    id integer NOT NULL,
    workflow_id character varying(50) NOT NULL,
    cbu_id integer,
    product_ids integer[] NOT NULL,
    workflow_status character varying(20) DEFAULT 'initiated'::character varying,
    priority character varying(20) DEFAULT 'medium'::character varying,
    target_go_live_date date,
    business_requirements jsonb DEFAULT '{}'::jsonb,
    compliance_requirements jsonb DEFAULT '{}'::jsonb,
    resource_requirements jsonb DEFAULT '{}'::jsonb,
    execution_plan jsonb DEFAULT '[]'::jsonb,
    current_stage character varying(100),
    completion_percentage integer DEFAULT 0,
    requested_by character varying(100),
    assigned_to character varying(100),
    approval_chain jsonb DEFAULT '[]'::jsonb,
    estimated_duration_days integer,
    actual_duration_days integer,
    created_by character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by character varying(100),
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT completion_percentage_check CHECK (((completion_percentage >= 0) AND (completion_percentage <= 100))),
    CONSTRAINT priority_check CHECK (((priority)::text = ANY ((ARRAY['low'::character varying, 'medium'::character varying, 'high'::character varying, 'critical'::character varying])::text[]))),
    CONSTRAINT workflow_status_check CHECK (((workflow_status)::text = ANY ((ARRAY['initiated'::character varying, 'in_progress'::character varying, 'completed'::character varying, 'failed'::character varying, 'cancelled'::character varying])::text[])))
);


--
-- Name: TABLE onboarding_workflows; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.onboarding_workflows IS 'Capability-driven onboarding workflows linking CBUs to products and resources';


--
-- Name: onboarding_workflows_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.onboarding_workflows_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: onboarding_workflows_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.onboarding_workflows_id_seq OWNED BY public.onboarding_workflows.id;


--
-- Name: persistence_entities_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.persistence_entities_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: persistence_entities_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.persistence_entities_id_seq OWNED BY public.persistence_entities.id;


--
-- Name: persistence_systems_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.persistence_systems_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: persistence_systems_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.persistence_systems_id_seq OWNED BY public.persistence_systems.id;


--
-- Name: product_option_service_mappings_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.product_option_service_mappings_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: product_option_service_mappings_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.product_option_service_mappings_id_seq OWNED BY public.product_option_service_mappings.id;


--
-- Name: product_options_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.product_options_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: product_options_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.product_options_id_seq OWNED BY public.product_options.id;


--
-- Name: product_service_mappings_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.product_service_mappings_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: product_service_mappings_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.product_service_mappings_id_seq OWNED BY public.product_service_mappings.id;


--
-- Name: product_services; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.product_services (
    id integer NOT NULL,
    product_id integer,
    service_id integer,
    is_required boolean DEFAULT true,
    configuration jsonb,
    pricing_component numeric(10,2),
    display_order integer,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: product_services_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.product_services_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: product_services_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.product_services_id_seq OWNED BY public.product_services.id;


--
-- Name: products_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.products_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: products_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.products_id_seq OWNED BY public.products.id;


--
-- Name: resource_attribute_dependencies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_attribute_dependencies (
    id integer NOT NULL,
    source_resource_id integer,
    target_resource_id integer,
    dependency_type character varying(50) NOT NULL,
    dependency_config jsonb DEFAULT '{}'::jsonb,
    is_active boolean DEFAULT true,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE resource_attribute_dependencies; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.resource_attribute_dependencies IS 'Dependencies between resource attribute sets';


--
-- Name: resource_attribute_dependencies_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resource_attribute_dependencies_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resource_attribute_dependencies_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resource_attribute_dependencies_id_seq OWNED BY public.resource_attribute_dependencies.id;


--
-- Name: resource_attribute_templates; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_attribute_templates (
    id integer NOT NULL,
    template_name character varying(100) NOT NULL,
    description text,
    category character varying(50),
    attributes_config jsonb NOT NULL,
    ui_layout_template jsonb DEFAULT '{}'::jsonb,
    validation_rules jsonb DEFAULT '{}'::jsonb,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE resource_attribute_templates; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.resource_attribute_templates IS 'Templates for common attribute set patterns';


--
-- Name: resource_attribute_templates_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resource_attribute_templates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resource_attribute_templates_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resource_attribute_templates_id_seq OWNED BY public.resource_attribute_templates.id;


--
-- Name: resource_attribute_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_attribute_versions (
    id integer NOT NULL,
    resource_id integer,
    version_number character varying(20) NOT NULL,
    change_description text,
    attributes_snapshot jsonb NOT NULL,
    schema_changes jsonb DEFAULT '{}'::jsonb,
    created_by character varying(100),
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    is_active boolean DEFAULT false
);


--
-- Name: TABLE resource_attribute_versions; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.resource_attribute_versions IS 'Version control for resource attribute sets';


--
-- Name: resource_attribute_versions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resource_attribute_versions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resource_attribute_versions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resource_attribute_versions_id_seq OWNED BY public.resource_attribute_versions.id;


--
-- Name: resource_capabilities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_capabilities (
    id integer NOT NULL,
    capability_id character varying(50) NOT NULL,
    capability_name character varying(200) NOT NULL,
    description text,
    capability_type character varying(50) NOT NULL,
    required_attributes jsonb DEFAULT '[]'::jsonb,
    optional_attributes jsonb DEFAULT '[]'::jsonb,
    output_attributes jsonb DEFAULT '[]'::jsonb,
    implementation_function character varying(200),
    validation_rules jsonb DEFAULT '{}'::jsonb,
    error_handling jsonb DEFAULT '{}'::jsonb,
    timeout_seconds integer DEFAULT 300,
    retry_attempts integer DEFAULT 3,
    status character varying(20) DEFAULT 'active'::character varying,
    created_by character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by character varying(100),
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE resource_capabilities; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.resource_capabilities IS 'Specific actions that resources can perform';


--
-- Name: resource_capabilities_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resource_capabilities_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resource_capabilities_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resource_capabilities_id_seq OWNED BY public.resource_capabilities.id;


--
-- Name: resource_dependencies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_dependencies (
    id integer NOT NULL,
    dependent_resource_id integer,
    prerequisite_resource_id integer,
    dependency_type character varying(50),
    description text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT resource_dependencies_check CHECK ((dependent_resource_id <> prerequisite_resource_id))
);


--
-- Name: resource_dependencies_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resource_dependencies_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resource_dependencies_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resource_dependencies_id_seq OWNED BY public.resource_dependencies.id;


--
-- Name: resource_dictionaries; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_dictionaries (
    id integer NOT NULL,
    dictionary_name character varying(100) NOT NULL,
    version character varying(20) DEFAULT '1.0'::character varying NOT NULL,
    description text,
    author character varying(100),
    creation_date timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    last_modified timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    status character varying(20) DEFAULT 'active'::character varying,
    metadata jsonb DEFAULT '{}'::jsonb,
    CONSTRAINT resource_dictionaries_status_check CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'inactive'::character varying, 'deprecated'::character varying])::text[])))
);


--
-- Name: resource_dictionaries_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resource_dictionaries_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resource_dictionaries_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resource_dictionaries_id_seq OWNED BY public.resource_dictionaries.id;


--
-- Name: resource_instances; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_instances (
    instance_id text DEFAULT (gen_random_uuid())::text NOT NULL,
    onboarding_request_id text NOT NULL,
    template_id text NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    instance_data jsonb DEFAULT '{}'::jsonb NOT NULL,
    execution_context jsonb DEFAULT '{}'::jsonb,
    error_message text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by text DEFAULT 'system'::text NOT NULL,
    CONSTRAINT resource_instances_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'active'::text, 'completed'::text, 'failed'::text])))
);


--
-- Name: resource_management_view; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.resource_management_view AS
 SELECT rd.dictionary_name,
    rd.version AS dictionary_version,
    ro.id AS resource_id,
    ro.resource_name,
    ro.description,
    ro.version AS resource_version,
    ro.category,
    ro.status,
    ro.ui_layout,
    count(ao.id) AS total_attributes,
    count(
        CASE
            WHEN ao.is_required THEN 1
            ELSE NULL::integer
        END) AS required_attributes,
    count(
        CASE
            WHEN ((ao.attribute_class)::text = 'derived'::text) THEN 1
            ELSE NULL::integer
        END) AS derived_attributes,
    count(
        CASE
            WHEN ((ao.attribute_class)::text = 'real'::text) THEN 1
            ELSE NULL::integer
        END) AS real_attributes,
    count(
        CASE
            WHEN (ao.extended_description IS NOT NULL) THEN 1
            ELSE NULL::integer
        END) AS documented_attributes,
    round((((count(
        CASE
            WHEN (ao.extended_description IS NOT NULL) THEN 1
            ELSE NULL::integer
        END))::numeric / (GREATEST(count(ao.id), (1)::bigint))::numeric) * (100)::numeric), 2) AS documentation_percentage,
    count(
        CASE
            WHEN (ao.ui_component_type IS NOT NULL) THEN 1
            ELSE NULL::integer
        END) AS ui_configured_attributes,
    round((((count(
        CASE
            WHEN (ao.ui_component_type IS NOT NULL) THEN 1
            ELSE NULL::integer
        END))::numeric / (GREATEST(count(ao.id), (1)::bigint))::numeric) * (100)::numeric), 2) AS ui_configuration_percentage,
    count(
        CASE
            WHEN ((ao.semantic_tags IS NOT NULL) AND (ao.semantic_tags <> '[]'::jsonb)) THEN 1
            ELSE NULL::integer
        END) AS ai_enhanced_attributes,
    round((((count(
        CASE
            WHEN ((ao.semantic_tags IS NOT NULL) AND (ao.semantic_tags <> '[]'::jsonb)) THEN 1
            ELSE NULL::integer
        END))::numeric / (GREATEST(count(ao.id), (1)::bigint))::numeric) * (100)::numeric), 2) AS ai_enhancement_percentage,
    count(
        CASE
            WHEN ((ao.validation_pattern IS NOT NULL) OR (ao.allowed_values IS NOT NULL)) THEN 1
            ELSE NULL::integer
        END) AS validated_attributes,
    array_agg(DISTINCT ao.ui_group ORDER BY ao.ui_group) FILTER (WHERE (ao.ui_group IS NOT NULL)) AS ui_groups,
    ro.updated_at AS last_modified,
    max(ao.updated_at) AS last_attribute_change
   FROM ((public.resource_dictionaries rd
     JOIN public.resource_objects ro ON ((ro.dictionary_id = rd.id)))
     LEFT JOIN public.attribute_objects ao ON ((ao.resource_id = ro.id)))
  GROUP BY rd.id, rd.dictionary_name, rd.version, ro.id, ro.resource_name, ro.description, ro.version, ro.category, ro.status, ro.ui_layout, ro.updated_at;


--
-- Name: VIEW resource_management_view; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON VIEW public.resource_management_view IS 'Comprehensive view of resource attribute set management metrics';


--
-- Name: resource_objects_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resource_objects_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resource_objects_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resource_objects_id_seq OWNED BY public.resource_objects.id;


--
-- Name: resource_sheet_execution_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_sheet_execution_logs (
    log_id integer NOT NULL,
    resource_id text NOT NULL,
    execution_id text NOT NULL,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL,
    step text NOT NULL,
    message text NOT NULL,
    log_level text NOT NULL,
    log_data jsonb DEFAULT '{}'::jsonb
);


--
-- Name: resource_sheet_execution_logs_log_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resource_sheet_execution_logs_log_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resource_sheet_execution_logs_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resource_sheet_execution_logs_log_id_seq OWNED BY public.resource_sheet_execution_logs.log_id;


--
-- Name: resource_sheet_relationships; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_sheet_relationships (
    relationship_id integer NOT NULL,
    parent_resource_id text NOT NULL,
    child_resource_id text NOT NULL,
    relationship_type text NOT NULL,
    sequence_order integer DEFAULT 0,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: resource_sheet_relationships_relationship_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resource_sheet_relationships_relationship_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resource_sheet_relationships_relationship_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resource_sheet_relationships_relationship_id_seq OWNED BY public.resource_sheet_relationships.relationship_id;


--
-- Name: resource_sheets; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_sheets (
    resource_id text NOT NULL,
    resource_type text NOT NULL,
    name text NOT NULL,
    description text,
    version text DEFAULT '1.0.0'::text NOT NULL,
    client_id text,
    product_id text,
    status text DEFAULT 'Pending'::text NOT NULL,
    json_data jsonb NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by text NOT NULL,
    tags jsonb DEFAULT '[]'::jsonb,
    domain_id integer,
    ebnf_template_id integer,
    dsl_code text
);


--
-- Name: resource_template_capabilities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_template_capabilities (
    id integer NOT NULL,
    template_id integer,
    capability_id integer,
    capability_order integer DEFAULT 1,
    is_required boolean DEFAULT true,
    configuration_overrides jsonb DEFAULT '{}'::jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE resource_template_capabilities; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.resource_template_capabilities IS 'Many-to-many mapping of templates to capabilities';


--
-- Name: resource_template_capabilities_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resource_template_capabilities_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resource_template_capabilities_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resource_template_capabilities_id_seq OWNED BY public.resource_template_capabilities.id;


--
-- Name: resource_templates; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_templates (
    id integer NOT NULL,
    template_id character varying(50) NOT NULL,
    template_name character varying(200) NOT NULL,
    description text,
    part_of_product character varying(100),
    implements_service character varying(100),
    resource_type character varying(50) NOT NULL,
    attributes jsonb DEFAULT '[]'::jsonb,
    capabilities jsonb DEFAULT '[]'::jsonb,
    dsl_template text,
    version character varying(20) DEFAULT '1.0'::character varying,
    status character varying(20) DEFAULT 'active'::character varying,
    created_by character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by character varying(100),
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE resource_templates; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.resource_templates IS 'Templates defining reusable resource configurations with capabilities';


--
-- Name: resource_templates_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resource_templates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resource_templates_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resource_templates_id_seq OWNED BY public.resource_templates.id;


--
-- Name: resource_validation_rules; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_validation_rules (
    id integer NOT NULL,
    resource_id integer,
    rule_name character varying(100) NOT NULL,
    rule_type character varying(50) NOT NULL,
    rule_config jsonb NOT NULL,
    is_active boolean DEFAULT true,
    severity character varying(20) DEFAULT 'error'::character varying,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE resource_validation_rules; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.resource_validation_rules IS 'Validation rules for resource attribute set integrity';


--
-- Name: resource_validation_rules_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resource_validation_rules_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resource_validation_rules_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resource_validation_rules_id_seq OWNED BY public.resource_validation_rules.id;


--
-- Name: resources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resources (
    id integer NOT NULL,
    resource_id character varying(100) NOT NULL,
    resource_name character varying(255) NOT NULL,
    resource_type character varying(50) NOT NULL,
    description text,
    location character varying(100),
    capacity_limits jsonb,
    operational_hours character varying(100),
    contact_information jsonb,
    technical_specifications jsonb,
    compliance_certifications text[],
    status character varying(20) DEFAULT 'active'::character varying,
    created_by character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by character varying(100),
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT resources_status_check CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'inactive'::character varying, 'maintenance'::character varying, 'deprecated'::character varying])::text[])))
);


--
-- Name: TABLE resources; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.resources IS 'Physical implementors (applications, teams) of logical services';


--
-- Name: resources_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.resources_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: resources_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.resources_id_seq OWNED BY public.resources.id;


--
-- Name: role_service_access; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.role_service_access (
    id integer NOT NULL,
    cbu_role_id integer,
    service_id integer,
    access_type character varying(50),
    interaction_mode character varying(50),
    business_justification text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: role_service_access_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.role_service_access_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: role_service_access_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.role_service_access_id_seq OWNED BY public.role_service_access.id;


--
-- Name: rule_categories; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rule_categories (
    id integer NOT NULL,
    category_key character varying(50) NOT NULL,
    name character varying(100) NOT NULL,
    description text,
    color character varying(7),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: rule_categories_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.rule_categories_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: rule_categories_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.rule_categories_id_seq OWNED BY public.rule_categories.id;


--
-- Name: rule_compilation_queue; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rule_compilation_queue (
    id integer NOT NULL,
    rule_id integer,
    compilation_type character varying(20),
    priority integer DEFAULT 5,
    status character varying(20) DEFAULT 'pending'::character varying,
    retry_count integer DEFAULT 0,
    max_retries integer DEFAULT 3,
    error_message text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    started_at timestamp with time zone,
    completed_at timestamp with time zone,
    worker_id character varying(50),
    CONSTRAINT rule_compilation_queue_compilation_type_check CHECK (((compilation_type)::text = ANY ((ARRAY['rust'::character varying, 'wasm'::character varying, 'both'::character varying])::text[]))),
    CONSTRAINT rule_compilation_queue_priority_check CHECK (((priority >= 1) AND (priority <= 10))),
    CONSTRAINT rule_compilation_queue_status_check CHECK (((status)::text = ANY ((ARRAY['pending'::character varying, 'processing'::character varying, 'completed'::character varying, 'failed'::character varying, 'cancelled'::character varying])::text[])))
);


--
-- Name: rule_compilation_queue_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.rule_compilation_queue_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: rule_compilation_queue_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.rule_compilation_queue_id_seq OWNED BY public.rule_compilation_queue.id;


--
-- Name: rule_dependencies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rule_dependencies (
    id integer NOT NULL,
    rule_id integer,
    attribute_id integer,
    dependency_type character varying(20) DEFAULT 'input'::character varying,
    CONSTRAINT rule_dependencies_dependency_type_check CHECK (((dependency_type)::text = ANY (ARRAY[('input'::character varying)::text, ('lookup'::character varying)::text, ('reference'::character varying)::text])))
);


--
-- Name: rule_dependencies_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.rule_dependencies_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: rule_dependencies_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.rule_dependencies_id_seq OWNED BY public.rule_dependencies.id;


--
-- Name: rule_execution_stats; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rule_execution_stats (
    id integer NOT NULL,
    rule_id integer,
    execution_date date DEFAULT CURRENT_DATE,
    total_executions integer DEFAULT 0,
    successful_executions integer DEFAULT 0,
    failed_executions integer DEFAULT 0,
    avg_execution_time_ms numeric(10,3),
    min_execution_time_ms numeric(10,3),
    max_execution_time_ms numeric(10,3),
    used_compiled_code boolean DEFAULT false
);


--
-- Name: rule_execution_stats_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.rule_execution_stats_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: rule_execution_stats_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.rule_execution_stats_id_seq OWNED BY public.rule_execution_stats.id;


--
-- Name: rule_executions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rule_executions (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    rule_id integer,
    execution_time timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    input_data jsonb,
    output_value jsonb,
    execution_duration_ms integer,
    success boolean,
    error_message text,
    context jsonb
);


--
-- Name: rule_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rule_versions (
    id integer NOT NULL,
    rule_id integer,
    version integer NOT NULL,
    rule_definition text NOT NULL,
    change_description text,
    created_by character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: rule_versions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.rule_versions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: rule_versions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.rule_versions_id_seq OWNED BY public.rule_versions.id;


--
-- Name: rules_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.rules_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: rules_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.rules_id_seq OWNED BY public.rules.id;


--
-- Name: service_resource_mappings_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.service_resource_mappings_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: service_resource_mappings_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.service_resource_mappings_id_seq OWNED BY public.service_resource_mappings.id;


--
-- Name: service_resources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.service_resources (
    id integer NOT NULL,
    service_id integer,
    resource_id integer,
    resource_role character varying(100),
    configuration jsonb,
    priority integer DEFAULT 1,
    health_check_endpoint character varying(255),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: service_resources_hierarchy; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.service_resources_hierarchy AS
 SELECT s.id AS service_id,
    s.service_id AS service_code,
    s.service_name,
    s.service_category,
    s.service_type,
    s.delivery_model,
    s.billable,
    s.description AS service_description,
    s.status AS service_status,
    ro.id AS resource_id,
    ro.resource_name,
    ro.description AS resource_description,
    ro.version AS resource_version,
    ro.category AS resource_category,
    ro.resource_type,
    ro.criticality_level,
    ro.operational_status,
    ro.owner_team,
    srm.usage_type,
    srm.resource_role,
    srm.cost_allocation_percentage,
    srm.dependency_level,
    srm.performance_requirements,
    srm.configuration_parameters
   FROM ((public.services s
     JOIN public.service_resource_mappings srm ON ((srm.service_id = s.id)))
     JOIN public.resource_objects ro ON ((ro.id = srm.resource_id)))
  WHERE (((s.status)::text = 'active'::text) AND ((ro.status)::text = 'active'::text))
  ORDER BY s.service_name, srm.dependency_level, ro.resource_name;


--
-- Name: service_resources_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.service_resources_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: service_resources_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.service_resources_id_seq OWNED BY public.service_resources.id;


--
-- Name: services_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.services_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: services_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.services_id_seq OWNED BY public.services.id;


--
-- Name: ui_component_templates; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ui_component_templates (
    id integer NOT NULL,
    template_name character varying(100) NOT NULL,
    component_type character varying(50) NOT NULL,
    template_config jsonb NOT NULL,
    styling_defaults jsonb DEFAULT '{}'::jsonb,
    behavior_defaults jsonb DEFAULT '{}'::jsonb,
    validation_rules jsonb DEFAULT '{}'::jsonb,
    accessibility_defaults jsonb DEFAULT '{}'::jsonb,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: ui_component_templates_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.ui_component_templates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ui_component_templates_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.ui_component_templates_id_seq OWNED BY public.ui_component_templates.id;


--
-- Name: ui_layout_groups; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ui_layout_groups (
    id integer NOT NULL,
    resource_id integer,
    group_name character varying(100) NOT NULL,
    display_order integer DEFAULT 0,
    is_collapsible boolean DEFAULT false,
    initially_collapsed boolean DEFAULT false,
    group_description text
);


--
-- Name: ui_layout_groups_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.ui_layout_groups_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ui_layout_groups_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.ui_layout_groups_id_seq OWNED BY public.ui_layout_groups.id;


--
-- Name: user_saved_filters; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_saved_filters (
    id integer NOT NULL,
    user_id character varying(100) NOT NULL,
    filter_name character varying(200) NOT NULL,
    filter_criteria jsonb NOT NULL,
    applied_clusters integer[],
    result_count integer,
    last_used timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    is_favorite boolean DEFAULT false,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE user_saved_filters; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.user_saved_filters IS 'User-saved filter preferences and frequently used filters';


--
-- Name: user_saved_filters_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_saved_filters_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_saved_filters_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_saved_filters_id_seq OWNED BY public.user_saved_filters.id;


--
-- Name: v_attribute_dependencies; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_attribute_dependencies AS
 SELECT r.rule_id,
    r.rule_name,
    (((da.entity_name)::text || '.'::text) || (da.attribute_name)::text) AS target_attribute,
    array_agg((((ba.entity_name)::text || '.'::text) || (ba.attribute_name)::text) ORDER BY ba.attribute_name) AS source_attributes
   FROM (((public.rules r
     LEFT JOIN public.derived_attributes da ON ((r.target_attribute_id = da.id)))
     LEFT JOIN public.rule_dependencies rd ON ((rd.rule_id = r.id)))
     LEFT JOIN public.business_attributes ba ON ((rd.attribute_id = ba.id)))
  GROUP BY r.id, r.rule_id, r.rule_name, da.entity_name, da.attribute_name;


--
-- Name: v_cbu_members_detail; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_cbu_members_detail AS
 SELECT cm.id,
    cbu.cbu_id,
    cbu.cbu_name,
    cr.role_code,
    cr.role_name,
    cr.role_category,
    cm.entity_id,
    cm.entity_name,
    cm.entity_lei,
    cm.is_primary,
    cm.effective_date,
    cm.expiry_date,
    cm.contact_email,
    cm.is_active,
    cm.has_trading_authority,
    cm.has_settlement_authority,
    cm.notes,
    cm.created_at,
    cm.updated_at
   FROM ((public.cbu_members cm
     JOIN public.client_business_units cbu ON ((cm.cbu_id = cbu.id)))
     JOIN public.cbu_roles cr ON ((cm.role_id = cr.id)))
  ORDER BY cbu.cbu_name, cr.display_order, cm.entity_name;


--
-- Name: v_cbu_product_subscriptions; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_cbu_product_subscriptions AS
 SELECT cbu.cbu_id,
    cbu.cbu_name,
    p.product_id,
    p.product_name,
    p.line_of_business,
    cps.subscription_status,
    cps.subscription_date,
    cps.activation_date,
    cr.role_name AS primary_contact_role
   FROM (((public.client_business_units cbu
     JOIN public.cbu_product_subscriptions cps ON ((cbu.id = cps.cbu_id)))
     JOIN public.products p ON ((cps.product_id = p.id)))
     LEFT JOIN public.cbu_roles cr ON ((cps.primary_contact_role_id = cr.id)))
  ORDER BY cbu.cbu_name, p.product_name;


--
-- Name: v_cbu_roles_taxonomy; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_cbu_roles_taxonomy AS
 SELECT cr.id,
    cr.role_code,
    cr.role_name,
    cr.description,
    cr.role_category,
    cr.display_order,
    cr.is_active,
    count(DISTINCT cm.cbu_id) AS usage_count
   FROM (public.cbu_roles cr
     LEFT JOIN public.cbu_members cm ON (((cr.id = cm.role_id) AND (cm.is_active = true))))
  GROUP BY cr.id, cr.role_code, cr.role_name, cr.description, cr.role_category, cr.display_order, cr.is_active
  ORDER BY cr.role_category, cr.display_order;


--
-- Name: v_cbu_summary; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_cbu_summary AS
SELECT
    NULL::integer AS id,
    NULL::character varying(100) AS cbu_id,
    NULL::character varying(255) AS cbu_name,
    NULL::text AS description,
    NULL::character varying(20) AS primary_lei,
    NULL::character(2) AS domicile_country,
    NULL::character varying(50) AS business_type,
    NULL::character varying(20) AS status,
    NULL::date AS created_date,
    NULL::bigint AS member_count,
    NULL::bigint AS role_count,
    NULL::text AS roles,
    NULL::timestamp with time zone AS created_at,
    NULL::timestamp with time zone AS updated_at;


--
-- Name: v_mandate_summary; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_mandate_summary AS
SELECT
    NULL::character varying(100) AS mandate_id,
    NULL::character varying(100) AS cbu_id,
    NULL::character varying(255) AS asset_owner_name,
    NULL::character varying(20) AS asset_owner_lei,
    NULL::character varying(255) AS investment_manager_name,
    NULL::character varying(20) AS investment_manager_lei,
    NULL::character(3) AS base_currency,
    NULL::date AS effective_date,
    NULL::date AS expiry_date,
    NULL::bigint AS instrument_count,
    NULL::bigint AS benchmark_count,
    NULL::timestamp with time zone AS created_at,
    NULL::timestamp with time zone AS updated_at;


--
-- Name: v_onboarding_progress; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_onboarding_progress AS
SELECT
    NULL::character varying(100) AS request_id,
    NULL::character varying(255) AS cbu_name,
    NULL::character varying(255) AS product_name,
    NULL::character varying(20) AS request_status,
    NULL::date AS target_go_live_date,
    NULL::bigint AS total_tasks,
    NULL::bigint AS completed_tasks,
    NULL::bigint AS blocked_tasks,
    NULL::numeric AS completion_percentage;


--
-- Name: VIEW v_onboarding_progress; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON VIEW public.v_onboarding_progress IS 'Real-time onboarding progress tracking with completion metrics';


--
-- Name: v_onboarding_task_details; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_onboarding_task_details AS
 SELECT ort.id AS task_id,
    ow.workflow_id,
    cbu.cbu_name,
    rt.template_name AS resource_template_name,
    rt.resource_type,
    rc.capability_name,
    rc.capability_type,
    ort.task_order,
    ort.task_status,
    ort.assigned_to,
    ort.started_at,
    ort.completed_at,
    ort.estimated_hours,
    ort.actual_hours,
    ort.blocking_issues,
    ort.retry_count,
    ort.created_at
   FROM ((((public.onboarding_resource_tasks ort
     JOIN public.onboarding_workflows ow ON ((ort.workflow_id = ow.id)))
     JOIN public.client_business_units cbu ON ((ow.cbu_id = cbu.id)))
     JOIN public.resource_templates rt ON ((ort.resource_template_id = rt.id)))
     JOIN public.resource_capabilities rc ON ((ort.capability_id = rc.id)))
  ORDER BY ow.workflow_id, ort.task_order;


--
-- Name: v_product_hierarchy; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_product_hierarchy AS
 SELECT p.product_id,
    p.product_name,
    p.line_of_business,
    p.status AS product_status,
    s.service_id,
    s.service_name,
    s.service_category,
    ps.is_required AS service_required,
    r.resource_id,
    r.resource_name,
    r.resource_type,
    sr.resource_role,
    sr.priority AS resource_priority
   FROM ((((public.products p
     JOIN public.product_services ps ON ((p.id = ps.product_id)))
     JOIN public.services s ON ((ps.service_id = s.id)))
     JOIN public.service_resources sr ON ((s.id = sr.service_id)))
     JOIN public.resources r ON ((sr.resource_id = r.id)))
  WHERE (((p.status)::text = 'active'::text) AND ((s.status)::text = 'active'::text) AND ((r.status)::text = 'active'::text))
  ORDER BY p.product_name, ps.display_order, sr.priority;


--
-- Name: VIEW v_product_hierarchy; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON VIEW public.v_product_hierarchy IS 'Complete view of product-service-resource relationships';


--
-- Name: v_product_service_resource_hierarchy; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_product_service_resource_hierarchy AS
 SELECT p.product_id,
    p.product_name,
    p.line_of_business,
    s.service_id,
    s.service_name,
    s.service_category,
    psm.is_mandatory,
    psm.delivery_sequence,
    r.id AS resource_id,
    r.resource_name,
    r.resource_type,
    srm.resource_role,
    srm.dependency_level AS resource_priority,
    rt.template_id,
    rt.template_name,
    rt.dsl_template
   FROM (((((public.products p
     LEFT JOIN public.product_service_mappings psm ON ((p.id = psm.product_id)))
     LEFT JOIN public.services s ON ((psm.service_id = s.id)))
     LEFT JOIN public.service_resource_mappings srm ON ((s.id = srm.service_id)))
     LEFT JOIN public.resource_objects r ON ((srm.resource_id = r.id)))
     LEFT JOIN public.resource_templates rt ON (((r.resource_type)::text = (rt.resource_type)::text)))
  WHERE (((p.status)::text = 'active'::text) AND ((s.status IS NULL) OR ((s.status)::text = 'active'::text)) AND ((r.status IS NULL) OR ((r.status)::text = 'active'::text)))
  ORDER BY p.product_name, psm.delivery_sequence, srm.dependency_level;


--
-- Name: v_resource_provisioning_status; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_resource_provisioning_status AS
 SELECT ow.workflow_id,
    rt.template_name AS resource_template_name,
    rt.resource_type,
    count(ort.*) AS total_capabilities,
    count(
        CASE
            WHEN ((ort.task_status)::text = 'completed'::text) THEN 1
            ELSE NULL::integer
        END) AS completed_capabilities,
    count(
        CASE
            WHEN ((ort.task_status)::text = 'failed'::text) THEN 1
            ELSE NULL::integer
        END) AS failed_capabilities,
    ( SELECT rc.capability_name
           FROM (public.resource_capabilities rc
             JOIN public.onboarding_resource_tasks ort2 ON ((rc.id = ort2.capability_id)))
          WHERE ((ort2.workflow_id = ow.id) AND ((ort2.task_status)::text = 'in_progress'::text))
         LIMIT 1) AS current_capability,
        CASE
            WHEN (count(
            CASE
                WHEN ((ort.task_status)::text = 'failed'::text) THEN 1
                ELSE NULL::integer
            END) > 0) THEN 'failed'::text
            WHEN (count(
            CASE
                WHEN ((ort.task_status)::text = 'completed'::text) THEN 1
                ELSE NULL::integer
            END) = count(ort.*)) THEN 'completed'::text
            WHEN (count(
            CASE
                WHEN ((ort.task_status)::text = ANY ((ARRAY['in_progress'::character varying, 'completed'::character varying])::text[])) THEN 1
                ELSE NULL::integer
            END) > 0) THEN 'in_progress'::text
            ELSE 'pending'::text
        END AS provision_status
   FROM ((public.onboarding_workflows ow
     JOIN public.onboarding_resource_tasks ort ON ((ow.id = ort.workflow_id)))
     JOIN public.resource_templates rt ON ((ort.resource_template_id = rt.id)))
  GROUP BY ow.workflow_id, rt.template_name, rt.resource_type, ow.id
  ORDER BY ow.workflow_id, rt.template_name;


--
-- Name: v_resource_template_capabilities; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_resource_template_capabilities AS
 SELECT rt.template_id,
    rt.template_name,
    rt.part_of_product,
    rt.implements_service,
    rc.capability_id,
    rc.capability_name,
    rc.capability_type,
    rc.required_attributes,
    rc.optional_attributes,
    rc.output_attributes,
    rtc.capability_order,
    rtc.is_required AS capability_required,
    rtc.configuration_overrides
   FROM ((public.resource_templates rt
     JOIN public.resource_template_capabilities rtc ON ((rt.id = rtc.template_id)))
     JOIN public.resource_capabilities rc ON ((rtc.capability_id = rc.id)))
  WHERE (((rt.status)::text = 'active'::text) AND ((rc.status)::text = 'active'::text))
  ORDER BY rt.template_name, rtc.capability_order;


--
-- Name: ai_attribute_contexts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_attribute_contexts ALTER COLUMN id SET DEFAULT nextval('public.ai_attribute_contexts_id_seq'::regclass);


--
-- Name: ai_metadata_contexts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_metadata_contexts ALTER COLUMN id SET DEFAULT nextval('public.ai_metadata_contexts_id_seq'::regclass);


--
-- Name: ai_prompt_contexts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_prompt_contexts ALTER COLUMN id SET DEFAULT nextval('public.ai_prompt_contexts_id_seq'::regclass);


--
-- Name: ai_training_examples id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_training_examples ALTER COLUMN id SET DEFAULT nextval('public.ai_training_examples_id_seq'::regclass);


--
-- Name: attribute_cluster_memberships id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_cluster_memberships ALTER COLUMN id SET DEFAULT nextval('public.attribute_cluster_memberships_id_seq'::regclass);


--
-- Name: attribute_documentation id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_documentation ALTER COLUMN id SET DEFAULT nextval('public.attribute_documentation_id_seq'::regclass);


--
-- Name: attribute_domain_mappings id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_domain_mappings ALTER COLUMN id SET DEFAULT nextval('public.attribute_domain_mappings_id_seq'::regclass);


--
-- Name: attribute_filter_configurations id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_filter_configurations ALTER COLUMN id SET DEFAULT nextval('public.attribute_filter_configurations_id_seq'::regclass);


--
-- Name: attribute_lineage id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_lineage ALTER COLUMN id SET DEFAULT nextval('public.attribute_lineage_id_seq'::regclass);


--
-- Name: attribute_objects id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_objects ALTER COLUMN id SET DEFAULT nextval('public.attribute_objects_id_seq'::regclass);


--
-- Name: attribute_persistence_mappings id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_persistence_mappings ALTER COLUMN id SET DEFAULT nextval('public.attribute_persistence_mappings_id_seq'::regclass);


--
-- Name: attribute_perspectives id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_perspectives ALTER COLUMN id SET DEFAULT nextval('public.attribute_perspectives_id_seq'::regclass);


--
-- Name: attribute_relationships id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_relationships ALTER COLUMN id SET DEFAULT nextval('public.attribute_relationships_id_seq'::regclass);


--
-- Name: attribute_semantic_relationships id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_semantic_relationships ALTER COLUMN id SET DEFAULT nextval('public.attribute_semantic_relationships_id_seq'::regclass);


--
-- Name: attribute_sources id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_sources ALTER COLUMN id SET DEFAULT nextval('public.attribute_sources_id_seq'::regclass);


--
-- Name: attribute_tag_assignments id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_tag_assignments ALTER COLUMN id SET DEFAULT nextval('public.attribute_tag_assignments_id_seq'::regclass);


--
-- Name: attribute_tags id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_tags ALTER COLUMN id SET DEFAULT nextval('public.attribute_tags_id_seq'::regclass);


--
-- Name: attribute_terminology_links id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_terminology_links ALTER COLUMN id SET DEFAULT nextval('public.attribute_terminology_links_id_seq'::regclass);


--
-- Name: attribute_value_audit id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_value_audit ALTER COLUMN id SET DEFAULT nextval('public.attribute_value_audit_id_seq'::regclass);


--
-- Name: attribute_vector_clusters id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_vector_clusters ALTER COLUMN id SET DEFAULT nextval('public.attribute_vector_clusters_id_seq'::regclass);


--
-- Name: business_attributes id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.business_attributes ALTER COLUMN id SET DEFAULT nextval('public.business_attributes_id_seq'::regclass);


--
-- Name: cbu_entity_associations id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_entity_associations ALTER COLUMN id SET DEFAULT nextval('public.cbu_entity_associations_id_seq'::regclass);


--
-- Name: cbu_members id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_members ALTER COLUMN id SET DEFAULT nextval('public.cbu_members_id_seq'::regclass);


--
-- Name: cbu_product_subscriptions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_product_subscriptions ALTER COLUMN id SET DEFAULT nextval('public.cbu_product_subscriptions_id_seq'::regclass);


--
-- Name: cbu_roles id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_roles ALTER COLUMN id SET DEFAULT nextval('public.cbu_roles_id_seq'::regclass);


--
-- Name: cbu_service_resources id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_service_resources ALTER COLUMN id SET DEFAULT nextval('public.cbu_service_resources_id_seq'::regclass);


--
-- Name: client_business_units id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.client_business_units ALTER COLUMN id SET DEFAULT nextval('public.client_business_units_id_seq'::regclass);


--
-- Name: commercial_contracts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.commercial_contracts ALTER COLUMN id SET DEFAULT nextval('public.commercial_contracts_id_seq'::regclass);


--
-- Name: data_domains id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.data_domains ALTER COLUMN id SET DEFAULT nextval('public.data_domains_id_seq'::regclass);


--
-- Name: derivation_execution_log id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derivation_execution_log ALTER COLUMN id SET DEFAULT nextval('public.derivation_execution_log_id_seq'::regclass);


--
-- Name: derived_attribute_quality_metrics id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derived_attribute_quality_metrics ALTER COLUMN id SET DEFAULT nextval('public.derived_attribute_quality_metrics_id_seq'::regclass);


--
-- Name: derived_attribute_rules id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derived_attribute_rules ALTER COLUMN id SET DEFAULT nextval('public.derived_attribute_rules_id_seq'::regclass);


--
-- Name: derived_attributes id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derived_attributes ALTER COLUMN id SET DEFAULT nextval('public.derived_attributes_id_seq'::regclass);


--
-- Name: domain_terminology_glossary id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.domain_terminology_glossary ALTER COLUMN id SET DEFAULT nextval('public.domain_terminology_glossary_id_seq'::regclass);


--
-- Name: ebnf_grammar_templates id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ebnf_grammar_templates ALTER COLUMN id SET DEFAULT nextval('public.ebnf_grammar_templates_id_seq'::regclass);


--
-- Name: entity_attribute_values id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.entity_attribute_values ALTER COLUMN id SET DEFAULT nextval('public.entity_attribute_values_id_seq'::regclass);


--
-- Name: entity_relationships id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.entity_relationships ALTER COLUMN id SET DEFAULT nextval('public.entity_relationships_id_seq'::regclass);


--
-- Name: form_layout_templates id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.form_layout_templates ALTER COLUMN id SET DEFAULT nextval('public.form_layout_templates_id_seq'::regclass);


--
-- Name: grammar_extensions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.grammar_extensions ALTER COLUMN id SET DEFAULT nextval('public.grammar_extensions_id_seq'::regclass);


--
-- Name: grammar_metadata id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.grammar_metadata ALTER COLUMN id SET DEFAULT nextval('public.grammar_metadata_id_seq'::regclass);


--
-- Name: grammar_rules id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.grammar_rules ALTER COLUMN id SET DEFAULT nextval('public.grammar_rules_id_seq'::regclass);


--
-- Name: instruction_formats id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.instruction_formats ALTER COLUMN id SET DEFAULT nextval('public.instruction_formats_id_seq'::regclass);


--
-- Name: instrument_taxonomy id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.instrument_taxonomy ALTER COLUMN id SET DEFAULT nextval('public.instrument_taxonomy_id_seq'::regclass);


--
-- Name: kyc_onboarding_domains id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.kyc_onboarding_domains ALTER COLUMN id SET DEFAULT nextval('public.kyc_onboarding_domains_id_seq'::regclass);


--
-- Name: legal_entities id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.legal_entities ALTER COLUMN id SET DEFAULT nextval('public.legal_entities_id_seq'::regclass);


--
-- Name: mandate_benchmarks id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_benchmarks ALTER COLUMN id SET DEFAULT nextval('public.mandate_benchmarks_id_seq'::regclass);


--
-- Name: mandate_instruction_channels id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instruction_channels ALTER COLUMN id SET DEFAULT nextval('public.mandate_instruction_channels_id_seq'::regclass);


--
-- Name: mandate_instrument_identifiers id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instrument_identifiers ALTER COLUMN id SET DEFAULT nextval('public.mandate_instrument_identifiers_id_seq'::regclass);


--
-- Name: mandate_instrument_venues id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instrument_venues ALTER COLUMN id SET DEFAULT nextval('public.mandate_instrument_venues_id_seq'::regclass);


--
-- Name: mandate_instruments id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instruments ALTER COLUMN id SET DEFAULT nextval('public.mandate_instruments_id_seq'::regclass);


--
-- Name: mandate_lifecycle_events id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_lifecycle_events ALTER COLUMN id SET DEFAULT nextval('public.mandate_lifecycle_events_id_seq'::regclass);


--
-- Name: onboarding_approvals id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_approvals ALTER COLUMN id SET DEFAULT nextval('public.onboarding_approvals_id_seq'::regclass);


--
-- Name: onboarding_dependencies id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_dependencies ALTER COLUMN id SET DEFAULT nextval('public.onboarding_dependencies_id_seq'::regclass);


--
-- Name: onboarding_requests id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_requests ALTER COLUMN id SET DEFAULT nextval('public.onboarding_requests_id_seq'::regclass);


--
-- Name: onboarding_resource_tasks id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_resource_tasks ALTER COLUMN id SET DEFAULT nextval('public.onboarding_resource_tasks_id_seq'::regclass);


--
-- Name: onboarding_tasks id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_tasks ALTER COLUMN id SET DEFAULT nextval('public.onboarding_tasks_id_seq'::regclass);


--
-- Name: onboarding_workflows id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_workflows ALTER COLUMN id SET DEFAULT nextval('public.onboarding_workflows_id_seq'::regclass);


--
-- Name: persistence_entities id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.persistence_entities ALTER COLUMN id SET DEFAULT nextval('public.persistence_entities_id_seq'::regclass);


--
-- Name: persistence_systems id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.persistence_systems ALTER COLUMN id SET DEFAULT nextval('public.persistence_systems_id_seq'::regclass);


--
-- Name: product_option_service_mappings id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_option_service_mappings ALTER COLUMN id SET DEFAULT nextval('public.product_option_service_mappings_id_seq'::regclass);


--
-- Name: product_options id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_options ALTER COLUMN id SET DEFAULT nextval('public.product_options_id_seq'::regclass);


--
-- Name: product_service_mappings id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_service_mappings ALTER COLUMN id SET DEFAULT nextval('public.product_service_mappings_id_seq'::regclass);


--
-- Name: product_services id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_services ALTER COLUMN id SET DEFAULT nextval('public.product_services_id_seq'::regclass);


--
-- Name: products id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products ALTER COLUMN id SET DEFAULT nextval('public.products_id_seq'::regclass);


--
-- Name: resource_attribute_dependencies id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_attribute_dependencies ALTER COLUMN id SET DEFAULT nextval('public.resource_attribute_dependencies_id_seq'::regclass);


--
-- Name: resource_attribute_templates id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_attribute_templates ALTER COLUMN id SET DEFAULT nextval('public.resource_attribute_templates_id_seq'::regclass);


--
-- Name: resource_attribute_versions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_attribute_versions ALTER COLUMN id SET DEFAULT nextval('public.resource_attribute_versions_id_seq'::regclass);


--
-- Name: resource_capabilities id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_capabilities ALTER COLUMN id SET DEFAULT nextval('public.resource_capabilities_id_seq'::regclass);


--
-- Name: resource_dependencies id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_dependencies ALTER COLUMN id SET DEFAULT nextval('public.resource_dependencies_id_seq'::regclass);


--
-- Name: resource_dictionaries id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_dictionaries ALTER COLUMN id SET DEFAULT nextval('public.resource_dictionaries_id_seq'::regclass);


--
-- Name: resource_objects id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_objects ALTER COLUMN id SET DEFAULT nextval('public.resource_objects_id_seq'::regclass);


--
-- Name: resource_sheet_execution_logs log_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_sheet_execution_logs ALTER COLUMN log_id SET DEFAULT nextval('public.resource_sheet_execution_logs_log_id_seq'::regclass);


--
-- Name: resource_sheet_relationships relationship_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_sheet_relationships ALTER COLUMN relationship_id SET DEFAULT nextval('public.resource_sheet_relationships_relationship_id_seq'::regclass);


--
-- Name: resource_template_capabilities id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_template_capabilities ALTER COLUMN id SET DEFAULT nextval('public.resource_template_capabilities_id_seq'::regclass);


--
-- Name: resource_templates id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_templates ALTER COLUMN id SET DEFAULT nextval('public.resource_templates_id_seq'::regclass);


--
-- Name: resource_validation_rules id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_validation_rules ALTER COLUMN id SET DEFAULT nextval('public.resource_validation_rules_id_seq'::regclass);


--
-- Name: resources id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resources ALTER COLUMN id SET DEFAULT nextval('public.resources_id_seq'::regclass);


--
-- Name: role_service_access id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.role_service_access ALTER COLUMN id SET DEFAULT nextval('public.role_service_access_id_seq'::regclass);


--
-- Name: rule_categories id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_categories ALTER COLUMN id SET DEFAULT nextval('public.rule_categories_id_seq'::regclass);


--
-- Name: rule_compilation_queue id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_compilation_queue ALTER COLUMN id SET DEFAULT nextval('public.rule_compilation_queue_id_seq'::regclass);


--
-- Name: rule_dependencies id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_dependencies ALTER COLUMN id SET DEFAULT nextval('public.rule_dependencies_id_seq'::regclass);


--
-- Name: rule_execution_stats id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_execution_stats ALTER COLUMN id SET DEFAULT nextval('public.rule_execution_stats_id_seq'::regclass);


--
-- Name: rule_versions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_versions ALTER COLUMN id SET DEFAULT nextval('public.rule_versions_id_seq'::regclass);


--
-- Name: rules id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rules ALTER COLUMN id SET DEFAULT nextval('public.rules_id_seq'::regclass);


--
-- Name: service_resource_mappings id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_resource_mappings ALTER COLUMN id SET DEFAULT nextval('public.service_resource_mappings_id_seq'::regclass);


--
-- Name: service_resources id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_resources ALTER COLUMN id SET DEFAULT nextval('public.service_resources_id_seq'::regclass);


--
-- Name: services id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.services ALTER COLUMN id SET DEFAULT nextval('public.services_id_seq'::regclass);


--
-- Name: ui_component_templates id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ui_component_templates ALTER COLUMN id SET DEFAULT nextval('public.ui_component_templates_id_seq'::regclass);


--
-- Name: ui_layout_groups id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ui_layout_groups ALTER COLUMN id SET DEFAULT nextval('public.ui_layout_groups_id_seq'::regclass);


--
-- Name: user_saved_filters id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_saved_filters ALTER COLUMN id SET DEFAULT nextval('public.user_saved_filters_id_seq'::regclass);


--
-- Name: ai_attribute_contexts ai_attribute_contexts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_attribute_contexts
    ADD CONSTRAINT ai_attribute_contexts_pkey PRIMARY KEY (id);


--
-- Name: ai_metadata_contexts ai_metadata_contexts_context_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_metadata_contexts
    ADD CONSTRAINT ai_metadata_contexts_context_name_key UNIQUE (context_name);


--
-- Name: ai_metadata_contexts ai_metadata_contexts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_metadata_contexts
    ADD CONSTRAINT ai_metadata_contexts_pkey PRIMARY KEY (id);


--
-- Name: ai_prompt_contexts ai_prompt_contexts_context_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_prompt_contexts
    ADD CONSTRAINT ai_prompt_contexts_context_name_key UNIQUE (context_name);


--
-- Name: ai_prompt_contexts ai_prompt_contexts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_prompt_contexts
    ADD CONSTRAINT ai_prompt_contexts_pkey PRIMARY KEY (id);


--
-- Name: ai_training_examples ai_training_examples_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_training_examples
    ADD CONSTRAINT ai_training_examples_pkey PRIMARY KEY (id);


--
-- Name: attribute_cluster_memberships attribute_cluster_memberships_attribute_id_cluster_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_cluster_memberships
    ADD CONSTRAINT attribute_cluster_memberships_attribute_id_cluster_id_key UNIQUE (attribute_id, cluster_id);


--
-- Name: attribute_cluster_memberships attribute_cluster_memberships_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_cluster_memberships
    ADD CONSTRAINT attribute_cluster_memberships_pkey PRIMARY KEY (id);


--
-- Name: attribute_documentation attribute_documentation_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_documentation
    ADD CONSTRAINT attribute_documentation_pkey PRIMARY KEY (id);


--
-- Name: attribute_domain_mappings attribute_domain_mappings_attribute_id_domain_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_domain_mappings
    ADD CONSTRAINT attribute_domain_mappings_attribute_id_domain_id_key UNIQUE (attribute_id, domain_id);


--
-- Name: attribute_domain_mappings attribute_domain_mappings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_domain_mappings
    ADD CONSTRAINT attribute_domain_mappings_pkey PRIMARY KEY (id);


--
-- Name: attribute_filter_configurations attribute_filter_configurations_filter_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_filter_configurations
    ADD CONSTRAINT attribute_filter_configurations_filter_name_key UNIQUE (filter_name);


--
-- Name: attribute_filter_configurations attribute_filter_configurations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_filter_configurations
    ADD CONSTRAINT attribute_filter_configurations_pkey PRIMARY KEY (id);


--
-- Name: attribute_lineage attribute_lineage_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_lineage
    ADD CONSTRAINT attribute_lineage_pkey PRIMARY KEY (id);


--
-- Name: attribute_lineage attribute_lineage_target_attribute_id_source_attribute_id_l_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_lineage
    ADD CONSTRAINT attribute_lineage_target_attribute_id_source_attribute_id_l_key UNIQUE (target_attribute_id, source_attribute_id, lineage_type);


--
-- Name: attribute_objects attribute_objects_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_objects
    ADD CONSTRAINT attribute_objects_pkey PRIMARY KEY (id);


--
-- Name: attribute_objects attribute_objects_resource_id_attribute_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_objects
    ADD CONSTRAINT attribute_objects_resource_id_attribute_name_key UNIQUE (resource_id, attribute_name);


--
-- Name: attribute_persistence_mappings attribute_persistence_mapping_attribute_id_persistence_enti_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_persistence_mappings
    ADD CONSTRAINT attribute_persistence_mapping_attribute_id_persistence_enti_key UNIQUE (attribute_id, persistence_entity_id);


--
-- Name: attribute_persistence_mappings attribute_persistence_mappings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_persistence_mappings
    ADD CONSTRAINT attribute_persistence_mappings_pkey PRIMARY KEY (id);


--
-- Name: attribute_perspectives attribute_perspectives_attribute_id_perspective_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_perspectives
    ADD CONSTRAINT attribute_perspectives_attribute_id_perspective_name_key UNIQUE (attribute_id, perspective_name);


--
-- Name: attribute_perspectives attribute_perspectives_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_perspectives
    ADD CONSTRAINT attribute_perspectives_pkey PRIMARY KEY (id);


--
-- Name: attribute_relationships attribute_relationships_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_relationships
    ADD CONSTRAINT attribute_relationships_pkey PRIMARY KEY (id);


--
-- Name: attribute_relationships attribute_relationships_source_attribute_id_target_attribut_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_relationships
    ADD CONSTRAINT attribute_relationships_source_attribute_id_target_attribut_key UNIQUE (source_attribute_id, target_attribute_id, relationship_type);


--
-- Name: attribute_semantic_relationships attribute_semantic_relationships_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_semantic_relationships
    ADD CONSTRAINT attribute_semantic_relationships_pkey PRIMARY KEY (id);


--
-- Name: attribute_sources attribute_sources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_sources
    ADD CONSTRAINT attribute_sources_pkey PRIMARY KEY (id);


--
-- Name: attribute_sources attribute_sources_source_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_sources
    ADD CONSTRAINT attribute_sources_source_key_key UNIQUE (source_key);


--
-- Name: attribute_tag_assignments attribute_tag_assignments_attribute_id_tag_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_tag_assignments
    ADD CONSTRAINT attribute_tag_assignments_attribute_id_tag_id_key UNIQUE (attribute_id, tag_id);


--
-- Name: attribute_tag_assignments attribute_tag_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_tag_assignments
    ADD CONSTRAINT attribute_tag_assignments_pkey PRIMARY KEY (id);


--
-- Name: attribute_tags attribute_tags_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_tags
    ADD CONSTRAINT attribute_tags_pkey PRIMARY KEY (id);


--
-- Name: attribute_tags attribute_tags_tag_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_tags
    ADD CONSTRAINT attribute_tags_tag_name_key UNIQUE (tag_name);


--
-- Name: attribute_terminology_links attribute_terminology_links_attribute_id_term_id_relationsh_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_terminology_links
    ADD CONSTRAINT attribute_terminology_links_attribute_id_term_id_relationsh_key UNIQUE (attribute_id, term_id, relationship_type);


--
-- Name: attribute_terminology_links attribute_terminology_links_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_terminology_links
    ADD CONSTRAINT attribute_terminology_links_pkey PRIMARY KEY (id);


--
-- Name: attribute_value_audit attribute_value_audit_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_value_audit
    ADD CONSTRAINT attribute_value_audit_pkey PRIMARY KEY (id);


--
-- Name: attribute_vector_clusters attribute_vector_clusters_cluster_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_vector_clusters
    ADD CONSTRAINT attribute_vector_clusters_cluster_name_key UNIQUE (cluster_name);


--
-- Name: attribute_vector_clusters attribute_vector_clusters_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_vector_clusters
    ADD CONSTRAINT attribute_vector_clusters_pkey PRIMARY KEY (id);


--
-- Name: business_attributes business_attributes_entity_name_attribute_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.business_attributes
    ADD CONSTRAINT business_attributes_entity_name_attribute_name_key UNIQUE (entity_name, attribute_name);


--
-- Name: business_attributes business_attributes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.business_attributes
    ADD CONSTRAINT business_attributes_pkey PRIMARY KEY (id);


--
-- Name: cbu_entity_associations cbu_entity_associations_cbu_id_entity_id_association_type_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_entity_associations
    ADD CONSTRAINT cbu_entity_associations_cbu_id_entity_id_association_type_key UNIQUE (cbu_id, entity_id, association_type);


--
-- Name: cbu_entity_associations cbu_entity_associations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_entity_associations
    ADD CONSTRAINT cbu_entity_associations_pkey PRIMARY KEY (id);


--
-- Name: cbu_members cbu_members_cbu_id_entity_id_role_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_members
    ADD CONSTRAINT cbu_members_cbu_id_entity_id_role_id_key UNIQUE (cbu_id, entity_id, role_id);


--
-- Name: cbu_members cbu_members_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_members
    ADD CONSTRAINT cbu_members_pkey PRIMARY KEY (id);


--
-- Name: cbu_product_subscriptions cbu_product_subscriptions_cbu_id_product_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_product_subscriptions
    ADD CONSTRAINT cbu_product_subscriptions_cbu_id_product_id_key UNIQUE (cbu_id, product_id);


--
-- Name: cbu_product_subscriptions cbu_product_subscriptions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_product_subscriptions
    ADD CONSTRAINT cbu_product_subscriptions_pkey PRIMARY KEY (id);


--
-- Name: cbu_roles cbu_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_roles
    ADD CONSTRAINT cbu_roles_pkey PRIMARY KEY (id);


--
-- Name: cbu_roles cbu_roles_role_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_roles
    ADD CONSTRAINT cbu_roles_role_code_key UNIQUE (role_code);


--
-- Name: cbu_service_resources cbu_service_resources_cbu_id_service_id_resource_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_service_resources
    ADD CONSTRAINT cbu_service_resources_cbu_id_service_id_resource_id_key UNIQUE (cbu_id, service_id, resource_id);


--
-- Name: cbu_service_resources cbu_service_resources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_service_resources
    ADD CONSTRAINT cbu_service_resources_pkey PRIMARY KEY (id);


--
-- Name: client_business_units client_business_units_cbu_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.client_business_units
    ADD CONSTRAINT client_business_units_cbu_id_key UNIQUE (cbu_id);


--
-- Name: client_business_units client_business_units_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.client_business_units
    ADD CONSTRAINT client_business_units_pkey PRIMARY KEY (id);


--
-- Name: commercial_contracts commercial_contracts_contract_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.commercial_contracts
    ADD CONSTRAINT commercial_contracts_contract_id_key UNIQUE (contract_id);


--
-- Name: commercial_contracts commercial_contracts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.commercial_contracts
    ADD CONSTRAINT commercial_contracts_pkey PRIMARY KEY (id);


--
-- Name: data_domains data_domains_domain_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.data_domains
    ADD CONSTRAINT data_domains_domain_name_key UNIQUE (domain_name);


--
-- Name: data_domains data_domains_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.data_domains
    ADD CONSTRAINT data_domains_pkey PRIMARY KEY (id);


--
-- Name: derivation_execution_log derivation_execution_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derivation_execution_log
    ADD CONSTRAINT derivation_execution_log_pkey PRIMARY KEY (id);


--
-- Name: derived_attribute_quality_metrics derived_attribute_quality_metrics_attribute_id_metric_date_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derived_attribute_quality_metrics
    ADD CONSTRAINT derived_attribute_quality_metrics_attribute_id_metric_date_key UNIQUE (attribute_id, metric_date);


--
-- Name: derived_attribute_quality_metrics derived_attribute_quality_metrics_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derived_attribute_quality_metrics
    ADD CONSTRAINT derived_attribute_quality_metrics_pkey PRIMARY KEY (id);


--
-- Name: derived_attribute_rules derived_attribute_rules_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derived_attribute_rules
    ADD CONSTRAINT derived_attribute_rules_pkey PRIMARY KEY (id);


--
-- Name: derived_attributes derived_attributes_entity_name_attribute_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derived_attributes
    ADD CONSTRAINT derived_attributes_entity_name_attribute_name_key UNIQUE (entity_name, attribute_name);


--
-- Name: derived_attributes derived_attributes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derived_attributes
    ADD CONSTRAINT derived_attributes_pkey PRIMARY KEY (id);


--
-- Name: domain_terminology_glossary domain_terminology_glossary_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.domain_terminology_glossary
    ADD CONSTRAINT domain_terminology_glossary_pkey PRIMARY KEY (id);


--
-- Name: domain_terminology_glossary domain_terminology_glossary_term_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.domain_terminology_glossary
    ADD CONSTRAINT domain_terminology_glossary_term_key UNIQUE (term);


--
-- Name: dsl_execution_logs dsl_execution_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dsl_execution_logs
    ADD CONSTRAINT dsl_execution_logs_pkey PRIMARY KEY (execution_id);


--
-- Name: ebnf_grammar_templates ebnf_grammar_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ebnf_grammar_templates
    ADD CONSTRAINT ebnf_grammar_templates_pkey PRIMARY KEY (id);


--
-- Name: ebnf_grammar_templates ebnf_grammar_templates_template_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ebnf_grammar_templates
    ADD CONSTRAINT ebnf_grammar_templates_template_name_key UNIQUE (template_name);


--
-- Name: entity_attribute_values entity_attribute_values_entity_id_attribute_id_effective_fr_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.entity_attribute_values
    ADD CONSTRAINT entity_attribute_values_entity_id_attribute_id_effective_fr_key UNIQUE (entity_id, attribute_id, effective_from);


--
-- Name: entity_attribute_values entity_attribute_values_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.entity_attribute_values
    ADD CONSTRAINT entity_attribute_values_pkey PRIMARY KEY (id);


--
-- Name: entity_relationships entity_relationships_parent_entity_id_child_entity_id_relat_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.entity_relationships
    ADD CONSTRAINT entity_relationships_parent_entity_id_child_entity_id_relat_key UNIQUE (parent_entity_id, child_entity_id, relationship_type);


--
-- Name: entity_relationships entity_relationships_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.entity_relationships
    ADD CONSTRAINT entity_relationships_pkey PRIMARY KEY (id);


--
-- Name: form_layout_templates form_layout_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.form_layout_templates
    ADD CONSTRAINT form_layout_templates_pkey PRIMARY KEY (id);


--
-- Name: form_layout_templates form_layout_templates_template_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.form_layout_templates
    ADD CONSTRAINT form_layout_templates_template_name_key UNIQUE (template_name);


--
-- Name: grammar_extensions grammar_extensions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.grammar_extensions
    ADD CONSTRAINT grammar_extensions_pkey PRIMARY KEY (id);


--
-- Name: grammar_metadata grammar_metadata_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.grammar_metadata
    ADD CONSTRAINT grammar_metadata_pkey PRIMARY KEY (id);


--
-- Name: grammar_rules grammar_rules_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.grammar_rules
    ADD CONSTRAINT grammar_rules_name_key UNIQUE (name);


--
-- Name: grammar_rules grammar_rules_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.grammar_rules
    ADD CONSTRAINT grammar_rules_pkey PRIMARY KEY (id);


--
-- Name: instruction_formats instruction_formats_format_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.instruction_formats
    ADD CONSTRAINT instruction_formats_format_id_key UNIQUE (format_id);


--
-- Name: instruction_formats instruction_formats_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.instruction_formats
    ADD CONSTRAINT instruction_formats_pkey PRIMARY KEY (id);


--
-- Name: instrument_taxonomy instrument_taxonomy_instrument_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.instrument_taxonomy
    ADD CONSTRAINT instrument_taxonomy_instrument_code_key UNIQUE (instrument_code);


--
-- Name: instrument_taxonomy instrument_taxonomy_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.instrument_taxonomy
    ADD CONSTRAINT instrument_taxonomy_pkey PRIMARY KEY (id);


--
-- Name: investment_mandates investment_mandates_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.investment_mandates
    ADD CONSTRAINT investment_mandates_pkey PRIMARY KEY (mandate_id);


--
-- Name: kyc_onboarding_domains kyc_onboarding_domains_domain_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.kyc_onboarding_domains
    ADD CONSTRAINT kyc_onboarding_domains_domain_name_key UNIQUE (domain_name);


--
-- Name: kyc_onboarding_domains kyc_onboarding_domains_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.kyc_onboarding_domains
    ADD CONSTRAINT kyc_onboarding_domains_pkey PRIMARY KEY (id);


--
-- Name: legal_entities legal_entities_entity_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.legal_entities
    ADD CONSTRAINT legal_entities_entity_id_key UNIQUE (entity_id);


--
-- Name: legal_entities legal_entities_lei_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.legal_entities
    ADD CONSTRAINT legal_entities_lei_code_key UNIQUE (lei_code);


--
-- Name: legal_entities legal_entities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.legal_entities
    ADD CONSTRAINT legal_entities_pkey PRIMARY KEY (id);


--
-- Name: mandate_benchmarks mandate_benchmarks_mandate_id_benchmark_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_benchmarks
    ADD CONSTRAINT mandate_benchmarks_mandate_id_benchmark_name_key UNIQUE (mandate_id, benchmark_name);


--
-- Name: mandate_benchmarks mandate_benchmarks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_benchmarks
    ADD CONSTRAINT mandate_benchmarks_pkey PRIMARY KEY (id);


--
-- Name: mandate_instruction_channels mandate_instruction_channels_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instruction_channels
    ADD CONSTRAINT mandate_instruction_channels_pkey PRIMARY KEY (id);


--
-- Name: mandate_instrument_identifiers mandate_instrument_identifier_instrument_id_identifier_type_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instrument_identifiers
    ADD CONSTRAINT mandate_instrument_identifier_instrument_id_identifier_type_key UNIQUE (instrument_id, identifier_type);


--
-- Name: mandate_instrument_identifiers mandate_instrument_identifiers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instrument_identifiers
    ADD CONSTRAINT mandate_instrument_identifiers_pkey PRIMARY KEY (id);


--
-- Name: mandate_instrument_venues mandate_instrument_venues_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instrument_venues
    ADD CONSTRAINT mandate_instrument_venues_pkey PRIMARY KEY (id);


--
-- Name: mandate_instruments mandate_instruments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instruments
    ADD CONSTRAINT mandate_instruments_pkey PRIMARY KEY (id);


--
-- Name: mandate_lifecycle_events mandate_lifecycle_events_instrument_id_event_type_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_lifecycle_events
    ADD CONSTRAINT mandate_lifecycle_events_instrument_id_event_type_key UNIQUE (instrument_id, event_type);


--
-- Name: mandate_lifecycle_events mandate_lifecycle_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_lifecycle_events
    ADD CONSTRAINT mandate_lifecycle_events_pkey PRIMARY KEY (id);


--
-- Name: onboarding_approvals onboarding_approvals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_approvals
    ADD CONSTRAINT onboarding_approvals_pkey PRIMARY KEY (id);


--
-- Name: onboarding_dependencies onboarding_dependencies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_dependencies
    ADD CONSTRAINT onboarding_dependencies_pkey PRIMARY KEY (id);


--
-- Name: onboarding_requests onboarding_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_requests
    ADD CONSTRAINT onboarding_requests_pkey PRIMARY KEY (id);


--
-- Name: onboarding_requests onboarding_requests_request_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_requests
    ADD CONSTRAINT onboarding_requests_request_id_key UNIQUE (request_id);


--
-- Name: onboarding_resource_tasks onboarding_resource_tasks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_resource_tasks
    ADD CONSTRAINT onboarding_resource_tasks_pkey PRIMARY KEY (id);


--
-- Name: onboarding_tasks onboarding_tasks_onboarding_request_id_task_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_tasks
    ADD CONSTRAINT onboarding_tasks_onboarding_request_id_task_id_key UNIQUE (onboarding_request_id, task_id);


--
-- Name: onboarding_tasks onboarding_tasks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_tasks
    ADD CONSTRAINT onboarding_tasks_pkey PRIMARY KEY (id);


--
-- Name: onboarding_workflows onboarding_workflows_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_workflows
    ADD CONSTRAINT onboarding_workflows_pkey PRIMARY KEY (id);


--
-- Name: onboarding_workflows onboarding_workflows_workflow_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_workflows
    ADD CONSTRAINT onboarding_workflows_workflow_id_key UNIQUE (workflow_id);


--
-- Name: persistence_entities persistence_entities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.persistence_entities
    ADD CONSTRAINT persistence_entities_pkey PRIMARY KEY (id);


--
-- Name: persistence_entities persistence_entities_system_id_entity_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.persistence_entities
    ADD CONSTRAINT persistence_entities_system_id_entity_name_key UNIQUE (system_id, entity_name);


--
-- Name: persistence_systems persistence_systems_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.persistence_systems
    ADD CONSTRAINT persistence_systems_pkey PRIMARY KEY (id);


--
-- Name: persistence_systems persistence_systems_system_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.persistence_systems
    ADD CONSTRAINT persistence_systems_system_name_key UNIQUE (system_name);


--
-- Name: product_option_service_mappings product_option_service_mappin_product_option_id_service_id__key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_option_service_mappings
    ADD CONSTRAINT product_option_service_mappin_product_option_id_service_id__key UNIQUE (product_option_id, service_id, mapping_relationship);


--
-- Name: product_option_service_mappings product_option_service_mappings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_option_service_mappings
    ADD CONSTRAINT product_option_service_mappings_pkey PRIMARY KEY (id);


--
-- Name: product_options product_options_option_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_options
    ADD CONSTRAINT product_options_option_id_key UNIQUE (option_id);


--
-- Name: product_options product_options_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_options
    ADD CONSTRAINT product_options_pkey PRIMARY KEY (id);


--
-- Name: product_service_mappings product_service_mappings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_service_mappings
    ADD CONSTRAINT product_service_mappings_pkey PRIMARY KEY (id);


--
-- Name: product_service_mappings product_service_mappings_product_id_service_id_mapping_type_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_service_mappings
    ADD CONSTRAINT product_service_mappings_product_id_service_id_mapping_type_key UNIQUE (product_id, service_id, mapping_type);


--
-- Name: product_services product_services_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_services
    ADD CONSTRAINT product_services_pkey PRIMARY KEY (id);


--
-- Name: product_services product_services_product_id_service_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_services
    ADD CONSTRAINT product_services_product_id_service_id_key UNIQUE (product_id, service_id);


--
-- Name: products products_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_pkey PRIMARY KEY (id);


--
-- Name: products products_product_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_product_id_key UNIQUE (product_id);


--
-- Name: resource_attribute_dependencies resource_attribute_dependenci_source_resource_id_target_res_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_attribute_dependencies
    ADD CONSTRAINT resource_attribute_dependenci_source_resource_id_target_res_key UNIQUE (source_resource_id, target_resource_id, dependency_type);


--
-- Name: resource_attribute_dependencies resource_attribute_dependencies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_attribute_dependencies
    ADD CONSTRAINT resource_attribute_dependencies_pkey PRIMARY KEY (id);


--
-- Name: resource_attribute_templates resource_attribute_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_attribute_templates
    ADD CONSTRAINT resource_attribute_templates_pkey PRIMARY KEY (id);


--
-- Name: resource_attribute_templates resource_attribute_templates_template_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_attribute_templates
    ADD CONSTRAINT resource_attribute_templates_template_name_key UNIQUE (template_name);


--
-- Name: resource_attribute_versions resource_attribute_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_attribute_versions
    ADD CONSTRAINT resource_attribute_versions_pkey PRIMARY KEY (id);


--
-- Name: resource_attribute_versions resource_attribute_versions_resource_id_version_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_attribute_versions
    ADD CONSTRAINT resource_attribute_versions_resource_id_version_number_key UNIQUE (resource_id, version_number);


--
-- Name: resource_capabilities resource_capabilities_capability_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_capabilities
    ADD CONSTRAINT resource_capabilities_capability_id_key UNIQUE (capability_id);


--
-- Name: resource_capabilities resource_capabilities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_capabilities
    ADD CONSTRAINT resource_capabilities_pkey PRIMARY KEY (id);


--
-- Name: resource_dependencies resource_dependencies_dependent_resource_id_prerequisite_re_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_dependencies
    ADD CONSTRAINT resource_dependencies_dependent_resource_id_prerequisite_re_key UNIQUE (dependent_resource_id, prerequisite_resource_id);


--
-- Name: resource_dependencies resource_dependencies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_dependencies
    ADD CONSTRAINT resource_dependencies_pkey PRIMARY KEY (id);


--
-- Name: resource_dictionaries resource_dictionaries_dictionary_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_dictionaries
    ADD CONSTRAINT resource_dictionaries_dictionary_name_key UNIQUE (dictionary_name);


--
-- Name: resource_dictionaries resource_dictionaries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_dictionaries
    ADD CONSTRAINT resource_dictionaries_pkey PRIMARY KEY (id);


--
-- Name: resource_instances resource_instances_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_instances
    ADD CONSTRAINT resource_instances_pkey PRIMARY KEY (instance_id);


--
-- Name: resource_objects resource_objects_dictionary_id_resource_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_objects
    ADD CONSTRAINT resource_objects_dictionary_id_resource_name_key UNIQUE (dictionary_id, resource_name);


--
-- Name: resource_objects resource_objects_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_objects
    ADD CONSTRAINT resource_objects_pkey PRIMARY KEY (id);


--
-- Name: resource_sheet_execution_logs resource_sheet_execution_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_sheet_execution_logs
    ADD CONSTRAINT resource_sheet_execution_logs_pkey PRIMARY KEY (log_id);


--
-- Name: resource_sheet_relationships resource_sheet_relationships_parent_resource_id_child_resou_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_sheet_relationships
    ADD CONSTRAINT resource_sheet_relationships_parent_resource_id_child_resou_key UNIQUE (parent_resource_id, child_resource_id, relationship_type);


--
-- Name: resource_sheet_relationships resource_sheet_relationships_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_sheet_relationships
    ADD CONSTRAINT resource_sheet_relationships_pkey PRIMARY KEY (relationship_id);


--
-- Name: resource_sheets resource_sheets_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_sheets
    ADD CONSTRAINT resource_sheets_pkey PRIMARY KEY (resource_id);


--
-- Name: resource_template_capabilities resource_template_capabilities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_template_capabilities
    ADD CONSTRAINT resource_template_capabilities_pkey PRIMARY KEY (id);


--
-- Name: resource_template_capabilities resource_template_capabilities_template_id_capability_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_template_capabilities
    ADD CONSTRAINT resource_template_capabilities_template_id_capability_id_key UNIQUE (template_id, capability_id);


--
-- Name: resource_templates resource_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_templates
    ADD CONSTRAINT resource_templates_pkey PRIMARY KEY (id);


--
-- Name: resource_templates resource_templates_template_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_templates
    ADD CONSTRAINT resource_templates_template_id_key UNIQUE (template_id);


--
-- Name: resource_validation_rules resource_validation_rules_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_validation_rules
    ADD CONSTRAINT resource_validation_rules_pkey PRIMARY KEY (id);


--
-- Name: resource_validation_rules resource_validation_rules_resource_id_rule_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_validation_rules
    ADD CONSTRAINT resource_validation_rules_resource_id_rule_name_key UNIQUE (resource_id, rule_name);


--
-- Name: resources resources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resources
    ADD CONSTRAINT resources_pkey PRIMARY KEY (id);


--
-- Name: resources resources_resource_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resources
    ADD CONSTRAINT resources_resource_id_key UNIQUE (resource_id);


--
-- Name: role_service_access role_service_access_cbu_role_id_service_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.role_service_access
    ADD CONSTRAINT role_service_access_cbu_role_id_service_id_key UNIQUE (cbu_role_id, service_id);


--
-- Name: role_service_access role_service_access_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.role_service_access
    ADD CONSTRAINT role_service_access_pkey PRIMARY KEY (id);


--
-- Name: rule_categories rule_categories_category_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_categories
    ADD CONSTRAINT rule_categories_category_key_key UNIQUE (category_key);


--
-- Name: rule_categories rule_categories_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_categories
    ADD CONSTRAINT rule_categories_pkey PRIMARY KEY (id);


--
-- Name: rule_compilation_queue rule_compilation_queue_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_compilation_queue
    ADD CONSTRAINT rule_compilation_queue_pkey PRIMARY KEY (id);


--
-- Name: rule_compilation_queue rule_compilation_queue_rule_id_compilation_type_status_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_compilation_queue
    ADD CONSTRAINT rule_compilation_queue_rule_id_compilation_type_status_key UNIQUE (rule_id, compilation_type, status);


--
-- Name: rule_dependencies rule_dependencies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_dependencies
    ADD CONSTRAINT rule_dependencies_pkey PRIMARY KEY (id);


--
-- Name: rule_dependencies rule_dependencies_rule_id_attribute_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_dependencies
    ADD CONSTRAINT rule_dependencies_rule_id_attribute_id_key UNIQUE (rule_id, attribute_id);


--
-- Name: rule_execution_stats rule_execution_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_execution_stats
    ADD CONSTRAINT rule_execution_stats_pkey PRIMARY KEY (id);


--
-- Name: rule_execution_stats rule_execution_stats_rule_id_execution_date_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_execution_stats
    ADD CONSTRAINT rule_execution_stats_rule_id_execution_date_key UNIQUE (rule_id, execution_date);


--
-- Name: rule_executions rule_executions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_executions
    ADD CONSTRAINT rule_executions_pkey PRIMARY KEY (id);


--
-- Name: rule_versions rule_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_versions
    ADD CONSTRAINT rule_versions_pkey PRIMARY KEY (id);


--
-- Name: rule_versions rule_versions_rule_id_version_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_versions
    ADD CONSTRAINT rule_versions_rule_id_version_key UNIQUE (rule_id, version);


--
-- Name: rules rules_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rules
    ADD CONSTRAINT rules_pkey PRIMARY KEY (id);


--
-- Name: rules rules_rule_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rules
    ADD CONSTRAINT rules_rule_id_key UNIQUE (rule_id);


--
-- Name: service_resource_mappings service_resource_mappings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_resource_mappings
    ADD CONSTRAINT service_resource_mappings_pkey PRIMARY KEY (id);


--
-- Name: service_resource_mappings service_resource_mappings_service_id_resource_id_usage_type_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_resource_mappings
    ADD CONSTRAINT service_resource_mappings_service_id_resource_id_usage_type_key UNIQUE (service_id, resource_id, usage_type);


--
-- Name: service_resources service_resources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_resources
    ADD CONSTRAINT service_resources_pkey PRIMARY KEY (id);


--
-- Name: service_resources service_resources_service_id_resource_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_resources
    ADD CONSTRAINT service_resources_service_id_resource_id_key UNIQUE (service_id, resource_id);


--
-- Name: services services_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.services
    ADD CONSTRAINT services_pkey PRIMARY KEY (id);


--
-- Name: services services_service_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.services
    ADD CONSTRAINT services_service_id_key UNIQUE (service_id);


--
-- Name: ui_component_templates ui_component_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ui_component_templates
    ADD CONSTRAINT ui_component_templates_pkey PRIMARY KEY (id);


--
-- Name: ui_component_templates ui_component_templates_template_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ui_component_templates
    ADD CONSTRAINT ui_component_templates_template_name_key UNIQUE (template_name);


--
-- Name: ui_layout_groups ui_layout_groups_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ui_layout_groups
    ADD CONSTRAINT ui_layout_groups_pkey PRIMARY KEY (id);


--
-- Name: ui_layout_groups ui_layout_groups_resource_id_group_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ui_layout_groups
    ADD CONSTRAINT ui_layout_groups_resource_id_group_name_key UNIQUE (resource_id, group_name);


--
-- Name: user_saved_filters user_saved_filters_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_saved_filters
    ADD CONSTRAINT user_saved_filters_pkey PRIMARY KEY (id);


--
-- Name: user_saved_filters user_saved_filters_user_id_filter_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_saved_filters
    ADD CONSTRAINT user_saved_filters_user_id_filter_name_key UNIQUE (user_id, filter_name);


--
-- Name: idx_ai_attribute_contexts_attribute; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_attribute_contexts_attribute ON public.ai_attribute_contexts USING btree (attribute_id);


--
-- Name: idx_ai_attribute_contexts_keywords; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_attribute_contexts_keywords ON public.ai_attribute_contexts USING gin (keywords);


--
-- Name: idx_ai_attribute_contexts_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_attribute_contexts_type ON public.ai_attribute_contexts USING btree (context_type);


--
-- Name: idx_ai_enhanced_view_full_text; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_enhanced_view_full_text ON public.attribute_objects USING gin (to_tsvector('english'::regconfig, (((((((((COALESCE(attribute_name, ''::character varying))::text || ' '::text) || COALESCE(description, ''::text)) || ' '::text) || COALESCE(extended_description, ''::text)) || ' '::text) || COALESCE(business_context, ''::text)) || ' '::text) || COALESCE(technical_context, ''::text))));


--
-- Name: idx_ai_training_examples_attribute; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_training_examples_attribute ON public.ai_training_examples USING btree (attribute_id);


--
-- Name: idx_ai_training_examples_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_training_examples_type ON public.ai_training_examples USING btree (example_type);


--
-- Name: idx_attribute_cluster_memberships_strength; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_cluster_memberships_strength ON public.attribute_cluster_memberships USING btree (membership_strength);


--
-- Name: idx_attribute_cluster_memberships_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_cluster_memberships_type ON public.attribute_cluster_memberships USING btree (membership_type);


--
-- Name: idx_attribute_comprehensive_search; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_comprehensive_search ON public.attribute_objects USING gin (to_tsvector('english'::regconfig, (((((((((COALESCE(attribute_name, ''::character varying))::text || ' '::text) || COALESCE(description, ''::text)) || ' '::text) || COALESCE(extended_description, ''::text)) || ' '::text) || COALESCE(business_context, ''::text)) || ' '::text) || COALESCE(technical_context, ''::text))));


--
-- Name: idx_attribute_documentation_attribute; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_documentation_attribute ON public.attribute_documentation USING btree (attribute_id);


--
-- Name: idx_attribute_domain_mappings_attribute; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_domain_mappings_attribute ON public.attribute_domain_mappings USING btree (attribute_id);


--
-- Name: idx_attribute_domain_mappings_domain; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_domain_mappings_domain ON public.attribute_domain_mappings USING btree (domain_id);


--
-- Name: idx_attribute_lineage_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_lineage_source ON public.attribute_lineage USING btree (source_attribute_id);


--
-- Name: idx_attribute_lineage_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_lineage_target ON public.attribute_lineage USING btree (target_attribute_id);


--
-- Name: idx_attribute_objects_ai_context; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_objects_ai_context ON public.attribute_objects USING gin (ai_context);


--
-- Name: idx_attribute_objects_class; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_objects_class ON public.attribute_objects USING btree (attribute_class);


--
-- Name: idx_attribute_objects_derivation_deps; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_objects_derivation_deps ON public.attribute_objects USING gin (derivation_dependencies);


--
-- Name: idx_attribute_objects_embedding; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_objects_embedding ON public.attribute_objects USING ivfflat (embedding_vector public.vector_cosine_ops);


--
-- Name: idx_attribute_objects_embedding_cosine; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_objects_embedding_cosine ON public.attribute_objects USING ivfflat (embedding_vector public.vector_cosine_ops) WITH (lists='100');


--
-- Name: idx_attribute_objects_group; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_objects_group ON public.attribute_objects USING btree (ui_group);


--
-- Name: idx_attribute_objects_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_objects_name ON public.attribute_objects USING btree (attribute_name);


--
-- Name: idx_attribute_objects_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_objects_resource ON public.attribute_objects USING btree (resource_id);


--
-- Name: idx_attribute_objects_search_keywords; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_objects_search_keywords ON public.attribute_objects USING gin (search_keywords);


--
-- Name: idx_attribute_objects_semantic_tags; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_objects_semantic_tags ON public.attribute_objects USING gin (semantic_tags);


--
-- Name: idx_attribute_objects_ui_component; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_objects_ui_component ON public.attribute_objects USING btree (ui_component_type);


--
-- Name: idx_attribute_objects_visibility; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_objects_visibility ON public.attribute_objects USING btree (visibility_scope);


--
-- Name: idx_attribute_persistence_mappings_attribute; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_persistence_mappings_attribute ON public.attribute_persistence_mappings USING btree (attribute_id);


--
-- Name: idx_attribute_persistence_mappings_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_persistence_mappings_entity ON public.attribute_persistence_mappings USING btree (persistence_entity_id);


--
-- Name: idx_attribute_perspectives_attr; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_perspectives_attr ON public.attribute_perspectives USING btree (attribute_id);


--
-- Name: idx_attribute_perspectives_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_perspectives_name ON public.attribute_perspectives USING btree (perspective_name);


--
-- Name: idx_attribute_semantic_relationships_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_semantic_relationships_source ON public.attribute_semantic_relationships USING btree (source_attribute_id);


--
-- Name: idx_attribute_semantic_relationships_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_semantic_relationships_target ON public.attribute_semantic_relationships USING btree (target_attribute_id);


--
-- Name: idx_attribute_tag_assignments_confidence; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_tag_assignments_confidence ON public.attribute_tag_assignments USING btree (assignment_confidence);


--
-- Name: idx_attribute_tags_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_tags_category ON public.attribute_tags USING btree (tag_category);


--
-- Name: idx_attribute_value_audit_attribute; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_value_audit_attribute ON public.attribute_value_audit USING btree (attribute_id);


--
-- Name: idx_attribute_value_audit_entity_instance; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_value_audit_entity_instance ON public.attribute_value_audit USING btree (entity_instance_id);


--
-- Name: idx_attribute_value_audit_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_value_audit_timestamp ON public.attribute_value_audit USING btree (change_timestamp);


--
-- Name: idx_attribute_vector_clusters_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attribute_vector_clusters_type ON public.attribute_vector_clusters USING btree (cluster_type);


--
-- Name: idx_benchmarks_mandate_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_benchmarks_mandate_id ON public.mandate_benchmarks USING btree (mandate_id);


--
-- Name: idx_business_attrs_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_business_attrs_entity ON public.business_attributes USING btree (entity_name);


--
-- Name: idx_cbu_business_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_business_type ON public.client_business_units USING btree (business_type);


--
-- Name: idx_cbu_cbu_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_cbu_id ON public.client_business_units USING btree (cbu_id);


--
-- Name: idx_cbu_country; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_country ON public.client_business_units USING btree (domicile_country);


--
-- Name: idx_cbu_created_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_created_date ON public.client_business_units USING btree (created_date);


--
-- Name: idx_cbu_entity_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_entity_active ON public.cbu_entity_associations USING btree (active_in_cbu);


--
-- Name: idx_cbu_entity_cbu; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_entity_cbu ON public.cbu_entity_associations USING btree (cbu_id);


--
-- Name: idx_cbu_entity_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_entity_entity ON public.cbu_entity_associations USING btree (entity_id);


--
-- Name: idx_cbu_entity_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_entity_type ON public.cbu_entity_associations USING btree (association_type);


--
-- Name: idx_cbu_members_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_members_active ON public.cbu_members USING btree (is_active);


--
-- Name: idx_cbu_members_cbu_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_members_cbu_id ON public.cbu_members USING btree (cbu_id);


--
-- Name: idx_cbu_members_effective_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_members_effective_date ON public.cbu_members USING btree (effective_date);


--
-- Name: idx_cbu_members_entity_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_members_entity_id ON public.cbu_members USING btree (entity_id);


--
-- Name: idx_cbu_members_entity_lei; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_members_entity_lei ON public.cbu_members USING btree (entity_lei);


--
-- Name: idx_cbu_members_primary; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_members_primary ON public.cbu_members USING btree (is_primary);


--
-- Name: idx_cbu_members_role_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_members_role_id ON public.cbu_members USING btree (role_id);


--
-- Name: idx_cbu_primary_lei; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_primary_lei ON public.client_business_units USING btree (primary_lei);


--
-- Name: idx_cbu_roles_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_roles_active ON public.cbu_roles USING btree (is_active);


--
-- Name: idx_cbu_roles_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_roles_category ON public.cbu_roles USING btree (role_category);


--
-- Name: idx_cbu_roles_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_roles_code ON public.cbu_roles USING btree (role_code);


--
-- Name: idx_cbu_service_resources_cbu; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_service_resources_cbu ON public.cbu_service_resources USING btree (cbu_id);


--
-- Name: idx_cbu_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_status ON public.client_business_units USING btree (status);


--
-- Name: idx_cbu_subscriptions_cbu; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_subscriptions_cbu ON public.cbu_product_subscriptions USING btree (cbu_id);


--
-- Name: idx_cbu_subscriptions_product; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_subscriptions_product ON public.cbu_product_subscriptions USING btree (product_id);


--
-- Name: idx_cbu_subscriptions_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cbu_subscriptions_status ON public.cbu_product_subscriptions USING btree (subscription_status);


--
-- Name: idx_channels_instrument_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_channels_instrument_id ON public.mandate_instruction_channels USING btree (instrument_id);


--
-- Name: idx_commercial_contracts_customer; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_commercial_contracts_customer ON public.commercial_contracts USING btree (customer_entity_id);


--
-- Name: idx_commercial_contracts_product; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_commercial_contracts_product ON public.commercial_contracts USING btree (product_id);


--
-- Name: idx_commercial_contracts_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_commercial_contracts_status ON public.commercial_contracts USING btree (contract_status);


--
-- Name: idx_compilation_queue_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_compilation_queue_status ON public.rule_compilation_queue USING btree (status, priority DESC);


--
-- Name: idx_derivation_execution_log_attr; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_derivation_execution_log_attr ON public.derivation_execution_log USING btree (derived_attribute_id);


--
-- Name: idx_derivation_execution_log_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_derivation_execution_log_timestamp ON public.derivation_execution_log USING btree (execution_timestamp);


--
-- Name: idx_derived_attribute_rules_derived_attr; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_derived_attribute_rules_derived_attr ON public.derived_attribute_rules USING btree (derived_attribute_id);


--
-- Name: idx_derived_attrs_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_derived_attrs_entity ON public.derived_attributes USING btree (entity_name);


--
-- Name: idx_domain_terminology_glossary_term; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_domain_terminology_glossary_term ON public.domain_terminology_glossary USING gin (to_tsvector('english'::regconfig, (((term)::text || ' '::text) || definition)));


--
-- Name: idx_dsl_execution_logs_executed; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_dsl_execution_logs_executed ON public.dsl_execution_logs USING btree (executed_at);


--
-- Name: idx_dsl_execution_logs_instance; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_dsl_execution_logs_instance ON public.dsl_execution_logs USING btree (instance_id);


--
-- Name: idx_dsl_execution_logs_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_dsl_execution_logs_status ON public.dsl_execution_logs USING btree (execution_status);


--
-- Name: idx_entity_attr_values_attr; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_entity_attr_values_attr ON public.entity_attribute_values USING btree (attribute_id);


--
-- Name: idx_entity_attr_values_current; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_entity_attr_values_current ON public.entity_attribute_values USING btree (is_current);


--
-- Name: idx_entity_attr_values_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_entity_attr_values_entity ON public.entity_attribute_values USING btree (entity_id);


--
-- Name: idx_entity_relationships_child; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_entity_relationships_child ON public.entity_relationships USING btree (child_entity_id);


--
-- Name: idx_entity_relationships_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_entity_relationships_parent ON public.entity_relationships USING btree (parent_entity_id);


--
-- Name: idx_entity_relationships_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_entity_relationships_type ON public.entity_relationships USING btree (relationship_type);


--
-- Name: idx_events_instrument_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_events_instrument_id ON public.mandate_lifecycle_events USING btree (instrument_id);


--
-- Name: idx_execution_logs_execution; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_execution_logs_execution ON public.resource_sheet_execution_logs USING btree (execution_id);


--
-- Name: idx_execution_logs_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_execution_logs_resource ON public.resource_sheet_execution_logs USING btree (resource_id);


--
-- Name: idx_execution_logs_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_execution_logs_timestamp ON public.resource_sheet_execution_logs USING btree ("timestamp");


--
-- Name: idx_executions_rule; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_executions_rule ON public.rule_executions USING btree (rule_id);


--
-- Name: idx_executions_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_executions_time ON public.rule_executions USING btree (execution_time);


--
-- Name: idx_grammar_extensions_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_grammar_extensions_category ON public.grammar_extensions USING btree (category);


--
-- Name: idx_grammar_extensions_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_grammar_extensions_type ON public.grammar_extensions USING btree (type);


--
-- Name: idx_grammar_rules_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_grammar_rules_category ON public.grammar_rules USING btree (category);


--
-- Name: idx_grammar_rules_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_grammar_rules_name ON public.grammar_rules USING btree (name);


--
-- Name: idx_identifiers_instrument_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_identifiers_instrument_id ON public.mandate_instrument_identifiers USING btree (instrument_id);


--
-- Name: idx_instruction_formats_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_instruction_formats_category ON public.instruction_formats USING btree (format_category);


--
-- Name: idx_instruction_formats_standard; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_instruction_formats_standard ON public.instruction_formats USING btree (message_standard);


--
-- Name: idx_instrument_taxonomy_asset_class; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_instrument_taxonomy_asset_class ON public.instrument_taxonomy USING btree (asset_class);


--
-- Name: idx_instrument_taxonomy_class; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_instrument_taxonomy_class ON public.instrument_taxonomy USING btree (instrument_class);


--
-- Name: idx_instrument_taxonomy_risk; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_instrument_taxonomy_risk ON public.instrument_taxonomy USING btree (risk_category);


--
-- Name: idx_instruments_family; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_instruments_family ON public.mandate_instruments USING btree (instrument_family);


--
-- Name: idx_instruments_mandate_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_instruments_mandate_id ON public.mandate_instruments USING btree (mandate_id);


--
-- Name: idx_investment_mandates_cbu; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_investment_mandates_cbu ON public.investment_mandates USING btree (cbu_id);


--
-- Name: idx_legal_entities_country; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_legal_entities_country ON public.legal_entities USING btree (incorporation_country);


--
-- Name: idx_legal_entities_kyc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_legal_entities_kyc ON public.legal_entities USING btree (kyc_status);


--
-- Name: idx_legal_entities_lei; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_legal_entities_lei ON public.legal_entities USING btree (lei_code);


--
-- Name: idx_legal_entities_reg_num; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_legal_entities_reg_num ON public.legal_entities USING btree (registration_number);


--
-- Name: idx_legal_entities_risk; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_legal_entities_risk ON public.legal_entities USING btree (risk_rating);


--
-- Name: idx_legal_entities_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_legal_entities_status ON public.legal_entities USING btree (status);


--
-- Name: idx_legal_entities_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_legal_entities_type ON public.legal_entities USING btree (entity_type);


--
-- Name: idx_mandates_asset_owner_lei; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mandates_asset_owner_lei ON public.investment_mandates USING btree (asset_owner_lei);


--
-- Name: idx_mandates_cbu_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mandates_cbu_id ON public.investment_mandates USING btree (cbu_id);


--
-- Name: idx_mandates_effective_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mandates_effective_date ON public.investment_mandates USING btree (effective_date);


--
-- Name: idx_mandates_expiry_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mandates_expiry_date ON public.investment_mandates USING btree (expiry_date);


--
-- Name: idx_mandates_investment_manager_lei; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mandates_investment_manager_lei ON public.investment_mandates USING btree (investment_manager_lei);


--
-- Name: idx_mv_data_dictionary_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mv_data_dictionary_entity ON public.mv_data_dictionary USING btree (entity_name);


--
-- Name: idx_mv_data_dictionary_path; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_mv_data_dictionary_path ON public.mv_data_dictionary USING btree (full_path);


--
-- Name: idx_mv_data_dictionary_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mv_data_dictionary_type ON public.mv_data_dictionary USING btree (attribute_type);


--
-- Name: idx_onboarding_approvals_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_approvals_status ON public.onboarding_approvals USING btree (approval_status);


--
-- Name: idx_onboarding_approvals_workflow; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_approvals_workflow ON public.onboarding_approvals USING btree (workflow_id);


--
-- Name: idx_onboarding_cbu; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_cbu ON public.onboarding_requests USING btree (cbu_id);


--
-- Name: idx_onboarding_dependencies_workflow; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_dependencies_workflow ON public.onboarding_dependencies USING btree (workflow_id);


--
-- Name: idx_onboarding_product; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_product ON public.onboarding_requests USING btree (product_id);


--
-- Name: idx_onboarding_resource_tasks_order; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_resource_tasks_order ON public.onboarding_resource_tasks USING btree (workflow_id, task_order);


--
-- Name: idx_onboarding_resource_tasks_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_resource_tasks_status ON public.onboarding_resource_tasks USING btree (task_status);


--
-- Name: idx_onboarding_resource_tasks_workflow; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_resource_tasks_workflow ON public.onboarding_resource_tasks USING btree (workflow_id);


--
-- Name: idx_onboarding_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_status ON public.onboarding_requests USING btree (request_status);


--
-- Name: idx_onboarding_tasks_request; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_tasks_request ON public.onboarding_tasks USING btree (onboarding_request_id);


--
-- Name: idx_onboarding_tasks_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_tasks_resource ON public.onboarding_tasks USING btree (resource_id);


--
-- Name: idx_onboarding_workflows_cbu; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_workflows_cbu ON public.onboarding_workflows USING btree (cbu_id);


--
-- Name: idx_onboarding_workflows_priority; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_workflows_priority ON public.onboarding_workflows USING btree (priority);


--
-- Name: idx_onboarding_workflows_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_workflows_status ON public.onboarding_workflows USING btree (workflow_status);


--
-- Name: idx_onboarding_workflows_target_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_onboarding_workflows_target_date ON public.onboarding_workflows USING btree (target_go_live_date);


--
-- Name: idx_product_option_service_mappings_option; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_option_service_mappings_option ON public.product_option_service_mappings USING btree (product_option_id);


--
-- Name: idx_product_option_service_mappings_relationship; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_option_service_mappings_relationship ON public.product_option_service_mappings USING btree (mapping_relationship);


--
-- Name: idx_product_option_service_mappings_service; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_option_service_mappings_service ON public.product_option_service_mappings USING btree (service_id);


--
-- Name: idx_product_options_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_options_category ON public.product_options USING btree (option_category);


--
-- Name: idx_product_options_product; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_options_product ON public.product_options USING btree (product_id);


--
-- Name: idx_product_options_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_options_status ON public.product_options USING btree (status);


--
-- Name: idx_product_options_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_options_type ON public.product_options USING btree (option_type);


--
-- Name: idx_product_service_mappings_product; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_service_mappings_product ON public.product_service_mappings USING btree (product_id);


--
-- Name: idx_product_service_mappings_service; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_service_mappings_service ON public.product_service_mappings USING btree (service_id);


--
-- Name: idx_product_service_mappings_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_service_mappings_type ON public.product_service_mappings USING btree (mapping_type);


--
-- Name: idx_product_services_product; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_services_product ON public.product_services USING btree (product_id);


--
-- Name: idx_product_services_service; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_services_service ON public.product_services USING btree (service_id);


--
-- Name: idx_products_commercial_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_products_commercial_status ON public.products USING btree (commercial_status);


--
-- Name: idx_products_contract_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_products_contract_type ON public.products USING btree (contract_type);


--
-- Name: idx_products_line_of_business; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_products_line_of_business ON public.products USING btree (line_of_business);


--
-- Name: idx_products_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_products_status ON public.products USING btree (status);


--
-- Name: idx_relationships_child; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_relationships_child ON public.resource_sheet_relationships USING btree (child_resource_id);


--
-- Name: idx_relationships_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_relationships_parent ON public.resource_sheet_relationships USING btree (parent_resource_id);


--
-- Name: idx_resource_attribute_templates_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_attribute_templates_category ON public.resource_attribute_templates USING btree (category);


--
-- Name: idx_resource_attribute_versions_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_attribute_versions_active ON public.resource_attribute_versions USING btree (is_active);


--
-- Name: idx_resource_attribute_versions_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_attribute_versions_resource ON public.resource_attribute_versions USING btree (resource_id);


--
-- Name: idx_resource_capabilities_capability_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_capabilities_capability_id ON public.resource_capabilities USING btree (capability_id);


--
-- Name: idx_resource_capabilities_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_capabilities_type ON public.resource_capabilities USING btree (capability_type);


--
-- Name: idx_resource_dictionaries_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_dictionaries_name ON public.resource_dictionaries USING btree (dictionary_name);


--
-- Name: idx_resource_instances_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_instances_created ON public.resource_instances USING btree (created_at);


--
-- Name: idx_resource_instances_data; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_instances_data ON public.resource_instances USING gin (instance_data);


--
-- Name: idx_resource_instances_onboarding; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_instances_onboarding ON public.resource_instances USING btree (onboarding_request_id);


--
-- Name: idx_resource_instances_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_instances_status ON public.resource_instances USING btree (status);


--
-- Name: idx_resource_instances_template; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_instances_template ON public.resource_instances USING btree (template_id);


--
-- Name: idx_resource_objects_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_objects_category ON public.resource_objects USING btree (category);


--
-- Name: idx_resource_objects_dictionary; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_objects_dictionary ON public.resource_objects USING btree (dictionary_id);


--
-- Name: idx_resource_objects_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_objects_name ON public.resource_objects USING btree (resource_name);


--
-- Name: idx_resource_sheets_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_sheets_client ON public.resource_sheets USING btree (client_id);


--
-- Name: idx_resource_sheets_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_sheets_created ON public.resource_sheets USING btree (created_at);


--
-- Name: idx_resource_sheets_json; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_sheets_json ON public.resource_sheets USING gin (json_data);


--
-- Name: idx_resource_sheets_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_sheets_status ON public.resource_sheets USING btree (status);


--
-- Name: idx_resource_sheets_tags; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_sheets_tags ON public.resource_sheets USING gin (tags);


--
-- Name: idx_resource_sheets_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_sheets_type ON public.resource_sheets USING btree (resource_type);


--
-- Name: idx_resource_template_capabilities_capability; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_template_capabilities_capability ON public.resource_template_capabilities USING btree (capability_id);


--
-- Name: idx_resource_template_capabilities_template; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_template_capabilities_template ON public.resource_template_capabilities USING btree (template_id);


--
-- Name: idx_resource_templates_product; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_templates_product ON public.resource_templates USING btree (part_of_product);


--
-- Name: idx_resource_templates_service; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_templates_service ON public.resource_templates USING btree (implements_service);


--
-- Name: idx_resource_templates_template_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_templates_template_id ON public.resource_templates USING btree (template_id);


--
-- Name: idx_resource_validation_rules_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_validation_rules_active ON public.resource_validation_rules USING btree (is_active);


--
-- Name: idx_resource_validation_rules_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_validation_rules_resource ON public.resource_validation_rules USING btree (resource_id);


--
-- Name: idx_resources_criticality; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_criticality ON public.resource_objects USING btree (criticality_level);


--
-- Name: idx_resources_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_status ON public.resources USING btree (status);


--
-- Name: idx_resources_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_type ON public.resources USING btree (resource_type);


--
-- Name: idx_rule_deps_attr; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rule_deps_attr ON public.rule_dependencies USING btree (attribute_id);


--
-- Name: idx_rule_deps_rule; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rule_deps_rule ON public.rule_dependencies USING btree (rule_id);


--
-- Name: idx_rule_execution_stats_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rule_execution_stats_date ON public.rule_execution_stats USING btree (execution_date DESC);


--
-- Name: idx_rules_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rules_category ON public.rules USING btree (category_id);


--
-- Name: idx_rules_compilation_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rules_compilation_status ON public.rules USING btree (compilation_status);


--
-- Name: idx_rules_embedding; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rules_embedding ON public.rules USING hnsw (embedding public.vector_cosine_ops);


--
-- Name: idx_rules_search; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rules_search ON public.rules USING gin (search_vector);


--
-- Name: idx_rules_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rules_status ON public.rules USING btree (status);


--
-- Name: idx_rules_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rules_target ON public.rules USING btree (target_attribute_id);


--
-- Name: idx_rules_target_attribute; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rules_target_attribute ON public.rules USING btree (target_attribute_id);


--
-- Name: idx_service_resource_mappings_dependency; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_resource_mappings_dependency ON public.service_resource_mappings USING btree (dependency_level);


--
-- Name: idx_service_resource_mappings_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_resource_mappings_resource ON public.service_resource_mappings USING btree (resource_id);


--
-- Name: idx_service_resource_mappings_service; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_resource_mappings_service ON public.service_resource_mappings USING btree (service_id);


--
-- Name: idx_service_resource_mappings_usage; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_resource_mappings_usage ON public.service_resource_mappings USING btree (usage_type);


--
-- Name: idx_service_resources_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_resources_resource ON public.service_resources USING btree (resource_id);


--
-- Name: idx_service_resources_service; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_resources_service ON public.service_resources USING btree (service_id);


--
-- Name: idx_services_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_services_category ON public.services USING btree (service_category);


--
-- Name: idx_services_service_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_services_service_type ON public.services USING btree (service_type);


--
-- Name: idx_services_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_services_status ON public.services USING btree (status);


--
-- Name: idx_ui_layout_groups_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ui_layout_groups_resource ON public.ui_layout_groups USING btree (resource_id);


--
-- Name: idx_user_saved_filters_last_used; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_saved_filters_last_used ON public.user_saved_filters USING btree (last_used);


--
-- Name: idx_user_saved_filters_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_saved_filters_user ON public.user_saved_filters USING btree (user_id);


--
-- Name: idx_venues_instrument_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_venues_instrument_id ON public.mandate_instrument_venues USING btree (instrument_id);


--
-- Name: enhanced_attributes_view _RETURN; Type: RULE; Schema: public; Owner: -
--

CREATE OR REPLACE VIEW public.enhanced_attributes_view AS
 SELECT ao.id,
    ao.resource_id,
    ao.attribute_name,
    ao.data_type,
    ao.description,
    ao.is_required,
    ao.min_length,
    ao.max_length,
    ao.min_value,
    ao.max_value,
    ao.allowed_values,
    ao.validation_pattern,
    ao.persistence_system,
    ao.persistence_entity,
    ao.persistence_identifier,
    ao.ui_group,
    ao.ui_display_order,
    ao.ui_render_hint,
    ao.ui_label,
    ao.ui_help_text,
    ao.wizard_step,
    ao.wizard_step_title,
    ao.wizard_next_button,
    ao.wizard_previous_button,
    ao.wizard_description,
    ao.generation_examples,
    ao.rules_dsl,
    ao.created_at,
    ao.updated_at,
    ao.semantic_tags,
    ao.ai_context,
    ao.embedding_vector,
    ao.search_keywords,
    ao.ui_component_type,
    ao.ui_layout_config,
    ao.ui_styling,
    ao.ui_behavior,
    ao.conditional_logic,
    ao.relationship_metadata,
    ao.ai_prompt_templates,
    ao.form_generation_rules,
    ao.accessibility_config,
    ao.responsive_config,
    ao.data_flow_config,
    ac.context_name,
    ac.prompt_template,
    uct.template_config AS ui_template_config,
    uct.styling_defaults,
    uct.behavior_defaults,
    array_agg(DISTINCT ar_source.relationship_type) FILTER (WHERE (ar_source.relationship_type IS NOT NULL)) AS outgoing_relationships,
    array_agg(DISTINCT ar_target.relationship_type) FILTER (WHERE (ar_target.relationship_type IS NOT NULL)) AS incoming_relationships,
    count(DISTINCT ap.id) AS perspective_count
   FROM (((((public.attribute_objects ao
     LEFT JOIN public.ai_metadata_contexts ac ON (((ac.context_name)::text = 'attribute_discovery'::text)))
     LEFT JOIN public.ui_component_templates uct ON (((uct.component_type)::text = (ao.ui_component_type)::text)))
     LEFT JOIN public.attribute_relationships ar_source ON ((ar_source.source_attribute_id = ao.id)))
     LEFT JOIN public.attribute_relationships ar_target ON ((ar_target.target_attribute_id = ao.id)))
     LEFT JOIN public.attribute_perspectives ap ON ((ap.attribute_id = ao.id)))
  GROUP BY ao.id, ac.context_name, ac.prompt_template, uct.template_config, uct.styling_defaults, uct.behavior_defaults;


--
-- Name: v_cbu_summary _RETURN; Type: RULE; Schema: public; Owner: -
--

CREATE OR REPLACE VIEW public.v_cbu_summary AS
 SELECT cbu.id,
    cbu.cbu_id,
    cbu.cbu_name,
    cbu.description,
    cbu.primary_lei,
    cbu.domicile_country,
    cbu.business_type,
    cbu.status,
    cbu.created_date,
    count(DISTINCT cm.id) AS member_count,
    count(DISTINCT cm.role_id) AS role_count,
    string_agg(DISTINCT (cr.role_name)::text, ', '::text ORDER BY (cr.role_name)::text) AS roles,
    cbu.created_at,
    cbu.updated_at
   FROM ((public.client_business_units cbu
     LEFT JOIN public.cbu_members cm ON (((cbu.id = cm.cbu_id) AND (cm.is_active = true))))
     LEFT JOIN public.cbu_roles cr ON ((cm.role_id = cr.id)))
  GROUP BY cbu.id;


--
-- Name: v_mandate_summary _RETURN; Type: RULE; Schema: public; Owner: -
--

CREATE OR REPLACE VIEW public.v_mandate_summary AS
 SELECT m.mandate_id,
    m.cbu_id,
    m.asset_owner_name,
    m.asset_owner_lei,
    m.investment_manager_name,
    m.investment_manager_lei,
    m.base_currency,
    m.effective_date,
    m.expiry_date,
    count(DISTINCT mi.id) AS instrument_count,
    count(DISTINCT mb.id) AS benchmark_count,
    m.created_at,
    m.updated_at
   FROM ((public.investment_mandates m
     LEFT JOIN public.mandate_instruments mi ON (((m.mandate_id)::text = (mi.mandate_id)::text)))
     LEFT JOIN public.mandate_benchmarks mb ON (((m.mandate_id)::text = (mb.mandate_id)::text)))
  GROUP BY m.mandate_id;


--
-- Name: v_onboarding_progress _RETURN; Type: RULE; Schema: public; Owner: -
--

CREATE OR REPLACE VIEW public.v_onboarding_progress AS
 SELECT or_main.request_id,
    cbu.cbu_name,
    p.product_name,
    or_main.request_status,
    or_main.target_go_live_date,
    count(ot.id) AS total_tasks,
    count(
        CASE
            WHEN ((ot.task_status)::text = 'completed'::text) THEN 1
            ELSE NULL::integer
        END) AS completed_tasks,
    count(
        CASE
            WHEN ((ot.task_status)::text = 'blocked'::text) THEN 1
            ELSE NULL::integer
        END) AS blocked_tasks,
    round((((count(
        CASE
            WHEN ((ot.task_status)::text = 'completed'::text) THEN 1
            ELSE NULL::integer
        END))::numeric / (NULLIF(count(ot.id), 0))::numeric) * (100)::numeric), 1) AS completion_percentage
   FROM (((public.onboarding_requests or_main
     JOIN public.client_business_units cbu ON ((or_main.cbu_id = cbu.id)))
     JOIN public.products p ON ((or_main.product_id = p.id)))
     LEFT JOIN public.onboarding_tasks ot ON ((or_main.id = ot.onboarding_request_id)))
  GROUP BY or_main.id, or_main.request_id, cbu.cbu_name, p.product_name, or_main.request_status, or_main.target_go_live_date
  ORDER BY or_main.created_at DESC;


--
-- Name: attribute_objects comprehensive_metadata_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER comprehensive_metadata_trigger BEFORE INSERT OR UPDATE ON public.attribute_objects FOR EACH ROW EXECUTE FUNCTION public.update_comprehensive_metadata();


--
-- Name: resource_instances resource_instances_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER resource_instances_updated_at BEFORE UPDATE ON public.resource_instances FOR EACH ROW EXECUTE FUNCTION public.update_resource_instances_updated_at();


--
-- Name: rules trigger_mark_rule_for_recompilation; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_mark_rule_for_recompilation BEFORE UPDATE ON public.rules FOR EACH ROW EXECUTE FUNCTION public.mark_rule_for_recompilation();


--
-- Name: attribute_objects update_attribute_embedding_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_attribute_embedding_trigger BEFORE INSERT OR UPDATE ON public.attribute_objects FOR EACH ROW EXECUTE FUNCTION public.update_embedding_trigger();


--
-- Name: attribute_objects update_attribute_objects_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_attribute_objects_updated_at BEFORE UPDATE ON public.attribute_objects FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: business_attributes update_business_attributes_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_business_attributes_updated_at BEFORE UPDATE ON public.business_attributes FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: cbu_members update_cbu_members_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_cbu_members_updated_at BEFORE UPDATE ON public.cbu_members FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: cbu_roles update_cbu_roles_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_cbu_roles_updated_at BEFORE UPDATE ON public.cbu_roles FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: cbu_service_resources update_cbu_service_resources_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_cbu_service_resources_updated_at BEFORE UPDATE ON public.cbu_service_resources FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: cbu_product_subscriptions update_cbu_subscriptions_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_cbu_subscriptions_updated_at BEFORE UPDATE ON public.cbu_product_subscriptions FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: client_business_units update_cbu_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_cbu_updated_at BEFORE UPDATE ON public.client_business_units FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: derived_attributes update_derived_attributes_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_derived_attributes_updated_at BEFORE UPDATE ON public.derived_attributes FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: grammar_extensions update_grammar_extensions_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_grammar_extensions_updated_at BEFORE UPDATE ON public.grammar_extensions FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: grammar_metadata update_grammar_metadata_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_grammar_metadata_updated_at BEFORE UPDATE ON public.grammar_metadata FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: grammar_rules update_grammar_rules_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_grammar_rules_updated_at BEFORE UPDATE ON public.grammar_rules FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: investment_mandates update_investment_mandates_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_investment_mandates_updated_at BEFORE UPDATE ON public.investment_mandates FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: onboarding_requests update_onboarding_requests_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_onboarding_requests_updated_at BEFORE UPDATE ON public.onboarding_requests FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: onboarding_resource_tasks update_onboarding_resource_tasks_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_onboarding_resource_tasks_updated_at BEFORE UPDATE ON public.onboarding_resource_tasks FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: onboarding_tasks update_onboarding_tasks_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_onboarding_tasks_updated_at BEFORE UPDATE ON public.onboarding_tasks FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: onboarding_workflows update_onboarding_workflows_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_onboarding_workflows_updated_at BEFORE UPDATE ON public.onboarding_workflows FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: products update_products_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_products_updated_at BEFORE UPDATE ON public.products FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: resource_dictionaries update_resource_dictionaries_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_resource_dictionaries_updated_at BEFORE UPDATE ON public.resource_dictionaries FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: resource_objects update_resource_objects_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_resource_objects_updated_at BEFORE UPDATE ON public.resource_objects FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: resources update_resources_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_resources_updated_at BEFORE UPDATE ON public.resources FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: rules update_rules_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_rules_updated_at BEFORE UPDATE ON public.rules FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: services update_services_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_services_updated_at BEFORE UPDATE ON public.services FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: onboarding_resource_tasks update_workflow_completion_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_workflow_completion_trigger AFTER INSERT OR UPDATE ON public.onboarding_resource_tasks FOR EACH ROW EXECUTE FUNCTION public.update_workflow_completion();


--
-- Name: attribute_objects validate_derived_attribute_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER validate_derived_attribute_trigger BEFORE INSERT OR UPDATE ON public.attribute_objects FOR EACH ROW WHEN (((new.attribute_class)::text = 'derived'::text)) EXECUTE FUNCTION public.validate_derived_attribute();


--
-- Name: ai_attribute_contexts ai_attribute_contexts_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_attribute_contexts
    ADD CONSTRAINT ai_attribute_contexts_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: ai_training_examples ai_training_examples_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_training_examples
    ADD CONSTRAINT ai_training_examples_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_cluster_memberships attribute_cluster_memberships_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_cluster_memberships
    ADD CONSTRAINT attribute_cluster_memberships_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_cluster_memberships attribute_cluster_memberships_cluster_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_cluster_memberships
    ADD CONSTRAINT attribute_cluster_memberships_cluster_id_fkey FOREIGN KEY (cluster_id) REFERENCES public.attribute_vector_clusters(id) ON DELETE CASCADE;


--
-- Name: attribute_documentation attribute_documentation_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_documentation
    ADD CONSTRAINT attribute_documentation_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_domain_mappings attribute_domain_mappings_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_domain_mappings
    ADD CONSTRAINT attribute_domain_mappings_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_domain_mappings attribute_domain_mappings_domain_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_domain_mappings
    ADD CONSTRAINT attribute_domain_mappings_domain_id_fkey FOREIGN KEY (domain_id) REFERENCES public.kyc_onboarding_domains(id) ON DELETE CASCADE;


--
-- Name: attribute_lineage attribute_lineage_source_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_lineage
    ADD CONSTRAINT attribute_lineage_source_attribute_id_fkey FOREIGN KEY (source_attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_lineage attribute_lineage_target_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_lineage
    ADD CONSTRAINT attribute_lineage_target_attribute_id_fkey FOREIGN KEY (target_attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_objects attribute_objects_primary_persistence_entity_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_objects
    ADD CONSTRAINT attribute_objects_primary_persistence_entity_id_fkey FOREIGN KEY (primary_persistence_entity_id) REFERENCES public.persistence_entities(id);


--
-- Name: attribute_objects attribute_objects_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_objects
    ADD CONSTRAINT attribute_objects_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resource_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_persistence_mappings attribute_persistence_mappings_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_persistence_mappings
    ADD CONSTRAINT attribute_persistence_mappings_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_persistence_mappings attribute_persistence_mappings_persistence_entity_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_persistence_mappings
    ADD CONSTRAINT attribute_persistence_mappings_persistence_entity_id_fkey FOREIGN KEY (persistence_entity_id) REFERENCES public.persistence_entities(id);


--
-- Name: attribute_perspectives attribute_perspectives_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_perspectives
    ADD CONSTRAINT attribute_perspectives_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_relationships attribute_relationships_source_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_relationships
    ADD CONSTRAINT attribute_relationships_source_attribute_id_fkey FOREIGN KEY (source_attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_relationships attribute_relationships_target_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_relationships
    ADD CONSTRAINT attribute_relationships_target_attribute_id_fkey FOREIGN KEY (target_attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_semantic_relationships attribute_semantic_relationships_source_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_semantic_relationships
    ADD CONSTRAINT attribute_semantic_relationships_source_attribute_id_fkey FOREIGN KEY (source_attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_semantic_relationships attribute_semantic_relationships_target_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_semantic_relationships
    ADD CONSTRAINT attribute_semantic_relationships_target_attribute_id_fkey FOREIGN KEY (target_attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_tag_assignments attribute_tag_assignments_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_tag_assignments
    ADD CONSTRAINT attribute_tag_assignments_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_tag_assignments attribute_tag_assignments_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_tag_assignments
    ADD CONSTRAINT attribute_tag_assignments_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.attribute_tags(id) ON DELETE CASCADE;


--
-- Name: attribute_tags attribute_tags_parent_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_tags
    ADD CONSTRAINT attribute_tags_parent_tag_id_fkey FOREIGN KEY (parent_tag_id) REFERENCES public.attribute_tags(id);


--
-- Name: attribute_terminology_links attribute_terminology_links_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_terminology_links
    ADD CONSTRAINT attribute_terminology_links_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: attribute_terminology_links attribute_terminology_links_term_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_terminology_links
    ADD CONSTRAINT attribute_terminology_links_term_id_fkey FOREIGN KEY (term_id) REFERENCES public.domain_terminology_glossary(id) ON DELETE CASCADE;


--
-- Name: attribute_value_audit attribute_value_audit_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_value_audit
    ADD CONSTRAINT attribute_value_audit_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.attribute_objects(id);


--
-- Name: attribute_value_audit attribute_value_audit_persistence_entity_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attribute_value_audit
    ADD CONSTRAINT attribute_value_audit_persistence_entity_id_fkey FOREIGN KEY (persistence_entity_id) REFERENCES public.persistence_entities(id);


--
-- Name: business_attributes business_attributes_domain_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.business_attributes
    ADD CONSTRAINT business_attributes_domain_id_fkey FOREIGN KEY (domain_id) REFERENCES public.data_domains(id);


--
-- Name: business_attributes business_attributes_source_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.business_attributes
    ADD CONSTRAINT business_attributes_source_id_fkey FOREIGN KEY (source_id) REFERENCES public.attribute_sources(id);


--
-- Name: cbu_entity_associations cbu_entity_associations_cbu_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_entity_associations
    ADD CONSTRAINT cbu_entity_associations_cbu_id_fkey FOREIGN KEY (cbu_id) REFERENCES public.client_business_units(id) ON DELETE CASCADE;


--
-- Name: cbu_entity_associations cbu_entity_associations_entity_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_entity_associations
    ADD CONSTRAINT cbu_entity_associations_entity_id_fkey FOREIGN KEY (entity_id) REFERENCES public.legal_entities(id) ON DELETE CASCADE;


--
-- Name: cbu_members cbu_members_cbu_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_members
    ADD CONSTRAINT cbu_members_cbu_id_fkey FOREIGN KEY (cbu_id) REFERENCES public.client_business_units(id) ON DELETE CASCADE;


--
-- Name: cbu_members cbu_members_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_members
    ADD CONSTRAINT cbu_members_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.cbu_roles(id);


--
-- Name: cbu_product_subscriptions cbu_product_subscriptions_cbu_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_product_subscriptions
    ADD CONSTRAINT cbu_product_subscriptions_cbu_id_fkey FOREIGN KEY (cbu_id) REFERENCES public.client_business_units(id) ON DELETE CASCADE;


--
-- Name: cbu_product_subscriptions cbu_product_subscriptions_primary_contact_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_product_subscriptions
    ADD CONSTRAINT cbu_product_subscriptions_primary_contact_role_id_fkey FOREIGN KEY (primary_contact_role_id) REFERENCES public.cbu_roles(id);


--
-- Name: cbu_product_subscriptions cbu_product_subscriptions_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_product_subscriptions
    ADD CONSTRAINT cbu_product_subscriptions_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id) ON DELETE CASCADE;


--
-- Name: cbu_service_resources cbu_service_resources_cbu_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_service_resources
    ADD CONSTRAINT cbu_service_resources_cbu_id_fkey FOREIGN KEY (cbu_id) REFERENCES public.client_business_units(id) ON DELETE CASCADE;


--
-- Name: cbu_service_resources cbu_service_resources_onboarding_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_service_resources
    ADD CONSTRAINT cbu_service_resources_onboarding_request_id_fkey FOREIGN KEY (onboarding_request_id) REFERENCES public.onboarding_requests(id);


--
-- Name: cbu_service_resources cbu_service_resources_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_service_resources
    ADD CONSTRAINT cbu_service_resources_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: cbu_service_resources cbu_service_resources_service_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cbu_service_resources
    ADD CONSTRAINT cbu_service_resources_service_id_fkey FOREIGN KEY (service_id) REFERENCES public.services(id) ON DELETE CASCADE;


--
-- Name: commercial_contracts commercial_contracts_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.commercial_contracts
    ADD CONSTRAINT commercial_contracts_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id);


--
-- Name: derivation_execution_log derivation_execution_log_derived_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derivation_execution_log
    ADD CONSTRAINT derivation_execution_log_derived_attribute_id_fkey FOREIGN KEY (derived_attribute_id) REFERENCES public.attribute_objects(id);


--
-- Name: derived_attribute_quality_metrics derived_attribute_quality_metrics_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derived_attribute_quality_metrics
    ADD CONSTRAINT derived_attribute_quality_metrics_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: derived_attribute_rules derived_attribute_rules_derived_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derived_attribute_rules
    ADD CONSTRAINT derived_attribute_rules_derived_attribute_id_fkey FOREIGN KEY (derived_attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: derived_attributes derived_attributes_domain_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.derived_attributes
    ADD CONSTRAINT derived_attributes_domain_id_fkey FOREIGN KEY (domain_id) REFERENCES public.data_domains(id);


--
-- Name: dsl_execution_logs dsl_execution_logs_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dsl_execution_logs
    ADD CONSTRAINT dsl_execution_logs_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.resource_instances(instance_id) ON DELETE CASCADE;


--
-- Name: entity_attribute_values entity_attribute_values_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.entity_attribute_values
    ADD CONSTRAINT entity_attribute_values_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.attribute_objects(id) ON DELETE CASCADE;


--
-- Name: entity_attribute_values entity_attribute_values_entity_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.entity_attribute_values
    ADD CONSTRAINT entity_attribute_values_entity_id_fkey FOREIGN KEY (entity_id) REFERENCES public.legal_entities(id) ON DELETE CASCADE;


--
-- Name: entity_relationships entity_relationships_child_entity_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.entity_relationships
    ADD CONSTRAINT entity_relationships_child_entity_id_fkey FOREIGN KEY (child_entity_id) REFERENCES public.legal_entities(id) ON DELETE CASCADE;


--
-- Name: entity_relationships entity_relationships_parent_entity_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.entity_relationships
    ADD CONSTRAINT entity_relationships_parent_entity_id_fkey FOREIGN KEY (parent_entity_id) REFERENCES public.legal_entities(id) ON DELETE CASCADE;


--
-- Name: mandate_benchmarks mandate_benchmarks_mandate_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_benchmarks
    ADD CONSTRAINT mandate_benchmarks_mandate_id_fkey FOREIGN KEY (mandate_id) REFERENCES public.investment_mandates(mandate_id) ON DELETE CASCADE;


--
-- Name: mandate_instruction_channels mandate_instruction_channels_instrument_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instruction_channels
    ADD CONSTRAINT mandate_instruction_channels_instrument_id_fkey FOREIGN KEY (instrument_id) REFERENCES public.mandate_instruments(id) ON DELETE CASCADE;


--
-- Name: mandate_instrument_identifiers mandate_instrument_identifiers_instrument_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instrument_identifiers
    ADD CONSTRAINT mandate_instrument_identifiers_instrument_id_fkey FOREIGN KEY (instrument_id) REFERENCES public.mandate_instruments(id) ON DELETE CASCADE;


--
-- Name: mandate_instrument_venues mandate_instrument_venues_instrument_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instrument_venues
    ADD CONSTRAINT mandate_instrument_venues_instrument_id_fkey FOREIGN KEY (instrument_id) REFERENCES public.mandate_instruments(id) ON DELETE CASCADE;


--
-- Name: mandate_instruments mandate_instruments_mandate_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_instruments
    ADD CONSTRAINT mandate_instruments_mandate_id_fkey FOREIGN KEY (mandate_id) REFERENCES public.investment_mandates(mandate_id) ON DELETE CASCADE;


--
-- Name: mandate_lifecycle_events mandate_lifecycle_events_instrument_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mandate_lifecycle_events
    ADD CONSTRAINT mandate_lifecycle_events_instrument_id_fkey FOREIGN KEY (instrument_id) REFERENCES public.mandate_instruments(id) ON DELETE CASCADE;


--
-- Name: onboarding_approvals onboarding_approvals_workflow_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_approvals
    ADD CONSTRAINT onboarding_approvals_workflow_id_fkey FOREIGN KEY (workflow_id) REFERENCES public.onboarding_workflows(id) ON DELETE CASCADE;


--
-- Name: onboarding_dependencies onboarding_dependencies_source_task_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_dependencies
    ADD CONSTRAINT onboarding_dependencies_source_task_id_fkey FOREIGN KEY (source_task_id) REFERENCES public.onboarding_resource_tasks(id) ON DELETE CASCADE;


--
-- Name: onboarding_dependencies onboarding_dependencies_target_task_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_dependencies
    ADD CONSTRAINT onboarding_dependencies_target_task_id_fkey FOREIGN KEY (target_task_id) REFERENCES public.onboarding_resource_tasks(id) ON DELETE CASCADE;


--
-- Name: onboarding_dependencies onboarding_dependencies_workflow_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_dependencies
    ADD CONSTRAINT onboarding_dependencies_workflow_id_fkey FOREIGN KEY (workflow_id) REFERENCES public.onboarding_workflows(id) ON DELETE CASCADE;


--
-- Name: onboarding_requests onboarding_requests_cbu_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_requests
    ADD CONSTRAINT onboarding_requests_cbu_id_fkey FOREIGN KEY (cbu_id) REFERENCES public.client_business_units(id) ON DELETE CASCADE;


--
-- Name: onboarding_requests onboarding_requests_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_requests
    ADD CONSTRAINT onboarding_requests_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id) ON DELETE CASCADE;


--
-- Name: onboarding_resource_tasks onboarding_resource_tasks_capability_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_resource_tasks
    ADD CONSTRAINT onboarding_resource_tasks_capability_id_fkey FOREIGN KEY (capability_id) REFERENCES public.resource_capabilities(id) ON DELETE CASCADE;


--
-- Name: onboarding_resource_tasks onboarding_resource_tasks_resource_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_resource_tasks
    ADD CONSTRAINT onboarding_resource_tasks_resource_template_id_fkey FOREIGN KEY (resource_template_id) REFERENCES public.resource_templates(id) ON DELETE CASCADE;


--
-- Name: onboarding_resource_tasks onboarding_resource_tasks_workflow_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_resource_tasks
    ADD CONSTRAINT onboarding_resource_tasks_workflow_id_fkey FOREIGN KEY (workflow_id) REFERENCES public.onboarding_workflows(id) ON DELETE CASCADE;


--
-- Name: onboarding_tasks onboarding_tasks_onboarding_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_tasks
    ADD CONSTRAINT onboarding_tasks_onboarding_request_id_fkey FOREIGN KEY (onboarding_request_id) REFERENCES public.onboarding_requests(id) ON DELETE CASCADE;


--
-- Name: onboarding_tasks onboarding_tasks_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_tasks
    ADD CONSTRAINT onboarding_tasks_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resources(id);


--
-- Name: onboarding_workflows onboarding_workflows_cbu_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.onboarding_workflows
    ADD CONSTRAINT onboarding_workflows_cbu_id_fkey FOREIGN KEY (cbu_id) REFERENCES public.client_business_units(id) ON DELETE CASCADE;


--
-- Name: persistence_entities persistence_entities_system_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.persistence_entities
    ADD CONSTRAINT persistence_entities_system_id_fkey FOREIGN KEY (system_id) REFERENCES public.persistence_systems(id);


--
-- Name: product_option_service_mappings product_option_service_mappings_product_option_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_option_service_mappings
    ADD CONSTRAINT product_option_service_mappings_product_option_id_fkey FOREIGN KEY (product_option_id) REFERENCES public.product_options(id) ON DELETE CASCADE;


--
-- Name: product_option_service_mappings product_option_service_mappings_service_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_option_service_mappings
    ADD CONSTRAINT product_option_service_mappings_service_id_fkey FOREIGN KEY (service_id) REFERENCES public.services(id) ON DELETE CASCADE;


--
-- Name: product_options product_options_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_options
    ADD CONSTRAINT product_options_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id) ON DELETE CASCADE;


--
-- Name: product_service_mappings product_service_mappings_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_service_mappings
    ADD CONSTRAINT product_service_mappings_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id) ON DELETE CASCADE;


--
-- Name: product_service_mappings product_service_mappings_service_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_service_mappings
    ADD CONSTRAINT product_service_mappings_service_id_fkey FOREIGN KEY (service_id) REFERENCES public.services(id) ON DELETE CASCADE;


--
-- Name: product_services product_services_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_services
    ADD CONSTRAINT product_services_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id) ON DELETE CASCADE;


--
-- Name: product_services product_services_service_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_services
    ADD CONSTRAINT product_services_service_id_fkey FOREIGN KEY (service_id) REFERENCES public.services(id) ON DELETE CASCADE;


--
-- Name: resource_attribute_dependencies resource_attribute_dependencies_source_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_attribute_dependencies
    ADD CONSTRAINT resource_attribute_dependencies_source_resource_id_fkey FOREIGN KEY (source_resource_id) REFERENCES public.resource_objects(id) ON DELETE CASCADE;


--
-- Name: resource_attribute_dependencies resource_attribute_dependencies_target_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_attribute_dependencies
    ADD CONSTRAINT resource_attribute_dependencies_target_resource_id_fkey FOREIGN KEY (target_resource_id) REFERENCES public.resource_objects(id) ON DELETE CASCADE;


--
-- Name: resource_attribute_versions resource_attribute_versions_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_attribute_versions
    ADD CONSTRAINT resource_attribute_versions_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resource_objects(id) ON DELETE CASCADE;


--
-- Name: resource_dependencies resource_dependencies_dependent_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_dependencies
    ADD CONSTRAINT resource_dependencies_dependent_resource_id_fkey FOREIGN KEY (dependent_resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: resource_dependencies resource_dependencies_prerequisite_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_dependencies
    ADD CONSTRAINT resource_dependencies_prerequisite_resource_id_fkey FOREIGN KEY (prerequisite_resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: resource_instances resource_instances_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_instances
    ADD CONSTRAINT resource_instances_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.resource_sheets(resource_id) ON DELETE RESTRICT;


--
-- Name: resource_objects resource_objects_dictionary_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_objects
    ADD CONSTRAINT resource_objects_dictionary_id_fkey FOREIGN KEY (dictionary_id) REFERENCES public.resource_dictionaries(id) ON DELETE CASCADE;


--
-- Name: resource_sheet_execution_logs resource_sheet_execution_logs_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_sheet_execution_logs
    ADD CONSTRAINT resource_sheet_execution_logs_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resource_sheets(resource_id) ON DELETE CASCADE;


--
-- Name: resource_sheet_relationships resource_sheet_relationships_child_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_sheet_relationships
    ADD CONSTRAINT resource_sheet_relationships_child_resource_id_fkey FOREIGN KEY (child_resource_id) REFERENCES public.resource_sheets(resource_id) ON DELETE CASCADE;


--
-- Name: resource_sheet_relationships resource_sheet_relationships_parent_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_sheet_relationships
    ADD CONSTRAINT resource_sheet_relationships_parent_resource_id_fkey FOREIGN KEY (parent_resource_id) REFERENCES public.resource_sheets(resource_id) ON DELETE CASCADE;


--
-- Name: resource_sheets resource_sheets_domain_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_sheets
    ADD CONSTRAINT resource_sheets_domain_id_fkey FOREIGN KEY (domain_id) REFERENCES public.kyc_onboarding_domains(id);


--
-- Name: resource_sheets resource_sheets_ebnf_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_sheets
    ADD CONSTRAINT resource_sheets_ebnf_template_id_fkey FOREIGN KEY (ebnf_template_id) REFERENCES public.ebnf_grammar_templates(id);


--
-- Name: resource_template_capabilities resource_template_capabilities_capability_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_template_capabilities
    ADD CONSTRAINT resource_template_capabilities_capability_id_fkey FOREIGN KEY (capability_id) REFERENCES public.resource_capabilities(id) ON DELETE CASCADE;


--
-- Name: resource_template_capabilities resource_template_capabilities_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_template_capabilities
    ADD CONSTRAINT resource_template_capabilities_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.resource_templates(id) ON DELETE CASCADE;


--
-- Name: resource_validation_rules resource_validation_rules_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_validation_rules
    ADD CONSTRAINT resource_validation_rules_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resource_objects(id) ON DELETE CASCADE;


--
-- Name: role_service_access role_service_access_cbu_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.role_service_access
    ADD CONSTRAINT role_service_access_cbu_role_id_fkey FOREIGN KEY (cbu_role_id) REFERENCES public.cbu_roles(id) ON DELETE CASCADE;


--
-- Name: role_service_access role_service_access_service_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.role_service_access
    ADD CONSTRAINT role_service_access_service_id_fkey FOREIGN KEY (service_id) REFERENCES public.services(id) ON DELETE CASCADE;


--
-- Name: rule_compilation_queue rule_compilation_queue_rule_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_compilation_queue
    ADD CONSTRAINT rule_compilation_queue_rule_id_fkey FOREIGN KEY (rule_id) REFERENCES public.rules(id) ON DELETE CASCADE;


--
-- Name: rule_dependencies rule_dependencies_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_dependencies
    ADD CONSTRAINT rule_dependencies_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.business_attributes(id);


--
-- Name: rule_dependencies rule_dependencies_rule_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_dependencies
    ADD CONSTRAINT rule_dependencies_rule_id_fkey FOREIGN KEY (rule_id) REFERENCES public.rules(id) ON DELETE CASCADE;


--
-- Name: rule_execution_stats rule_execution_stats_rule_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_execution_stats
    ADD CONSTRAINT rule_execution_stats_rule_id_fkey FOREIGN KEY (rule_id) REFERENCES public.rules(id) ON DELETE CASCADE;


--
-- Name: rule_executions rule_executions_rule_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_executions
    ADD CONSTRAINT rule_executions_rule_id_fkey FOREIGN KEY (rule_id) REFERENCES public.rules(id);


--
-- Name: rule_versions rule_versions_rule_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rule_versions
    ADD CONSTRAINT rule_versions_rule_id_fkey FOREIGN KEY (rule_id) REFERENCES public.rules(id) ON DELETE CASCADE;


--
-- Name: rules rules_category_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rules
    ADD CONSTRAINT rules_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.rule_categories(id);


--
-- Name: rules rules_target_attribute_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rules
    ADD CONSTRAINT rules_target_attribute_id_fkey FOREIGN KEY (target_attribute_id) REFERENCES public.derived_attributes(id);


--
-- Name: service_resource_mappings service_resource_mappings_failover_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_resource_mappings
    ADD CONSTRAINT service_resource_mappings_failover_resource_id_fkey FOREIGN KEY (failover_resource_id) REFERENCES public.resource_objects(id);


--
-- Name: service_resource_mappings service_resource_mappings_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_resource_mappings
    ADD CONSTRAINT service_resource_mappings_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resource_objects(id) ON DELETE CASCADE;


--
-- Name: service_resource_mappings service_resource_mappings_service_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_resource_mappings
    ADD CONSTRAINT service_resource_mappings_service_id_fkey FOREIGN KEY (service_id) REFERENCES public.services(id) ON DELETE CASCADE;


--
-- Name: service_resources service_resources_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_resources
    ADD CONSTRAINT service_resources_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: service_resources service_resources_service_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_resources
    ADD CONSTRAINT service_resources_service_id_fkey FOREIGN KEY (service_id) REFERENCES public.services(id) ON DELETE CASCADE;


--
-- Name: ui_layout_groups ui_layout_groups_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ui_layout_groups
    ADD CONSTRAINT ui_layout_groups_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resource_objects(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

\unrestrict 45sLfvTajVPmRdsVA53Za6LGK6Ug5PdHHh0dcEgkGqPHBkwkQ5FTlayh7esnald

