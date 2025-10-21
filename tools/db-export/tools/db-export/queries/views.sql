-- Export view definitions
SELECT
    table_schema,
    table_name,
    view_definition,
    check_option,
    is_updatable,
    is_insertable_into,
    is_trigger_updatable,
    is_trigger_deletable,
    is_trigger_insertable_into
FROM information_schema.views
WHERE table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY table_schema, table_name;

-- Export materialized views
SELECT
    n.nspname AS schema_name,
    c.relname AS view_name,
    pg_get_viewdef(c.oid) AS view_definition,
    c.relispopulated AS is_populated
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE c.relkind = 'm'  -- materialized views
    AND n.nspname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY n.nspname, c.relname;

-- Export view column details
SELECT
    table_schema,
    table_name,
    column_name,
    ordinal_position,
    column_default,
    is_nullable,
    data_type,
    character_maximum_length,
    numeric_precision,
    numeric_scale,
    is_updatable
FROM information_schema.columns
WHERE table_name IN (
    SELECT table_name
    FROM information_schema.views
    WHERE table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
)
AND table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY table_schema, table_name, ordinal_position;