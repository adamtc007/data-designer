-- Export function and procedure definitions
SELECT
    n.nspname AS schema_name,
    p.proname AS function_name,
    pg_catalog.pg_get_function_identity_arguments(p.oid) AS arguments,
    t.typname AS return_type,
    CASE p.prokind
        WHEN 'f' THEN 'FUNCTION'
        WHEN 'p' THEN 'PROCEDURE'
        WHEN 'a' THEN 'AGGREGATE'
        WHEN 'w' THEN 'WINDOW'
        ELSE 'OTHER'
    END AS function_type,
    l.lanname AS language,
    p.prosrc AS source_code,
    p.provolatile AS volatility,
    p.proisstrict AS is_strict,
    p.prosecdef AS security_definer,
    p.proleakproof AS leak_proof,
    p.procost AS cost,
    p.prorows AS estimated_rows,
    pg_catalog.pg_get_functiondef(p.oid) AS function_definition
FROM pg_proc p
JOIN pg_namespace n ON n.oid = p.pronamespace
JOIN pg_type t ON t.oid = p.prorettype
JOIN pg_language l ON l.oid = p.prolang
WHERE n.nspname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
    AND p.prokind IN ('f', 'p')  -- functions and procedures
ORDER BY n.nspname, p.proname, pg_catalog.pg_get_function_identity_arguments(p.oid);

-- Export aggregate functions
SELECT
    n.nspname AS schema_name,
    p.proname AS aggregate_name,
    pg_catalog.pg_get_function_identity_arguments(p.oid) AS arguments,
    t.typname AS return_type,
    pg_catalog.pg_get_functiondef(p.oid) AS aggregate_definition
FROM pg_proc p
JOIN pg_namespace n ON n.oid = p.pronamespace
JOIN pg_type t ON t.oid = p.prorettype
WHERE n.nspname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
    AND p.prokind = 'a'  -- aggregates
ORDER BY n.nspname, p.proname;

-- Export triggers
SELECT
    n.nspname AS schema_name,
    c.relname AS table_name,
    t.tgname AS trigger_name,
    pg_catalog.pg_get_triggerdef(t.oid) AS trigger_definition,
    CASE t.tgtype & 66
        WHEN 2 THEN 'BEFORE'
        WHEN 64 THEN 'INSTEAD OF'
        ELSE 'AFTER'
    END AS trigger_timing,
    CASE t.tgtype & 28
        WHEN 4 THEN 'INSERT'
        WHEN 8 THEN 'DELETE'
        WHEN 16 THEN 'UPDATE'
        WHEN 12 THEN 'INSERT OR DELETE'
        WHEN 20 THEN 'INSERT OR UPDATE'
        WHEN 24 THEN 'UPDATE OR DELETE'
        WHEN 28 THEN 'INSERT OR UPDATE OR DELETE'
    END AS trigger_events,
    t.tgenabled AS is_enabled
FROM pg_trigger t
JOIN pg_class c ON c.oid = t.tgrelid
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE n.nspname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
    AND NOT t.tgisinternal  -- exclude internal triggers
ORDER BY n.nspname, c.relname, t.tgname;