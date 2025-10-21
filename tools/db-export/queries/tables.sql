-- Export table definitions
SELECT
    schemaname,
    tablename,
    tableowner,
    tablespace,
    hasindexes,
    hasrules,
    hastriggers,
    rowsecurity
FROM pg_tables
WHERE schemaname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY schemaname, tablename;

-- Get detailed table structure with DDL
SELECT
    t.schemaname,
    t.tablename,
    pg_get_table_def(c.oid) as table_ddl
FROM pg_tables t
JOIN pg_class c ON c.relname = t.tablename
JOIN pg_namespace n ON n.oid = c.relnamespace AND n.nspname = t.schemaname
WHERE t.schemaname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY t.schemaname, t.tablename;