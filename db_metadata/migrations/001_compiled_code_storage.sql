-- Migration: Add compiled code storage and data dictionary
-- Date: 2025-01-09
-- Description: Enhance rules table with compiled code storage and create unified data dictionary

-- 1. Add compiled code columns to rules table
ALTER TABLE rules ADD COLUMN IF NOT EXISTS compiled_rust_code TEXT;
ALTER TABLE rules ADD COLUMN IF NOT EXISTS compiled_wasm_binary BYTEA;
ALTER TABLE rules ADD COLUMN IF NOT EXISTS compilation_status VARCHAR(20) DEFAULT 'pending'
    CHECK (compilation_status IN ('pending', 'compiling', 'success', 'failed', 'outdated'));
ALTER TABLE rules ADD COLUMN IF NOT EXISTS compilation_error TEXT;
ALTER TABLE rules ADD COLUMN IF NOT EXISTS compilation_timestamp TIMESTAMP;
ALTER TABLE rules ADD COLUMN IF NOT EXISTS compiler_version VARCHAR(50);
ALTER TABLE rules ADD COLUMN IF NOT EXISTS execution_count BIGINT DEFAULT 0;
ALTER TABLE rules ADD COLUMN IF NOT EXISTS avg_execution_time_ms NUMERIC(10,3);

-- 2. Create rule compilation queue for async compilation
CREATE TABLE IF NOT EXISTS rule_compilation_queue (
    id SERIAL PRIMARY KEY,
    rule_id INTEGER REFERENCES rules(id) ON DELETE CASCADE,
    compilation_type VARCHAR(20) CHECK (compilation_type IN ('rust', 'wasm', 'both')),
    priority INTEGER DEFAULT 5 CHECK (priority BETWEEN 1 AND 10),
    status VARCHAR(20) DEFAULT 'pending'
        CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'cancelled')),
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    worker_id VARCHAR(50),
    UNIQUE(rule_id, compilation_type, status)
);

-- 3. Create comprehensive data dictionary view
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_data_dictionary AS
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
    WHERE is_active = true OR is_active IS NULL

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
            ELSE 'String'
        END as rust_type,
        pgd.description,
        c.is_nullable = 'NO' as required,
        NULL as validation_pattern,
        NULL::TEXT as rule_definition,
        NULL::INTEGER as rule_id,
        'active' as status
    FROM information_schema.columns c
    LEFT JOIN pg_catalog.pg_statio_all_tables st ON c.table_name = st.relname
    LEFT JOIN pg_catalog.pg_description pgd ON pgd.objoid = st.relid
        AND pgd.objsubid = (
            SELECT attnum FROM pg_attribute
            WHERE attrelid = st.relid AND attname = c.column_name
        )
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

-- Create index for faster queries
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_data_dictionary_path
    ON mv_data_dictionary(full_path);
CREATE INDEX IF NOT EXISTS idx_mv_data_dictionary_entity
    ON mv_data_dictionary(entity_name);
CREATE INDEX IF NOT EXISTS idx_mv_data_dictionary_type
    ON mv_data_dictionary(attribute_type);

-- 4. Create function to refresh data dictionary
CREATE OR REPLACE FUNCTION refresh_data_dictionary()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_data_dictionary;
END;
$$ LANGUAGE plpgsql;

-- 5. Create function to discover all available attributes dynamically
CREATE OR REPLACE FUNCTION discover_all_attributes(
    p_entity_filter VARCHAR DEFAULT NULL,
    p_include_system BOOLEAN DEFAULT true
)
RETURNS TABLE (
    attribute_type VARCHAR,
    entity_name VARCHAR,
    attribute_name VARCHAR,
    full_path VARCHAR,
    data_type VARCHAR,
    sql_type VARCHAR,
    rust_type VARCHAR,
    description TEXT,
    is_nullable BOOLEAN,
    is_derived BOOLEAN,
    derivation_rule TEXT,
    rule_id INTEGER
) AS $$
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
$$ LANGUAGE plpgsql;

-- 6. Create function to compile rule to Rust code
CREATE OR REPLACE FUNCTION compile_rule_to_rust(
    p_rule_id INTEGER,
    p_dsl_code TEXT
)
RETURNS TEXT AS $$
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
$$ LANGUAGE plpgsql;

-- 7. Create trigger to mark rules for recompilation when updated
CREATE OR REPLACE FUNCTION mark_rule_for_recompilation()
RETURNS TRIGGER AS $$
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
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_mark_rule_for_recompilation ON rules;
CREATE TRIGGER trigger_mark_rule_for_recompilation
    BEFORE UPDATE ON rules
    FOR EACH ROW
    EXECUTE FUNCTION mark_rule_for_recompilation();

-- 8. Create view for attribute dependencies
CREATE OR REPLACE VIEW v_attribute_dependencies AS
SELECT
    r.rule_id,
    r.rule_name,
    da.entity_name || '.' || da.attribute_name as target_attribute,
    array_agg(
        ba.entity_name || '.' || ba.attribute_name ORDER BY ba.attribute_name
    ) as source_attributes
FROM rules r
LEFT JOIN derived_attributes da ON r.target_attribute_id = da.id
LEFT JOIN rule_dependencies rd ON rd.rule_id = r.id
LEFT JOIN business_attributes ba ON rd.attribute_id = ba.id
GROUP BY r.id, r.rule_id, r.rule_name, da.entity_name, da.attribute_name;

-- 9. Add performance tracking
CREATE TABLE IF NOT EXISTS rule_execution_stats (
    id SERIAL PRIMARY KEY,
    rule_id INTEGER REFERENCES rules(id) ON DELETE CASCADE,
    execution_date DATE DEFAULT CURRENT_DATE,
    total_executions INTEGER DEFAULT 0,
    successful_executions INTEGER DEFAULT 0,
    failed_executions INTEGER DEFAULT 0,
    avg_execution_time_ms NUMERIC(10,3),
    min_execution_time_ms NUMERIC(10,3),
    max_execution_time_ms NUMERIC(10,3),
    used_compiled_code BOOLEAN DEFAULT false,
    UNIQUE(rule_id, execution_date)
);

-- 10. Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_rules_compilation_status
    ON rules(compilation_status);
CREATE INDEX IF NOT EXISTS idx_rules_target_attribute
    ON rules(target_attribute_id);
CREATE INDEX IF NOT EXISTS idx_compilation_queue_status
    ON rule_compilation_queue(status, priority DESC);
CREATE INDEX IF NOT EXISTS idx_rule_execution_stats_date
    ON rule_execution_stats(execution_date DESC);

-- Initial data dictionary refresh
REFRESH MATERIALIZED VIEW mv_data_dictionary;

-- Grant permissions
GRANT SELECT ON mv_data_dictionary TO data_designer_app;
GRANT EXECUTE ON FUNCTION discover_all_attributes TO data_designer_app;
GRANT EXECUTE ON FUNCTION refresh_data_dictionary TO data_designer_app;
GRANT ALL ON rule_compilation_queue TO data_designer_app;
GRANT ALL ON rule_execution_stats TO data_designer_app;