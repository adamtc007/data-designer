use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub context: CompletionContext,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionContext {
    pub current_line: String,
    pub preceding_lines: Vec<String>,
    pub following_lines: Vec<String>,
    pub cursor_position: usize,
    pub file_path: Option<String>,
    pub available_attributes: Vec<String>,
    pub available_functions: Vec<String>,
    pub data_dictionary_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub suggestions: Vec<Suggestion>,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub text: String,
    pub confidence: f32,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    pub rule: String,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

#[async_trait]
pub trait AIAgent: Send + Sync {
    async fn get_completions(&self, request: CompletionRequest) -> Result<CompletionResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn validate_rule(&self, request: ValidationRequest) -> Result<ValidationResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn explain_rule(&self, rule: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn optimize_rule(&self, rule: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn generate_test_cases(&self, rule: &str) -> Result<Vec<TestCase>, Box<dyn std::error::Error + Send + Sync>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub description: String,
    pub input: HashMap<String, serde_json::Value>,
    pub expected_output: serde_json::Value,
}

// Gemini AI Implementation
pub struct GeminiAgent {
    api_key: String,
    model: String,
    base_url: String,
}

impl GeminiAgent {
    pub fn new(api_key: String) -> Self {
        GeminiAgent {
            api_key,
            model: "gemini-pro".to_string(),
            base_url: "https://generativelanguage.googleapis.com".to_string(),
        }
    }

    async fn call_api(&self, prompt: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Implementation would call actual Gemini API
        // For now, return a mock response
        Ok("Mock Gemini response".to_string())
    }
}

#[async_trait]
impl AIAgent for GeminiAgent {
    async fn get_completions(&self, request: CompletionRequest) -> Result<CompletionResponse, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "You are a DSL code completion assistant. \n\
             Current line: {}\n\
             Cursor position: {}\n\
             Available attributes: {:?}\n\
             Available functions: {:?}\n\
             Suggest completions for the cursor position.",
            request.context.current_line,
            request.context.cursor_position,
            request.context.available_attributes,
            request.context.available_functions
        );

        let response = self.call_api(prompt).await?;

        // Parse response and return suggestions
        Ok(CompletionResponse {
            suggestions: vec![
                Suggestion {
                    text: "suggested_completion".to_string(),
                    confidence: 0.9,
                    description: Some("AI suggested completion".to_string()),
                }
            ],
            explanation: Some(response),
        })
    }

    async fn validate_rule(&self, request: ValidationRequest) -> Result<ValidationResponse, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "Validate this DSL rule: {}\n\
             Context: {:?}\n\
             Check for syntax errors, logical issues, and best practices.",
            request.rule, request.context
        );

        let response = self.call_api(prompt).await?;

        Ok(ValidationResponse {
            is_valid: true,
            issues: vec![],
            suggestions: vec![response],
        })
    }

    async fn explain_rule(&self, rule: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "Explain this DSL rule in plain English: {}\n\
             Break down the logic and explain what it does.",
            rule
        );

        self.call_api(prompt).await
    }

    async fn optimize_rule(&self, rule: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "Optimize this DSL rule for better performance and readability: {}\n\
             Suggest improvements while maintaining the same logic.",
            rule
        );

        self.call_api(prompt).await
    }

    async fn generate_test_cases(&self, rule: &str) -> Result<Vec<TestCase>, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "Generate test cases for this DSL rule: {}\n\
             Include edge cases and typical scenarios.",
            rule
        );

        let response = self.call_api(prompt).await?;

        // Parse response into test cases
        Ok(vec![
            TestCase {
                description: "Test case 1".to_string(),
                input: HashMap::new(),
                expected_output: serde_json::Value::Bool(true),
            }
        ])
    }
}

// GitHub Copilot Integration (via Language Server Protocol)
pub struct CopilotAgent {
    endpoint: String,
}

impl CopilotAgent {
    pub fn new() -> Self {
        CopilotAgent {
            endpoint: "http://localhost:3031".to_string(), // Local Copilot LSP proxy
        }
    }
}

#[async_trait]
impl AIAgent for CopilotAgent {
    async fn get_completions(&self, request: CompletionRequest) -> Result<CompletionResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Implementation would communicate with Copilot LSP
        Ok(CompletionResponse {
            suggestions: vec![
                Suggestion {
                    text: "copilot_suggestion".to_string(),
                    confidence: 0.85,
                    description: Some("Copilot suggestion".to_string()),
                }
            ],
            explanation: None,
        })
    }

    async fn validate_rule(&self, request: ValidationRequest) -> Result<ValidationResponse, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ValidationResponse {
            is_valid: true,
            issues: vec![],
            suggestions: vec![],
        })
    }

    async fn explain_rule(&self, rule: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(format!("Copilot explanation for: {}", rule))
    }

    async fn optimize_rule(&self, rule: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(rule.to_string())
    }

    async fn generate_test_cases(&self, rule: &str) -> Result<Vec<TestCase>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }
}

// Local Mock Agent for testing
pub struct MockAgent;

#[async_trait]
impl AIAgent for MockAgent {
    async fn get_completions(&self, request: CompletionRequest) -> Result<CompletionResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut suggestions = vec![];

        // Simple pattern-based suggestions
        let line = &request.context.current_line;
        let pos = request.context.cursor_position;

        if line.contains("IF ") && !line.contains("THEN") {
            suggestions.push(Suggestion {
                text: "THEN ".to_string(),
                confidence: 0.95,
                description: Some("Complete IF statement with THEN".to_string()),
            });
        }

        if line.ends_with("LOOKUP(") {
            suggestions.push(Suggestion {
                text: "key, \"table_name\")".to_string(),
                confidence: 0.9,
                description: Some("LOOKUP function template".to_string()),
            });
        }

        if line.ends_with("IS_") {
            suggestions.push(Suggestion {
                text: "EMAIL(".to_string(),
                confidence: 0.8,
                description: Some("Email validation function".to_string()),
            });
            suggestions.push(Suggestion {
                text: "LEI(".to_string(),
                confidence: 0.8,
                description: Some("LEI validation function".to_string()),
            });
        }

        Ok(CompletionResponse {
            suggestions,
            explanation: Some("Pattern-based suggestions".to_string()),
        })
    }

    async fn validate_rule(&self, request: ValidationRequest) -> Result<ValidationResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut issues = vec![];

        // Basic validation checks
        if request.rule.contains("IF") && !request.rule.contains("THEN") {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                message: "IF statement missing THEN clause".to_string(),
                line: None,
                column: None,
            });
        }

        let parentheses_balance: i32 = request.rule.chars()
            .map(|c| match c {
                '(' => 1,
                ')' => -1,
                _ => 0,
            })
            .sum();

        if parentheses_balance != 0 {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                message: "Unbalanced parentheses".to_string(),
                line: None,
                column: None,
            });
        }

        Ok(ValidationResponse {
            is_valid: issues.is_empty(),
            issues,
            suggestions: vec![],
        })
    }

    async fn explain_rule(&self, rule: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(format!("This rule evaluates: {}", rule))
    }

    async fn optimize_rule(&self, rule: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Simple optimization: remove unnecessary parentheses
        Ok(rule.replace("((", "(").replace("))", ")"))
    }

    async fn generate_test_cases(&self, _rule: &str) -> Result<Vec<TestCase>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![
            TestCase {
                description: "Basic test case".to_string(),
                input: {
                    let mut map = HashMap::new();
                    map.insert("value".to_string(), serde_json::Value::String("test".to_string()));
                    map
                },
                expected_output: serde_json::Value::Bool(true),
            }
        ])
    }
}

pub struct AIAgentManager {
    agents: HashMap<String, Box<dyn AIAgent>>,
    active_agent: String,
}

impl std::fmt::Debug for AIAgentManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AIAgentManager")
            .field("active_agent", &self.active_agent)
            .field("agents", &self.agents.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl AIAgentManager {
    pub fn new() -> Self {
        let mut agents: HashMap<String, Box<dyn AIAgent>> = HashMap::new();

        // Add mock agent by default
        agents.insert("mock".to_string(), Box::new(MockAgent));

        AIAgentManager {
            agents,
            active_agent: "mock".to_string(),
        }
    }

    pub fn add_agent(&mut self, name: String, agent: Box<dyn AIAgent>) {
        self.agents.insert(name, agent);
    }

    pub fn set_active_agent(&mut self, name: String) -> Result<(), String> {
        if self.agents.contains_key(&name) {
            self.active_agent = name;
            Ok(())
        } else {
            Err(format!("Agent '{}' not found", name))
        }
    }

    pub fn get_active_agent(&self) -> Option<&Box<dyn AIAgent>> {
        self.agents.get(&self.active_agent)
    }

    pub async fn get_completions(&self, request: CompletionRequest) -> Result<CompletionResponse, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(agent) = self.get_active_agent() {
            agent.get_completions(request).await
        } else {
            Err("No active agent".into())
        }
    }

    pub async fn validate_rule(&self, request: ValidationRequest) -> Result<ValidationResponse, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(agent) = self.get_active_agent() {
            agent.validate_rule(request).await
        } else {
            Err("No active agent".into())
        }
    }
}