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
    // Original fields
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

    // Enhanced AI RAG and UI metadata fields
    pub semantic_tags: Option<serde_json::Value>,
    pub ai_context: Option<serde_json::Value>,
    pub search_keywords: Option<Vec<String>>,
    pub ui_component_type: Option<String>,
    pub ui_layout_config: Option<serde_json::Value>,
    pub ui_styling: Option<serde_json::Value>,
    pub ui_behavior: Option<serde_json::Value>,
    pub conditional_logic: Option<serde_json::Value>,
    pub relationship_metadata: Option<serde_json::Value>,
    pub ai_prompt_templates: Option<serde_json::Value>,
    pub form_generation_rules: Option<serde_json::Value>,
    pub accessibility_config: Option<serde_json::Value>,
    pub responsive_config: Option<serde_json::Value>,
    pub data_flow_config: Option<serde_json::Value>,

    // Comprehensive AI LLM Integration Fields
    pub extended_description: Option<String>,
    pub business_context: Option<String>,
    pub technical_context: Option<String>,
    pub user_guidance: Option<String>,
    pub ai_summary: Option<String>,
    pub domain_terminology: Option<serde_json::Value>,
    pub contextual_examples: Option<serde_json::Value>,
    pub llm_prompt_context: Option<String>,
    pub semantic_embedding: Option<Vec<f32>>,
    pub context_embedding: Option<Vec<f32>>,
    pub similarity_threshold: Option<f32>,

    // Attribute Classification and Derivation
    pub attribute_class: Option<String>, // 'real', 'derived'
    pub derivation_rule: Option<String>,
    pub ebnf_grammar: Option<String>,
    pub derivation_dependencies: Option<Vec<String>>,
    pub quality_score: Option<f32>,
    pub last_derived_at: Option<chrono::NaiveDateTime>,
    pub derivation_error: Option<String>,

    // Persistence Integration
    pub primary_persistence_entity_id: Option<i32>,
    pub backup_persistence_entities: Option<Vec<i32>>,
    pub value_lifecycle: Option<serde_json::Value>,
    pub data_governance: Option<serde_json::Value>,
    pub compliance_metadata: Option<serde_json::Value>,

    // Filtering and Clustering
    pub cluster_assignments: Option<Vec<i32>>,
    pub filter_tags: Option<Vec<String>>,
    pub visibility_rules: Option<serde_json::Value>,
    pub access_control: Option<serde_json::Value>,

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicFormConfiguration {
    pub form_id: String,
    pub title: String,
    pub layout_type: String, // 'wizard', 'tabbed', 'accordion', 'single_page'
    pub groups: Vec<FormGroupConfig>,
    pub navigation: FormNavigationConfig,
    pub validation_rules: serde_json::Value,
    pub conditional_display: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormGroupConfig {
    pub group_id: String,
    pub title: String,
    pub description: Option<String>,
    pub layout: String, // 'grid', 'flex', 'vertical', 'horizontal'
    pub attributes: Vec<AttributeFormConfig>,
    pub conditional_rules: Option<serde_json::Value>,
    pub styling: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeFormConfig {
    pub attribute: AttributeObject,
    pub component_type: String, // 'text_input', 'dropdown', 'checkbox', 'file_upload', etc.
    pub layout_config: serde_json::Value,
    pub validation_config: serde_json::Value,
    pub conditional_visibility: Option<serde_json::Value>,
    pub ai_assistance: Option<AIAssistanceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAssistanceConfig {
    pub enabled: bool,
    pub auto_complete: bool,
    pub suggestions: bool,
    pub validation_hints: bool,
    pub context_help: bool,
    pub prompt_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormNavigationConfig {
    pub navigation_type: String, // 'linear', 'tree', 'free_form'
    pub steps: Vec<NavigationStep>,
    pub progress_indicator: bool,
    pub save_draft: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationStep {
    pub step_id: String,
    pub title: String,
    pub groups: Vec<String>, // group_ids
    pub prerequisites: Vec<String>,
    pub validation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeFilterCriteria {
    pub search_text: Option<String>,
    pub data_types: Option<Vec<String>>,
    pub attribute_classes: Option<Vec<String>>, // 'real', 'derived'
    pub semantic_clusters: Option<Vec<i32>>,
    pub tags: Option<Vec<String>>,
    pub compliance_levels: Option<Vec<String>>,
    pub ui_groups: Option<Vec<String>>,
    pub vector_similarity: Option<VectorSimilarityFilter>,
    pub date_range: Option<DateRangeFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSimilarityFilter {
    pub reference_attribute_id: i32,
    pub similarity_threshold: f32,
    pub max_results: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRangeFilter {
    pub start_date: Option<chrono::NaiveDateTime>,
    pub end_date: Option<chrono::NaiveDateTime>,
    pub date_field: String, // 'created_at', 'updated_at', 'last_derived_at'
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

    /// Generate dynamic form configuration based on metadata
    pub async fn generate_dynamic_form(
        pool: &DbPool,
        resource_name: &str,
        perspective: Option<&str>,
        filter_criteria: Option<&AttributeFilterCriteria>
    ) -> Result<DynamicFormConfiguration, String> {
        // Get full resource configuration
        let config = match Self::get_full_resource_config(pool, resource_name).await? {
            Some(c) => c,
            None => return Err(format!("Resource '{}' not found", resource_name)),
        };

        // Apply filtering if specified
        let filtered_attributes = if let Some(criteria) = filter_criteria {
            Self::filter_attributes(&config.attributes, criteria).await?
        } else {
            config.attributes.clone()
        };

        // Group attributes by UI group
        let mut groups_map: std::collections::HashMap<String, Vec<AttributeFormConfig>> = std::collections::HashMap::new();

        for attr_with_persp in filtered_attributes {
            let group_name = attr_with_persp.attribute.ui_group
                .clone()
                .unwrap_or_else(|| "default".to_string());

            let component_type = Self::determine_component_type(&attr_with_persp.attribute);
            let ai_assistance = Self::create_ai_assistance_config(&attr_with_persp.attribute);

            let form_config = AttributeFormConfig {
                attribute: attr_with_persp.attribute.clone(),
                component_type,
                layout_config: attr_with_persp.attribute.ui_layout_config
                    .clone()
                    .unwrap_or_else(|| serde_json::json!({})),
                validation_config: Self::create_validation_config(&attr_with_persp.attribute),
                conditional_visibility: attr_with_persp.attribute.conditional_logic.clone(),
                ai_assistance: Some(ai_assistance),
            };

            groups_map.entry(group_name).or_insert_with(Vec::new).push(form_config);
        }

        // Create form groups
        let mut groups = Vec::new();
        for (group_name, attributes) in groups_map {
            let group = FormGroupConfig {
                group_id: group_name.clone(),
                title: group_name.clone(),
                description: None,
                layout: "vertical".to_string(),
                attributes,
                conditional_rules: None,
                styling: None,
            };
            groups.push(group);
        }

        // Determine layout type based on resource configuration
        let layout_type = config.resource.ui_layout.clone();

        // Create navigation configuration
        let navigation = FormNavigationConfig {
            navigation_type: if layout_type == "wizard" { "linear".to_string() } else { "free_form".to_string() },
            steps: Self::create_navigation_steps(&groups),
            progress_indicator: layout_type == "wizard",
            save_draft: true,
        };

        Ok(DynamicFormConfiguration {
            form_id: format!("{}_{}", resource_name, chrono::Utc::now().timestamp()),
            title: config.resource.description.unwrap_or_else(|| resource_name.to_string()),
            layout_type,
            groups,
            navigation,
            validation_rules: serde_json::json!({}),
            conditional_display: serde_json::json!({}),
        })
    }

    /// Filter attributes based on comprehensive criteria
    pub async fn filter_attributes(
        attributes: &[AttributeWithPerspectives],
        criteria: &AttributeFilterCriteria
    ) -> Result<Vec<AttributeWithPerspectives>, String> {
        let mut filtered = attributes.to_vec();

        // Text search filter
        if let Some(search_text) = &criteria.search_text {
            filtered.retain(|attr| {
                let search_lower = search_text.to_lowercase();
                attr.attribute.attribute_name.to_lowercase().contains(&search_lower) ||
                attr.attribute.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&search_lower)) ||
                attr.attribute.extended_description.as_ref().map_or(false, |d| d.to_lowercase().contains(&search_lower)) ||
                attr.attribute.business_context.as_ref().map_or(false, |d| d.to_lowercase().contains(&search_lower))
            });
        }

        // Data type filter
        if let Some(data_types) = &criteria.data_types {
            filtered.retain(|attr| data_types.contains(&attr.attribute.data_type));
        }

        // Attribute class filter (real vs derived)
        if let Some(classes) = &criteria.attribute_classes {
            filtered.retain(|attr| {
                attr.attribute.attribute_class.as_ref().map_or(false, |c| classes.contains(c))
            });
        }

        // Semantic cluster filter
        if let Some(clusters) = &criteria.semantic_clusters {
            filtered.retain(|attr| {
                if let Some(assignments) = &attr.attribute.cluster_assignments {
                    assignments.iter().any(|c| clusters.contains(c))
                } else {
                    false
                }
            });
        }

        // Tags filter
        if let Some(tags) = &criteria.tags {
            filtered.retain(|attr| {
                if let Some(attr_tags) = &attr.attribute.filter_tags {
                    tags.iter().any(|t| attr_tags.contains(t))
                } else {
                    false
                }
            });
        }

        Ok(filtered)
    }

    /// Determine appropriate UI component type based on attribute metadata
    fn determine_component_type(attribute: &AttributeObject) -> String {
        // Check explicit UI component type first
        if let Some(component_type) = &attribute.ui_component_type {
            return component_type.clone();
        }

        // Infer from data type and constraints
        match attribute.data_type.as_str() {
            "boolean" => "checkbox".to_string(),
            "integer" | "decimal" => {
                if attribute.allowed_values.is_some() {
                    "dropdown".to_string()
                } else {
                    "number_input".to_string()
                }
            },
            "string" => {
                if let Some(allowed_values) = &attribute.allowed_values {
                    if allowed_values.as_array().map_or(false, |arr| arr.len() <= 10) {
                        "dropdown".to_string()
                    } else {
                        "autocomplete".to_string()
                    }
                } else if attribute.max_length.map_or(false, |len| len > 255) {
                    "textarea".to_string()
                } else {
                    "text_input".to_string()
                }
            },
            "date" => "date_picker".to_string(),
            "datetime" => "datetime_picker".to_string(),
            "file" => "file_upload".to_string(),
            _ => "text_input".to_string(),
        }
    }

    /// Create AI assistance configuration
    fn create_ai_assistance_config(attribute: &AttributeObject) -> AIAssistanceConfig {
        AIAssistanceConfig {
            enabled: attribute.ai_context.is_some() || attribute.ai_prompt_templates.is_some(),
            auto_complete: attribute.semantic_tags.is_some(),
            suggestions: attribute.contextual_examples.is_some(),
            validation_hints: attribute.business_context.is_some(),
            context_help: attribute.user_guidance.is_some(),
            prompt_template: attribute.llm_prompt_context.clone(),
        }
    }

    /// Create validation configuration from attribute metadata
    fn create_validation_config(attribute: &AttributeObject) -> serde_json::Value {
        let mut validation = serde_json::json!({
            "required": attribute.is_required
        });

        if let Some(min_len) = attribute.min_length {
            validation["minLength"] = serde_json::Value::Number(min_len.into());
        }
        if let Some(max_len) = attribute.max_length {
            validation["maxLength"] = serde_json::Value::Number(max_len.into());
        }
        if let Some(pattern) = &attribute.validation_pattern {
            validation["pattern"] = serde_json::Value::String(pattern.clone());
        }
        if let Some(allowed) = &attribute.allowed_values {
            validation["allowedValues"] = allowed.clone();
        }

        validation
    }

    /// Create navigation steps for wizard-style forms
    fn create_navigation_steps(groups: &[FormGroupConfig]) -> Vec<NavigationStep> {
        groups.iter().enumerate().map(|(idx, group)| {
            NavigationStep {
                step_id: format!("step_{}", idx + 1),
                title: group.title.clone(),
                groups: vec![group.group_id.clone()],
                prerequisites: if idx > 0 { vec![format!("step_{}", idx)] } else { vec![] },
                validation_required: true,
            }
        }).collect()
    }

    /// Get filtered attributes with advanced criteria including vector similarity
    pub async fn get_filtered_attributes(
        pool: &DbPool,
        resource_name: &str,
        criteria: &AttributeFilterCriteria
    ) -> Result<Vec<AttributeWithPerspectives>, String> {
        // Start with full resource configuration
        let config = match Self::get_full_resource_config(pool, resource_name).await? {
            Some(c) => c,
            None => return Err(format!("Resource '{}' not found", resource_name)),
        };

        // Apply basic filtering
        let mut filtered = Self::filter_attributes(&config.attributes, criteria).await?;

        // Apply vector similarity filtering if specified
        if let Some(vector_filter) = &criteria.vector_similarity {
            filtered = Self::apply_vector_similarity_filter(pool, filtered, vector_filter).await?;
        }

        Ok(filtered)
    }

    /// Apply vector similarity filtering
    async fn apply_vector_similarity_filter(
        pool: &DbPool,
        attributes: Vec<AttributeWithPerspectives>,
        filter: &VectorSimilarityFilter
    ) -> Result<Vec<AttributeWithPerspectives>, String> {
        // Get reference attribute embedding
        let reference_embedding = sqlx::query_scalar::<_, Vec<f32>>(
            "SELECT semantic_embedding FROM attribute_objects WHERE id = $1"
        )
        .bind(filter.reference_attribute_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to fetch reference embedding: {}", e))?;

        let reference_embedding = match reference_embedding {
            Some(Some(embedding)) => embedding,
            _ => return Ok(attributes), // No embedding available, return as-is
        };

        // Calculate similarities and filter
        let mut similarities: Vec<(usize, f32)> = Vec::new();
        for (idx, attr) in attributes.iter().enumerate() {
            if let Some(embedding) = &attr.attribute.semantic_embedding {
                let similarity = Self::calculate_cosine_similarity(&reference_embedding, embedding);
                if similarity >= filter.similarity_threshold {
                    similarities.push((idx, similarity));
                }
            }
        }

        // Sort by similarity (highest first)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Apply max results limit
        if let Some(max_results) = filter.max_results {
            similarities.truncate(max_results as usize);
        }

        // Return filtered attributes in similarity order
        Ok(similarities.into_iter()
            .map(|(idx, _)| attributes[idx].clone())
            .collect())
    }

    /// Calculate cosine similarity between two vectors
    fn calculate_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
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