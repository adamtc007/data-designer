//! CBU DSL Language Server Protocol Implementation
//! Provides IDE features for the CBU DSL including syntax highlighting,
//! code completion, error diagnostics, and validation.

use std::collections::HashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::{info, warn, error};

// Import CBU DSL components
use data_designer_core::lisp_cbu_dsl::{LispCbuParser, LispValue, LispDslError};
use data_designer_core::cbu_dsl::CbuDslParser;
use data_designer_core::parser::parse_expression;

pub struct CbuDslLanguageServer {
    client: Client,
    document_map: tokio::sync::RwLock<HashMap<Url, String>>,
    lisp_parser: tokio::sync::RwLock<LispCbuParser>,
}

impl CbuDslLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: tokio::sync::RwLock::new(HashMap::new()),
            lisp_parser: tokio::sync::RwLock::new(LispCbuParser::new(None)),
        }
    }

    /// Validate CBU DSL content and return diagnostics
    async fn validate_document(&self, uri: &Url, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Try parsing as S-expression first
        let mut parser = self.lisp_parser.write().await;
        match parser.parse_and_eval(text) {
            Ok(result) => {
                if !result.success {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: text.len() as u32 },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("CBU_DSL_ERROR".to_string())),
                        code_description: None,
                        source: Some("cbu-dsl-lsp".to_string()),
                        message: result.message,
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
            Err(error) => {
                let (line, character) = self.get_error_position(text, &error);
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line, character },
                        end: Position { line, character: character + 10 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("PARSE_ERROR".to_string())),
                    code_description: None,
                    source: Some("cbu-dsl-lsp".to_string()),
                    message: format!("{}", error),
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }

        diagnostics
    }

    /// Get approximate position of error in text
    fn get_error_position(&self, _text: &str, error: &LispDslError) -> (u32, u32) {
        // Simple heuristic - in real implementation, would track positions during parsing
        match error {
            LispDslError::ParseError(_) => (0, 0),
            LispDslError::UnknownFunction(_) => (0, 0),
            _ => (0, 0),
        }
    }

    /// Provide code completion suggestions
    fn get_completion_items(&self, _text: &str, _position: Position) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // S-expression functions
        items.extend(vec![
            CompletionItem {
                label: "create-cbu".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Create a new CBU".to_string()),
                documentation: Some(Documentation::String(
                    "(create-cbu \"name\" \"description\" (entities ...))".to_string()
                )),
                insert_text: Some("create-cbu \"$1\" \"$2\"".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "entity".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Define an entity".to_string()),
                documentation: Some(Documentation::String(
                    "(entity \"id\" \"name\" role)".to_string()
                )),
                insert_text: Some("entity \"$1\" \"$2\" $3".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "entities".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Group multiple entities".to_string()),
                documentation: Some(Documentation::String(
                    "(entities (entity ...) (entity ...))".to_string()
                )),
                insert_text: Some("entities\n  $0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]);

        // Entity roles
        let roles = vec![
            "asset-owner", "investment-manager", "managing-company",
            "general-partner", "limited-partner", "prime-broker",
            "administrator", "custodian"
        ];

        for role in roles {
            items.push(CompletionItem {
                label: role.to_string(),
                kind: Some(CompletionItemKind::ENUM),
                detail: Some("Entity role".to_string()),
                documentation: Some(Documentation::String(
                    format!("Entity role: {}", role)
                )),
                insert_text: Some(role.to_string()),
                ..Default::default()
            });
        }

        // CRUD operations
        items.extend(vec![
            CompletionItem {
                label: "update-cbu".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Update an existing CBU".to_string()),
                insert_text: Some("update-cbu \"$1\"".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "delete-cbu".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Delete a CBU".to_string()),
                insert_text: Some("delete-cbu \"$1\"".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "query-cbu".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Query CBUs".to_string()),
                insert_text: Some("query-cbu".to_string()),
                ..Default::default()
            },
        ]);

        items
    }

    /// Provide hover information
    async fn get_hover_info(&self, text: &str, position: Position) -> Option<Hover> {
        let lines: Vec<&str> = text.lines().collect();
        if let Some(line) = lines.get(position.line as usize) {
            let word = self.get_word_at_position(line, position.character as usize);

            match word.as_str() {
                "create-cbu" => Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: "**create-cbu**\n\nCreate a new Client Business Unit\n\n```lisp\n(create-cbu \"name\" \"description\" (entities ...))\n```".to_string(),
                    }),
                    range: None,
                }),
                "entity" => Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: "**entity**\n\nDefine an entity with ID, name, and role\n\n```lisp\n(entity \"id\" \"name\" role)\n```".to_string(),
                    }),
                    range: None,
                }),
                "asset-owner" => Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: "**asset-owner**\n\nEntity role: The legal owner of assets being managed".to_string(),
                    }),
                    range: None,
                }),
                "investment-manager" => Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: "**investment-manager**\n\nEntity role: Responsible for making investment decisions".to_string(),
                    }),
                    range: None,
                }),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Extract word at cursor position
    fn get_word_at_position(&self, line: &str, character: usize) -> String {
        let chars: Vec<char> = line.chars().collect();
        if character >= chars.len() {
            return String::new();
        }

        let mut start = character;
        let mut end = character;

        // Find word boundaries
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '-' || chars[start - 1] == '_') {
            start -= 1;
        }

        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '-' || chars[end] == '_') {
            end += 1;
        }

        chars[start..end].iter().collect()
    }

    /// Provide semantic tokens for syntax highlighting
    fn get_semantic_tokens(&self, text: &str) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let mut char_idx = 0;
            let mut chars = line.chars().peekable();

            while let Some(ch) = chars.next() {
                match ch {
                    '(' | ')' => {
                        tokens.push(SemanticToken {
                            delta_line: if tokens.is_empty() { line_idx as u32 } else { 0 },
                            delta_start: if tokens.is_empty() { char_idx as u32 } else { 1 },
                            length: 1,
                            token_type: 0, // Delimiter
                            token_modifiers_bitset: 0,
                        });
                    }
                    '"' => {
                        // String literal
                        let start_char = char_idx;
                        let mut length = 1;
                        while let Some(str_ch) = chars.next() {
                            char_idx += 1;
                            length += 1;
                            if str_ch == '"' {
                                break;
                            }
                        }
                        tokens.push(SemanticToken {
                            delta_line: if tokens.is_empty() { line_idx as u32 } else { 0 },
                            delta_start: if tokens.is_empty() { start_char as u32 } else { (start_char - if tokens.is_empty() { 0 } else { char_idx - length + 1 }) as u32 },
                            length: length as u32,
                            token_type: 1, // String
                            token_modifiers_bitset: 0,
                        });
                    }
                    ';' => {
                        // Comment - rest of line
                        let remaining: String = chars.collect();
                        tokens.push(SemanticToken {
                            delta_line: if tokens.is_empty() { line_idx as u32 } else { 0 },
                            delta_start: if tokens.is_empty() { char_idx as u32 } else { 1 },
                            length: (remaining.len() + 1) as u32,
                            token_type: 2, // Comment
                            token_modifiers_bitset: 0,
                        });
                        break;
                    }
                    _ if ch.is_alphabetic() || ch == '-' => {
                        // Keyword or identifier
                        let start_char = char_idx;
                        let mut word = String::new();
                        word.push(ch);

                        while let Some(&next_ch) = chars.peek() {
                            if next_ch.is_alphanumeric() || next_ch == '-' || next_ch == '_' {
                                word.push(chars.next().unwrap());
                                char_idx += 1;
                            } else {
                                break;
                            }
                        }

                        let token_type = match word.as_str() {
                            "create-cbu" | "update-cbu" | "delete-cbu" | "query-cbu" | "entity" | "entities" => 3, // Function
                            "asset-owner" | "investment-manager" | "managing-company" | "custodian" |
                            "administrator" | "prime-broker" | "general-partner" | "limited-partner" => 4, // Enum
                            "true" | "false" | "nil" => 5, // Keyword
                            _ => 6, // Variable
                        };

                        tokens.push(SemanticToken {
                            delta_line: if tokens.is_empty() { line_idx as u32 } else { 0 },
                            delta_start: if tokens.is_empty() { start_char as u32 } else { (start_char - if tokens.is_empty() { 0 } else { char_idx - word.len() + 1 }) as u32 },
                            length: word.len() as u32,
                            token_type,
                            token_modifiers_bitset: 0,
                        });
                    }
                    _ => {}
                }
                char_idx += 1;
            }
        }

        tokens
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for CbuDslLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        info!("CBU DSL Language Server initializing...");

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "CBU DSL Language Server".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec!["(".to_string(), " ".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("cbu-dsl".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: Default::default(),
                    }
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            work_done_progress_options: Default::default(),
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::new("delimiter"),
                                    SemanticTokenType::new("string"),
                                    SemanticTokenType::new("comment"),
                                    SemanticTokenType::new("function"),
                                    SemanticTokenType::new("enum"),
                                    SemanticTokenType::new("keyword"),
                                    SemanticTokenType::new("variable"),
                                ],
                                token_modifiers: vec![],
                            },
                            range: Some(true),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        }
                    )
                ),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("CBU DSL Language Server initialized!");
        self.client
            .log_message(MessageType::INFO, "CBU DSL Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        info!("CBU DSL Language Server shutting down...");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        info!("Document opened: {}", params.text_document.uri);

        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();

        // Store document
        self.document_map.write().await.insert(uri.clone(), text.clone());

        // Validate and send diagnostics
        let diagnostics = self.validate_document(&uri, &text).await;
        self.client
            .publish_diagnostics(uri, diagnostics, Some(params.text_document.version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text;

            // Update document
            self.document_map.write().await.insert(uri.clone(), text.clone());

            // Validate and send diagnostics
            let diagnostics = self.validate_document(&uri, &text).await;
            self.client
                .publish_diagnostics(uri, diagnostics, Some(params.text_document.version))
                .await;
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(text) = self.document_map.read().await.get(uri) {
            let items = self.get_completion_items(text, position);
            Ok(Some(CompletionResponse::Array(items)))
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(text) = self.document_map.read().await.get(uri) {
            Ok(self.get_hover_info(text, position).await)
        } else {
            Ok(None)
        }
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        let uri = &params.text_document.uri;

        if let Some(text) = self.document_map.read().await.get(uri) {
            let tokens = self.get_semantic_tokens(text);
            Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: tokens,
            })))
        } else {
            Ok(None)
        }
    }
}

/// Create and configure the LSP service
pub fn create_lsp_service() -> (LspService<CbuDslLanguageServer>, tower_lsp::ClientSocket) {
    LspService::new(|client| CbuDslLanguageServer::new(client))
}