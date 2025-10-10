pub mod data_dictionary;
pub mod ai_agent;
pub mod grammar_loader;

use dashmap::DashMap;
use lazy_static::lazy_static;
use regex::Regex;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use data_designer::parser::parse_rule;
use crate::data_dictionary::DataDictionary;
use crate::ai_agent::{AIAgentManager, CompletionRequest, CompletionContext, ValidationRequest};
use crate::grammar_loader::GrammarLoader;
use tokio::sync::RwLock;

// DSL Keywords and functions based on EBNF
lazy_static! {
    static ref DSL_KEYWORDS: Vec<&'static str> = vec![
        "IF", "THEN", "ELSE", "AND", "OR", "NOT", "true", "false", "null"
    ];

    static ref DSL_FUNCTIONS: Vec<(&'static str, &'static str)> = vec![
        ("CONCAT", "Concatenates multiple string values"),
        ("SUBSTRING", "Extracts substring (string, start, end)"),
        ("LOOKUP", "Looks up value from a table (key, table_name)"),
        ("UPPER", "Converts string to uppercase"),
        ("LOWER", "Converts string to lowercase"),
        ("LENGTH", "Returns string length"),
        ("ROUND", "Rounds a number to specified decimals"),
        ("ABS", "Returns absolute value"),
        ("MAX", "Returns maximum of values"),
        ("MIN", "Returns minimum of values"),
        // Regex validation functions for KYC
        ("IS_EMAIL", "Validates email format: IS_EMAIL(email)"),
        ("IS_LEI", "Validates Legal Entity Identifier: IS_LEI(lei)"),
        ("IS_SWIFT", "Validates SWIFT/BIC code: IS_SWIFT(code)"),
        ("IS_PHONE", "Validates phone number: IS_PHONE(number)"),
        ("VALIDATE", "Generic pattern validation: VALIDATE(value, pattern)"),
        ("EXTRACT", "Extract pattern matches: EXTRACT(value, pattern)"),
        ("MATCHES", "Pattern matching function: MATCHES(text, pattern)"),
    ];

    static ref DSL_OPERATORS: Vec<(&'static str, &'static str)> = vec![
        ("+", "Addition or string concatenation"),
        ("-", "Subtraction"),
        ("*", "Multiplication"),
        ("/", "Division"),
        ("%", "Modulo"),
        ("&", "String concatenation"),
        ("==", "Equality comparison"),
        ("!=", "Inequality comparison"),
        ("<", "Less than"),
        (">", "Greater than"),
        ("<=", "Less than or equal"),
        (">=", "Greater than or equal"),
        ("=", "Assignment"),
        ("MATCHES", "Regex pattern matching: text MATCHES /pattern/"),
        ("~", "Regex match shorthand: text ~ /pattern/"),
    ];

    static ref IDENTIFIER_PATTERN: Regex = Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap();
    static ref NUMBER_PATTERN: Regex = Regex::new(r"\b\d+(\.\d+)?\b").unwrap();
    static ref STRING_PATTERN: Regex = Regex::new(r#"["'][^"']*["']"#).unwrap();
    static ref REGEX_PATTERN: Regex = Regex::new(r#"/[^/]+/|r"[^"]+"#).unwrap();
}

#[derive(Debug)]
pub struct Backend {
    client: Client,
    document_map: Arc<DashMap<Url, Rope>>,
    semantic_tokens: Arc<DashMap<Url, Vec<DslSemanticToken>>>,
    data_dictionary: Arc<RwLock<DataDictionary>>,
    ai_agent_manager: Arc<RwLock<AIAgentManager>>,
    grammar_loader: Arc<GrammarLoader>,
}

#[derive(Debug, Clone)]
struct DslSemanticToken {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        // Initialize with default KYC data dictionary
        let data_dictionary = DataDictionary::create_default_kyc_dictionary();

        // Initialize AI agent manager
        let ai_agent_manager = AIAgentManager::new();

        // Initialize grammar loader
        let grammar_loader = Arc::new(GrammarLoader::default());

        Backend {
            client,
            document_map: Arc::new(DashMap::new()),
            semantic_tokens: Arc::new(DashMap::new()),
            data_dictionary: Arc::new(RwLock::new(data_dictionary)),
            ai_agent_manager: Arc::new(RwLock::new(ai_agent_manager)),
            grammar_loader,
        }
    }

    pub async fn load_data_dictionary(&self, path: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let dictionary = DataDictionary::load_from_directory(path)?;
        let mut dict_guard = self.data_dictionary.write().await;
        *dict_guard = dictionary;

        self.client
            .log_message(MessageType::INFO, format!("Loaded data dictionary from {}", path))
            .await;

        Ok(())
    }

    pub async fn set_ai_agent(&self, agent_type: &str, config: Option<String>) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut manager = self.ai_agent_manager.write().await;

        match agent_type {
            "gemini" => {
                if let Some(api_key) = config {
                    let agent = ai_agent::GeminiAgent::new(api_key);
                    manager.add_agent("gemini".to_string(), Box::new(agent));
                    manager.set_active_agent("gemini".to_string())?;
                } else {
                    return Err("Gemini API key required".into());
                }
            },
            "copilot" => {
                let agent = ai_agent::CopilotAgent::new();
                manager.add_agent("copilot".to_string(), Box::new(agent));
                manager.set_active_agent("copilot".to_string())?;
            },
            "mock" | _ => {
                manager.set_active_agent("mock".to_string())?;
            }
        }

        self.client
            .log_message(MessageType::INFO, format!("AI agent set to: {}", agent_type))
            .await;

        Ok(())
    }

    async fn on_change(&self, params: TextDocumentItem) {
        let rope = Rope::from_str(&params.text);
        self.document_map.insert(params.uri.clone(), rope);

        // Perform diagnostics with nom parser
        self.validate_document(params.uri, params.text).await;
    }

    async fn validate_document(&self, uri: Url, text: String) {
        let mut diagnostics = Vec::new();

        // Parse line by line for better error reporting
        for (line_num, line) in text.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }

            match parse_rule(line) {
                Ok((remaining, _ast)) => {
                    if !remaining.trim().is_empty() && !remaining.trim().starts_with('#') {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: line_num as u32,
                                    character: (line.len() - remaining.len()) as u32,
                                },
                                end: Position {
                                    line: line_num as u32,
                                    character: line.len() as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: Some(NumberOrString::String("unparsed".to_string())),
                            source: Some("dsl-lsp".to_string()),
                            message: format!("Unparsed content: '{}'", remaining.trim()),
                            ..Default::default()
                        });
                    }
                }
                Err(e) => {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: 0,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: line.len() as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("parse_error".to_string())),
                        source: Some("dsl-lsp".to_string()),
                        message: format!("Parse error: {:?}", e),
                        ..Default::default()
                    });
                }
            }
        }

        // AI-based validation if available
        let ai_manager = self.ai_agent_manager.read().await;
        if let Some(_agent) = ai_manager.get_active_agent() {
            let validation_request = ValidationRequest {
                rule: text.clone(),
                context: std::collections::HashMap::new(),
            };

            if let Ok(validation_response) = ai_manager.validate_rule(validation_request).await {
                for issue in validation_response.issues {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: issue.line.unwrap_or(0) as u32,
                                character: issue.column.unwrap_or(0) as u32,
                            },
                            end: Position {
                                line: issue.line.unwrap_or(0) as u32,
                                character: issue.column.unwrap_or(100) as u32,
                            },
                        },
                        severity: Some(match issue.severity {
                            ai_agent::IssueSeverity::Error => DiagnosticSeverity::ERROR,
                            ai_agent::IssueSeverity::Warning => DiagnosticSeverity::WARNING,
                            ai_agent::IssueSeverity::Info => DiagnosticSeverity::INFORMATION,
                        }),
                        code: Some(NumberOrString::String("ai_validation".to_string())),
                        source: Some("ai-agent".to_string()),
                        message: issue.message,
                        ..Default::default()
                    });
                }
            }
        }

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn get_completions(&self, line: &str, character: usize) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Get the current word being typed
        let before = &line[..character.min(line.len())];
        let word_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.').map(|i| i + 1).unwrap_or(0);
        let current_word = &before[word_start..];

        // Add keyword completions
        for keyword in DSL_KEYWORDS.iter() {
            if keyword.to_lowercase().starts_with(&current_word.to_lowercase()) {
                completions.push(CompletionItem {
                    label: keyword.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some("Keyword".to_string()),
                    insert_text: Some(keyword.to_string()),
                    ..Default::default()
                });
            }
        }

        // Add function completions
        for (func, desc) in DSL_FUNCTIONS.iter() {
            if func.to_lowercase().starts_with(&current_word.to_lowercase()) {
                completions.push(CompletionItem {
                    label: func.to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some(desc.to_string()),
                    insert_text: Some(format!("{}($1)", func)),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                });
            }
        }

        // Add operator completions if appropriate
        if current_word.is_empty() || "+-*/%&=<>!~".contains(current_word.chars().next().unwrap_or(' ')) {
            for (op, desc) in DSL_OPERATORS.iter() {
                if op.starts_with(current_word) {
                    completions.push(CompletionItem {
                        label: op.to_string(),
                        kind: Some(CompletionItemKind::OPERATOR),
                        detail: Some(desc.to_string()),
                        insert_text: Some(format!("{} ", op)),
                        ..Default::default()
                    });
                }
            }
        }

        // Add data dictionary completions
        let dict = self.data_dictionary.blocking_read();

        // Add entity attributes
        for (full_name, attr) in dict.get_all_attributes() {
            if full_name.to_lowercase().contains(&current_word.to_lowercase()) {
                // Build a concise detail string with type info
                let mut detail = String::new();
                if let Some(sql_type) = &attr.sql_type {
                    detail.push_str(&sql_type);
                } else {
                    detail.push_str(&format!("{:?}", attr.data_type));
                }
                if let Some(domain) = &attr.domain {
                    detail.push_str(&format!(" [{}]", domain));
                }

                // Build comprehensive documentation
                let mut doc = format!("### {}\n\n{}\n\n", full_name, attr.description);

                // Add type information
                doc.push_str("**Type Information:**\n");
                if let Some(sql_type) = &attr.sql_type {
                    doc.push_str(&format!("- SQL: `{}`\n", sql_type));
                }
                if let Some(rust_type) = &attr.rust_type {
                    doc.push_str(&format!("- Rust: `{}`\n", rust_type));
                }

                // Add format mask if present
                if let Some(format_mask) = &attr.format_mask {
                    doc.push_str(&format!("\n**Format:** `{}`\n", format_mask));
                }

                // Add validation pattern if present
                if let Some(pattern) = &attr.pattern {
                    doc.push_str(&format!("\n**Pattern:** `{}`\n", pattern));
                }

                // Add constraints
                if attr.required {
                    doc.push_str("\n**Required:** ✅\n");
                }

                // Add examples
                if !attr.examples.is_empty() {
                    doc.push_str(&format!("\n**Examples:** {}", attr.examples.join(", ")));
                }

                completions.push(CompletionItem {
                    label: full_name.clone(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some(detail),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: doc,
                    })),
                    insert_text: Some(full_name),
                    ..Default::default()
                });
            }
        }

        // Add domain value completions
        if current_word.contains('.') {
            let parts: Vec<&str> = current_word.split('.').collect();
            if parts.len() == 2 {
                if let Some(attr) = dict.get_attribute_info(parts[0]) {
                    if let Some(domain_name) = &attr.domain {
                        for value in dict.get_domain_values(domain_name) {
                            if value.to_lowercase().starts_with(&parts[1].to_lowercase()) {
                                completions.push(CompletionItem {
                                    label: format!("\"{}\"", value),
                                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                                    detail: Some(format!("Domain value for {}", domain_name)),
                                    insert_text: Some(format!("\"{}\"", value)),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }
        }

        // Add lookup table completions
        if line.contains("LOOKUP") && line.contains("\"") {
            for (table_name, table) in &dict.lookups {
                if table_name.to_lowercase().contains(&current_word.to_lowercase()) {
                    completions.push(CompletionItem {
                        label: format!("\"{}\"", table_name),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some(table.description.clone()),
                        insert_text: Some(format!("\"{}\"", table_name)),
                        ..Default::default()
                    });
                }
            }
        }

        // Get AI-powered completions
        let ai_manager = self.ai_agent_manager.blocking_read();
        if let Some(_agent) = ai_manager.get_active_agent() {
            let context = CompletionContext {
                current_line: line.to_string(),
                preceding_lines: vec![],
                following_lines: vec![],
                cursor_position: character,
                file_path: None,
                available_attributes: dict.get_all_attributes().iter()
                    .map(|(name, _)| name.clone())
                    .collect(),
                available_functions: {
                    let grammar_loader = self.grammar_loader.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async move {
                            grammar_loader.get_functions().await.iter()
                                .map(|(name, _)| name.clone())
                                .collect()
                        })
                    })
                },
                data_dictionary_context: None,
            };

            let request = CompletionRequest {
                prompt: format!("Complete DSL at position {} in line: {}", character, line),
                context,
                max_tokens: Some(50),
                temperature: Some(0.3),
            };

            // Use blocking task for async operation
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                if let Ok(ai_response) = handle.block_on(ai_manager.get_completions(request)) {
                    for suggestion in ai_response.suggestions {
                        completions.push(CompletionItem {
                            label: format!("🤖 {}", suggestion.text),
                            kind: Some(CompletionItemKind::TEXT),
                            detail: suggestion.description,
                            insert_text: Some(suggestion.text),
                            sort_text: Some(format!("zzz{}", suggestion.confidence)), // Sort AI suggestions last
                            ..Default::default()
                        });
                    }
                }
            }
        }

        completions
    }

    fn get_hover_info(&self, line: &str, character: usize) -> Option<Hover> {
        // Find the word at the cursor position
        let start = line[..character.min(line.len())]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .map(|i| i + 1)
            .unwrap_or(0);

        let end = line[character..]
            .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .map(|i| i + character)
            .unwrap_or(line.len());

        let word = &line[start..end];

        // Check if it's an attribute from the data dictionary
        let dict = self.data_dictionary.blocking_read();
        if let Some(attr_info) = dict.get_attribute_info(word) {
            let mut hover_content = format!("### 📊 Attribute: `{}`\n\n", word);
            hover_content.push_str(&format!("**Description:** {}\n\n", attr_info.description));

            // Type information
            hover_content.push_str("#### Type Information\n");
            hover_content.push_str(&format!("- **DSL Type:** `{:?}`\n", attr_info.data_type));
            if let Some(sql_type) = &attr_info.sql_type {
                hover_content.push_str(&format!("- **SQL Type:** `{}`\n", sql_type));
            }
            if let Some(rust_type) = &attr_info.rust_type {
                hover_content.push_str(&format!("- **Rust Type:** `{}`\n", rust_type));
            }

            // Format and validation
            if let Some(format_mask) = &attr_info.format_mask {
                hover_content.push_str(&format!("\n#### Format\n- **Mask:** `{}`\n", format_mask));
            }
            if let Some(pattern) = &attr_info.pattern {
                hover_content.push_str(&format!("- **Pattern:** `{}`\n", pattern));
            }

            // Constraints
            hover_content.push_str("\n#### Constraints\n");
            hover_content.push_str(&format!("- **Required:** {}\n", if attr_info.required { "✅ Yes" } else { "❌ No" }));
            if let Some(min_length) = attr_info.min_length {
                hover_content.push_str(&format!("- **Min Length:** {}\n", min_length));
            }
            if let Some(max_length) = attr_info.max_length {
                hover_content.push_str(&format!("- **Max Length:** {}\n", max_length));
            }
            if let Some(min_value) = &attr_info.min_value {
                hover_content.push_str(&format!("- **Min Value:** {}\n", min_value));
            }
            if let Some(max_value) = &attr_info.max_value {
                hover_content.push_str(&format!("- **Max Value:** {}\n", max_value));
            }

            // Domain values
            if let Some(domain) = &attr_info.domain {
                hover_content.push_str(&format!("\n#### Domain: `{}`\n", domain));
                let domain_values = dict.get_domain_values(domain);
                if !domain_values.is_empty() {
                    hover_content.push_str("Valid values:\n");
                    for value in domain_values.iter().take(5) {
                        hover_content.push_str(&format!("- `{}`\n", value));
                    }
                    if domain_values.len() > 5 {
                        hover_content.push_str(&format!("- ... and {} more\n", domain_values.len() - 5));
                    }
                }
            }

            // Validation rules
            if !attr_info.validation_rules.is_empty() {
                hover_content.push_str("\n#### Validation Rules\n");
                for rule in &attr_info.validation_rules {
                    hover_content.push_str(&format!("- `{}`\n", rule));
                }
            }

            // Examples
            if !attr_info.examples.is_empty() {
                hover_content.push_str("\n#### Examples\n");
                for example in attr_info.examples.iter().take(3) {
                    hover_content.push_str(&format!("- `{}`\n", example));
                }
            }

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_content,
                }),
                range: None,
            });
        }

        // Check if it's a function
        for (func, desc) in DSL_FUNCTIONS.iter() {
            if word == *func {
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("**Function: {}**\n\n{}", func, desc),
                    }),
                    range: None,
                });
            }
        }

        // Check if it's a keyword
        if DSL_KEYWORDS.contains(&word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**Keyword: {}**", word),
                }),
                range: None,
            });
        }

        // Check if it's an operator
        for (op, desc) in DSL_OPERATORS.iter() {
            if line[start..].starts_with(op) {
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("**Operator: {}**\n\n{}", op, desc),
                    }),
                    range: None,
                });
            }
        }

        None
    }

    async fn get_semantic_tokens(&self, uri: &Url) -> Option<Vec<DslSemanticToken>> {
        if let Some(rope) = self.document_map.get(uri) {
            let mut tokens = Vec::new();

            for (line_num, line) in rope.lines().enumerate() {
                let line_str = line.to_string();

                // Tokenize keywords
                for keyword in DSL_KEYWORDS.iter() {
                    let mut start = 0;
                    while let Some(pos) = line_str[start..].find(keyword) {
                        let abs_pos = start + pos;
                        // Check it's a whole word
                        let before_ok = abs_pos == 0 || !line_str.chars().nth(abs_pos - 1).unwrap().is_alphanumeric();
                        let after_ok = abs_pos + keyword.len() >= line_str.len() ||
                            !line_str.chars().nth(abs_pos + keyword.len()).unwrap().is_alphanumeric();

                        if before_ok && after_ok {
                            tokens.push(DslSemanticToken {
                                line: line_num as u32,
                                start: abs_pos as u32,
                                length: keyword.len() as u32,
                                token_type: 0, // KEYWORD
                            });
                        }
                        start = abs_pos + keyword.len();
                    }
                }

                // Tokenize functions
                for (func, _) in DSL_FUNCTIONS.iter() {
                    if let Some(pos) = line_str.find(func) {
                        tokens.push(DslSemanticToken {
                            line: line_num as u32,
                            start: pos as u32,
                            length: func.len() as u32,
                            token_type: 5, // FUNCTION
                        });
                    }
                }

                // Tokenize strings
                for mat in STRING_PATTERN.find_iter(&line_str) {
                    tokens.push(DslSemanticToken {
                        line: line_num as u32,
                        start: mat.start() as u32,
                        length: mat.len() as u32,
                        token_type: 2, // STRING
                    });
                }

                // Tokenize numbers
                for mat in NUMBER_PATTERN.find_iter(&line_str) {
                    tokens.push(DslSemanticToken {
                        line: line_num as u32,
                        start: mat.start() as u32,
                        length: mat.len() as u32,
                        token_type: 3, // NUMBER
                    });
                }

                // Tokenize regex patterns
                for mat in REGEX_PATTERN.find_iter(&line_str) {
                    tokens.push(DslSemanticToken {
                        line: line_num as u32,
                        start: mat.start() as u32,
                        length: mat.len() as u32,
                        token_type: 2, // STRING (for regex patterns)
                    });
                }

                // Tokenize comments
                if let Some(comment_pos) = line_str.find('#') {
                    tokens.push(DslSemanticToken {
                        line: line_num as u32,
                        start: comment_pos as u32,
                        length: (line_str.len() - comment_pos) as u32,
                        token_type: 6, // COMMENT
                    });
                }
            }

            return Some(tokens);
        }
        None
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        "(".to_string(),
                        " ".to_string(),
                        "\"".to_string(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: None,
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: Default::default(),
                    },
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::OPERATOR,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::COMMENT,
                                ],
                                token_modifiers: vec![],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            ..Default::default()
                        },
                    ),
                ),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "dsl.explainRule".to_string(),
                        "dsl.optimizeRule".to_string(),
                        "dsl.generateTests".to_string(),
                        "dsl.loadDataDictionary".to_string(),
                        "dsl.setAIAgent".to_string(),
                        "dsl.reloadGrammar".to_string(),
                    ],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        // Load grammar on startup
        if let Err(e) = self.grammar_loader.load_grammar().await {
            self.client
                .log_message(MessageType::ERROR, format!("Failed to load grammar: {}", e))
                .await;
        }

        self.client
            .log_message(MessageType::INFO, "DSL Language Server initialized with AI support and dynamic grammar!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            language_id: params.text_document.language_id,
            version: params.text_document.version,
            text: params.text_document.text,
        })
        .await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            self.on_change(TextDocumentItem {
                uri: params.text_document.uri,
                language_id: "dsl".to_string(),
                version: params.text_document.version,
                text: change.text,
            })
            .await
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;

        if let Some(rope) = self.document_map.get(&uri) {
            let line = params.text_document_position.position.line as usize;
            let character = params.text_document_position.position.character as usize;

            if let Some(line_str) = rope.get_line(line) {
                let line_text = line_str.to_string();
                let completions = self.get_completions(&line_text, character).await;
                return Ok(Some(CompletionResponse::Array(completions)));
            }
        }

        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;

        if let Some(rope) = self.document_map.get(&uri) {
            let line = params.text_document_position_params.position.line as usize;
            let character = params.text_document_position_params.position.character as usize;

            if let Some(line_str) = rope.get_line(line) {
                let line_text = line_str.to_string();
                return Ok(self.get_hover_info(&line_text, character));
            }
        }

        Ok(None)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;

        if let Some(tokens) = self.get_semantic_tokens(&uri).await {
            let mut data = Vec::new();
            let mut prev_line = 0;
            let mut prev_start = 0;

            for token in tokens {
                let delta_line = token.line - prev_line;
                let delta_start = if delta_line == 0 {
                    token.start - prev_start
                } else {
                    token.start
                };

                data.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length: token.length,
                    token_type: token.token_type,
                    token_modifiers_bitset: 0,
                });

                prev_line = token.line;
                prev_start = token.start;
            }

            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data,
            })));
        }

        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let mut actions = Vec::new();

        // Add explain rule action
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Explain Rule".to_string(),
            kind: Some(CodeActionKind::REFACTOR_EXTRACT),
            command: Some(Command {
                title: "Explain Rule".to_string(),
                command: "dsl.explainRule".to_string(),
                arguments: Some(vec![serde_json::to_value(&params.text_document.uri).unwrap()]),
            }),
            ..Default::default()
        }));

        // Add optimize rule action
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Optimize Rule".to_string(),
            kind: Some(CodeActionKind::REFACTOR_REWRITE),
            command: Some(Command {
                title: "Optimize Rule".to_string(),
                command: "dsl.optimizeRule".to_string(),
                arguments: Some(vec![serde_json::to_value(&params.text_document.uri).unwrap()]),
            }),
            ..Default::default()
        }));

        // Add generate tests action
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Generate Test Cases".to_string(),
            kind: Some(CodeActionKind::SOURCE_ORGANIZE_IMPORTS),
            command: Some(Command {
                title: "Generate Test Cases".to_string(),
                command: "dsl.generateTests".to_string(),
                arguments: Some(vec![serde_json::to_value(&params.text_document.uri).unwrap()]),
            }),
            ..Default::default()
        }));

        Ok(Some(actions))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<serde_json::Value>> {
        match params.command.as_str() {
            "dsl.explainRule" => {
                if let Some(uri) = params.arguments.get(0).and_then(|v| v.as_str()) {
                    let uri = Url::parse(uri).unwrap();
                    if let Some(rope) = self.document_map.get(&uri) {
                        let content = rope.to_string();
                        let ai_manager = self.ai_agent_manager.read().await;
                        if let Some(agent) = ai_manager.get_active_agent() {
                            if let Ok(explanation) = agent.explain_rule(&content).await {
                                self.client
                                    .show_message(MessageType::INFO, explanation)
                                    .await;
                            }
                        }
                    }
                }
            },
            "dsl.optimizeRule" => {
                if let Some(uri) = params.arguments.get(0).and_then(|v| v.as_str()) {
                    let uri = Url::parse(uri).unwrap();
                    if let Some(rope) = self.document_map.get(&uri) {
                        let content = rope.to_string();
                        let ai_manager = self.ai_agent_manager.read().await;
                        if let Some(agent) = ai_manager.get_active_agent() {
                            if let Ok(optimized) = agent.optimize_rule(&content).await {
                                self.client
                                    .show_message(MessageType::INFO, format!("Optimized:\n{}", optimized))
                                    .await;
                            }
                        }
                    }
                }
            },
            "dsl.generateTests" => {
                if let Some(uri) = params.arguments.get(0).and_then(|v| v.as_str()) {
                    let uri = Url::parse(uri).unwrap();
                    if let Some(rope) = self.document_map.get(&uri) {
                        let content = rope.to_string();
                        let ai_manager = self.ai_agent_manager.read().await;
                        if let Some(agent) = ai_manager.get_active_agent() {
                            if let Ok(tests) = agent.generate_test_cases(&content).await {
                                let test_json = serde_json::to_string_pretty(&tests).unwrap();
                                self.client
                                    .show_message(MessageType::INFO, format!("Test cases:\n{}", test_json))
                                    .await;
                            }
                        }
                    }
                }
            },
            "dsl.loadDataDictionary" => {
                if let Some(path) = params.arguments.get(0).and_then(|v| v.as_str()) {
                    if let Err(e) = self.load_data_dictionary(path).await {
                        self.client
                            .show_message(MessageType::ERROR, format!("Failed to load data dictionary: {}", e))
                            .await;
                    }
                }
            },
            "dsl.setAIAgent" => {
                if let Some(agent_type) = params.arguments.get(0).and_then(|v| v.as_str()) {
                    let config = params.arguments.get(1).and_then(|v| v.as_str()).map(String::from);
                    if let Err(e) = self.set_ai_agent(agent_type, config).await {
                        self.client
                            .show_message(MessageType::ERROR, format!("Failed to set AI agent: {}", e))
                            .await;
                    }
                }
            },
            "dsl.reloadGrammar" => {
                if let Err(e) = self.grammar_loader.reload_if_changed().await {
                    self.client
                        .show_message(MessageType::ERROR, format!("Failed to reload grammar: {}", e))
                        .await;
                } else {
                    self.client
                        .show_message(MessageType::INFO, "Grammar reloaded successfully!")
                        .await;
                }
            },
            _ => {}
        }

        Ok(None)
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}