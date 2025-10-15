use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use data_designer_core::models::{Expression, Value, DataDictionary};
use data_designer_core::db::{DbPool, EmbeddingOperations};
use reqwest;
use keyring::Entry;

/// AI Assistant for intelligent DSL development
#[derive(Clone)]
pub struct AiAssistant {
    pub provider: AiProvider,
    pub context: DslContext,
    pub suggestions_cache: HashMap<String, Vec<AiSuggestion>>,
    pub conversation_history: Vec<AiMessage>,
    pub db_pool: Option<DbPool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AiProvider {
    OpenAI { api_key: Option<String> },
    Anthropic { api_key: Option<String> },
    Offline, // Fallback mode with pattern-based responses
}

#[derive(Debug, Clone)]
pub struct DslContext {
    pub current_rule: String,
    pub cursor_position: usize,
    pub data_dictionary: Option<DataDictionary>,
    pub available_functions: Vec<String>,
    pub available_attributes: Vec<String>,
    pub recent_errors: Vec<String>,
    pub target_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestion {
    pub suggestion_type: SuggestionType,
    pub title: String,
    pub description: String,
    pub code_snippet: Option<String>,
    pub confidence: f32, // 0.0 to 1.0
    pub context_relevance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    CodeCompletion,
    ErrorFix,
    Optimization,
    FunctionUsage,
    BestPractice,
    Alternative,
    Documentation,
    SimilarPattern,
    PatternMatch,
    AutoComplete,
    SnippetCompletion,
    ErrorAnalysis,
    QuickFix,
}

#[derive(Debug, Clone)]
pub struct AiMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIRequestMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct OpenAIRequestMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicRequestMessage>,
}

#[derive(Debug, Serialize)]
struct AnthropicRequestMessage {
    role: String,
    content: String,
}

impl Default for AiAssistant {
    fn default() -> Self {
        Self {
            provider: AiProvider::Offline,
            context: DslContext::default(),
            suggestions_cache: HashMap::new(),
            conversation_history: Vec::new(),
            db_pool: None,
        }
    }
}

impl Default for DslContext {
    fn default() -> Self {
        Self {
            current_rule: String::new(),
            cursor_position: 0,
            data_dictionary: None,
            available_functions: vec![
                "CONCAT".to_string(), "UPPER".to_string(), "LOWER".to_string(),
                "LENGTH".to_string(), "TRIM".to_string(), "SUBSTRING".to_string(),
                "IS_EMAIL".to_string(), "IS_LEI".to_string(), "IS_SWIFT".to_string(),
                "LOOKUP".to_string(), "ROUND".to_string(), "ABS".to_string(),
            ],
            available_attributes: Vec::new(),
            recent_errors: Vec::new(),
            target_language: "Rust".to_string(),
        }
    }
}

impl AiAssistant {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_provider(mut self, provider: AiProvider) -> Self {
        self.provider = provider;
        self
    }

    /// Get AI suggestions based on current context
    pub async fn get_suggestions(&mut self, query: &str) -> Vec<AiSuggestion> {
        // Check cache first
        if let Some(cached) = self.suggestions_cache.get(query) {
            return cached.clone();
        }

        let suggestions = match &self.provider {
            AiProvider::OpenAI { api_key } => {
                if api_key.is_some() {
                    self.get_openai_suggestions(query).await.unwrap_or_else(|_| self.get_offline_suggestions(query))
                } else {
                    self.get_offline_suggestions(query)
                }
            }
            AiProvider::Anthropic { api_key } => {
                if api_key.is_some() {
                    self.get_anthropic_suggestions(query).await.unwrap_or_else(|_| self.get_offline_suggestions(query))
                } else {
                    self.get_offline_suggestions(query)
                }
            }
            AiProvider::Offline => self.get_offline_suggestions(query),
        };

        // Cache the results
        self.suggestions_cache.insert(query.to_string(), suggestions.clone());
        suggestions
    }

    /// OpenAI-powered suggestions
    async fn get_openai_suggestions(&self, query: &str) -> Result<Vec<AiSuggestion>, Box<dyn std::error::Error>> {
        let prompt = self.build_context_prompt(query);

        // Get API key from provider
        let api_key = match &self.provider {
            AiProvider::OpenAI { api_key } => {
                api_key.as_ref().ok_or("No OpenAI API key available")?
            }
            _ => return Err("Not using OpenAI provider".into()),
        };

        // Create HTTP client
        let client = reqwest::Client::new();

        // Build request
        let request = OpenAIRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![
                OpenAIRequestMessage {
                    role: "system".to_string(),
                    content: "You are an expert assistant for a financial KYC DSL system. Provide helpful code suggestions in JSON format.".to_string(),
                },
                OpenAIRequestMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            max_tokens: 1000,
            temperature: 0.3,
        };

        // Make API call
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error: {}", error_text).into());
        }

        let openai_response: OpenAIResponse = response.json().await?;

        // Parse the response and convert to suggestions
        let mut suggestions = Vec::new();
        if let Some(choice) = openai_response.choices.first() {
            let content = &choice.message.content;

            // Try to parse as structured suggestions, fallback to text parsing
            suggestions.extend(self.parse_ai_response_to_suggestions(content));
        }

        // If we got no suggestions from AI, fallback to offline suggestions
        if suggestions.is_empty() {
            suggestions = self.get_enhanced_offline_suggestions(query);
        }

        Ok(suggestions)
    }

    /// Anthropic Claude-powered suggestions
    async fn get_anthropic_suggestions(&self, query: &str) -> Result<Vec<AiSuggestion>, Box<dyn std::error::Error>> {
        let prompt = self.build_context_prompt(query);

        // Get API key from provider
        let api_key = match &self.provider {
            AiProvider::Anthropic { api_key } => {
                api_key.as_ref().ok_or("No Anthropic API key available")?
            }
            _ => return Err("Not using Anthropic provider".into()),
        };

        // Create HTTP client
        let client = reqwest::Client::new();

        // Build request
        let request = AnthropicRequest {
            model: "claude-3-haiku-20240307".to_string(),
            max_tokens: 1000,
            messages: vec![
                AnthropicRequestMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
        };

        // Make API call
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Anthropic API error: {}", error_text).into());
        }

        let anthropic_response: AnthropicResponse = response.json().await?;

        // Parse the response and convert to suggestions
        let mut suggestions = Vec::new();
        if let Some(content) = anthropic_response.content.first() {
            let text = &content.text;

            // Try to parse as structured suggestions, fallback to text parsing
            suggestions.extend(self.parse_ai_response_to_suggestions(text));
        }

        // If we got no suggestions from AI, fallback to offline suggestions
        if suggestions.is_empty() {
            suggestions = self.get_enhanced_offline_suggestions(query);
        }

        Ok(suggestions)
    }

    /// Intelligent offline suggestions using pattern matching
    pub fn get_offline_suggestions(&self, query: &str) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();
        let query_lower = query.to_lowercase();

        // Code completion suggestions
        if query_lower.contains("email") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::FunctionUsage,
                title: "Email Validation".to_string(),
                description: "Use IS_EMAIL function to validate email addresses".to_string(),
                code_snippet: Some("IS_EMAIL(Client.email)".to_string()),
                confidence: 0.9,
                context_relevance: 0.95,
            });

            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::CodeCompletion,
                title: "Email Pattern Match".to_string(),
                description: "Use regex pattern to validate email format".to_string(),
                code_snippet: Some("Client.email ~ /^[\\w.-]+@[\\w.-]+\\.[A-Za-z]{2,}$/".to_string()),
                confidence: 0.8,
                context_relevance: 0.85,
            });
        }

        if query_lower.contains("concat") || query_lower.contains("string") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::FunctionUsage,
                title: "String Concatenation".to_string(),
                description: "Combine multiple strings into one".to_string(),
                code_snippet: Some("CONCAT(first_name, \" \", last_name)".to_string()),
                confidence: 0.95,
                context_relevance: 0.9,
            });

            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::Alternative,
                title: "String Concatenation with &".to_string(),
                description: "Alternative syntax for string concatenation".to_string(),
                code_snippet: Some("first_name & \" \" & last_name".to_string()),
                confidence: 0.85,
                context_relevance: 0.8,
            });
        }

        if query_lower.contains("if") || query_lower.contains("condition") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::CodeCompletion,
                title: "Conditional Expression".to_string(),
                description: "IF-THEN-ELSE conditional logic".to_string(),
                code_snippet: Some("IF condition THEN value1 ELSE value2".to_string()),
                confidence: 0.9,
                context_relevance: 0.9,
            });

            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::BestPractice,
                title: "Nested Conditions".to_string(),
                description: "Handle multiple conditions efficiently".to_string(),
                code_snippet: Some("IF risk_score > 80 THEN \"HIGH\" ELSE IF risk_score > 50 THEN \"MEDIUM\" ELSE \"LOW\"".to_string()),
                confidence: 0.8,
                context_relevance: 0.75,
            });
        }

        if query_lower.contains("lookup") || query_lower.contains("table") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::FunctionUsage,
                title: "Lookup Table Usage".to_string(),
                description: "Retrieve values from lookup tables".to_string(),
                code_snippet: Some("LOOKUP(country_code, \"countries\")".to_string()),
                confidence: 0.9,
                context_relevance: 0.85,
            });
        }

        if query_lower.contains("error") || self.context.recent_errors.len() > 0 {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::ErrorFix,
                title: "Common Syntax Fixes".to_string(),
                description: "Check for missing quotes, parentheses, or operators".to_string(),
                code_snippet: None,
                confidence: 0.7,
                context_relevance: 0.8,
            });
        }

        // Function-specific suggestions based on context
        for func in &self.context.available_functions {
            if query_lower.contains(&func.to_lowercase()) {
                suggestions.push(self.get_function_suggestion(func));
            }
        }

        // KYC-specific suggestions
        if query_lower.contains("kyc") || query_lower.contains("compliance") {
            suggestions.extend(self.get_kyc_suggestions());
        }

        // Optimization suggestions
        if query_lower.contains("optimize") || query_lower.contains("performance") {
            suggestions.extend(self.get_optimization_suggestions());
        }

        suggestions.sort_by(|a, b| {
            (b.confidence * b.context_relevance).partial_cmp(&(a.confidence * a.context_relevance)).unwrap()
        });

        suggestions.into_iter().take(10).collect() // Limit to top 10
    }

    /// Enhanced offline suggestions with better context awareness
    fn get_enhanced_offline_suggestions(&self, query: &str) -> Vec<AiSuggestion> {
        let mut base_suggestions = self.get_offline_suggestions(query);

        // Add context-aware suggestions based on current rule
        if !self.context.current_rule.is_empty() {
            base_suggestions.extend(self.analyze_current_rule());
        }

        // Add data dictionary-aware suggestions
        if let Some(dict) = &self.context.data_dictionary {
            base_suggestions.extend(self.get_dictionary_suggestions(dict, query));
        }

        base_suggestions
    }

    fn get_function_suggestion(&self, func_name: &str) -> AiSuggestion {
        let (description, example) = match func_name {
            "CONCAT" => ("Concatenate multiple strings", "CONCAT(\"Hello\", \" \", \"World\")"),
            "UPPER" => ("Convert string to uppercase", "UPPER(Client.name)"),
            "LOWER" => ("Convert string to lowercase", "LOWER(Client.email)"),
            "LENGTH" => ("Get string length", "LENGTH(Client.description)"),
            "TRIM" => ("Remove whitespace", "TRIM(Client.notes)"),
            "SUBSTRING" => ("Extract substring", "SUBSTRING(Client.id, 1, 5)"),
            "IS_EMAIL" => ("Validate email format", "IS_EMAIL(Client.email)"),
            "IS_LEI" => ("Validate LEI code", "IS_LEI(Client.lei_code)"),
            "IS_SWIFT" => ("Validate SWIFT code", "IS_SWIFT(Client.swift_code)"),
            "LOOKUP" => ("Lookup value from table", "LOOKUP(Client.country, \"countries\")"),
            "ROUND" => ("Round number", "ROUND(Client.balance, 2)"),
            "ABS" => ("Absolute value", "ABS(Client.net_position)"),
            _ => ("Function usage", "function call"),
        };

        AiSuggestion {
            suggestion_type: SuggestionType::FunctionUsage,
            title: format!("{} Function", func_name),
            description: description.to_string(),
            code_snippet: Some(example.to_string()),
            confidence: 0.9,
            context_relevance: 0.8,
        }
    }

    fn get_kyc_suggestions(&self) -> Vec<AiSuggestion> {
        vec![
            AiSuggestion {
                suggestion_type: SuggestionType::BestPractice,
                title: "KYC Risk Assessment".to_string(),
                description: "Standard risk calculation for KYC".to_string(),
                code_snippet: Some("risk_score = IF Client.pep_status THEN 50 ELSE 0 + IF Client.high_risk_country THEN 30 ELSE 0".to_string()),
                confidence: 0.9,
                context_relevance: 0.95,
            },
            AiSuggestion {
                suggestion_type: SuggestionType::CodeCompletion,
                title: "LEI Validation".to_string(),
                description: "Validate Legal Entity Identifier".to_string(),
                code_snippet: Some("lei_valid = IS_LEI(Client.lei_code) AND LENGTH(Client.lei_code) == 20".to_string()),
                confidence: 0.85,
                context_relevance: 0.9,
            },
            AiSuggestion {
                suggestion_type: SuggestionType::BestPractice,
                title: "Compliance Status Check".to_string(),
                description: "Comprehensive compliance validation".to_string(),
                code_snippet: Some("compliance_ok = IS_EMAIL(Client.email) AND IS_LEI(Client.lei_code) AND Client.kyc_status == \"APPROVED\"".to_string()),
                confidence: 0.8,
                context_relevance: 0.85,
            },
        ]
    }

    fn get_optimization_suggestions(&self) -> Vec<AiSuggestion> {
        vec![
            AiSuggestion {
                suggestion_type: SuggestionType::Optimization,
                title: "Constant Folding".to_string(),
                description: "Combine constants at compile time".to_string(),
                code_snippet: Some("// Instead of: 2 + 3 + tax\n// Use: 5 + tax".to_string()),
                confidence: 0.7,
                context_relevance: 0.6,
            },
            AiSuggestion {
                suggestion_type: SuggestionType::Optimization,
                title: "Early Return Pattern".to_string(),
                description: "Exit early for better performance".to_string(),
                code_snippet: Some("IF Client.kyc_status != \"APPROVED\" THEN \"REJECTED\" ELSE complex_calculation()".to_string()),
                confidence: 0.75,
                context_relevance: 0.7,
            },
        ]
    }

    fn analyze_current_rule(&self) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();
        let rule = &self.context.current_rule;

        // Detect incomplete patterns
        if rule.contains("IF") && !rule.contains("THEN") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::CodeCompletion,
                title: "Complete IF Statement".to_string(),
                description: "Add THEN clause to IF statement".to_string(),
                code_snippet: Some("IF condition THEN result".to_string()),
                confidence: 0.9,
                context_relevance: 0.95,
            });
        }

        if rule.contains("THEN") && !rule.contains("ELSE") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::CodeCompletion,
                title: "Add ELSE Clause".to_string(),
                description: "Consider adding ELSE for complete condition".to_string(),
                code_snippet: Some("ELSE default_value".to_string()),
                confidence: 0.7,
                context_relevance: 0.8,
            });
        }

        // Detect unmatched parentheses
        let open_parens = rule.matches('(').count();
        let close_parens = rule.matches(')').count();
        if open_parens > close_parens {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::ErrorFix,
                title: "Missing Closing Parenthesis".to_string(),
                description: format!("Add {} closing parenthesis", open_parens - close_parens),
                code_snippet: Some(")".repeat(open_parens - close_parens)),
                confidence: 0.95,
                context_relevance: 0.95,
            });
        }

        suggestions
    }

    fn get_dictionary_suggestions(&self, dict: &DataDictionary, query: &str) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();

        // Suggest available attributes
        for dataset in &dict.datasets {
            for (attr_name, _) in &dataset.attributes {
                if attr_name.to_lowercase().contains(&query.to_lowercase()) {
                    suggestions.push(AiSuggestion {
                        suggestion_type: SuggestionType::CodeCompletion,
                        title: format!("Use {}.{}", dataset.name, attr_name),
                        description: format!("Reference attribute from {}", dataset.name),
                        code_snippet: Some(format!("{}.{}", dataset.name, attr_name)),
                        confidence: 0.8,
                        context_relevance: 0.9,
                    });
                }
            }
        }

        suggestions
    }

    /// Build context-aware prompt for AI APIs
    fn build_context_prompt(&self, query: &str) -> String {
        let mut prompt = String::new();

        prompt.push_str("You are an expert DSL assistant for a financial KYC system. ");
        prompt.push_str("Help users write, debug, and optimize DSL expressions.\n\n");

        prompt.push_str("Context:\n");
        prompt.push_str(&format!("- Current rule: {}\n", self.context.current_rule));
        prompt.push_str(&format!("- Target language: {}\n", self.context.target_language));
        prompt.push_str(&format!("- Available functions: {:?}\n", self.context.available_functions));

        if !self.context.recent_errors.is_empty() {
            prompt.push_str(&format!("- Recent errors: {:?}\n", self.context.recent_errors));
        }

        prompt.push_str("\nAvailable DSL features:\n");
        prompt.push_str("- Operators: +, -, *, /, ==, !=, <, >, <=, >=, AND, OR, NOT\n");
        prompt.push_str("- Functions: CONCAT, UPPER, LOWER, LENGTH, TRIM, SUBSTRING, IS_EMAIL, IS_LEI, IS_SWIFT, LOOKUP, ROUND, ABS\n");
        prompt.push_str("- Conditionals: IF condition THEN value ELSE value\n");
        prompt.push_str("- Pattern matching: value ~ /regex/\n");

        prompt.push_str(&format!("\nUser query: {}\n", query));
        prompt.push_str("Provide helpful, actionable suggestions with code examples.");

        prompt
    }

    /// Update context with current editor state
    pub fn update_context(&mut self, rule: String, cursor_pos: usize, errors: Vec<String>) {
        self.context.current_rule = rule;
        self.context.cursor_position = cursor_pos;
        self.context.recent_errors = errors;

        // Clear cache when context changes significantly
        if self.suggestions_cache.len() > 100 {
            self.suggestions_cache.clear();
        }
    }

    /// Set data dictionary for context-aware suggestions
    pub fn set_data_dictionary(&mut self, dict: DataDictionary) {
        self.context.data_dictionary = Some(dict);
    }

    /// Get quick help for common DSL patterns
    pub fn get_quick_help(&self, topic: &str) -> Vec<AiSuggestion> {
        match topic.to_lowercase().as_str() {
            "functions" => self.get_all_function_suggestions(),
            "operators" => self.get_operator_suggestions(),
            "examples" => self.get_example_suggestions(),
            "errors" => self.get_error_help(),
            _ => self.get_offline_suggestions(topic),
        }
    }

    fn get_all_function_suggestions(&self) -> Vec<AiSuggestion> {
        self.context.available_functions.iter()
            .map(|func| self.get_function_suggestion(func))
            .collect()
    }

    fn get_operator_suggestions(&self) -> Vec<AiSuggestion> {
        vec![
            AiSuggestion {
                suggestion_type: SuggestionType::Documentation,
                title: "Arithmetic Operators".to_string(),
                description: "Basic math operations: + - * / %".to_string(),
                code_snippet: Some("price * quantity + tax".to_string()),
                confidence: 1.0,
                context_relevance: 0.8,
            },
            AiSuggestion {
                suggestion_type: SuggestionType::Documentation,
                title: "Comparison Operators".to_string(),
                description: "Compare values: == != < > <= >=".to_string(),
                code_snippet: Some("age >= 18 AND status == \"ACTIVE\"".to_string()),
                confidence: 1.0,
                context_relevance: 0.8,
            },
            AiSuggestion {
                suggestion_type: SuggestionType::Documentation,
                title: "Logical Operators".to_string(),
                description: "Combine conditions: AND OR NOT".to_string(),
                code_snippet: Some("active AND NOT suspended".to_string()),
                confidence: 1.0,
                context_relevance: 0.8,
            },
        ]
    }

    fn get_example_suggestions(&self) -> Vec<AiSuggestion> {
        vec![
            AiSuggestion {
                suggestion_type: SuggestionType::Documentation,
                title: "KYC Risk Calculation".to_string(),
                description: "Complete example of risk assessment".to_string(),
                code_snippet: Some("risk_score = IF Client.pep_status THEN 50 ELSE 0 + IF LOOKUP(Client.country, \"high_risk\") THEN 30 ELSE 10".to_string()),
                confidence: 0.9,
                context_relevance: 0.95,
            },
            AiSuggestion {
                suggestion_type: SuggestionType::Documentation,
                title: "Email Validation".to_string(),
                description: "Comprehensive email checking".to_string(),
                code_snippet: Some("email_valid = IS_EMAIL(Client.email) AND LENGTH(Client.email) > 5 AND Client.email ~ /@company\\.com$/".to_string()),
                confidence: 0.9,
                context_relevance: 0.9,
            },
        ]
    }

    fn get_error_help(&self) -> Vec<AiSuggestion> {
        vec![
            AiSuggestion {
                suggestion_type: SuggestionType::ErrorFix,
                title: "Common Syntax Errors".to_string(),
                description: "Most frequent mistakes and fixes".to_string(),
                code_snippet: None,
                confidence: 0.8,
                context_relevance: 0.9,
            },
        ]
    }

    /// Set database pool for semantic search
    pub fn set_db_pool(&mut self, pool: DbPool) {
        self.db_pool = Some(pool);
    }

    /// Enhanced offline suggestions with semantic search
    pub async fn get_enhanced_suggestions(&self, query: &str) -> Vec<AiSuggestion> {
        let mut suggestions = self.get_offline_suggestions(query);

        // Add semantic search suggestions if database is available
        if let Some(pool) = &self.db_pool {
            if let Ok(semantic_suggestions) = self.get_semantic_suggestions(pool, query).await {
                suggestions.extend(semantic_suggestions);
            }
        }

        // Sort by relevance and confidence
        suggestions.sort_by(|a, b| {
            let score_a = a.confidence * a.context_relevance;
            let score_b = b.confidence * b.context_relevance;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to top 10 suggestions
        suggestions.truncate(10);
        suggestions
    }

    /// Get semantic search suggestions from database
    async fn get_semantic_suggestions(&self, pool: &DbPool, query: &str) -> Result<Vec<AiSuggestion>, String> {
        let mut suggestions = Vec::new();

        // Search for similar rules based on current input or query
        let search_text = if !self.context.current_rule.is_empty() {
            &self.context.current_rule
        } else {
            query
        };

        match EmbeddingOperations::find_similar_rules(pool, search_text, 5).await {
            Ok(similar_rules) => {
                for rule in similar_rules {
                    // Convert similarity score to confidence (higher similarity = higher confidence)
                    let confidence = 1.0 - rule.similarity.min(1.0);

                    if confidence > 0.3 { // Only include reasonably similar rules
                        suggestions.push(AiSuggestion {
                            suggestion_type: SuggestionType::SimilarPattern,
                            title: format!("Similar Rule: {}", rule.rule_name),
                            description: format!("Found similar pattern with {:.0}% similarity", confidence * 100.0),
                            code_snippet: Some(rule.rule_definition),
                            confidence,
                            context_relevance: confidence * 0.9, // Slightly lower context relevance
                        });
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️ Semantic search failed: {}", e);
            }
        }

        // Add pattern-based suggestions using embeddings
        if query.len() > 3 {
            if let Ok(pattern_suggestions) = self.get_pattern_suggestions(pool, query).await {
                suggestions.extend(pattern_suggestions);
            }
        }

        Ok(suggestions)
    }

    /// Get pattern-based suggestions using semantic analysis
    async fn get_pattern_suggestions(&self, pool: &DbPool, query: &str) -> Result<Vec<AiSuggestion>, String> {
        let mut suggestions = Vec::new();

        // Analyze query for common patterns and suggest related rules
        let keywords = self.extract_keywords(query);

        for keyword in keywords {
            if let Ok(related_rules) = EmbeddingOperations::find_similar_rules(pool, &keyword, 3).await {
                for rule in related_rules {
                    let confidence = 1.0 - rule.similarity.min(1.0);

                    if confidence > 0.4 {
                        suggestions.push(AiSuggestion {
                            suggestion_type: SuggestionType::PatternMatch,
                            title: format!("Pattern: {}", keyword),
                            description: format!("Rule using similar pattern: {}", rule.rule_name),
                            code_snippet: Some(self.extract_pattern_snippet(&rule.rule_definition, &keyword)),
                            confidence: confidence * 0.8,
                            context_relevance: 0.7,
                        });
                    }
                }
            }
        }

        Ok(suggestions)
    }

    /// Extract keywords from query for semantic search
    fn extract_keywords(&self, query: &str) -> Vec<String> {
        let stop_words = ["the", "and", "or", "is", "are", "was", "were", "in", "on", "at", "to", "for", "of", "with", "by"];

        query
            .to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 2 && !stop_words.contains(word))
            .map(|word| word.to_string())
            .collect()
    }

    /// Extract relevant snippet from rule definition
    fn extract_pattern_snippet(&self, rule_definition: &str, keyword: &str) -> String {
        // Find the part of the rule that contains the keyword
        let lines: Vec<&str> = rule_definition.lines().collect();

        for line in &lines {
            if line.to_lowercase().contains(&keyword.to_lowercase()) {
                return line.trim().to_string();
            }
        }

        // If no line contains the keyword, return first meaningful line
        lines.iter()
            .find(|line| line.trim().len() > 10)
            .unwrap_or(&rule_definition)
            .trim()
            .to_string()
    }

    /// Batch update embeddings for all rules in database
    pub async fn refresh_embeddings(&self) -> Result<(), String> {
        if let Some(pool) = &self.db_pool {
            EmbeddingOperations::generate_all_embeddings(pool).await?;
            println!("✅ Successfully refreshed all rule embeddings");
        } else {
            return Err("No database connection available".to_string());
        }
        Ok(())
    }

    /// Search for rules by semantic similarity to a specific DSL expression
    pub async fn find_similar_dsl_patterns(&self, dsl_expression: &str) -> Result<Vec<AiSuggestion>, String> {
        if let Some(pool) = &self.db_pool {
            let similar_rules = EmbeddingOperations::find_similar_rules(pool, dsl_expression, 10).await?;

            let suggestions = similar_rules
                .into_iter()
                .map(|rule| {
                    let confidence = 1.0 - rule.similarity.min(1.0);
                    AiSuggestion {
                        suggestion_type: SuggestionType::SimilarPattern,
                        title: format!("Similar: {}", rule.rule_name),
                        description: format!("Semantic similarity: {:.1}%", confidence * 100.0),
                        code_snippet: Some(rule.rule_definition),
                        confidence,
                        context_relevance: confidence,
                    }
                })
                .filter(|s| s.confidence > 0.2) // Only include reasonably similar patterns
                .collect();

            Ok(suggestions)
        } else {
            Err("No database connection available".to_string())
        }
    }

    /// Get intelligent code completions based on cursor position and partial input
    pub fn get_code_completions(&self, input: &str, cursor_pos: usize) -> Vec<AiSuggestion> {
        let mut completions = Vec::new();

        // Get current word being typed
        let (current_word, _word_start) = self.get_current_word(input, cursor_pos);

        if current_word.is_empty() {
            return self.get_contextual_suggestions(input, cursor_pos);
        }

        // Function completions
        completions.extend(self.get_function_completions(&current_word));

        // Attribute completions
        completions.extend(self.get_attribute_completions(&current_word));

        // Operator completions
        completions.extend(self.get_operator_completions(&current_word));

        // Keyword completions
        completions.extend(self.get_keyword_completions(&current_word));

        // Snippet completions
        completions.extend(self.get_snippet_completions(&current_word, input, cursor_pos));

        // Sort by relevance
        completions.sort_by(|a, b| {
            let score_a = a.confidence * a.context_relevance;
            let score_b = b.confidence * b.context_relevance;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to top 8 completions
        completions.truncate(8);
        completions
    }

    /// Extract current word being typed at cursor position
    fn get_current_word(&self, input: &str, cursor_pos: usize) -> (String, usize) {
        let chars: Vec<char> = input.chars().collect();

        if cursor_pos > chars.len() {
            return (String::new(), cursor_pos);
        }

        // Find word boundaries
        let mut start = cursor_pos;
        let mut end = cursor_pos;

        // Move backward to find word start
        while start > 0 && self.is_word_char(chars[start - 1]) {
            start -= 1;
        }

        // Move forward to find word end
        while end < chars.len() && self.is_word_char(chars[end]) {
            end += 1;
        }

        let word: String = chars[start..end].iter().collect();
        (word, start)
    }

    /// Check if character is part of a word (identifier)
    fn is_word_char(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_' || c == '.'
    }

    /// Get function name completions
    fn get_function_completions(&self, partial: &str) -> Vec<AiSuggestion> {
        let mut completions = Vec::new();
        let partial_lower = partial.to_lowercase();

        for func in &self.context.available_functions {
            if func.to_lowercase().starts_with(&partial_lower) {
                let (description, example) = self.get_function_info(func);

                completions.push(AiSuggestion {
                    suggestion_type: SuggestionType::AutoComplete,
                    title: func.clone(),
                    description: description.to_string(),
                    code_snippet: Some(example.to_string()),
                    confidence: self.calculate_match_confidence(partial, func),
                    context_relevance: 0.9,
                });
            }
        }

        completions
    }

    /// Get attribute name completions
    fn get_attribute_completions(&self, partial: &str) -> Vec<AiSuggestion> {
        let mut completions = Vec::new();
        let partial_lower = partial.to_lowercase();

        for attr in &self.context.available_attributes {
            if attr.to_lowercase().contains(&partial_lower) {
                completions.push(AiSuggestion {
                    suggestion_type: SuggestionType::AutoComplete,
                    title: attr.clone(),
                    description: format!("Reference attribute: {}", attr),
                    code_snippet: Some(attr.clone()),
                    confidence: self.calculate_match_confidence(partial, attr),
                    context_relevance: 0.85,
                });
            }
        }

        completions
    }

    /// Get operator completions
    fn get_operator_completions(&self, partial: &str) -> Vec<AiSuggestion> {
        let operators = vec![
            ("AND", "Logical AND operator", "condition1 AND condition2"),
            ("OR", "Logical OR operator", "condition1 OR condition2"),
            ("NOT", "Logical NOT operator", "NOT condition"),
            ("IF", "Conditional expression", "IF condition THEN value ELSE fallback"),
            ("THEN", "IF-THEN clause", "IF condition THEN value"),
            ("ELSE", "IF-ELSE clause", "IF condition THEN value ELSE fallback"),
        ];

        let mut completions = Vec::new();
        let partial_lower = partial.to_lowercase();

        for (op, desc, example) in operators {
            if op.to_lowercase().starts_with(&partial_lower) {
                completions.push(AiSuggestion {
                    suggestion_type: SuggestionType::AutoComplete,
                    title: op.to_string(),
                    description: desc.to_string(),
                    code_snippet: Some(example.to_string()),
                    confidence: self.calculate_match_confidence(partial, op),
                    context_relevance: 0.8,
                });
            }
        }

        completions
    }

    /// Get keyword completions
    fn get_keyword_completions(&self, partial: &str) -> Vec<AiSuggestion> {
        let keywords = vec![
            ("Client", "Client entity reference", "Client.attribute_name"),
            ("Product", "Product entity reference", "Product.attribute_name"),
            ("Account", "Account entity reference", "Account.attribute_name"),
            ("Transaction", "Transaction entity reference", "Transaction.attribute_name"),
            ("true", "Boolean true value", "true"),
            ("false", "Boolean false value", "false"),
            ("null", "Null value", "null"),
        ];

        let mut completions = Vec::new();
        let partial_lower = partial.to_lowercase();

        for (keyword, desc, example) in keywords {
            if keyword.to_lowercase().starts_with(&partial_lower) {
                completions.push(AiSuggestion {
                    suggestion_type: SuggestionType::AutoComplete,
                    title: keyword.to_string(),
                    description: desc.to_string(),
                    code_snippet: Some(example.to_string()),
                    confidence: self.calculate_match_confidence(partial, keyword),
                    context_relevance: 0.75,
                });
            }
        }

        completions
    }

    /// Get snippet completions for common patterns
    fn get_snippet_completions(&self, partial: &str, _input: &str, _cursor_pos: usize) -> Vec<AiSuggestion> {
        let snippets = vec![
            ("email", "Email validation pattern",
             "IS_EMAIL(Client.email) AND LENGTH(Client.email) > 5"),
            ("risk", "Risk calculation pattern",
             "IF condition THEN high_risk_score ELSE low_risk_score"),
            ("lookup", "Lookup table pattern",
             "LOOKUP(Client.country, \"countries\")"),
            ("concat", "String concatenation pattern",
             "CONCAT(first_name, \" \", last_name)"),
            ("validation", "Validation rule pattern",
             "field != null AND LENGTH(TRIM(field)) > 0"),
            ("range", "Numeric range check",
             "value >= min AND value <= max"),
            ("regex", "Regex pattern match",
             "field ~ /pattern/"),
        ];

        let mut completions = Vec::new();
        let partial_lower = partial.to_lowercase();

        for (trigger, desc, snippet) in snippets {
            if trigger.starts_with(&partial_lower) && partial.len() >= 2 {
                completions.push(AiSuggestion {
                    suggestion_type: SuggestionType::SnippetCompletion,
                    title: format!("Snippet: {}", trigger),
                    description: desc.to_string(),
                    code_snippet: Some(snippet.to_string()),
                    confidence: self.calculate_match_confidence(partial, trigger),
                    context_relevance: 0.95,
                });
            }
        }

        completions
    }

    /// Get contextual suggestions when no partial word
    fn get_contextual_suggestions(&self, input: &str, cursor_pos: usize) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();

        // Analyze context around cursor
        let before_cursor = &input[..cursor_pos.min(input.len())];

        // Suggest operators after expressions
        if self.is_after_expression(before_cursor) {
            suggestions.extend(self.get_operator_suggestions_contextual());
        }

        // Suggest functions at start or after operators
        if self.is_function_context(before_cursor) {
            suggestions.extend(self.get_function_suggestions_contextual());
        }

        // Suggest common patterns
        if before_cursor.trim().is_empty() {
            suggestions.extend(self.get_starter_suggestions());
        }

        suggestions
    }

    /// Check if cursor is after an expression (for operator suggestions)
    fn is_after_expression(&self, before_cursor: &str) -> bool {
        let trimmed = before_cursor.trim_end();
        trimmed.ends_with(')') ||
        trimmed.chars().last().map_or(false, |c| c.is_alphanumeric() || c == '_')
    }

    /// Check if cursor is in function context
    fn is_function_context(&self, before_cursor: &str) -> bool {
        let trimmed = before_cursor.trim_end();
        trimmed.is_empty() ||
        trimmed.ends_with('(') ||
        trimmed.ends_with(',') ||
        trimmed.ends_with("AND ") ||
        trimmed.ends_with("OR ") ||
        trimmed.ends_with("IF ")
    }

    /// Get contextual operator suggestions
    fn get_operator_suggestions_contextual(&self) -> Vec<AiSuggestion> {
        vec![
            AiSuggestion {
                suggestion_type: SuggestionType::AutoComplete,
                title: "AND".to_string(),
                description: "Logical AND - combine conditions".to_string(),
                code_snippet: Some(" AND ".to_string()),
                confidence: 0.9,
                context_relevance: 0.9,
            },
            AiSuggestion {
                suggestion_type: SuggestionType::AutoComplete,
                title: "OR".to_string(),
                description: "Logical OR - alternative conditions".to_string(),
                code_snippet: Some(" OR ".to_string()),
                confidence: 0.9,
                context_relevance: 0.9,
            },
        ]
    }

    /// Get contextual function suggestions
    fn get_function_suggestions_contextual(&self) -> Vec<AiSuggestion> {
        vec![
            AiSuggestion {
                suggestion_type: SuggestionType::AutoComplete,
                title: "IF".to_string(),
                description: "Conditional expression".to_string(),
                code_snippet: Some("IF condition THEN value ELSE fallback".to_string()),
                confidence: 0.95,
                context_relevance: 0.95,
            },
            AiSuggestion {
                suggestion_type: SuggestionType::AutoComplete,
                title: "CONCAT".to_string(),
                description: "String concatenation".to_string(),
                code_snippet: Some("CONCAT(string1, string2)".to_string()),
                confidence: 0.8,
                context_relevance: 0.8,
            },
        ]
    }

    /// Get starter suggestions for empty input
    fn get_starter_suggestions(&self) -> Vec<AiSuggestion> {
        vec![
            AiSuggestion {
                suggestion_type: SuggestionType::SnippetCompletion,
                title: "Email Validation".to_string(),
                description: "Complete email validation rule".to_string(),
                code_snippet: Some("IS_EMAIL(Client.email) AND LENGTH(Client.email) > 5".to_string()),
                confidence: 0.9,
                context_relevance: 0.9,
            },
            AiSuggestion {
                suggestion_type: SuggestionType::SnippetCompletion,
                title: "Risk Assessment".to_string(),
                description: "Risk calculation with conditions".to_string(),
                code_snippet: Some("IF Client.pep_status THEN 50 ELSE 10".to_string()),
                confidence: 0.9,
                context_relevance: 0.9,
            },
        ]
    }

    /// Calculate match confidence based on partial input
    fn calculate_match_confidence(&self, partial: &str, candidate: &str) -> f32 {
        if partial.is_empty() {
            return 0.5;
        }

        let partial_lower = partial.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.starts_with(&partial_lower) {
            // Perfect prefix match
            0.9 + (partial.len() as f32 / candidate.len() as f32) * 0.1
        } else if candidate_lower.contains(&partial_lower) {
            // Contains match
            0.6 + (partial.len() as f32 / candidate.len() as f32) * 0.2
        } else {
            // Fuzzy match
            self.fuzzy_match_score(&partial_lower, &candidate_lower)
        }
    }

    /// Simple fuzzy matching score
    fn fuzzy_match_score(&self, pattern: &str, candidate: &str) -> f32 {
        let mut score = 0.0;
        let mut pattern_chars = pattern.chars().peekable();

        for c in candidate.chars() {
            if let Some(&pc) = pattern_chars.peek() {
                if c == pc {
                    score += 1.0;
                    pattern_chars.next();
                }
            }
        }

        if pattern_chars.peek().is_none() {
            score / candidate.len() as f32 * 0.5
        } else {
            0.1
        }
    }

    /// Get function information for completions
    fn get_function_info(&self, func_name: &str) -> (&str, &str) {
        match func_name {
            "CONCAT" => ("Concatenate multiple strings", "CONCAT(\"Hello\", \" \", \"World\")"),
            "UPPER" => ("Convert string to uppercase", "UPPER(Client.name)"),
            "LOWER" => ("Convert string to lowercase", "LOWER(Client.email)"),
            "LENGTH" => ("Get string length", "LENGTH(Client.description)"),
            "TRIM" => ("Remove whitespace", "TRIM(Client.address)"),
            "SUBSTRING" => ("Extract substring", "SUBSTRING(Client.phone, 1, 3)"),
            "IS_EMAIL" => ("Validate email format", "IS_EMAIL(Client.email)"),
            "IS_LEI" => ("Validate LEI code", "IS_LEI(Client.lei_code)"),
            "IS_SWIFT" => ("Validate SWIFT code", "IS_SWIFT(Client.swift_code)"),
            "LOOKUP" => ("Lookup value from table", "LOOKUP(Client.country, \"countries\")"),
            "ROUND" => ("Round number", "ROUND(Client.balance, 2)"),
            "ABS" => ("Absolute value", "ABS(Client.net_position)"),
            _ => ("Function usage", "function call"),
        }
    }

    /// Analyze errors and provide intelligent explanations and fixes
    pub fn analyze_error(&self, error_message: &str, dsl_input: &str) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();

        // Analyze different types of errors
        suggestions.extend(self.analyze_syntax_errors(error_message, dsl_input));
        suggestions.extend(self.analyze_semantic_errors(error_message, dsl_input));
        suggestions.extend(self.analyze_function_errors(error_message, dsl_input));
        suggestions.extend(self.analyze_type_errors(error_message, dsl_input));
        suggestions.extend(self.analyze_logic_errors(error_message, dsl_input));

        // Add general error patterns if no specific matches found
        if suggestions.is_empty() {
            suggestions.extend(self.get_general_error_help(error_message, dsl_input));
        }

        // Sort by confidence and relevance
        suggestions.sort_by(|a, b| {
            let score_a = a.confidence * a.context_relevance;
            let score_b = b.confidence * b.context_relevance;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to top 5 error suggestions
        suggestions.truncate(5);
        suggestions
    }

    /// Analyze syntax errors (missing quotes, parentheses, etc.)
    fn analyze_syntax_errors(&self, error_message: &str, dsl_input: &str) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();
        let error_lower = error_message.to_lowercase();

        // Missing closing quote
        if error_lower.contains("unterminated") || error_lower.contains("unclosed") ||
           error_lower.contains("quote") || self.has_unmatched_quotes(dsl_input) {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::ErrorAnalysis,
                title: "Missing Closing Quote".to_string(),
                description: "String literals must be properly closed with matching quotes".to_string(),
                code_snippet: Some(self.fix_unmatched_quotes(dsl_input)),
                confidence: 0.9,
                context_relevance: 0.95,
            });
        }

        // Missing closing parenthesis
        if error_lower.contains("expected ')'") || error_lower.contains("parenthes") ||
           self.has_unmatched_parentheses(dsl_input) {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::QuickFix,
                title: "Missing Closing Parenthesis".to_string(),
                description: "Function calls and expressions must have balanced parentheses".to_string(),
                code_snippet: Some(self.fix_unmatched_parentheses(dsl_input)),
                confidence: 0.9,
                context_relevance: 0.95,
            });
        }

        // Missing semicolon or separator
        if error_lower.contains("expected ';'") || error_lower.contains("separator") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::QuickFix,
                title: "Missing Statement Separator".to_string(),
                description: "Multiple statements may need to be separated".to_string(),
                code_snippet: Some(format!("{};", dsl_input.trim())),
                confidence: 0.8,
                context_relevance: 0.8,
            });
        }

        // Invalid character
        if error_lower.contains("unexpected character") || error_lower.contains("invalid character") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::ErrorAnalysis,
                title: "Invalid Character".to_string(),
                description: "Check for special characters that aren't allowed in DSL expressions".to_string(),
                code_snippet: Some(self.clean_invalid_characters(dsl_input)),
                confidence: 0.7,
                context_relevance: 0.8,
            });
        }

        suggestions
    }

    /// Analyze semantic errors (undefined functions, wrong parameters)
    fn analyze_semantic_errors(&self, error_message: &str, dsl_input: &str) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();
        let error_lower = error_message.to_lowercase();

        // Undefined function
        if error_lower.contains("undefined") || error_lower.contains("unknown function") ||
           error_lower.contains("not found") {

            // Try to find the problematic function name
            if let Some(func_name) = self.extract_function_from_error(error_message) {
                let similar_functions = self.find_similar_functions(&func_name);

                for similar in similar_functions {
                    suggestions.push(AiSuggestion {
                        suggestion_type: SuggestionType::QuickFix,
                        title: format!("Did you mean '{}'?", similar),
                        description: format!("Replace '{}' with '{}'", func_name, similar),
                        code_snippet: Some(dsl_input.replace(&func_name, &similar)),
                        confidence: self.calculate_similarity_confidence(&func_name, &similar),
                        context_relevance: 0.9,
                    });
                }
            }
        }

        // Wrong number of arguments
        if error_lower.contains("wrong number") || error_lower.contains("expected") && error_lower.contains("argument") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::ErrorAnalysis,
                title: "Incorrect Function Arguments".to_string(),
                description: "Function called with wrong number of parameters".to_string(),
                code_snippet: Some(self.suggest_correct_function_call(dsl_input)),
                confidence: 0.85,
                context_relevance: 0.9,
            });
        }

        // Type mismatch
        if error_lower.contains("type") && (error_lower.contains("mismatch") || error_lower.contains("expected")) {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::ErrorAnalysis,
                title: "Type Mismatch".to_string(),
                description: "Value types don't match expected function parameter types".to_string(),
                code_snippet: Some(self.suggest_type_conversion(dsl_input)),
                confidence: 0.8,
                context_relevance: 0.85,
            });
        }

        suggestions
    }

    /// Analyze function-specific errors
    fn analyze_function_errors(&self, error_message: &str, dsl_input: &str) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();

        // CONCAT function errors
        if dsl_input.contains("CONCAT") {
            if error_message.contains("argument") || error_message.contains("parameter") {
                suggestions.push(AiSuggestion {
                    suggestion_type: SuggestionType::QuickFix,
                    title: "Fix CONCAT Usage".to_string(),
                    description: "CONCAT requires at least 2 string arguments".to_string(),
                    code_snippet: Some("CONCAT(\"first\", \"second\")".to_string()),
                    confidence: 0.9,
                    context_relevance: 0.9,
                });
            }
        }

        // IF-THEN-ELSE errors
        if dsl_input.contains("IF") && !dsl_input.contains("THEN") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::QuickFix,
                title: "Missing THEN Clause".to_string(),
                description: "IF statements require THEN and optionally ELSE".to_string(),
                code_snippet: Some(format!("{} THEN value ELSE fallback", dsl_input)),
                confidence: 0.95,
                context_relevance: 0.95,
            });
        }

        // Regex pattern errors
        if dsl_input.contains("~") || dsl_input.contains("/") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::ErrorAnalysis,
                title: "Regex Pattern Issue".to_string(),
                description: "Check regex pattern syntax and escaping".to_string(),
                code_snippet: Some("field ~ /^[a-zA-Z0-9]+$/".to_string()),
                confidence: 0.75,
                context_relevance: 0.8,
            });
        }

        suggestions
    }

    /// Analyze type-related errors
    fn analyze_type_errors(&self, error_message: &str, dsl_input: &str) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();
        let error_lower = error_message.to_lowercase();

        if error_lower.contains("string") && error_lower.contains("number") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::QuickFix,
                title: "String/Number Conversion".to_string(),
                description: "Use quotes for strings, remove quotes for numbers".to_string(),
                code_snippet: Some(self.suggest_string_number_fix(dsl_input)),
                confidence: 0.85,
                context_relevance: 0.9,
            });
        }

        if error_lower.contains("boolean") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::QuickFix,
                title: "Boolean Value Error".to_string(),
                description: "Use 'true' or 'false' for boolean values".to_string(),
                code_snippet: Some(self.suggest_boolean_fix(dsl_input)),
                confidence: 0.9,
                context_relevance: 0.9,
            });
        }

        suggestions
    }

    /// Analyze logical errors
    fn analyze_logic_errors(&self, _error_message: &str, dsl_input: &str) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();

        // Missing logical operators
        if self.has_adjacent_conditions(dsl_input) {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::QuickFix,
                title: "Missing Logical Operator".to_string(),
                description: "Multiple conditions need AND/OR operators".to_string(),
                code_snippet: Some(self.add_missing_operators(dsl_input)),
                confidence: 0.8,
                context_relevance: 0.85,
            });
        }

        // Operator precedence issues
        if dsl_input.contains("AND") && dsl_input.contains("OR") && !dsl_input.contains("(") {
            suggestions.push(AiSuggestion {
                suggestion_type: SuggestionType::ErrorAnalysis,
                title: "Operator Precedence".to_string(),
                description: "Use parentheses to clarify AND/OR precedence".to_string(),
                code_snippet: Some(self.add_precedence_parentheses(dsl_input)),
                confidence: 0.7,
                context_relevance: 0.8,
            });
        }

        suggestions
    }

    /// Provide general error help when specific patterns don't match
    fn get_general_error_help(&self, error_message: &str, _dsl_input: &str) -> Vec<AiSuggestion> {
        vec![
            AiSuggestion {
                suggestion_type: SuggestionType::ErrorAnalysis,
                title: "General Syntax Check".to_string(),
                description: format!("Error: {}. Check syntax, quotes, and function names.",
                    error_message.chars().take(50).collect::<String>()),
                code_snippet: None,
                confidence: 0.5,
                context_relevance: 0.6,
            },
            AiSuggestion {
                suggestion_type: SuggestionType::Alternative,
                title: "Try Simple Expression".to_string(),
                description: "Start with a basic expression to test syntax".to_string(),
                code_snippet: Some("Client.name != null".to_string()),
                confidence: 0.6,
                context_relevance: 0.7,
            },
        ]
    }

    // Helper methods for error analysis

    fn has_unmatched_quotes(&self, input: &str) -> bool {
        let mut in_single = false;
        let mut in_double = false;
        let mut escape_next = false;

        for c in input.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match c {
                '\\' => escape_next = true,
                '"' if !in_single => in_double = !in_double,
                '\'' if !in_double => in_single = !in_single,
                _ => {}
            }
        }

        in_single || in_double
    }

    fn has_unmatched_parentheses(&self, input: &str) -> bool {
        let mut count = 0;
        for c in input.chars() {
            match c {
                '(' => count += 1,
                ')' => count -= 1,
                _ => {}
            }
        }
        count != 0
    }

    fn fix_unmatched_quotes(&self, input: &str) -> String {
        if self.has_unmatched_quotes(input) {
            if input.chars().filter(|&c| c == '"').count() % 2 == 1 {
                format!("{}\"", input)
            } else if input.chars().filter(|&c| c == '\'').count() % 2 == 1 {
                format!("{}'", input)
            } else {
                input.to_string()
            }
        } else {
            input.to_string()
        }
    }

    fn fix_unmatched_parentheses(&self, input: &str) -> String {
        let mut count = 0;
        for c in input.chars() {
            match c {
                '(' => count += 1,
                ')' => count -= 1,
                _ => {}
            }
        }

        if count > 0 {
            format!("{}{}", input, ")".repeat(count as usize))
        } else if count < 0 {
            format!("{}{}", "(".repeat((-count) as usize), input)
        } else {
            input.to_string()
        }
    }

    fn clean_invalid_characters(&self, input: &str) -> String {
        input.chars()
            .filter(|&c| c.is_alphanumeric() || " ()\"'.,+-*/=<>!&|~".contains(c))
            .collect()
    }

    fn extract_function_from_error(&self, error_message: &str) -> Option<String> {
        // Simple extraction - look for function names in error messages
        let words: Vec<&str> = error_message.split_whitespace().collect();
        for word in words {
            if word.chars().all(|c| c.is_alphanumeric() || c == '_') &&
               word.chars().any(|c| c.is_uppercase()) {
                return Some(word.to_string());
            }
        }
        None
    }

    fn find_similar_functions(&self, func_name: &str) -> Vec<String> {
        let mut similar = Vec::new();
        let func_lower = func_name.to_lowercase();

        for available_func in &self.context.available_functions {
            let available_lower = available_func.to_lowercase();
            if self.strings_similar(&func_lower, &available_lower) {
                similar.push(available_func.clone());
            }
        }

        similar.sort_by(|a, b| {
            let score_a = self.fuzzy_match_score(&func_lower, &a.to_lowercase());
            let score_b = self.fuzzy_match_score(&func_lower, &b.to_lowercase());
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        similar.truncate(3);
        similar
    }

    fn strings_similar(&self, a: &str, b: &str) -> bool {
        if a == b { return true; }
        if a.is_empty() || b.is_empty() { return false; }

        // Check if one contains the other
        if a.contains(b) || b.contains(a) { return true; }

        // Check edit distance
        self.edit_distance(a, b) <= 2
    }

    fn edit_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let m = a_chars.len();
        let n = b_chars.len();

        if m == 0 { return n; }
        if n == 0 { return m; }

        let mut dp = vec![vec![0; n + 1]; m + 1];

        for i in 0..=m { dp[i][0] = i; }
        for j in 0..=n { dp[0][j] = j; }

        for i in 1..=m {
            for j in 1..=n {
                let cost = if a_chars[i-1] == b_chars[j-1] { 0 } else { 1 };
                dp[i][j] = (dp[i-1][j] + 1)
                    .min(dp[i][j-1] + 1)
                    .min(dp[i-1][j-1] + cost);
            }
        }

        dp[m][n]
    }

    fn calculate_similarity_confidence(&self, original: &str, similar: &str) -> f32 {
        let distance = self.edit_distance(&original.to_lowercase(), &similar.to_lowercase());
        let max_len = original.len().max(similar.len());

        if max_len == 0 { return 1.0; }

        1.0 - (distance as f32 / max_len as f32)
    }

    fn suggest_correct_function_call(&self, input: &str) -> String {
        // Try to fix common function call issues
        if input.contains("CONCAT(") && !input.contains(",") {
            "CONCAT(\"first\", \"second\")".to_string()
        } else if input.contains("IF(") {
            "IF condition THEN value ELSE fallback".to_string()
        } else {
            input.to_string()
        }
    }

    fn suggest_type_conversion(&self, input: &str) -> String {
        // Simple type conversion suggestions
        if input.contains("\"") && input.contains("+") {
            input.replace("\"", "")
        } else if input.chars().any(|c| c.is_numeric()) && !input.contains("\"") {
            format!("\"{}\"", input)
        } else {
            input.to_string()
        }
    }

    fn suggest_string_number_fix(&self, input: &str) -> String {
        // If it looks like a number but has quotes, remove them
        if input.starts_with("\"") && input.ends_with("\"") {
            let inner = &input[1..input.len()-1];
            if inner.parse::<f64>().is_ok() {
                return inner.to_string();
            }
        }
        // If it looks like a string but no quotes, add them
        if !input.contains("\"") && !input.chars().all(|c| c.is_numeric() || c == '.') {
            return format!("\"{}\"", input);
        }
        input.to_string()
    }

    fn suggest_boolean_fix(&self, input: &str) -> String {
        let lower = input.to_lowercase();
        if lower.contains("yes") || lower.contains("y") || lower.contains("1") {
            input.replace(&lower, "true")
        } else if lower.contains("no") || lower.contains("n") || lower.contains("0") {
            input.replace(&lower, "false")
        } else {
            input.to_string()
        }
    }

    fn has_adjacent_conditions(&self, input: &str) -> bool {
        // Simple check for conditions that might need operators
        input.contains("==") && input.matches("==").count() > 1 &&
        !input.contains("AND") && !input.contains("OR")
    }

    fn add_missing_operators(&self, input: &str) -> String {
        // Very basic - add AND between conditions
        if self.has_adjacent_conditions(input) {
            input.replace(" ", " AND ")
        } else {
            input.to_string()
        }
    }

    fn add_precedence_parentheses(&self, input: &str) -> String {
        // Add parentheses around OR conditions when AND is present
        if input.contains("AND") && input.contains("OR") {
            format!("({})", input)
        } else {
            input.to_string()
        }
    }

    /// RAG (Retrieval-Augmented Generation) functionality
    /// Get contextual help suggestions based on similar patterns in the database
    pub async fn get_rag_suggestions(&self, query: &str, limit: i32) -> Vec<AiSuggestion> {
        if let Some(db_pool) = &self.db_pool {
            match EmbeddingOperations::find_similar_rules(db_pool, query, limit).await {
                Ok(similar_rules) => {
                    let mut suggestions = Vec::new();

                    for rule in similar_rules {
                        suggestions.push(AiSuggestion {
                            suggestion_type: SuggestionType::SimilarPattern,
                            title: format!("Similar Pattern: {}", rule.rule_name),
                            description: format!(
                                "Found similar rule with {:.2}% similarity. Pattern: {}",
                                (1.0 - rule.similarity) * 100.0,
                                rule.rule_definition
                            ),
                            code_snippet: Some(rule.rule_definition),
                            confidence: 1.0 - rule.similarity,
                            context_relevance: 0.9,
                        });
                    }

                    suggestions
                }
                Err(e) => {
                    eprintln!("RAG query failed: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        }
    }


    /// Initialize AI assistant with database connection for RAG
    pub fn with_database(mut self, db_pool: DbPool) -> Self {
        self.db_pool = Some(db_pool);
        self
    }

    /// Load API keys from environment variables
    pub fn with_env_api_keys(mut self) -> Self {
        match &mut self.provider {
            AiProvider::OpenAI { api_key } => {
                if api_key.is_none() {
                    *api_key = std::env::var("OPENAI_API_KEY").ok();
                }
            }
            AiProvider::Anthropic { api_key } => {
                if api_key.is_none() {
                    *api_key = std::env::var("ANTHROPIC_API_KEY").ok();
                }
            }
            AiProvider::Offline => {
                // Check if any API key is available and switch provider
                if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
                    self.provider = AiProvider::OpenAI { api_key: Some(openai_key) };
                } else if let Ok(anthropic_key) = std::env::var("ANTHROPIC_API_KEY") {
                    self.provider = AiProvider::Anthropic { api_key: Some(anthropic_key) };
                }
            }
        }
        self
    }

    /// Check if API keys are available
    pub fn has_valid_api_key(&self) -> bool {
        match &self.provider {
            AiProvider::OpenAI { api_key } => api_key.is_some(),
            AiProvider::Anthropic { api_key } => api_key.is_some(),
            AiProvider::Offline => false,
        }
    }

    /// Get the current provider status
    pub fn get_provider_status(&self) -> String {
        match &self.provider {
            AiProvider::OpenAI { api_key } => {
                if api_key.is_some() {
                    "🔮 OpenAI (Connected)".to_string()
                } else {
                    "🔮 OpenAI (No API Key)".to_string()
                }
            }
            AiProvider::Anthropic { api_key } => {
                if api_key.is_some() {
                    "🧠 Anthropic (Connected)".to_string()
                } else {
                    "🧠 Anthropic (No API Key)".to_string()
                }
            }
            AiProvider::Offline => "🤖 Offline Mode".to_string(),
        }
    }

    /// Parse AI response text into structured suggestions
    fn parse_ai_response_to_suggestions(&self, response_text: &str) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();

        // Try to parse as JSON first
        if let Ok(json_suggestions) = self.parse_json_suggestions(response_text) {
            return json_suggestions;
        }

        // Fallback to text parsing for common patterns
        let lines: Vec<&str> = response_text.lines().collect();
        let mut current_suggestion: Option<AiSuggestion> = None;

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Look for suggestion patterns
            if line.starts_with("1.") || line.starts_with("-") || line.starts_with("*") {
                // Save previous suggestion
                if let Some(suggestion) = current_suggestion.take() {
                    suggestions.push(suggestion);
                }

                // Start new suggestion
                let title = line.trim_start_matches("1.")
                    .trim_start_matches("-")
                    .trim_start_matches("*")
                    .trim()
                    .to_string();

                current_suggestion = Some(AiSuggestion {
                    suggestion_type: SuggestionType::CodeCompletion,
                    title: if title.len() > 50 { title[..47].to_string() + "..." } else { title },
                    description: "AI-generated suggestion".to_string(),
                    code_snippet: None,
                    confidence: 0.8,
                    context_relevance: 0.7,
                });
            } else if line.starts_with("```") && line.len() > 3 {
                // Code block
                if let Some(ref mut suggestion) = current_suggestion {
                    // Extract code between backticks
                    if let Some(code_start) = response_text.find("```") {
                        if let Some(code_end) = response_text[code_start + 3..].find("```") {
                            let code = response_text[code_start + 3..code_start + 3 + code_end].trim();
                            suggestion.code_snippet = Some(code.to_string());
                        }
                    }
                }
            } else if let Some(ref mut suggestion) = current_suggestion {
                // Add to description
                if !suggestion.description.is_empty() && suggestion.description != "AI-generated suggestion" {
                    suggestion.description.push(' ');
                }
                suggestion.description.push_str(line);
            }
        }

        // Add final suggestion
        if let Some(suggestion) = current_suggestion {
            suggestions.push(suggestion);
        }

        // If no structured suggestions found, create a generic one
        if suggestions.is_empty() && !response_text.trim().is_empty() {
            // Look for code-like patterns in the response
            if response_text.contains("IF") || response_text.contains("CONCAT") || response_text.contains("==") {
                suggestions.push(AiSuggestion {
                    suggestion_type: SuggestionType::CodeCompletion,
                    title: "AI Suggestion".to_string(),
                    description: response_text.lines().next().unwrap_or("AI-generated code suggestion").to_string(),
                    code_snippet: Some(response_text.trim().to_string()),
                    confidence: 0.7,
                    context_relevance: 0.6,
                });
            } else {
                suggestions.push(AiSuggestion {
                    suggestion_type: SuggestionType::Documentation,
                    title: "AI Guidance".to_string(),
                    description: if response_text.len() > 100 {
                        response_text[..97].to_string() + "..."
                    } else {
                        response_text.to_string()
                    },
                    code_snippet: None,
                    confidence: 0.6,
                    context_relevance: 0.5,
                });
            }
        }

        suggestions
    }

    /// Try to parse JSON-formatted suggestions
    fn parse_json_suggestions(&self, text: &str) -> Result<Vec<AiSuggestion>, serde_json::Error> {
        // Look for JSON arrays or objects
        if let Some(start) = text.find('[') {
            if let Some(end) = text.rfind(']') {
                let json_text = &text[start..=end];
                return serde_json::from_str(json_text);
            }
        }

        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                let json_text = &text[start..=end];
                // Try as single suggestion
                let suggestion: AiSuggestion = serde_json::from_str(json_text)?;
                return Ok(vec![suggestion]);
            }
        }

        Err(serde_json::from_str::<AiSuggestion>("").unwrap_err())
    }

    /// Save API key securely to OS keychain
    pub fn save_api_key_to_keychain(&self, service: &str, api_key: &str) -> Result<(), String> {
        let entry = Entry::new("data-designer", service)
            .map_err(|e| format!("Failed to create keychain entry: {}", e))?;

        entry.set_password(api_key)
            .map_err(|e| format!("Failed to save API key to keychain: {}", e))
    }

    /// Load API key securely from OS keychain
    pub fn load_api_key_from_keychain(&self, service: &str) -> Result<String, String> {
        let entry = Entry::new("data-designer", service)
            .map_err(|e| format!("Failed to create keychain entry: {}", e))?;

        entry.get_password()
            .map_err(|e| format!("Failed to load API key from keychain: {}", e))
    }

    /// Delete API key from OS keychain
    pub fn delete_api_key_from_keychain(&self, service: &str) -> Result<(), String> {
        let entry = Entry::new("data-designer", service)
            .map_err(|e| format!("Failed to create keychain entry: {}", e))?;

        entry.delete_password()
            .map_err(|e| format!("Failed to delete API key from keychain: {}", e))
    }

    /// Load API keys from keychain on startup
    pub fn load_keys_from_keychain(mut self) -> Self {
        // Try to load OpenAI key
        if let Ok(openai_key) = self.load_api_key_from_keychain("openai") {
            if matches!(self.provider, AiProvider::Offline) {
                self.provider = AiProvider::OpenAI { api_key: Some(openai_key) };
            } else if let AiProvider::OpenAI { ref mut api_key } = self.provider {
                *api_key = Some(openai_key);
            }
        }

        // Try to load Anthropic key
        if let Ok(anthropic_key) = self.load_api_key_from_keychain("anthropic") {
            if matches!(self.provider, AiProvider::Offline) {
                self.provider = AiProvider::Anthropic { api_key: Some(anthropic_key) };
            } else if let AiProvider::Anthropic { ref mut api_key } = self.provider {
                *api_key = Some(anthropic_key);
            }
        }

        self
    }

    /// Set API key and save to keychain
    pub fn set_and_save_api_key(&mut self, provider_type: &str, api_key: String) -> Result<(), String> {
        // Save to keychain first
        self.save_api_key_to_keychain(provider_type, &api_key)?;

        // Update in-memory provider
        match provider_type {
            "openai" => {
                self.provider = AiProvider::OpenAI { api_key: Some(api_key) };
            }
            "anthropic" => {
                self.provider = AiProvider::Anthropic { api_key: Some(api_key) };
            }
            _ => return Err("Unknown provider type".to_string()),
        }

        Ok(())
    }
}