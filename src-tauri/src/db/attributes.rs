use super::{DbPool, DbOperations};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use chrono::{DateTime, Utc};

// Attribute-related DTOs
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BusinessAttribute {
    pub id: i32,
    pub entity_name: String,
    pub attribute_name: String,
    pub full_path: String,
    pub data_type: String,
    pub description: Option<String>,
    pub is_key: bool,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub validation_rules: Option<serde_json::Value>,
    pub business_glossary_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DerivedAttribute {
    pub id: i32,
    pub name: String,
    pub data_type: String,
    pub description: Option<String>,
    pub rule_logic: Option<String>,
    pub status: String,
    pub tags: Option<Vec<String>>,
    pub performance_metrics: Option<serde_json::Value>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

// Import from data_dictionary module to avoid duplication
use super::data_dictionary::{CreateDerivedAttributeRequest, DataDictionaryResponse};

// Attribute database operations
pub struct AttributeOperations;

impl AttributeOperations {
    // Get data dictionary
    pub async fn get_data_dictionary(
        pool: &DbPool,
        search_term: Option<&str>,
    ) -> Result<DataDictionaryResponse, String> {
        let base_query = "
            SELECT attribute_name, full_path, data_type, description,
                   attribute_type, entity_name, is_key, is_nullable
            FROM mv_data_dictionary
        ";

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
            let attribute_type = row.get::<&str, _>("attribute_type");
            match attribute_type {
                "business" => business_count += 1,
                "derived" => derived_count += 1,
                "system" => system_count += 1,
                _ => {}
            }

            let attr = serde_json::json!({
                "attribute_name": row.get::<&str, _>("attribute_name"),
                "full_path": row.get::<&str, _>("full_path"),
                "data_type": row.get::<&str, _>("data_type"),
                "description": row.get::<Option<&str>, _>("description"),
                "attribute_type": attribute_type,
                "entity_name": row.get::<&str, _>("entity_name"),
                "is_key": row.get::<bool, _>("is_key"),
                "is_nullable": row.get::<bool, _>("is_nullable")
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

    // Create derived attribute
    pub async fn create_derived_attribute(
        pool: &DbPool,
        request: CreateDerivedAttributeRequest,
    ) -> Result<i32, String> {
        let query = "
            INSERT INTO derived_attributes (name, data_type, description, rule_logic, tags, status, created_by)
            VALUES ($1, $2, $3, $4, $5, 'draft', 'system')
            RETURNING id
        ";

        let row: (i32,) = sqlx::query_as(query)
            .bind(&request.name)
            .bind(&request.data_type)
            .bind(&request.description)
            .bind(&request.rule_logic)
            .bind(&request.tags)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to create derived attribute: {}", e))?;

        Ok(row.0)
    }

    // Search attributes
    pub async fn search_attributes(
        pool: &DbPool,
        query_text: &str,
        limit: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, String> {
        let search_query = "
            SELECT attribute_name, full_path, data_type, description,
                   attribute_type, entity_name
            FROM mv_data_dictionary
            WHERE attribute_name ILIKE $1
               OR full_path ILIKE $1
               OR description ILIKE $1
            ORDER BY
                CASE WHEN attribute_name ILIKE $1 THEN 1 ELSE 2 END,
                attribute_name
            LIMIT $2
        ";

        let search_pattern = format!("%{}%", query_text);
        let limit_value = limit.unwrap_or(50);

        let rows = DbOperations::query_raw_all_two_params(
            pool,
            search_query,
            &search_pattern,
            &limit_value
        ).await?;

        let mut attributes = Vec::new();
        for row in rows {
            let attr = serde_json::json!({
                "attribute_name": row.get::<&str, _>("attribute_name"),
                "full_path": row.get::<&str, _>("full_path"),
                "data_type": row.get::<&str, _>("data_type"),
                "description": row.get::<Option<&str>, _>("description"),
                "attribute_type": row.get::<&str, _>("attribute_type"),
                "entity_name": row.get::<&str, _>("entity_name")
            });
            attributes.push(attr);
        }

        Ok(attributes)
    }

    // Get rule dependencies
    pub async fn get_rule_dependencies(
        pool: &DbPool,
        rule_id: i32,
    ) -> Result<Vec<String>, String> {
        let query = "
            SELECT ba.attribute_name
            FROM rule_dependencies rd
            JOIN business_attributes ba ON rd.source_attribute_id = ba.id::text
            WHERE rd.rule_id = $1
            ORDER BY ba.attribute_name
        ";

        let rows = DbOperations::query_raw_all_one_i32_param(pool, query, rule_id).await?;

        let dependencies = rows
            .iter()
            .map(|row| row.get::<&str, _>("attribute_name").to_string())
            .collect();

        Ok(dependencies)
    }

    // Update attribute
    pub async fn update_attribute(
        pool: &DbPool,
        attribute_id: i32,
        description: Option<&str>,
        data_type: Option<&str>,
    ) -> Result<(), String> {
        let query = "
            UPDATE derived_attributes
            SET description = COALESCE($2, description),
                data_type = COALESCE($3, data_type),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
        ";

        sqlx::query(query)
            .bind(attribute_id)
            .bind(description)
            .bind(data_type)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update attribute: {}", e))?;

        Ok(())
    }
}