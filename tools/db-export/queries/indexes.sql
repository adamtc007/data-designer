-- Export index definitions
SELECT
    schemaname,
    tablename,
    indexname,
    tablespace,
    indexdef
FROM pg_indexes
WHERE schemaname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY schemaname, tablename, indexname;

-- Additional index metadata
SELECT
    n.nspname AS schema_name,
    t.relname AS table_name,
    i.relname AS index_name,
    idx.indisunique AS is_unique,
    idx.indisprimary AS is_primary,
    idx.indisexclusion AS is_exclusion,
    idx.indimmediate AS is_immediate,
    idx.indisclustered AS is_clustered,
    idx.indisvalid AS is_valid,
    idx.indcheckxmin AS check_xmin,
    idx.indisready AS is_ready,
    idx.indislive AS is_live,
    idx.indisreplident AS is_replica_identity,
    array_to_string(array_agg(a.attname ORDER BY a.attnum), ', ') AS indexed_columns
FROM pg_index idx
JOIN pg_class i ON i.oid = idx.indexrelid
JOIN pg_class t ON t.oid = idx.indrelid
JOIN pg_namespace n ON n.oid = t.relnamespace
JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(idx.indkey)
WHERE n.nspname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
GROUP BY n.nspname, t.relname, i.relname, idx.indisunique, idx.indisprimary,
         idx.indisexclusion, idx.indimmediate, idx.indisclustered, idx.indisvalid,
         idx.indcheckxmin, idx.indisready, idx.indislive, idx.indisreplident
ORDER BY n.nspname, t.relname, i.relname;