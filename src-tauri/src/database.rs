use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

// Database connection pool
pub type DbPool = Pool<Postgres>;

// DTOs for database entities
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
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_by: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BusinessAttribute {
    pub id: i32,
    pub entity_name: String,
    pub attribute_name: String,
    pub full_path: String,
    pub data_type: String,
    pub sql_type: Option<String>,
    pub rust_type: Option<String>,
    pub format_mask: Option<String>,
    pub validation_pattern: Option<String>,
    pub required: bool,
    pub editable: bool,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DerivedAttribute {
    pub id: i32,
    pub entity_name: String,
    pub attribute_name: String,
    pub full_path: String,
    pub data_type: String,
    pub sql_type: Option<String>,
    pub rust_type: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RuleCategory {
    pub id: i32,
    pub category_key: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}


// Database connection management (unused - keeping for potential future use)
#[allow(dead_code)]
pub async fn create_pool() -> Result<DbPool> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://adamtc007@localhost/data_designer".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

// Rule operations
pub async fn get_all_rules(pool: &DbPool) -> Result<Vec<Rule>> {
    let rules = sqlx::query_as::<_, Rule>(
        r#"
        SELECT * FROM rules
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rules)
}

pub async fn get_rule_by_id(pool: &DbPool, rule_id: &str) -> Result<Option<Rule>> {
    let rule = sqlx::query_as::<_, Rule>(
        r#"
        SELECT * FROM rules
        WHERE rule_id = $1
        "#,
    )
    .bind(rule_id)
    .fetch_optional(pool)
    .await?;

    Ok(rule)
}

pub async fn create_rule(pool: &DbPool, request: CreateRuleRequest) -> Result<Rule> {
    // Get category ID
    let category = sqlx::query_as::<_, RuleCategory>(
        "SELECT * FROM rule_categories WHERE category_key = $1",
    )
    .bind(&request.category_key)
    .fetch_optional(pool)
    .await?;

    let category_id = category.map(|c| c.id);

    // Get or create target attribute
    let target_parts: Vec<&str> = request.target_attribute.split('.').collect();
    if target_parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid target attribute format"));
    }

    let derived_attr = sqlx::query_as::<_, DerivedAttribute>(
        r#"
        INSERT INTO derived_attributes (entity_name, attribute_name, data_type, sql_type, rust_type, description)
        VALUES ($1, $2, 'Unknown', 'VARCHAR(255)', 'String', $3)
        ON CONFLICT (entity_name, attribute_name) DO UPDATE
        SET updated_at = CURRENT_TIMESTAMP
        RETURNING *
        "#,
    )
    .bind(target_parts[0])
    .bind(target_parts[1])
    .bind(&request.description)
    .fetch_one(pool)
    .await?;

    // Create the rule
    let rule = sqlx::query_as::<_, Rule>(
        r#"
        INSERT INTO rules (
            rule_id, rule_name, description, category_id,
            target_attribute_id, rule_definition, status, tags
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'active', $7)
        RETURNING *
        "#,
    )
    .bind(&request.rule_id)
    .bind(&request.rule_name)
    .bind(&request.description)
    .bind(category_id)
    .bind(derived_attr.id)
    .bind(&request.rule_definition)
    .bind(&request.tags)
    .fetch_one(pool)
    .await?;

    // Add source dependencies
    for source_attr in request.source_attributes {
        let attr_parts: Vec<&str> = source_attr.split('.').collect();
        if attr_parts.len() == 2 {
            if let Ok(Some(business_attr)) = get_business_attribute(pool, attr_parts[0], attr_parts[1]).await {
                sqlx::query(
                    r#"
                    INSERT INTO rule_dependencies (rule_id, attribute_id, dependency_type)
                    VALUES ($1, $2, 'input')
                    ON CONFLICT DO NOTHING
                    "#,
                )
                .bind(rule.id)
                .bind(business_attr.id)
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(rule)
}

pub async fn update_rule(pool: &DbPool, rule_id: &str, rule_definition: &str) -> Result<Rule> {
    // First increment version
    sqlx::query(
        r#"
        INSERT INTO rule_versions (rule_id, version, rule_definition, change_description)
        SELECT id, version, rule_definition, 'Previous version'
        FROM rules WHERE rule_id = $1
        "#,
    )
    .bind(rule_id)
    .execute(pool)
    .await?;

    // Update the rule
    let rule = sqlx::query_as::<_, Rule>(
        r#"
        UPDATE rules
        SET rule_definition = $1,
            version = version + 1,
            updated_at = CURRENT_TIMESTAMP
        WHERE rule_id = $2
        RETURNING *
        "#,
    )
    .bind(rule_definition)
    .bind(rule_id)
    .fetch_one(pool)
    .await?;

    Ok(rule)
}

// Attribute operations
pub async fn get_all_business_attributes(pool: &DbPool) -> Result<Vec<BusinessAttribute>> {
    let attributes = sqlx::query_as::<_, BusinessAttribute>(
        r#"
        SELECT * FROM business_attributes
        ORDER BY entity_name, attribute_name
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(attributes)
}

pub async fn get_business_attribute(
    pool: &DbPool,
    entity: &str,
    attribute: &str,
) -> Result<Option<BusinessAttribute>> {
    let attr = sqlx::query_as::<_, BusinessAttribute>(
        r#"
        SELECT * FROM business_attributes
        WHERE entity_name = $1 AND attribute_name = $2
        "#,
    )
    .bind(entity)
    .bind(attribute)
    .fetch_optional(pool)
    .await?;

    Ok(attr)
}

pub async fn get_all_derived_attributes(pool: &DbPool) -> Result<Vec<DerivedAttribute>> {
    let attributes = sqlx::query_as::<_, DerivedAttribute>(
        r#"
        SELECT * FROM derived_attributes
        ORDER BY entity_name, attribute_name
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(attributes)
}

// Rule execution logging (unused - keeping for potential future use)
#[allow(dead_code)]
pub async fn log_rule_execution(
    pool: &DbPool,
    rule_id: i32,
    input_data: serde_json::Value,
    output_value: Option<serde_json::Value>,
    duration_ms: i32,
    success: bool,
    error_message: Option<String>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO rule_executions (
            rule_id, input_data, output_value,
            execution_duration_ms, success, error_message
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(rule_id)
    .bind(input_data)
    .bind(output_value)
    .bind(duration_ms)
    .bind(success)
    .bind(error_message)
    .execute(pool)
    .await?;

    Ok(())
}

// Search operations
pub async fn search_rules(pool: &DbPool, query: &str) -> Result<Vec<Rule>> {
    let rules = sqlx::query_as::<_, Rule>(
        r#"
        SELECT * FROM rules
        WHERE search_vector @@ plainto_tsquery('english', $1)
        ORDER BY ts_rank(search_vector, plainto_tsquery('english', $1)) DESC
        LIMIT 20
        "#,
    )
    .bind(query)
    .fetch_all(pool)
    .await?;

    Ok(rules)
}

// Category operations
pub async fn get_all_categories(pool: &DbPool) -> Result<Vec<RuleCategory>> {
    let categories = sqlx::query_as::<_, RuleCategory>(
        r#"
        SELECT * FROM rule_categories
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(categories)
}