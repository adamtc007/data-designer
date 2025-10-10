use super::{DbPool, DbOperations};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use chrono::{DateTime, Utc};

// Rule-related DTOs
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Rule {
    pub id: i32,
    pub rule_id: String,
    pub rule_name: String,
    pub description: Option<String>,
    pub category_id: Option<i32>,
    pub target_attribute_id: Option<i32>,
    pub rule_definition: String,
    pub parsed_ast: Option<serde_json::Value>,
    pub status: String,
    pub version: i32,
    pub tags: Option<Vec<String>>,
    pub performance_metrics: Option<serde_json::Value>,
    pub embedding_data: Option<serde_json::Value>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRuleRequest {
    pub rule_id: String,
    pub rule_name: String,
    pub description: Option<String>,
    pub category_key: String,
    pub target_attribute: String,
    pub source_attributes: Vec<String>,
    pub rule_definition: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRuleWithTemplateRequest {
    pub rule_id: String,
    pub rule_name: String,
    pub description: String,
    pub target_attribute_name: String,
    pub source_attributes: Vec<String>,
    pub rule_definition: String,
}

// Rule database operations
pub struct RuleOperations;

impl RuleOperations {
    // Check if attribute exists
    pub async fn check_attribute_exists(
        pool: &DbPool,
        attribute_name: &str,
    ) -> Result<bool, String> {
        let query = "
            SELECT COUNT(*) as count
            FROM mv_data_dictionary
            WHERE attribute_name = $1 OR full_path = $1
        ";

        let count = DbOperations::query_count(pool, query, attribute_name).await?;
        Ok(count > 0)
    }

    // Get business attributes
    pub async fn get_business_attributes(
        pool: &DbPool,
    ) -> Result<Vec<serde_json::Value>, String> {
        let query = "
            SELECT attribute_name, full_path, data_type, description
            FROM mv_data_dictionary
            WHERE attribute_type = 'business'
            ORDER BY entity_name, attribute_name
        ";

        let rows = sqlx::query(query)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let mut attributes = Vec::new();
        for row in rows {
            let attr = serde_json::json!({
                "attribute_name": row.get::<&str, _>("attribute_name"),
                "full_path": row.get::<&str, _>("full_path"),
                "data_type": row.get::<&str, _>("data_type"),
                "description": row.get::<Option<&str>, _>("description")
            });
            attributes.push(attr);
        }

        Ok(attributes)
    }

    // Create rule with template
    pub async fn create_rule_with_template(
        pool: &DbPool,
        request: CreateRuleWithTemplateRequest,
    ) -> Result<(), String> {
        let mut tx = DbOperations::begin_transaction(pool).await?;

        // First, create or get the derived attribute
        let attr_query = "
            INSERT INTO derived_attributes (name, data_type, description, status)
            VALUES ($1, 'String', $2, 'draft')
            ON CONFLICT (name) DO UPDATE SET
                description = EXCLUDED.description,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id
        ";

        let attr_row: (i32,) = sqlx::query_as(attr_query)
            .bind(&request.target_attribute_name)
            .bind(&request.description)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("Failed to create derived attribute: {}", e))?;

        let target_attribute_id = attr_row.0;

        // Create the rule
        let rule_query = "
            INSERT INTO rules (
                rule_id, rule_name, description, target_attribute_id,
                rule_definition, status, created_by
            )
            VALUES ($1, $2, $3, $4, $5, 'draft', 'system')
        ";

        sqlx::query(rule_query)
            .bind(&request.rule_id)
            .bind(&request.rule_name)
            .bind(&request.description)
            .bind(target_attribute_id)
            .bind(&request.rule_definition)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to create rule: {}", e))?;

        // Get the rule internal ID for dependencies
        let rule_internal_id: (i32,) = sqlx::query_as("SELECT id FROM rules WHERE rule_id = $1")
            .bind(&request.rule_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("Failed to get rule ID: {}", e))?;

        // Create dependencies for each source attribute
        for source_attr in &request.source_attributes {
            // Try to find the source attribute in business attributes
            let source_query = "
                SELECT id FROM business_attributes
                WHERE attribute_name = $1 OR full_path = $1
                LIMIT 1
            ";

            if let Ok(source_row) = sqlx::query_as::<_, (String,)>(source_query)
                .bind(source_attr)
                .fetch_one(&mut *tx)
                .await
            {
                // Create dependency relationship
                let dependency_query = "
                    INSERT INTO rule_dependencies (rule_id, source_attribute_id, dependency_type)
                    VALUES ($1, $2, 'direct')
                ";

                sqlx::query(dependency_query)
                    .bind(rule_internal_id.0)
                    .bind(&source_row.0)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| format!("Failed to create dependency: {}", e))?;
            }
        }

        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(())
    }

    // Get existing rules
    pub async fn get_existing_rules(
        pool: &DbPool,
    ) -> Result<Vec<serde_json::Value>, String> {
        let query = "
            SELECT rule_id, rule_name, description, status, created_at
            FROM rules
            WHERE status != 'deprecated'
            ORDER BY created_at DESC
        ";

        let rows = sqlx::query(query)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let mut rules = Vec::new();
        for row in rows {
            let rule = serde_json::json!({
                "rule_id": row.get::<&str, _>("rule_id"),
                "rule_name": row.get::<&str, _>("rule_name"),
                "description": row.get::<Option<&str>, _>("description"),
                "status": row.get::<&str, _>("status"),
                "created_at": row.get::<chrono::NaiveDateTime, _>("created_at").to_string()
            });
            rules.push(rule);
        }

        Ok(rules)
    }

    // Get rule by ID
    pub async fn get_rule_by_id(
        pool: &DbPool,
        rule_id: &str,
    ) -> Result<serde_json::Value, String> {
        let query = "
            SELECT rule_id, rule_name, description, rule_definition, status
            FROM rules
            WHERE rule_id = $1
        ";

        let row = sqlx::query(query)
            .bind(rule_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Rule not found: {}", e))?;

        let rule = serde_json::json!({
            "rule_id": row.get::<&str, _>("rule_id"),
            "rule_name": row.get::<&str, _>("rule_name"),
            "description": row.get::<Option<&str>, _>("description"),
            "rule_definition": row.get::<&str, _>("rule_definition"),
            "status": row.get::<&str, _>("status")
        });

        Ok(rule)
    }

    // Log rule execution (future use)
    pub async fn log_rule_execution(
        pool: &DbPool,
        rule_id: &str,
        execution_time_ms: i64,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<(), String> {
        let query = "
            INSERT INTO rule_execution_log (rule_id, execution_time_ms, success, error_message)
            VALUES ($1, $2, $3, $4)
        ";

        sqlx::query(query)
            .bind(rule_id)
            .bind(execution_time_ms)
            .bind(success)
            .bind(error_message)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to log rule execution: {}", e))?;

        Ok(())
    }
}