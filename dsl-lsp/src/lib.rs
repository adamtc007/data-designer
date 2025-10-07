use dashmap::DashMap;
use lazy_static::lazy_static;
use regex::Regex;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use data_designer::{BusinessRule, parse_rule};

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
}

#[derive(Debug)]
pub struct Backend {
    client: Client,
    document_map: Arc<DashMap<Url, Rope>>,
    semantic_tokens: Arc<DashMap<Url, Vec<SemanticToken>>>,
}

#[derive(Debug, Clone)]
struct SemanticToken {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend {
            client,
            document_map: Arc::new(DashMap::new()),
            semantic_tokens: Arc::new(DashMap::new()),
        }
    }

    async fn on_change(&self, params: TextDocumentItem) {
        let rope = Rope::from_str(&params.text);
        self.document_map.insert(params.uri.clone(), rope);

        // Perform diagnostics
        self.validate_document(params.uri, params.text).await;
    }

    async fn validate_document(&self, uri: Url, text: String) {
        let mut diagnostics = Vec::new();

        // Parse the DSL using the nom parser
        for (line_num, line) in text.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }

            // Try to parse each line/statement
            match parse_rule(line) {
                Ok((remaining, _ast)) => {
                    // Check for unparsed content
                    if !remaining.trim().is_empty() {
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
                            message: format!("Unparsed content: '{}'", remaining),
                            ..Default::default()
                        });
                    }
                }
                Err(_) => {
                    // Parse error
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
                        message: "Failed to parse DSL expression".to_string(),
                        ..Default::default()
                    });
                }
            }
        }

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    fn get_completions(&self, line: &str, character: usize) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Get the current word being typed
        let before = &line[..character.min(line.len())];
        let word_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_').map(|i| i + 1).unwrap_or(0);
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
        if current_word.is_empty() || "+-*/%&=<>!".contains(current_word.chars().next().unwrap_or(' ')) {
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

        // Add common attributes from KYC domain
        let kyc_attributes = vec![
            ("client_id", "Client identifier"),
            ("legal_entity_name", "Legal entity name"),
            ("risk_rating", "Risk rating level"),
            ("aum_usd", "Assets under management in USD"),
            ("kyc_completeness", "KYC completion percentage"),
            ("documents_received", "Number of documents received"),
            ("documents_required", "Number of documents required"),
            ("aml_risk_score", "AML risk score"),
            ("pep_status", "PEP status flag"),
            ("sanctions_check", "Sanctions check result"),
        ];

        for (attr, desc) in kyc_attributes {
            if attr.starts_with(current_word) {
                completions.push(CompletionItem {
                    label: attr.to_string(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    detail: Some(desc.to_string()),
                    insert_text: Some(attr.to_string()),
                    ..Default::default()
                });
            }
        }

        completions
    }

    fn get_hover_info(&self, line: &str, character: usize) -> Option<Hover> {
        // Find the word at the cursor position
        let start = line[..character.min(line.len())]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        let end = line[character..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + character)
            .unwrap_or(line.len());

        let word = &line[start..end];

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
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "DSL Language Server initialized!")
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
                let completions = self.get_completions(&line_text, character);
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

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}