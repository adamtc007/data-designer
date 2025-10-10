use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::db::{DbPool, DataDictionaryOperations, EmbeddingOperations, DbOperations};
use std::collections::HashMap;

// State management - now uses centralized DbPool
pub struct DataDictionaryState {
    pub db_pool: DbPool,
}

impl DataDictionaryState {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }
}

// Re-export types from centralized db module
pub use crate::db::{AttributeDefinition, CreateDerivedAttributeRequest, DataDictionaryResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupedAttributes {
    pub business: Vec<AttributeDefinition>,
    pub derived: Vec<AttributeDefinition>,
    pub system: Vec<AttributeDefinition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompiledRule {
    pub rule_id: i32,
    pub rust_code: String,
    pub wasm_bytes: Option<Vec<u8>>,
    pub compilation_status: String,
    pub compiler_version: Option<String>,
}

// === WRAPPER FUNCTIONS THAT DELEGATE TO CENTRALIZED OPERATIONS ===

/// Get all attributes from the data dictionary
pub async fn get_data_dictionary(
    pool: &DbPool,
    entity_filter: Option<String>,
) -> Result<DataDictionaryResponse, String> {
    // Use centralized operations
    DataDictionaryOperations::get_data_dictionary(pool, entity_filter.as_deref()).await
}

/// Refresh the materialized view
pub async fn refresh_data_dictionary(pool: &DbPool) -> Result<(), String> {
    // Use centralized operations
    DataDictionaryOperations::refresh_data_dictionary(pool).await
}

/// Create a new derived attribute
pub async fn create_derived_attribute(
    pool: &DbPool,
    request: CreateDerivedAttributeRequest,
) -> Result<i32, String> {
    // Use centralized operations
    DataDictionaryOperations::create_derived_attribute(pool, request).await
}

/// Search attributes by name or description
pub async fn search_attributes(
    pool: &DbPool,
    search_term: String,
) -> Result<Vec<AttributeDefinition>, String> {
    // Use centralized operations
    DataDictionaryOperations::search_attributes(pool, &search_term, Some(50)).await
}

/// Get rule dependencies for a specific rule
pub async fn get_rule_dependencies(
    pool: &DbPool,
    rule_id: i32,
) -> Result<Vec<AttributeDefinition>, String> {
    // Use centralized operations
    DataDictionaryOperations::get_rule_dependencies(pool, rule_id).await
}

/// Generate test context for rule evaluation
pub async fn generate_test_context(
    pool: &DbPool,
    attribute_names: Vec<String>,
) -> Result<HashMap<String, serde_json::Value>, String> {
    // Use centralized operations
    DataDictionaryOperations::generate_test_context(pool, attribute_names).await
}

/// Create and compile a rule
pub async fn create_and_compile_rule(
    pool: &DbPool,
    rule_name: String,
    dsl_code: String,
    target_attribute_id: i32,
    dependencies: Vec<String>,
) -> Result<CompiledRule, String> {
    // Start a transaction using centralized transaction management
    let mut tx = DbOperations::begin_transaction(pool).await?;

    // Create the rule using centralized query operations
    let rule_id_query = r#"
        INSERT INTO rules (
            rule_id, rule_name, rule_definition, target_attribute_id, status
        ) VALUES ($1, $2, $3, $4, 'active')
        RETURNING id
    "#;

    let rule_id = format!("RULE_{:06}", rand::random::<u32>() % 1000000);
    let rule_result: (i32,) = sqlx::query_as(rule_id_query)
        .bind(&rule_id)
        .bind(&rule_name)
        .bind(&dsl_code)
        .bind(target_attribute_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create rule: {}", e))?;

    let rule_id = rule_result.0;

    // Create rule dependencies using centralized operations
    for dep_path in &dependencies {
        let parts: Vec<&str> = dep_path.split('.').collect();
        if parts.len() == 2 {
            let entity = parts[0];
            let attribute = parts[1];

            // Find the business attribute ID
            let attr_query = r#"
                SELECT id FROM business_attributes
                WHERE entity_name = $1 AND attribute_name = $2
            "#;

            if let Ok(attr_result) = sqlx::query_as::<_, (i32,)>(attr_query)
                .bind(entity)
                .bind(attribute)
                .fetch_optional(&mut *tx)
                .await
            {
                if let Some((attr_id,)) = attr_result {
                    let dep_query = r#"
                        INSERT INTO rule_dependencies (rule_id, attribute_id, dependency_type)
                        VALUES ($1, $2, 'input')
                        ON CONFLICT (rule_id, attribute_id) DO NOTHING
                    "#;

                    sqlx::query(dep_query)
                        .bind(rule_id)
                        .bind(attr_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| format!("Failed to create dependency: {}", e))?;
                }
            }
        }
    }

    // Generate Rust code (simplified for now)
    let rust_code = generate_rust_code(&dsl_code, &dependencies);

    // Update rule with compiled code
    let update_query = r#"
        UPDATE rules SET
            compiled_rust_code = $1,
            compilation_status = 'success',
            compilation_timestamp = NOW(),
            compiler_version = $2
        WHERE id = $3
    "#;

    sqlx::query(update_query)
        .bind(&rust_code)
        .bind("1.0.0")
        .bind(rule_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to update compiled code: {}", e))?;

    // Add to compilation queue for WASM
    let queue_query = r#"
        INSERT INTO rule_compilation_queue (rule_id, compilation_type, priority)
        VALUES ($1, 'wasm', 5)
        ON CONFLICT (rule_id, compilation_type, status) DO NOTHING
    "#;

    sqlx::query(queue_query)
        .bind(rule_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to queue for WASM compilation: {}", e))?;

    // Commit transaction
    tx.commit()
        .await
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    // Refresh the materialized view
    refresh_data_dictionary(pool).await?;

    Ok(CompiledRule {
        rule_id,
        rust_code,
        wasm_bytes: None,
        compilation_status: "success".to_string(),
        compiler_version: Some("1.0.0".to_string()),
    })
}

// Simple DSL to Rust code generator (placeholder)
fn generate_rust_code(dsl_code: &str, dependencies: &[String]) -> String {
    // This is a simplified version - actual implementation would use the parser
    format!(
        r#"
use std::collections::HashMap;
use serde_json::Value;

/// Auto-generated rule from DSL
/// DSL: {}
/// Dependencies: {:?}
pub fn execute_rule(context: &HashMap<String, Value>) -> Result<Value, String> {{
    // TODO: Implement actual DSL to Rust transpilation
    // This is a placeholder implementation

    // Extract dependencies from context
    {}

    // Execute rule logic
    // {}

    Ok(Value::Null)
}}
"#,
        dsl_code,
        dependencies,
        dependencies
            .iter()
            .map(|d| format!("    let {} = context.get(\"{}\");", d.replace('.', "_"), d))
            .collect::<Vec<_>>()
            .join("\n"),
        dsl_code
    )
}