-- Export table privileges
SELECT
    table_catalog,
    table_schema,
    table_name,
    grantor,
    grantee,
    privilege_type,
    is_grantable,
    with_hierarchy
FROM information_schema.table_privileges
WHERE table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY table_schema, table_name, grantee, privilege_type;

-- Export column privileges
SELECT
    table_catalog,
    table_schema,
    table_name,
    column_name,
    grantor,
    grantee,
    privilege_type,
    is_grantable
FROM information_schema.column_privileges
WHERE table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY table_schema, table_name, column_name, grantee, privilege_type;

-- Export schema privileges
SELECT
    catalog_name,
    schema_name,
    schema_owner,
    default_character_set_catalog,
    default_character_set_schema,
    default_character_set_name,
    sql_path
FROM information_schema.schemata
WHERE schema_name NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY schema_name;

-- Export routine (function/procedure) privileges
SELECT
    specific_catalog,
    specific_schema,
    specific_name,
    routine_catalog,
    routine_schema,
    routine_name,
    grantor,
    grantee,
    privilege_type,
    is_grantable
FROM information_schema.routine_privileges
WHERE routine_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY routine_schema, routine_name, grantee, privilege_type;

-- Export database-level privileges
SELECT
    r.rolname AS role_name,
    d.datname AS database_name,
    has_database_privilege(r.rolname, d.datname, 'CONNECT') AS can_connect,
    has_database_privilege(r.rolname, d.datname, 'CREATE') AS can_create,
    has_database_privilege(r.rolname, d.datname, 'TEMPORARY') AS can_temp
FROM pg_roles r
CROSS JOIN pg_database d
WHERE d.datname = current_database()
    AND r.rolname NOT LIKE 'pg_%'
    AND r.rolname != 'postgres'
ORDER BY r.rolname;