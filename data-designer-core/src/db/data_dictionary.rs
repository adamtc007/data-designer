use super::{DbPool, DbOperations};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AttributeDefinition {
    pub attribute_type: String,
    pub entity_name: String,
    pub attribute_name: String,
    pub full_path: String,
    pub data_type: String,
    pub sql_type: Option<String>,
    pub rust_type: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDerivedAttributeRequest {
    pub name: String,
    pub data_type: String,
    pub description: Option<String>,
    pub rule_logic: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataDictionaryResponse {
    pub attributes: Vec<serde_json::Value>,
    pub total_count: i64,
    pub business_count: i64,
    pub derived_count: i64,
    pub system_count: i64,
}

pub struct DataDictionaryOperations;

impl DataDictionaryOperations {
    /// Get comprehensive data dictionary with all attributes
    pub async fn get_data_dictionary(
        pool: &DbPool,
        search_term: Option<&str>,
    ) -> Result<DataDictionaryResponse, String> {
        let base_query = r#"
            SELECT attribute_name, full_path, data_type, description,
                   attribute_type, entity_name
            FROM mv_data_dictionary
        "#;

        let rows = match search_term {
            Some(term) => {
                let search_query = format!("{} WHERE attribute_name ILIKE $1 OR description ILIKE $1 ORDER BY attribute_name", base_query);
                let search_pattern = format!("%{}%", term);
                DbOperations::query_raw_all_one_param(pool, &search_query, &search_pattern).await?
            }
            None => {
                let query = format!("{} ORDER BY entity_name, attribute_name", base_query);
                DbOperations::query_raw_all_no_params(pool, &query).await?
            }
        };

        let mut attributes = Vec::new();
        let mut business_count = 0i64;
        let mut derived_count = 0i64;
        let mut system_count = 0i64;

        for row in rows {
            let attribute_type = row.try_get::<&str, _>("attribute_type")
                .map_err(|e| format!("Failed to get attribute_type: {}", e))?;

            match attribute_type {
                "business" => business_count += 1,
                "derived" => derived_count += 1,
                "system" => system_count += 1,
                _ => {}
            }

            let attr = serde_json::json!({
                "attribute_name": row.try_get::<&str, _>("attribute_name")
                    .map_err(|e| format!("Failed to get attribute_name: {}", e))?,
                "full_path": row.try_get::<&str, _>("full_path")
                    .map_err(|e| format!("Failed to get full_path: {}", e))?,
                "data_type": row.try_get::<&str, _>("data_type")
                    .map_err(|e| format!("Failed to get data_type: {}", e))?,
                "description": row.try_get::<Option<&str>, _>("description")
                    .map_err(|e| format!("Failed to get description: {}", e))?,
                "attribute_type": attribute_type,
                "entity_name": row.try_get::<&str, _>("entity_name")
                    .map_err(|e| format!("Failed to get entity_name: {}", e))?,
                "is_key": false,
                "is_nullable": true
            });
            attributes.push(attr);
        }

        let total_count = attributes.len() as i64;

        Ok(DataDictionaryResponse {
            attributes,
            total_count,
            business_count,
            derived_count,
            system_count,
        })
    }

    /// Create a new derived attribute
    pub async fn create_derived_attribute(
        pool: &DbPool,
        request: CreateDerivedAttributeRequest,
    ) -> Result<i32, String> {
        let query = r#"
            INSERT INTO derived_attributes (name, data_type, description, rule_logic, tags, status, created_by)
            VALUES ($1, $2, $3, $4, $5, 'draft', 'system')
            RETURNING id
        "#;

        let tags_json = request.tags.map(|t| serde_json::to_value(t))
            .transpose()
            .map_err(|e| format!("Failed to serialize tags: {}", e))?;

        let row: (i32,) = sqlx::query_as(query)
            .bind(&request.name)
            .bind(&request.data_type)
            .bind(&request.description)
            .bind(&request.rule_logic)
            .bind(tags_json)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to create derived attribute: {}", e))?;

        Ok(row.0)
    }

    /// Refresh the materialized view for data dictionary
    pub async fn refresh_data_dictionary(pool: &DbPool) -> Result<(), String> {
        let query = "REFRESH MATERIALIZED VIEW mv_data_dictionary";
        DbOperations::execute(pool, query).await?;
        Ok(())
    }

    /// Search attributes with advanced filtering
    pub async fn search_attributes(
        pool: &DbPool,
        query_text: &str,
        limit: Option<i32>,
    ) -> Result<Vec<AttributeDefinition>, String> {
        let search_query = r#"
            SELECT
                'business' as attribute_type,
                entity_name,
                attribute_name,
                entity_name || '.' || attribute_name as full_path,
                data_type,
                sql_type,
                rust_type,
                description
            FROM business_attributes
            WHERE attribute_name ILIKE $1
               OR full_path ILIKE $1
               OR description ILIKE $1
            ORDER BY
                CASE WHEN attribute_name ILIKE $1 THEN 1 ELSE 2 END,
                attribute_name
            LIMIT $2
        "#;

        let search_pattern = format!("%{}%", query_text);
        let limit_value = limit.unwrap_or(50);

        DbOperations::query_all_with_two_params(pool, search_query, search_pattern, limit_value).await
    }

    /// Get rule dependencies for a specific rule
    pub async fn get_rule_dependencies(
        pool: &DbPool,
        rule_id: i32,
    ) -> Result<Vec<AttributeDefinition>, String> {
        let query = r#"
            SELECT
                'business' as attribute_type,
                ba.entity_name,
                ba.attribute_name,
                ba.entity_name || '.' || ba.attribute_name as full_path,
                ba.data_type,
                ba.sql_type,
                ba.rust_type,
                ba.description
            FROM rule_dependencies rd
            JOIN business_attributes ba ON rd.source_attribute_id = ba.id::text
            WHERE rd.rule_id = $1
            ORDER BY ba.attribute_name
        "#;

        DbOperations::query_all_with_param(pool, query, &rule_id.to_string()).await
    }

    /// Generate test context for rule evaluation
    pub async fn generate_test_context(
        pool: &DbPool,
        attribute_names: Vec<String>,
    ) -> Result<HashMap<String, serde_json::Value>, String> {
        let mut context = HashMap::new();

        for attr_name in attribute_names {
            // Get attribute metadata
            let query = r#"
                SELECT data_type, description
                FROM mv_data_dictionary
                WHERE attribute_name = $1
                LIMIT 1
            "#;

            if let Ok(rows) = DbOperations::query_raw_all_one_param(pool, query, &attr_name).await {
                if let Some(row) = rows.first() {
                    let data_type = row.try_get::<&str, _>("data_type").unwrap_or("string");

                    // Generate sample data based on type and name
                    let sample_value = match data_type.to_lowercase().as_str() {
                        "integer" | "int" | "bigint" => {
                            if attr_name.contains("age") {
                                serde_json::Value::Number(35.into())
                            } else if attr_name.contains("amount") || attr_name.contains("balance") {
                                serde_json::Value::Number(50000.into())
                            } else {
                                serde_json::Value::Number(1.into())
                            }
                        },
                        "decimal" | "numeric" | "real" | "double" => {
                            serde_json::Value::Number(serde_json::Number::from_f64(100.50).unwrap())
                        },
                        "boolean" | "bool" => {
                            serde_json::Value::Bool(true)
                        },
                        _ => {
                            if attr_name.ends_with("_id") {
                                serde_json::Value::String("CUST_12345".to_string())
                            } else if attr_name.contains("country") {
                                serde_json::Value::String("USA".to_string())
                            } else if attr_name.contains("name") {
                                serde_json::Value::String("Sample Name".to_string())
                            } else {
                                serde_json::Value::String("sample_value".to_string())
                            }
                        }
                    };

                    context.insert(attr_name, sample_value);
                }
            }
        }

        Ok(context)
    }
}