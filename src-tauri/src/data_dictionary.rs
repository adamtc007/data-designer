use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use std::collections::HashMap;

// State management for Leptos SSR
pub struct DataDictionaryState {
    pub db_pool: PgPool,
}

impl DataDictionaryState {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AttributeDefinition {
    pub attribute_type: String,
    pub entity_name: String,
    pub attribute_name: String,
    pub full_path: String,
    pub data_type: String,
    pub sql_type: String,
    pub rust_type: String,
    pub description: Option<String>,
    pub required: bool,
    pub validation_pattern: Option<String>,
    pub rule_definition: Option<String>,
    pub rule_id: Option<i32>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupedAttributes {
    pub business: Vec<AttributeDefinition>,
    pub derived: Vec<AttributeDefinition>,
    pub system: Vec<AttributeDefinition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataDictionaryResponse {
    pub entities: HashMap<String, GroupedAttributes>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDerivedAttributeRequest {
    pub entity_name: String,
    pub attribute_name: String,
    pub data_type: String,
    pub description: Option<String>,
    pub dependencies: Vec<String>, // List of attribute full paths
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompiledRule {
    pub rule_id: i32,
    pub rust_code: String,
    pub wasm_bytes: Option<Vec<u8>>,
    pub compilation_status: String,
    pub compiler_version: Option<String>,
}

// Get all attributes from the data dictionary
pub async fn get_data_dictionary(
    pool: &PgPool,
    entity_filter: Option<String>,
) -> Result<DataDictionaryResponse, String> {
    let query = if let Some(entity) = entity_filter {
        sqlx::query_as::<_, AttributeDefinition>(
            "SELECT * FROM mv_data_dictionary WHERE entity_name = $1 ORDER BY entity_name, attribute_type, attribute_name"
        )
        .bind(entity)
    } else {
        sqlx::query_as::<_, AttributeDefinition>(
            "SELECT * FROM mv_data_dictionary ORDER BY entity_name, attribute_type, attribute_name"
        )
    };

    let attributes = query
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch data dictionary: {}", e))?;

    // Group attributes by entity and type
    let mut entities: HashMap<String, GroupedAttributes> = HashMap::new();
    let total_count = attributes.len();

    for attr in attributes {
        let entity = entities.entry(attr.entity_name.clone()).or_insert_with(|| {
            GroupedAttributes {
                business: Vec::new(),
                derived: Vec::new(),
                system: Vec::new(),
            }
        });

        match attr.attribute_type.as_str() {
            "business" => entity.business.push(attr),
            "derived" => entity.derived.push(attr),
            "system" => entity.system.push(attr),
            _ => {}
        }
    }

    Ok(DataDictionaryResponse {
        entities,
        total_count,
    })
}

// Refresh the materialized view
pub async fn refresh_data_dictionary(pool: &PgPool) -> Result<(), String> {
    sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY mv_data_dictionary")
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to refresh data dictionary: {}", e))?;

    Ok(())
}

// Create a new derived attribute
pub async fn create_derived_attribute(
    pool: &PgPool,
    request: CreateDerivedAttributeRequest,
) -> Result<i32, String> {
    // Determine SQL and Rust types based on data_type
    let (sql_type, rust_type) = match request.data_type.as_str() {
        "String" => ("VARCHAR(255)", "String"),
        "Number" => ("NUMERIC(15,3)", "f64"),
        "Integer" => ("INTEGER", "i32"),
        "Boolean" => ("BOOLEAN", "bool"),
        "Date" => ("DATE", "NaiveDate"),
        "DateTime" => ("TIMESTAMP", "DateTime<Utc>"),
        "Json" => ("JSONB", "serde_json::Value"),
        _ => ("VARCHAR(255)", "String"),
    };

    let result = sqlx::query!(
        r#"
        INSERT INTO derived_attributes (
            entity_name, attribute_name, data_type, sql_type, rust_type, description
        ) VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        request.entity_name,
        request.attribute_name,
        request.data_type,
        sql_type,
        rust_type,
        request.description
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to create derived attribute: {}", e))?;

    // Refresh the materialized view
    refresh_data_dictionary(pool).await?;

    Ok(result.id)
}

// Create and compile a rule
pub async fn create_and_compile_rule(
    pool: &PgPool,
    rule_name: String,
    dsl_code: String,
    target_attribute_id: i32,
    dependencies: Vec<String>,
) -> Result<CompiledRule, String> {
    // Start a transaction
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // Create the rule
    let rule = sqlx::query!(
        r#"
        INSERT INTO rules (
            rule_id, rule_name, rule_definition, target_attribute_id, status
        ) VALUES ($1, $2, $3, $4, 'active')
        RETURNING id
        "#,
        format!("RULE_{:06}", rand::random::<u32>() % 1000000),
        rule_name,
        dsl_code.clone(),
        target_attribute_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("Failed to create rule: {}", e))?;

    // Create rule dependencies
    for dep_path in &dependencies {
        // Parse the dependency path (e.g., "Client.email")
        let parts: Vec<&str> = dep_path.split('.').collect();
        if parts.len() == 2 {
            let entity = parts[0];
            let attribute = parts[1];

            // Find the business attribute ID
            let attr = sqlx::query!(
                r#"
                SELECT id FROM business_attributes
                WHERE entity_name = $1 AND attribute_name = $2
                "#,
                entity,
                attribute
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| format!("Failed to find attribute {}: {}", dep_path, e))?;

            if let Some(attr) = attr {
                sqlx::query!(
                    r#"
                    INSERT INTO rule_dependencies (rule_id, attribute_id, dependency_type)
                    VALUES ($1, $2, 'input')
                    ON CONFLICT (rule_id, attribute_id) DO NOTHING
                    "#,
                    rule.id,
                    attr.id
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Failed to create dependency: {}", e))?;
            }
        }
    }

    // Generate Rust code (simplified for now)
    let rust_code = generate_rust_code(&dsl_code, &dependencies);

    // Update rule with compiled code
    sqlx::query!(
        r#"
        UPDATE rules SET
            compiled_rust_code = $1,
            compilation_status = 'success',
            compilation_timestamp = NOW(),
            compiler_version = $2
        WHERE id = $3
        "#,
        rust_code.clone(),
        "1.0.0",
        rule.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to update compiled code: {}", e))?;

    // Add to compilation queue for WASM
    sqlx::query!(
        r#"
        INSERT INTO rule_compilation_queue (rule_id, compilation_type, priority)
        VALUES ($1, 'wasm', 5)
        ON CONFLICT (rule_id, compilation_type, status) DO NOTHING
        "#,
        rule.id
    )
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
        rule_id: rule.id,
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

// Get attribute dependencies for a rule (unused - keeping for potential future use)
#[allow(dead_code)]
pub async fn get_rule_dependencies(
    pool: &PgPool,
    rule_id: i32,
) -> Result<Vec<AttributeDefinition>, String> {
    let deps = sqlx::query_as::<_, AttributeDefinition>(
        r#"
        SELECT
            'business' as attribute_type,
            ba.entity_name,
            ba.attribute_name,
            ba.entity_name || '.' || ba.attribute_name as full_path,
            ba.data_type,
            ba.sql_type,
            ba.rust_type,
            ba.description,
            ba.required,
            ba.validation_pattern,
            NULL::TEXT as rule_definition,
            NULL::INTEGER as rule_id,
            'active' as status
        FROM rule_dependencies rd
        JOIN business_attributes ba ON rd.attribute_id = ba.id
        WHERE rd.rule_id = $1
        ORDER BY ba.entity_name, ba.attribute_name
        "#
    )
    .bind(rule_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch rule dependencies: {}", e))?;

    Ok(deps)
}

// Search attributes by name or description
pub async fn search_attributes(
    pool: &PgPool,
    search_term: String,
) -> Result<Vec<AttributeDefinition>, String> {
    let search_pattern = format!("%{}%", search_term.to_lowercase());

    let attributes = sqlx::query_as::<_, AttributeDefinition>(
        r#"
        SELECT * FROM mv_data_dictionary
        WHERE LOWER(attribute_name) LIKE $1
           OR LOWER(full_path) LIKE $1
           OR LOWER(COALESCE(description, '')) LIKE $1
        ORDER BY entity_name, attribute_type, attribute_name
        LIMIT 50
        "#
    )
    .bind(search_pattern)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to search attributes: {}", e))?;

    Ok(attributes)
}