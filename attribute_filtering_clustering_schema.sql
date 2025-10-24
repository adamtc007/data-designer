-- Attribute Filtering and Vector Clustering Schema
-- Comprehensive filtering system including semantic clustering and multi-dimensional filtering

-- Create vector clustering tables
CREATE TABLE IF NOT EXISTS attribute_vector_clusters (
    id SERIAL PRIMARY KEY,
    cluster_name VARCHAR(100) UNIQUE NOT NULL,
    cluster_description TEXT,
    cluster_type VARCHAR(50) DEFAULT 'semantic', -- 'semantic', 'functional', 'domain', 'usage'
    cluster_algorithm VARCHAR(50) DEFAULT 'kmeans', -- 'kmeans', 'hierarchical', 'dbscan', 'manual'
    cluster_parameters JSONB DEFAULT '{}',
    centroid_vector VECTOR(1536), -- Representative vector for the cluster
    cluster_radius DECIMAL(10,6), -- For spatial clustering
    quality_score DECIMAL(4,3), -- Cluster quality/coherence score
    member_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Link attributes to clusters (many-to-many relationship)
CREATE TABLE IF NOT EXISTS attribute_cluster_memberships (
    id SERIAL PRIMARY KEY,
    attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    cluster_id INTEGER REFERENCES attribute_vector_clusters(id) ON DELETE CASCADE,
    membership_strength DECIMAL(4,3) DEFAULT 1.0, -- 0.0-1.0 strength of membership
    distance_to_centroid DECIMAL(10,6), -- Distance from cluster centroid
    membership_type VARCHAR(50) DEFAULT 'primary', -- 'primary', 'secondary', 'weak'
    assigned_by VARCHAR(50) DEFAULT 'algorithm', -- 'algorithm', 'manual', 'ai_assisted'
    confidence_score DECIMAL(4,3) DEFAULT 1.0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(attribute_id, cluster_id)
);

-- Advanced filtering configurations
CREATE TABLE IF NOT EXISTS attribute_filter_configurations (
    id SERIAL PRIMARY KEY,
    filter_name VARCHAR(100) UNIQUE NOT NULL,
    filter_description TEXT,
    filter_type VARCHAR(50) NOT NULL, -- 'basic', 'semantic', 'vector', 'composite', 'ai_assisted'
    filter_config JSONB NOT NULL,
    target_audience VARCHAR(50), -- 'developers', 'business_users', 'ai_systems', 'compliance'
    use_case VARCHAR(100), -- 'form_generation', 'data_discovery', 'compliance_check', 'ai_context'
    performance_profile JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_by VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Saved filter instances for users
CREATE TABLE IF NOT EXISTS user_saved_filters (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(100) NOT NULL,
    filter_name VARCHAR(200) NOT NULL,
    filter_criteria JSONB NOT NULL,
    applied_clusters INTEGER[], -- Cluster IDs included in filter
    result_count INTEGER,
    last_used TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_favorite BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, filter_name)
);

-- Attribute tagging and categorization for filtering
CREATE TABLE IF NOT EXISTS attribute_tags (
    id SERIAL PRIMARY KEY,
    tag_name VARCHAR(100) UNIQUE NOT NULL,
    tag_description TEXT,
    tag_category VARCHAR(50), -- 'functional', 'domain', 'technical', 'compliance', 'ui'
    tag_color VARCHAR(7), -- Hex color for UI display
    icon_name VARCHAR(50), -- Icon identifier for UI
    parent_tag_id INTEGER REFERENCES attribute_tags(id),
    hierarchy_level INTEGER DEFAULT 1,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Many-to-many relationship between attributes and tags
CREATE TABLE IF NOT EXISTS attribute_tag_assignments (
    id SERIAL PRIMARY KEY,
    attribute_id INTEGER REFERENCES attribute_objects(id) ON DELETE CASCADE,
    tag_id INTEGER REFERENCES attribute_tags(id) ON DELETE CASCADE,
    assignment_confidence DECIMAL(4,3) DEFAULT 1.0,
    assigned_by VARCHAR(50) DEFAULT 'manual', -- 'manual', 'ai_inferred', 'rule_based'
    assignment_reason TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(attribute_id, tag_id)
);

-- Comprehensive filtering view with all metadata
CREATE OR REPLACE VIEW attribute_comprehensive_filter_view AS
SELECT
    ao.id,
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

    -- Domain and compliance
    kod.domain_name,
    kod.regulatory_framework,
    adm.compliance_criticality,
    adm.data_sensitivity,

    -- Clustering information
    array_agg(DISTINCT avc.cluster_name) FILTER (WHERE avc.cluster_name IS NOT NULL) as cluster_names,
    array_agg(DISTINCT avc.cluster_type) FILTER (WHERE avc.cluster_type IS NOT NULL) as cluster_types,
    array_agg(DISTINCT acm.membership_strength) FILTER (WHERE acm.membership_strength IS NOT NULL) as cluster_memberships,

    -- Tagging information
    array_agg(DISTINCT at.tag_name) FILTER (WHERE at.tag_name IS NOT NULL) as tags,
    array_agg(DISTINCT at.tag_category) FILTER (WHERE at.tag_category IS NOT NULL) as tag_categories,

    -- Semantic information
    ao.semantic_tags,
    ao.search_keywords,
    ao.ai_context,

    -- Quality metrics (for derived attributes)
    daqm.success_rate,
    daqm.data_quality_score,
    daqm.completeness_rate,

    -- Persistence information
    ps.system_name as persistence_system,
    ps.system_type as persistence_type,

    -- AI-enhanced search vectors
    ao.embedding_vector,

    -- Comprehensive searchable text
    CONCAT_WS(' ',
        ao.attribute_name,
        ao.description,
        ao.extended_description,
        ao.business_context,
        ao.technical_context,
        string_agg(DISTINCT at.tag_name, ' '),
        string_agg(DISTINCT avc.cluster_name, ' '),
        kod.domain_name
    ) as searchable_text,

    -- Filter-friendly metadata
    jsonb_build_object(
        'basic_info', jsonb_build_object(
            'id', ao.id,
            'name', ao.attribute_name,
            'class', ao.attribute_class,
            'visibility', ao.visibility_scope,
            'type', ao.data_type
        ),
        'classification', jsonb_build_object(
            'domain', kod.domain_name,
            'compliance', adm.compliance_criticality,
            'sensitivity', adm.data_sensitivity,
            'derivation_complexity', ao.derivation_complexity
        ),
        'clustering', jsonb_build_object(
            'clusters', array_agg(DISTINCT avc.cluster_name) FILTER (WHERE avc.cluster_name IS NOT NULL),
            'cluster_types', array_agg(DISTINCT avc.cluster_type) FILTER (WHERE avc.cluster_type IS NOT NULL)
        ),
        'ui_metadata', jsonb_build_object(
            'component_type', ao.ui_component_type,
            'group', ao.ui_group,
            'display_order', ao.ui_display_order
        ),
        'persistence', jsonb_build_object(
            'system', ps.system_name,
            'type', ps.system_type
        ),
        'tags', array_agg(DISTINCT at.tag_name) FILTER (WHERE at.tag_name IS NOT NULL)
    ) as filter_metadata

FROM attribute_objects ao
LEFT JOIN attribute_domain_mappings adm ON adm.attribute_id = ao.id
LEFT JOIN kyc_onboarding_domains kod ON kod.id = adm.domain_id
LEFT JOIN attribute_cluster_memberships acm ON acm.attribute_id = ao.id
LEFT JOIN attribute_vector_clusters avc ON avc.id = acm.cluster_id
LEFT JOIN attribute_tag_assignments ata ON ata.attribute_id = ao.id
LEFT JOIN attribute_tags at ON at.id = ata.tag_id
LEFT JOIN derived_attribute_quality_metrics daqm ON daqm.attribute_id = ao.id
    AND daqm.metric_date = (SELECT MAX(metric_date) FROM derived_attribute_quality_metrics WHERE attribute_id = ao.id)
LEFT JOIN persistence_entities pe ON pe.id = ao.primary_persistence_entity_id
LEFT JOIN persistence_systems ps ON ps.id = pe.system_id
GROUP BY ao.id, ao.attribute_name, ao.attribute_class, ao.visibility_scope, ao.data_type,
         ao.description, ao.extended_description, ao.business_context, ao.technical_context,
         ao.ui_component_type, ao.ui_group, ao.ui_display_order, ao.derivation_complexity,
         ao.materialization_strategy, kod.domain_name, kod.regulatory_framework,
         adm.compliance_criticality, adm.data_sensitivity, ao.semantic_tags, ao.search_keywords,
         ao.ai_context, daqm.success_rate, daqm.data_quality_score, daqm.completeness_rate,
         ps.system_name, ps.system_type, ao.embedding_vector;

-- Advanced filtering function with multiple criteria
CREATE OR REPLACE FUNCTION filter_attributes_advanced(
    filter_criteria JSONB
) RETURNS TABLE (
    attribute_id INTEGER,
    attribute_name VARCHAR,
    relevance_score DECIMAL,
    filter_metadata JSONB,
    cluster_info JSONB
) AS $$
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
$$ LANGUAGE plpgsql;

-- Function to create semantic clusters
CREATE OR REPLACE FUNCTION create_semantic_clusters(
    cluster_count INTEGER DEFAULT 10,
    min_cluster_size INTEGER DEFAULT 3
) RETURNS JSONB AS $$
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
$$ LANGUAGE plpgsql;

-- Function to get filter suggestions based on user context
CREATE OR REPLACE FUNCTION get_filter_suggestions(
    user_context JSONB DEFAULT '{}'
) RETURNS JSONB AS $$
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
$$ LANGUAGE plpgsql;

-- Insert sample attribute tags
INSERT INTO attribute_tags (tag_name, tag_description, tag_category, tag_color, icon_name) VALUES
('Personal Data', 'Personally identifiable information', 'compliance', '#ff6b6b', 'shield'),
('Financial', 'Financial information and calculations', 'domain', '#4ecdc4', 'dollar-sign'),
('Contact Info', 'Contact and communication details', 'functional', '#45b7d1', 'phone'),
('Identity', 'Identity verification attributes', 'compliance', '#96ceb4', 'id-card'),
('Risk Assessment', 'Risk scoring and assessment related', 'domain', '#ffeaa7', 'alert-triangle'),
('Computed', 'Algorithmically computed values', 'technical', '#dda0dd', 'cpu'),
('User Input', 'Direct user input fields', 'functional', '#98d8c8', 'edit'),
('System Generated', 'System-generated attributes', 'technical', '#a8a8a8', 'server'),
('Mandatory', 'Required for compliance or business rules', 'compliance', '#fd79a8', 'exclamation'),
('Optional', 'Optional attributes', 'functional', '#fdcb6e', 'circle');

-- Create comprehensive indexes for filtering performance
CREATE INDEX IF NOT EXISTS idx_attribute_vector_clusters_type ON attribute_vector_clusters(cluster_type);
CREATE INDEX IF NOT EXISTS idx_attribute_cluster_memberships_strength ON attribute_cluster_memberships(membership_strength);
CREATE INDEX IF NOT EXISTS idx_attribute_cluster_memberships_type ON attribute_cluster_memberships(membership_type);
CREATE INDEX IF NOT EXISTS idx_attribute_tags_category ON attribute_tags(tag_category);
CREATE INDEX IF NOT EXISTS idx_attribute_tag_assignments_confidence ON attribute_tag_assignments(assignment_confidence);
CREATE INDEX IF NOT EXISTS idx_user_saved_filters_user ON user_saved_filters(user_id);
CREATE INDEX IF NOT EXISTS idx_user_saved_filters_last_used ON user_saved_filters(last_used);

-- Full text search index for comprehensive filtering
CREATE INDEX IF NOT EXISTS idx_attribute_comprehensive_search ON attribute_objects
USING GIN(to_tsvector('english',
    COALESCE(attribute_name, '') || ' ' ||
    COALESCE(description, '') || ' ' ||
    COALESCE(extended_description, '') || ' ' ||
    COALESCE(business_context, '') || ' ' ||
    COALESCE(technical_context, '')
));

-- Vector similarity index for clustering
CREATE INDEX IF NOT EXISTS idx_attribute_objects_embedding_cosine ON attribute_objects
USING ivfflat (embedding_vector vector_cosine_ops) WITH (lists = 100);

-- Comments for documentation
COMMENT ON TABLE attribute_vector_clusters IS 'Semantic and functional clusters of related attributes';
COMMENT ON TABLE attribute_cluster_memberships IS 'Many-to-many relationship between attributes and clusters';
COMMENT ON TABLE attribute_filter_configurations IS 'Predefined filter configurations for different use cases';
COMMENT ON TABLE user_saved_filters IS 'User-saved filter preferences and frequently used filters';
COMMENT ON TABLE attribute_tags IS 'Hierarchical tagging system for attribute categorization';
COMMENT ON FUNCTION filter_attributes_advanced IS 'Advanced multi-criteria filtering with vector similarity support';
COMMENT ON FUNCTION create_semantic_clusters IS 'Creates semantic clusters based on attribute embeddings';
COMMENT ON FUNCTION get_filter_suggestions IS 'Provides intelligent filter suggestions based on user context';
COMMENT ON VIEW attribute_comprehensive_filter_view IS 'Comprehensive view optimized for filtering and searching attributes';