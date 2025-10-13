use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceDictionary {
    pub id: i32,
    pub dictionary_name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub creation_date: chrono::NaiveDateTime,
    pub last_modified: chrono::NaiveDateTime,
    pub status: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceObject {
    pub id: i32,
    pub dictionary_id: i32,
    pub resource_name: String,
    pub description: Option<String>,
    pub version: String,
    pub category: Option<String>,
    pub owner_team: Option<String>,
    pub status: String,
    pub ui_layout: String,
    pub group_order: Vec<String>,
    pub navigation_config: serde_json::Value,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AttributeObject {
    pub id: i32,
    pub resource_id: i32,
    pub attribute_name: String,
    pub data_type: String,
    pub description: Option<String>,
    pub is_required: bool,
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    pub min_value: Option<rust_decimal::Decimal>,
    pub max_value: Option<rust_decimal::Decimal>,
    pub allowed_values: Option<serde_json::Value>,
    pub validation_pattern: Option<String>,
    pub persistence_system: Option<String>,
    pub persistence_entity: Option<String>,
    pub persistence_identifier: Option<String>,
    pub ui_group: Option<String>,
    pub ui_display_order: i32,
    pub ui_render_hint: Option<String>,
    pub ui_label: Option<String>,
    pub ui_help_text: Option<String>,
    pub wizard_step: Option<i32>,
    pub wizard_step_title: Option<String>,
    pub wizard_next_button: Option<String>,
    pub wizard_previous_button: Option<String>,
    pub wizard_description: Option<String>,
    pub generation_examples: serde_json::Value,
    pub rules_dsl: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AttributePerspective {
    pub id: i32,
    pub attribute_id: i32,
    pub perspective_name: String,
    pub description: Option<String>,
    pub ui_group: Option<String>,
    pub ui_label: Option<String>,
    pub ui_help_text: Option<String>,
    pub generation_examples: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullResourceConfiguration {
    pub resource: ResourceObject,
    pub attributes: Vec<AttributeWithPerspectives>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeWithPerspectives {
    pub attribute: AttributeObject,
    pub perspectives: Vec<AttributePerspective>,
}

pub struct ConfigDrivenOperations;

impl ConfigDrivenOperations {
    /// Get all resource dictionaries
    pub async fn get_dictionaries(pool: &DbPool) -> Result<Vec<ResourceDictionary>, String> {
        let dictionaries = sqlx::query_as::<_, ResourceDictionary>(
            "SELECT * FROM resource_dictionaries ORDER BY dictionary_name"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch dictionaries: {}", e))?;

        Ok(dictionaries)
    }

    /// Get all resources in a dictionary
    pub async fn get_resources_by_dictionary(
        pool: &DbPool,
        dictionary_id: i32
    ) -> Result<Vec<ResourceObject>, String> {
        let resources = sqlx::query_as::<_, ResourceObject>(
            "SELECT * FROM resource_objects WHERE dictionary_id = $1 ORDER BY resource_name"
        )
        .bind(dictionary_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch resources: {}", e))?;

        Ok(resources)
    }

    /// Get a specific resource by name
    pub async fn get_resource_by_name(
        pool: &DbPool,
        resource_name: &str
    ) -> Result<Option<ResourceObject>, String> {
        let resource = sqlx::query_as::<_, ResourceObject>(
            "SELECT * FROM resource_objects WHERE resource_name = $1"
        )
        .bind(resource_name)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to fetch resource: {}", e))?;

        Ok(resource)
    }

    /// Get full resource configuration with attributes and perspectives
    pub async fn get_full_resource_config(
        pool: &DbPool,
        resource_name: &str
    ) -> Result<Option<FullResourceConfiguration>, String> {
        // Get the resource
        let resource = match Self::get_resource_by_name(pool, resource_name).await? {
            Some(r) => r,
            None => return Ok(None),
        };

        // Get attributes for this resource
        let attributes = sqlx::query_as::<_, AttributeObject>(
            r#"SELECT * FROM attribute_objects
               WHERE resource_id = $1
               ORDER BY ui_display_order, attribute_name"#
        )
        .bind(resource.id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch attributes: {}", e))?;

        // Get perspectives for each attribute
        let mut attributes_with_perspectives = Vec::new();
        for attribute in attributes {
            let perspectives = sqlx::query_as::<_, AttributePerspective>(
                "SELECT * FROM attribute_perspectives WHERE attribute_id = $1 ORDER BY perspective_name"
            )
            .bind(attribute.id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to fetch perspectives: {}", e))?;

            attributes_with_perspectives.push(AttributeWithPerspectives {
                attribute,
                perspectives,
            });
        }

        Ok(Some(FullResourceConfiguration {
            resource,
            attributes: attributes_with_perspectives,
        }))
    }

    /// Get available perspectives for a resource
    pub async fn get_resource_perspectives(
        pool: &DbPool,
        resource_name: &str
    ) -> Result<Vec<String>, String> {
        let perspectives = sqlx::query_scalar::<_, String>(
            r#"SELECT DISTINCT ap.perspective_name
               FROM attribute_perspectives ap
               JOIN attribute_objects ao ON ap.attribute_id = ao.id
               JOIN resource_objects ro ON ao.resource_id = ro.id
               WHERE ro.resource_name = $1
               ORDER BY ap.perspective_name"#
        )
        .bind(resource_name)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch perspectives: {}", e))?;

        Ok(perspectives)
    }

    /// Search resources by name or category
    pub async fn search_resources(
        pool: &DbPool,
        search_term: &str
    ) -> Result<Vec<ResourceObject>, String> {
        let resources = sqlx::query_as::<_, ResourceObject>(
            r#"SELECT * FROM resource_objects
               WHERE resource_name ILIKE $1
                  OR category ILIKE $1
                  OR description ILIKE $1
               ORDER BY resource_name"#
        )
        .bind(format!("%{}%", search_term))
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to search resources: {}", e))?;

        Ok(resources)
    }

    /// Get attributes by group for a specific resource
    pub async fn get_attributes_by_group(
        pool: &DbPool,
        resource_name: &str,
        group_name: &str
    ) -> Result<Vec<AttributeObject>, String> {
        let attributes = sqlx::query_as::<_, AttributeObject>(
            r#"SELECT ao.* FROM attribute_objects ao
               JOIN resource_objects ro ON ao.resource_id = ro.id
               WHERE ro.resource_name = $1 AND ao.ui_group = $2
               ORDER BY ao.ui_display_order, ao.attribute_name"#
        )
        .bind(resource_name)
        .bind(group_name)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch attributes by group: {}", e))?;

        Ok(attributes)
    }

    /// Convert database objects to frontend JSON format
    pub fn convert_to_frontend_format(config: &FullResourceConfiguration) -> serde_json::Value {
        let mut resource_json = serde_json::json!({
            "resourceName": config.resource.resource_name,
            "description": config.resource.description,
            "version": config.resource.version,
            "ui": {
                "layout": config.resource.ui_layout,
                "groupOrder": config.resource.group_order,
                "navigation": config.resource.navigation_config
            },
            "category": config.resource.category,
            "owner": config.resource.owner_team,
            "status": config.resource.status,
            "attributes": []
        });

        let mut attributes_array = Vec::new();
        for attr_with_persp in &config.attributes {
            let attr = &attr_with_persp.attribute;

            let mut constraints = serde_json::json!({
                "required": attr.is_required
            });

            if attr.min_length.is_some() {
                constraints["minLength"] = serde_json::Value::from(attr.min_length);
            }
            if attr.max_length.is_some() {
                constraints["maxLength"] = serde_json::Value::from(attr.max_length);
            }
            if attr.min_value.is_some() {
                constraints["min"] = serde_json::Value::from(attr.min_value.unwrap().to_string());
            }
            if attr.max_value.is_some() {
                constraints["max"] = serde_json::Value::from(attr.max_value.unwrap().to_string());
            }
            if let Some(allowed_values) = &attr.allowed_values {
                constraints["allowedValues"] = allowed_values.clone();
            }

            let mut attribute_json = serde_json::json!({
                "name": attr.attribute_name,
                "dataType": attr.data_type,
                "description": attr.description,
                "constraints": constraints,
                "persistence_locator": {
                    "system": attr.persistence_system,
                    "entity": attr.persistence_entity,
                    "identifier": attr.persistence_identifier
                },
                "ui": {
                    "group": attr.ui_group,
                    "displayOrder": attr.ui_display_order,
                    "renderHint": attr.ui_render_hint,
                    "label": attr.ui_label,
                    "helpText": attr.ui_help_text
                },
                "generationExamples": attr.generation_examples
            });

            // Add wizard configuration if present
            if attr.wizard_step.is_some() {
                attribute_json["ui"]["wizard"] = serde_json::json!({
                    "step": attr.wizard_step,
                    "title": attr.wizard_step_title,
                    "nextButton": attr.wizard_next_button,
                    "previousButton": attr.wizard_previous_button,
                    "description": attr.wizard_description
                });
            }

            // Add rules DSL if present
            if let Some(rules_dsl) = &attr.rules_dsl {
                attribute_json["rules_dsl"] = serde_json::Value::String(rules_dsl.clone());
            }

            // Add perspectives
            if !attr_with_persp.perspectives.is_empty() {
                let mut perspectives_json = serde_json::json!({});
                for perspective in &attr_with_persp.perspectives {
                    perspectives_json[&perspective.perspective_name] = serde_json::json!({
                        "description": perspective.description,
                        "ui": {
                            "group": perspective.ui_group,
                            "label": perspective.ui_label,
                            "helpText": perspective.ui_help_text
                        },
                        "generationExamples": perspective.generation_examples
                    });
                }
                attribute_json["perspectives"] = perspectives_json;
            }

            attributes_array.push(attribute_json);
        }

        resource_json["attributes"] = serde_json::Value::Array(attributes_array);
        resource_json
    }
}