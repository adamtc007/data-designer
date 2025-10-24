-- Fix materialized view creation
-- Date: 2025-01-09

-- Drop if exists first
DROP MATERIALIZED VIEW IF EXISTS mv_data_dictionary CASCADE;

-- Create comprehensive data dictionary view
CREATE MATERIALIZED VIEW mv_data_dictionary AS
WITH all_attributes AS (
    -- Business attributes (source data from configured tables)
    SELECT
        'business' as attribute_type,
        entity_name,
        attribute_name,
        entity_name || '.' || attribute_name as full_path,
        data_type,
        sql_type,
        rust_type,
        description,
        required,
        validation_pattern,
        NULL::TEXT as rule_definition,
        NULL::INTEGER as rule_id,
        'active' as status
    FROM business_attributes
    WHERE is_active = true

    UNION ALL

    -- Derived attributes (calculated via rules)
    SELECT
        'derived' as attribute_type,
        da.entity_name,
        da.attribute_name,
        da.entity_name || '.' || da.attribute_name as full_path,
        da.data_type,
        da.sql_type,
        da.rust_type,
        da.description,
        false as required,
        NULL as validation_pattern,
        r.rule_definition,
        r.id as rule_id,
        COALESCE(r.status, 'draft') as status
    FROM derived_attributes da
    LEFT JOIN rules r ON r.target_attribute_id = da.id

    UNION ALL

    -- System attributes from investment_mandates schema
    SELECT DISTINCT
        'system' as attribute_type,
        c.table_name as entity_name,
        c.column_name as attribute_name,
        c.table_name || '.' || c.column_name as full_path,
        c.data_type,
        c.data_type as sql_type,
        CASE
            WHEN c.data_type LIKE '%int%' THEN 'i32'
            WHEN c.data_type LIKE '%numeric%' OR c.data_type LIKE '%decimal%' THEN 'f64'
            WHEN c.data_type = 'boolean' THEN 'bool'
            WHEN c.data_type LIKE '%json%' THEN 'JsonValue'
            WHEN c.data_type LIKE 'vector%' THEN 'Vec<f32>'
            WHEN c.data_type LIKE '%timestamp%' THEN 'DateTime'
            WHEN c.data_type = 'date' THEN 'Date'
            WHEN c.data_type LIKE 'text%' THEN 'String'
            WHEN c.data_type LIKE 'character%' THEN 'String'
            ELSE 'String'
        END as rust_type,
        obj_description(pgc.oid, 'pg_class') as description,
        c.is_nullable = 'NO' as required,
        NULL as validation_pattern,
        NULL::TEXT as rule_definition,
        NULL::INTEGER as rule_id,
        'active' as status
    FROM information_schema.columns c
    LEFT JOIN pg_class pgc ON pgc.relname = c.table_name
    WHERE c.table_schema = 'public'
        AND c.table_name IN (
            'investment_mandates', 'mandate_instruments', 'mandate_benchmarks',
            'mandate_instrument_venues', 'mandate_instruction_channels',
            'mandate_instrument_identifiers', 'mandate_lifecycle_events'
        )
)
SELECT
    attribute_type,
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
ORDER BY entity_name, attribute_type, attribute_name;

-- Create indexes for faster queries
CREATE UNIQUE INDEX idx_mv_data_dictionary_path
    ON mv_data_dictionary(full_path);
CREATE INDEX idx_mv_data_dictionary_entity
    ON mv_data_dictionary(entity_name);
CREATE INDEX idx_mv_data_dictionary_type
    ON mv_data_dictionary(attribute_type);

-- Grant permissions
GRANT SELECT ON mv_data_dictionary TO data_designer_app;

-- Show summary
SELECT
    attribute_type,
    entity_name,
    COUNT(*) as attribute_count
FROM mv_data_dictionary
GROUP BY attribute_type, entity_name
ORDER BY attribute_type, entity_name;