/// Centralized DSL State Manager
///
/// This module implements a single, focused state manager for DSL content
/// to address the scattered state management issues in the UI.
///
/// Key principles:
/// - Single source of truth for DSL content
/// - Clear state transitions
/// - Testable public API
/// - Immutable updates where possible
use crate::cbu_state_manager::CbuContext;
use crate::grpc_client::{GrpcClient, CbuRecord};
use crate::wasm_utils;

#[derive(Debug, Clone, PartialEq)]
pub enum DslState {
    Empty,
    CreateNew { template: String },
    EditingCbu { cbu_id: String, content: String },
    Modified { original: String, current: String },
}

/// Centralized DSL State Manager
///
/// This replaces the scattered dsl_script management across 26+ locations
/// with a single, testable state management system.
#[derive(Debug, Clone)]
pub struct DslStateManager {
    state: DslState,
    available_cbus: Vec<CbuRecord>,
    validation_enabled: bool,
    syntax_errors: Vec<String>,
}

impl Default for DslStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DslStateManager {
    pub fn new() -> Self {
        Self {
            state: DslState::Empty,
            available_cbus: Vec::new(),
            validation_enabled: true,
            syntax_errors: Vec::new(),
        }
    }

    /// Get current DSL content as string
    pub fn get_content(&self) -> String {
        match &self.state {
            DslState::Empty => String::new(),
            DslState::CreateNew { template } => template.clone(),
            DslState::EditingCbu { content, .. } => content.clone(),
            DslState::Modified { current, .. } => current.clone(),
        }
    }

    /// Update available CBUs for selection
    pub fn set_available_cbus(&mut self, cbus: Vec<CbuRecord>) {
        self.available_cbus = cbus;
    }

    /// Get available CBUs
    pub fn get_available_cbus(&self) -> &[CbuRecord] {
        &self.available_cbus
    }

    /// Load DSL for a specific CBU
    pub fn load_cbu_dsl(&mut self, cbu_id: &str, grpc_client: Option<&GrpcClient>) -> Result<(), String> {
        let Some(_client) = grpc_client else {
            return Err("No gRPC client available for CBU DSL loading".to_string());
        };

        if let Some(cbu) = self.available_cbus.iter().find(|c| c.cbu_id == cbu_id) {
            let dsl_content = format!(
                "# Editing CBU: {}\nUPDATE CBU {} SET description = '{}'\n  # Add entity updates as needed",
                cbu.cbu_name, cbu.cbu_id, cbu.description.as_deref().unwrap_or("")
            );

            self.state = DslState::EditingCbu {
                cbu_id: cbu_id.to_string(),
                content: dsl_content,
            };

            wasm_utils::console_log(&format!("📝 Generated DSL for CBU: {} ({})", cbu.cbu_name, cbu.cbu_id));
            Ok(())
        } else {
            Err(format!("CBU not found: {}", cbu_id))
        }
    }

    /// Update DSL content (preserves edit state)
    pub fn update_content(&mut self, new_content: String) -> Result<(), String> {
        if self.validation_enabled {
            self.validate_syntax(&new_content)?;
        }

        match &self.state {
            DslState::Empty => {
                if !new_content.trim().is_empty() {
                    self.state = DslState::CreateNew { template: new_content };
                }
            },
            DslState::CreateNew { template } => {
                if new_content != *template {
                    self.state = DslState::Modified {
                        original: template.clone(),
                        current: new_content,
                    };
                }
            },
            DslState::EditingCbu { cbu_id, content } => {
                if new_content != *content {
                    self.state = DslState::Modified {
                        original: content.clone(),
                        current: new_content,
                    };
                } else {
                    // Content reverted to original - go back to EditingCbu state
                    self.state = DslState::EditingCbu {
                        cbu_id: cbu_id.clone(),
                        content: content.clone(),
                    };
                }
            },
            DslState::Modified { original, .. } => {
                if new_content == *original {
                    // Reverted to original - determine original state
                    if original.starts_with("# Editing CBU:") {
                        // Extract CBU ID from original content
                        if let Some(cbu_id) = self.extract_cbu_id_from_content(original) {
                            self.state = DslState::EditingCbu {
                                cbu_id,
                                content: original.clone(),
                            };
                        } else {
                            self.state = DslState::CreateNew { template: original.clone() };
                        }
                    } else {
                        self.state = DslState::CreateNew { template: original.clone() };
                    }
                } else {
                    self.state = DslState::Modified {
                        original: original.clone(),
                        current: new_content,
                    };
                }
            }
        }

        Ok(())
    }

    /// Clear all DSL content
    pub fn clear(&mut self) {
        self.state = DslState::Empty;
        self.syntax_errors.clear();
    }

    /// Check if content has been modified
    pub fn is_modified(&self) -> bool {
        matches!(self.state, DslState::Modified { .. })
    }

    /// Get current editing context
    pub fn get_context(&self) -> CbuContext {
        match &self.state {
            DslState::Empty => CbuContext::None,
            DslState::CreateNew { .. } => CbuContext::CreateNew,
            DslState::EditingCbu { .. } => CbuContext::EditExisting,
            DslState::Modified { .. } => CbuContext::EditExisting,
        }
    }

    /// Get current CBU ID if editing a CBU
    pub fn get_current_cbu_id(&self) -> Option<String> {
        match &self.state {
            DslState::EditingCbu { cbu_id, .. } => Some(cbu_id.clone()),
            DslState::Modified { original, .. } => {
                self.extract_cbu_id_from_content(original)
            },
            _ => None,
        }
    }

    /// Validate DSL syntax
    pub fn validate_syntax(&mut self, content: &str) -> Result<(), String> {
        self.syntax_errors.clear();

        // Basic validation - can be extended
        if content.trim().is_empty() {
            return Ok(());
        }

        let lines: Vec<&str> = content.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            // Validate CBU operations
            if trimmed.starts_with("CREATE CBU") || trimmed.starts_with("UPDATE CBU") {
                if !trimmed.contains("SET") {
                    let error = format!("Line {}: CBU operation missing SET clause", line_num + 1);
                    self.syntax_errors.push(error.clone());
                    return Err(error);
                }
            }
        }

        Ok(())
    }

    /// Get validation errors
    pub fn get_syntax_errors(&self) -> &[String] {
        &self.syntax_errors
    }

    /// Enable/disable syntax validation
    pub fn set_validation_enabled(&mut self, enabled: bool) {
        self.validation_enabled = enabled;
    }

    /// Extract CBU information from DSL content
    pub fn extract_cbu_info(&self) -> (Option<String>, Option<String>) {
        let content = self.get_content();
        self.extract_cbu_info_from_content(&content)
    }

    // Private helper methods
    fn extract_cbu_id_from_content(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("UPDATE CBU ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 3 {
                    return Some(parts[2].to_string());
                }
            }
        }
        None
    }

    fn extract_cbu_info_from_content(&self, content: &str) -> (Option<String>, Option<String>) {
        if content.is_empty() {
            return (None, None);
        }

        let mut cbu_name = None;
        let mut cbu_description = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("CREATE CBU") || trimmed.starts_with("UPDATE CBU") {
                // Extract CBU name/ID
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name_part = parts[2];
                    cbu_name = Some(name_part.trim_matches('\'').trim_matches('"').to_string());
                }

                // Extract description
                if let Some(desc_start) = trimmed.find("description = ") {
                    let desc_part = &trimmed[desc_start + 14..];
                    if let Some(quote_start) = desc_part.find('\'') {
                        if let Some(quote_end) = desc_part[quote_start + 1..].find('\'') {
                            cbu_description = Some(desc_part[quote_start + 1..quote_start + 1 + quote_end].to_string());
                        }
                    }
                }
                break;
            }
        }

        (cbu_name, cbu_description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cbu() -> CbuRecord {
        CbuRecord {
            id: 1,
            cbu_id: "TEST_CBU_001".to_string(),
            cbu_name: "Test Investment Fund".to_string(),
            description: Some("Test fund for DSL validation".to_string()),
            legal_entity_name: None,
            business_model: None,
            status: "active".to_string(),
            created_at: Some("2024-01-01".to_string()),
            updated_at: None,
            dsl_content: None,
            dsl_metadata: None,
        }
    }

    #[test]
    fn test_new_manager_starts_empty() {
        let manager = DslStateManager::new();
        assert_eq!(manager.get_content(), "");
        assert_eq!(manager.get_context(), CbuContext::None);
        assert!(!manager.is_modified());
    }

    #[test]
    fn test_load_cbu_dsl_without_client_fails() {
        let mut manager = DslStateManager::new();
        manager.set_available_cbus(vec![create_test_cbu()]);

        let result = manager.load_cbu_dsl("TEST_CBU_001", None);
        assert!(result.is_err());
        assert_eq!(manager.get_content(), "");
    }

    #[test]
    fn test_content_update_preserves_state() {
        let mut manager = DslStateManager::new();

        // Start with new content
        manager.update_content("# Test content".to_string()).unwrap();
        assert_eq!(manager.get_context(), CbuContext::CreateNew);

        // Modify content
        manager.update_content("# Modified content".to_string()).unwrap();
        assert!(manager.is_modified());

        // Revert to original
        manager.update_content("# Test content".to_string()).unwrap();
        assert!(!manager.is_modified());
    }

    #[test]
    fn test_syntax_validation() {
        let mut manager = DslStateManager::new();

        // Valid syntax
        let valid_dsl = "UPDATE CBU TEST_001 SET description = 'Test'";
        assert!(manager.update_content(valid_dsl.to_string()).is_ok());

        // Invalid syntax
        let invalid_dsl = "UPDATE CBU TEST_001 description = 'Test'"; // Missing SET
        assert!(manager.update_content(invalid_dsl.to_string()).is_err());
    }

    #[test]
    fn test_extract_cbu_info() {
        let mut manager = DslStateManager::new();
        manager.update_content("UPDATE CBU TEST_001 SET description = 'Test Description'".to_string()).unwrap();

        let (name, desc) = manager.extract_cbu_info();
        assert_eq!(name, Some("TEST_001".to_string()));
        assert_eq!(desc, Some("Test Description".to_string()));
    }
}