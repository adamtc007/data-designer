-- Export row-level security policies
SELECT
    schemaname,
    tablename,
    policyname,
    permissive,
    roles,
    cmd,
    qual,
    with_check
FROM pg_policies
WHERE schemaname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY schemaname, tablename, policyname;

-- Export RLS status for tables
SELECT
    n.nspname AS schema_name,
    c.relname AS table_name,
    c.relrowsecurity AS row_security_enabled,
    c.relforcerowsecurity AS force_row_security
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE c.relkind = 'r'  -- regular tables only
    AND n.nspname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
    AND (c.relrowsecurity OR c.relforcerowsecurity)
ORDER BY n.nspname, c.relname;