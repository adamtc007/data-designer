use anyhow::{Result, Context};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::collections::HashMap;
use uuid::Uuid;

use data_designer_core::db::*;

/// Test database manager with isolation and cleanup
pub struct TestDatabase {
    pool: PgPool,
    test_schema: String,
    cleanup_list: Vec<String>,
}

impl TestDatabase {
    /// Create a new test database with isolated schema
    pub async fn new() -> Result<Self> {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .unwrap_or_else(|_| "postgresql://adamtc007@localhost/data_designer_test".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .context("Failed to connect to test database")?;

        // Create a unique schema for this test run
        let test_schema = format!("test_{}", Uuid::new_v4().to_string().replace('-', "_"));

        let instance = Self {
            pool,
            test_schema,
            cleanup_list: Vec::new(),
        };

        instance.setup_test_schema().await?;
        Ok(instance)
    }

    /// Set up the test schema with all required tables
    async fn setup_test_schema(&self) -> Result<()> {
        // Create schema
        sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS {}", self.test_schema))
            .execute(&self.pool)
            .await?;

        // Set search path for this connection
        sqlx::query(&format!("SET search_path TO {}, public", self.test_schema))
            .execute(&self.pool)
            .await?;

        // Create all required tables in the test schema
        self.create_test_tables().await?;

        tracing::info!("Created test database schema: {}", self.test_schema);
        Ok(())
    }

    /// Create all required tables for testing
    async fn create_test_tables(&self) -> Result<()> {
        // Read and execute schema from the main database schema file
        let schema_sql = tokio::fs::read_to_string("database/schema.sql").await
            .context("Failed to read database schema file")?;

        // Execute schema creation
        for statement in schema_sql.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() && !statement.starts_with("--") {
                sqlx::query(statement)
                    .execute(&self.pool)
                    .await
                    .with_context(|| format!("Failed to execute schema statement: {}", statement))?;
            }
        }

        Ok(())
    }

    /// Get the database pool for tests
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Insert test rule and track for cleanup
    pub async fn insert_test_rule(&mut self, rule: &rules::CreateRuleRequest) -> Result<i32> {
        let rule_id = sqlx::query_scalar!(
            "INSERT INTO rules (rule_id, rule_name, description, rule_definition, status)
             VALUES ($1, $2, $3, $4, 'active') RETURNING id",
            rule.rule_id,
            rule.rule_name,
            rule.description,
            rule.rule_definition
        )
        .fetch_one(&self.pool)
        .await?;

        self.cleanup_list.push(format!("rules.{}", rule_id));
        Ok(rule_id)
    }

    /// Insert test derived attribute
    pub async fn insert_test_derived_attribute(&mut self, attr: &attributes::CreateDerivedAttributeRequest) -> Result<i32> {
        let attr_id = sqlx::query_scalar!(
            "INSERT INTO derived_attributes (name, data_type, description, rule_logic, status)
             VALUES ($1, $2, $3, $4, 'active') RETURNING id",
            attr.name,
            attr.data_type,
            attr.description,
            attr.rule_logic
        )
        .fetch_one(&self.pool)
        .await?;

        self.cleanup_list.push(format!("derived_attributes.{}", attr_id));
        Ok(attr_id)
    }

    /// Find rule by ID
    pub async fn find_rule_by_id(&self, rule_id: i32) -> Result<Option<rules::Rule>> {
        let rule = sqlx::query_as!(
            rules::Rule,
            "SELECT * FROM rules WHERE id = $1",
            rule_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(rule)
    }

    /// Find rule by rule_id string
    pub async fn find_rule_by_rule_id(&self, rule_id: &str) -> Result<Option<rules::Rule>> {
        let rule = sqlx::query_as!(
            rules::Rule,
            "SELECT * FROM rules WHERE rule_id = $1",
            rule_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(rule)
    }

    /// Count records in a table
    pub async fn count_records(&self, table: &str) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", table))
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    /// Execute custom SQL query
    pub async fn execute_query(&self, sql: &str) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            let mut result = HashMap::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value: serde_json::Value = match column.type_info().name() {
                    "INT4" => row.try_get::<Option<i32>, _>(i)?.map(|v| v.into()).unwrap_or(serde_json::Value::Null),
                    "INT8" => row.try_get::<Option<i64>, _>(i)?.map(|v| v.into()).unwrap_or(serde_json::Value::Null),
                    "TEXT" | "VARCHAR" => row.try_get::<Option<String>, _>(i)?.map(|v| v.into()).unwrap_or(serde_json::Value::Null),
                    "BOOL" => row.try_get::<Option<bool>, _>(i)?.map(|v| v.into()).unwrap_or(serde_json::Value::Null),
                    "TIMESTAMPTZ" => {
                        if let Ok(Some(ts)) = row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(i) {
                            serde_json::Value::String(ts.to_rfc3339())
                        } else {
                            serde_json::Value::Null
                        }
                    },
                    _ => serde_json::Value::Null,
                };
                result.insert(column.name().to_string(), value);
            }
            results.push(result);
        }

        Ok(results)
    }

    /// Simulate database failure for testing error handling
    pub async fn simulate_failure(&self, operation: &str, error: DatabaseError) {
        // This would typically involve a mock or test double
        // For now, we'll log the simulated failure
        tracing::warn!("Simulating database failure for operation '{}': {:?}", operation, error);
    }

    /// Get database connection info
    pub fn get_connection_info(&self) -> DatabaseConnectionInfo {
        DatabaseConnectionInfo {
            schema: self.test_schema.clone(),
            max_connections: 5,
            active_connections: 1, // Simplified for testing
        }
    }

    /// Reset all data in test schema
    pub async fn reset_data(&self) -> Result<()> {
        // Truncate all tables in dependency order
        let tables = vec![
            "rule_execution_logs",
            "derived_attribute_dependencies",
            "derived_attributes",
            "rules",
            "business_attributes",
            "attribute_categories"
        ];

        for table in tables {
            sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table))
                .execute(&self.pool)
                .await
                .with_context(|| format!("Failed to truncate table {}", table))?;
        }

        self.cleanup_list.clear();
        tracing::info!("Reset all data in test schema: {}", self.test_schema);
        Ok(())
    }

    /// Clean up test resources
    pub async fn cleanup(&self) -> Result<()> {
        // Drop the entire test schema
        sqlx::query(&format!("DROP SCHEMA IF EXISTS {} CASCADE", self.test_schema))
            .execute(&self.pool)
            .await
            .with_context(|| format!("Failed to drop test schema: {}", self.test_schema))?;

        tracing::info!("Cleaned up test database schema: {}", self.test_schema);
        Ok(())
    }

    /// Begin a transaction for testing
    pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>> {
        self.pool.begin().await.context("Failed to begin transaction")
    }

    /// Insert multiple test records efficiently
    pub async fn bulk_insert_rules(&mut self, rules: Vec<rules::CreateRuleRequest>) -> Result<Vec<i32>> {
        let mut rule_ids = Vec::new();

        for rule in rules {
            let rule_id = self.insert_test_rule(&rule).await?;
            rule_ids.push(rule_id);
        }

        Ok(rule_ids)
    }

    /// Verify database state for testing
    pub async fn verify_state(&self) -> Result<DatabaseStateVerification> {
        let rule_count = self.count_records("rules").await?;
        let attr_count = self.count_records("derived_attributes").await?;
        let business_attr_count = self.count_records("business_attributes").await?;

        Ok(DatabaseStateVerification {
            rule_count,
            derived_attribute_count: attr_count,
            business_attribute_count: business_attr_count,
            schema_name: self.test_schema.clone(),
        })
    }
}

/// Database error types for testing
#[derive(Debug)]
pub enum DatabaseError {
    ConnectionLost,
    QueryTimeout,
    ConstraintViolation,
    InsufficientPermissions,
}

/// Database connection information
#[derive(Debug)]
pub struct DatabaseConnectionInfo {
    pub schema: String,
    pub max_connections: u32,
    pub active_connections: u32,
}

/// Database state verification result
#[derive(Debug)]
pub struct DatabaseStateVerification {
    pub rule_count: i64,
    pub derived_attribute_count: i64,
    pub business_attribute_count: i64,
    pub schema_name: String,
}