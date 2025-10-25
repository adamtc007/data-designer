-- Migration 004: Cleanup Unused Tables
-- Drops tables that are not referenced in grpc-server, onboarding, web-ui, or data-designer-core

-- SAFETY: This migration drops tables with no code references
-- Backup your data before running if any of these tables contain important data

-- Drop unused rule-related tables
DROP TABLE IF EXISTS rule_dependencies CASCADE;
DROP TABLE IF EXISTS rule_executions CASCADE;
DROP TABLE IF EXISTS rule_versions CASCADE;
DROP TABLE IF EXISTS rule_categories CASCADE;

-- Drop unused attribute-related tables
DROP TABLE IF EXISTS derived_attributes CASCADE;
DROP TABLE IF EXISTS business_attributes CASCADE;
DROP TABLE IF EXISTS attribute_sources CASCADE;
DROP TABLE IF EXISTS data_domains CASCADE;

-- Drop unused resource dependency table
DROP TABLE IF EXISTS resource_dependencies CASCADE;

COMMENT ON DATABASE data_designer IS 'Data Designer - Cleaned up unused tables in migration 004';

-- Note: Keeping these tables despite low/no usage:
-- - rules (0 rows, but 5 code references in data-designer-core)
-- - product_services (FK integrity)
-- - service_resources (FK integrity)
-- - onboarding_requests, onboarding_tasks (future use)
-- - cbu_product_subscriptions, cbu_service_resources, role_service_access (FK integrity)
