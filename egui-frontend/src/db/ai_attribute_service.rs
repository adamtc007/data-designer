use crate::db::{DbPool, config_driven::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormPanelConfig {
    pub panel_type: PanelType,
    pub spawn_behavior: SpawnBehavior,
    pub layout_strategy: LayoutStrategy,
    pub form_config: DynamicFormConfiguration,
    pub ai_enhancements: AIEnhancementConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelType {
    Modal,           // Pop-up modal dialog
    SidePanel,       // Side drawer panel
    TabPanel,        // New tab in existing interface
    FullScreen,      // Full screen overlay
    InlineExpansion, // Expands within current view
    FloatingWindow,  // Separate floating window
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpawnBehavior {
    Replace,         // Replace current content
    Overlay,         // Show over current content
    Adjacent,        // Show beside current content
    NewContext,      // Create entirely new context
    Modal,           // Block interaction with parent
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutStrategy {
    Wizard,          // Step-by-step progression
    Tabbed,          // Multiple tabs for different sections
    Accordion,       // Collapsible sections
    Grid,            // Grid-based layout
    Masonry,         // Pinterest-style masonry
    Timeline,        // Chronological progression
    TreeView,        // Hierarchical tree structure
    CardView,        // Card-based layout
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIEnhancementConfig {
    pub smart_suggestions: bool,
    pub auto_completion: bool,
    pub context_aware_help: bool,
    pub semantic_search: bool,
    pub intelligent_validation: bool,
    pub adaptive_ui: bool,
    pub personalization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeViewRequest {
    pub resource_name: String,
    pub perspective: Option<String>,
    pub filter_criteria: Option<AttributeFilterCriteria>,
    pub view_context: ViewContext,
    pub ai_preferences: Option<AIPreferences>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewContext {
    pub user_id: Option<String>,
    pub role: Option<String>,
    pub current_workflow: Option<String>,
    pub session_context: HashMap<String, serde_json::Value>,
    pub device_type: DeviceType,
    pub screen_size: ScreenSize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Tablet,
    Mobile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPreferences {
    pub assistance_level: AssistanceLevel,
    pub suggestion_frequency: SuggestionFrequency,
    pub auto_save: bool,
    pub smart_validation: bool,
    pub contextual_hints: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssistanceLevel {
    Minimal,
    Moderate,
    Full,
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionFrequency {
    Never,
    OnRequest,
    Periodic,
    Continuous,
}

pub struct AIAttributeService;

impl AIAttributeService {
    /// Main entry point - determines how to spawn the form/panel based on context
    pub async fn create_dynamic_view(
        pool: &DbPool,
        request: &AttributeViewRequest
    ) -> Result<FormPanelConfig, String> {
        // Generate the dynamic form configuration
        let form_config = ConfigDrivenOperations::generate_dynamic_form(
            pool,
            &request.resource_name,
            request.perspective.as_deref(),
            request.filter_criteria.as_ref()
        ).await?;

        // Determine optimal panel type and spawn behavior
        let panel_type = Self::determine_panel_type(&request.view_context, &form_config);
        let spawn_behavior = Self::determine_spawn_behavior(&request.view_context, &panel_type);
        let layout_strategy = Self::determine_layout_strategy(&form_config, &request.view_context);

        // Configure AI enhancements based on user preferences and context
        let ai_enhancements = Self::configure_ai_enhancements(
            &request.ai_preferences,
            &request.view_context,
            &form_config
        );

        Ok(FormPanelConfig {
            panel_type,
            spawn_behavior,
            layout_strategy,
            form_config,
            ai_enhancements,
        })
    }

    /// Determine the most appropriate panel type based on context
    fn determine_panel_type(context: &ViewContext, form_config: &DynamicFormConfiguration) -> PanelType {
        // Consider device type
        match context.device_type {
            DeviceType::Mobile => {
                if form_config.groups.len() > 3 {
                    PanelType::FullScreen
                } else {
                    PanelType::Modal
                }
            },
            DeviceType::Tablet => {
                if form_config.layout_type == "wizard" {
                    PanelType::FullScreen
                } else {
                    PanelType::SidePanel
                }
            },
            DeviceType::Desktop => {
                // Consider screen real estate
                if context.screen_size.width < 1200 {
                    PanelType::Modal
                } else if form_config.groups.len() > 5 {
                    PanelType::TabPanel
                } else {
                    PanelType::SidePanel
                }
            }
        }
    }

    /// Determine spawn behavior based on current workflow context
    fn determine_spawn_behavior(context: &ViewContext, panel_type: &PanelType) -> SpawnBehavior {
        // Check if user is in a workflow that shouldn't be interrupted
        if let Some(workflow) = &context.current_workflow {
            match workflow.as_str() {
                "data_entry" | "form_filling" => SpawnBehavior::Adjacent,
                "review" | "approval" => SpawnBehavior::Overlay,
                "analysis" | "exploration" => SpawnBehavior::NewContext,
                _ => match panel_type {
                    PanelType::Modal => SpawnBehavior::Modal,
                    PanelType::FullScreen => SpawnBehavior::Replace,
                    _ => SpawnBehavior::Overlay,
                }
            }
        } else {
            match panel_type {
                PanelType::Modal => SpawnBehavior::Modal,
                PanelType::FullScreen => SpawnBehavior::Replace,
                PanelType::SidePanel => SpawnBehavior::Adjacent,
                PanelType::TabPanel => SpawnBehavior::NewContext,
                _ => SpawnBehavior::Overlay,
            }
        }
    }

    /// Determine optimal layout strategy based on form structure and context
    fn determine_layout_strategy(
        form_config: &DynamicFormConfiguration,
        context: &ViewContext
    ) -> LayoutStrategy {
        // If explicitly configured in the form
        match form_config.layout_type.as_str() {
            "wizard" => LayoutStrategy::Wizard,
            "tabbed" => LayoutStrategy::Tabbed,
            "accordion" => LayoutStrategy::Accordion,
            "grid" => LayoutStrategy::Grid,
            _ => {
                // Infer based on form characteristics
                let total_attributes: usize = form_config.groups.iter()
                    .map(|g| g.attributes.len())
                    .sum();

                match (form_config.groups.len(), total_attributes, &context.device_type) {
                    // Many groups with few attributes each -> Tabbed
                    (groups, _, DeviceType::Desktop) if groups > 4 => LayoutStrategy::Tabbed,

                    // Many attributes -> Wizard for mobile, Accordion for desktop
                    (_, attrs, DeviceType::Mobile) if attrs > 10 => LayoutStrategy::Wizard,
                    (_, attrs, _) if attrs > 15 => LayoutStrategy::Accordion,

                    // Few groups with many attributes -> Grid
                    (groups, attrs, _) if groups <= 3 && attrs > 6 => LayoutStrategy::Grid,

                    // Timeline if there are wizard steps with clear progression
                    _ if form_config.navigation.navigation_type == "linear" => LayoutStrategy::Timeline,

                    // Default to card view for moderate complexity
                    _ => LayoutStrategy::CardView,
                }
            }
        }
    }

    /// Configure AI enhancements based on preferences and capabilities
    fn configure_ai_enhancements(
        preferences: &Option<AIPreferences>,
        context: &ViewContext,
        form_config: &DynamicFormConfiguration
    ) -> AIEnhancementConfig {
        let default_prefs = AIPreferences {
            assistance_level: AssistanceLevel::Moderate,
            suggestion_frequency: SuggestionFrequency::OnRequest,
            auto_save: true,
            smart_validation: true,
            contextual_hints: true,
        };

        let prefs = preferences.as_ref().unwrap_or(&default_prefs);

        // Check if any attributes have AI capabilities
        let has_ai_attributes = form_config.groups.iter()
            .any(|group| group.attributes.iter()
                .any(|attr| attr.ai_assistance.as_ref().map_or(false, |ai| ai.enabled)));

        AIEnhancementConfig {
            smart_suggestions: has_ai_attributes && matches!(prefs.assistance_level,
                AssistanceLevel::Moderate | AssistanceLevel::Full | AssistanceLevel::Adaptive),

            auto_completion: has_ai_attributes && matches!(prefs.suggestion_frequency,
                SuggestionFrequency::Periodic | SuggestionFrequency::Continuous),

            context_aware_help: prefs.contextual_hints && has_ai_attributes,

            semantic_search: form_config.groups.iter()
                .any(|group| group.attributes.len() > 10), // Enable for complex forms

            intelligent_validation: prefs.smart_validation && has_ai_attributes,

            adaptive_ui: matches!(prefs.assistance_level, AssistanceLevel::Adaptive),

            personalization: context.user_id.is_some() &&
                matches!(prefs.assistance_level, AssistanceLevel::Full | AssistanceLevel::Adaptive),
        }
    }

    /// Get smart attribute recommendations based on context and AI analysis
    pub async fn get_smart_recommendations(
        pool: &DbPool,
        resource_name: &str,
        current_values: &HashMap<String, serde_json::Value>,
        context: &ViewContext
    ) -> Result<Vec<AttributeRecommendation>, String> {
        // Get resource configuration
        let config = match ConfigDrivenOperations::get_full_resource_config(pool, resource_name).await? {
            Some(c) => c,
            None => return Err(format!("Resource '{}' not found", resource_name)),
        };

        let mut recommendations = Vec::new();

        // Analyze current values to suggest next attributes
        for attr_with_persp in &config.attributes {
            let attr = &attr_with_persp.attribute;

            // Skip if already filled
            if current_values.contains_key(&attr.attribute_name) {
                continue;
            }

            // Calculate recommendation score based on various factors
            let score = Self::calculate_recommendation_score(attr, current_values, context);

            if score > 0.3 { // Threshold for recommendations
                recommendations.push(AttributeRecommendation {
                    attribute_id: attr.id,
                    attribute_name: attr.attribute_name.clone(),
                    recommendation_type: Self::determine_recommendation_type(attr, current_values),
                    confidence_score: score,
                    reasoning: Self::generate_reasoning(attr, current_values, context),
                    suggested_value: Self::suggest_value(attr, current_values).await,
                });
            }
        }

        // Sort by confidence score
        recommendations.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());

        Ok(recommendations)
    }

    /// Calculate how relevant an attribute is for recommendation
    fn calculate_recommendation_score(
        attribute: &AttributeObject,
        current_values: &HashMap<String, serde_json::Value>,
        context: &ViewContext
    ) -> f32 {
        let mut score = 0.0;

        // Base score from attribute metadata
        if attribute.is_required {
            score += 0.5;
        }

        // Boost score based on dependencies
        if let Some(deps) = &attribute.derivation_dependencies {
            let filled_deps = deps.iter()
                .filter(|dep| current_values.contains_key(*dep))
                .count();

            if filled_deps > 0 {
                score += (filled_deps as f32 / deps.len() as f32) * 0.3;
            }
        }

        // Context-based scoring
        if let Some(workflow) = &context.current_workflow {
            if let Some(business_context) = &attribute.business_context {
                if business_context.contains(workflow) {
                    score += 0.2;
                }
            }
        }

        // UI grouping hints
        if let Some(group) = &attribute.ui_group {
            let group_progress = current_values.iter()
                .filter(|(key, _)| {
                    // This would need to be enhanced with actual group membership lookup
                    key.contains(group)
                })
                .count();

            if group_progress > 0 {
                score += 0.15;
            }
        }

        score.min(1.0)
    }

    fn determine_recommendation_type(
        attribute: &AttributeObject,
        _current_values: &HashMap<String, serde_json::Value>
    ) -> RecommendationType {
        if attribute.is_required {
            RecommendationType::Required
        } else if attribute.attribute_class.as_ref().map_or(false, |c| c == "derived") {
            RecommendationType::Computed
        } else {
            RecommendationType::Suggested
        }
    }

    fn generate_reasoning(
        attribute: &AttributeObject,
        _current_values: &HashMap<String, serde_json::Value>,
        _context: &ViewContext
    ) -> String {
        if attribute.is_required {
            format!("'{}' is required to complete this form", attribute.attribute_name)
        } else if let Some(business_context) = &attribute.business_context {
            format!("Based on your workflow: {}", business_context)
        } else if let Some(description) = &attribute.description {
            format!("Recommended: {}", description)
        } else {
            "This field may be relevant based on your current progress".to_string()
        }
    }

    async fn suggest_value(
        attribute: &AttributeObject,
        current_values: &HashMap<String, serde_json::Value>
    ) -> Option<serde_json::Value> {
        // If it's a derived attribute with EBNF rules, try to compute
        if attribute.attribute_class.as_ref().map_or(false, |c| c == "derived") {
            if let Some(_ebnf_rule) = &attribute.ebnf_grammar {
                // This would integrate with the EBNF parser to compute derived values
                // For now, return None to indicate computation needed
                return None;
            }
        }

        // Suggest default values based on context
        if let Some(allowed_values) = &attribute.allowed_values {
            if let Some(array) = allowed_values.as_array() {
                if !array.is_empty() {
                    return Some(array[0].clone());
                }
            }
        }

        // Context-based suggestions could be added here
        // For example, if we see patterns in current_values that suggest a value

        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeRecommendation {
    pub attribute_id: i32,
    pub attribute_name: String,
    pub recommendation_type: RecommendationType,
    pub confidence_score: f32,
    pub reasoning: String,
    pub suggested_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    Required,    // Must be filled
    Suggested,   // Recommended based on context
    Computed,    // Can be automatically calculated
    Related,     // Related to current data
}