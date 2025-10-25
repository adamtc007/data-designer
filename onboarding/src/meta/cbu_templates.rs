use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbuTemplates {
    #[serde(rename = "cbuTemplates")]
    pub cbu_templates: Vec<CbuTemplate>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbuTemplate {
    pub id: String,
    pub description: Option<String>,
    #[serde(rename = "requiredRoles")]
    pub required_roles: Vec<RequiredRole>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredRole {
    pub role: String,
    #[serde(default, rename = "entityTypeConstraint")]
    pub entity_type_constraint: Vec<String>,
}
