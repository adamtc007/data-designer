-- Export constraint definitions
SELECT
    tc.constraint_catalog,
    tc.constraint_schema,
    tc.constraint_name,
    tc.table_catalog,
    tc.table_schema,
    tc.table_name,
    tc.constraint_type,
    tc.is_deferrable,
    tc.initially_deferred,
    tc.enforced,
    rc.match_option,
    rc.update_rule,
    rc.delete_rule,
    rc.unique_constraint_catalog,
    rc.unique_constraint_schema,
    rc.unique_constraint_name
FROM information_schema.table_constraints tc
LEFT JOIN information_schema.referential_constraints rc
    ON tc.constraint_catalog = rc.constraint_catalog
    AND tc.constraint_schema = rc.constraint_schema
    AND tc.constraint_name = rc.constraint_name
WHERE tc.table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY tc.table_schema, tc.table_name, tc.constraint_type, tc.constraint_name;

-- Export constraint column usage
SELECT
    kcu.constraint_catalog,
    kcu.constraint_schema,
    kcu.constraint_name,
    kcu.table_catalog,
    kcu.table_schema,
    kcu.table_name,
    kcu.column_name,
    kcu.ordinal_position,
    kcu.position_in_unique_constraint
FROM information_schema.key_column_usage kcu
JOIN information_schema.table_constraints tc
    ON kcu.constraint_catalog = tc.constraint_catalog
    AND kcu.constraint_schema = tc.constraint_schema
    AND kcu.constraint_name = tc.constraint_name
WHERE kcu.table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY kcu.table_schema, kcu.table_name, kcu.constraint_name, kcu.ordinal_position;

-- Export check constraints
SELECT
    cc.constraint_catalog,
    cc.constraint_schema,
    cc.constraint_name,
    cc.check_clause
FROM information_schema.check_constraints cc
JOIN information_schema.table_constraints tc
    ON cc.constraint_catalog = tc.constraint_catalog
    AND cc.constraint_schema = tc.constraint_schema
    AND cc.constraint_name = tc.constraint_name
WHERE tc.table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY cc.constraint_schema, tc.table_name, cc.constraint_name;