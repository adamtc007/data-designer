use std::collections::HashMap;
use std::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

// Import the core logic from our other crate
use data_designer_core::{manager, models::DataDictionary, parser};

// --- The State of our Language Server ---
// It now holds a map of open documents to provide contextual information.
struct Backend {
    client: Client,
    dictionary: Mutex<DataDictionary>,
    document_map: Mutex<HashMap<Url, String>>,
}

/// A helper function to find the boundaries of a word at a given cursor position.
/// This is a simple implementation that considers alphanumeric characters, '_', and '.' as part of a word.
fn get_word_at_position(document: &str, position: Position) -> Option<String> {
    let line = document.lines().nth(position.line as usize)?;
    let char_pos = position.character as usize;

    // Find the start of the word by walking backwards from the cursor
    let start = line
        .char_indices()
        .rev()
        .skip_while(|(i, _)| *i >= char_pos)
        .find(|(_, c)| !c.is_alphanumeric() && *c != '.' && *c != '_')
        .map(|(i, _)| i + 1)
        .unwrap_or(0);

    // Find the end of the word by walking forwards from the cursor
    let end = line
        .char_indices()
        .skip_while(|(i, _)| *i < char_pos)
        .find(|(_, c)| !c.is_alphanumeric() && *c != '.' && *c != '_')
        .map(|(i, _)| i)
        .unwrap_or_else(|| line.len());

    if start >= end {
        return None;
    }

    Some(line[start..end].to_string())
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(
                MessageType::INFO,
                "Data Designer Language Server initialized.",
            )
            .await;
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Data Designer Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), "(".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "Server is ready.").await;
    }

    // --- Feature 1: Live Validation (with document caching) ---
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let content = &params.content_changes[0].text;

        // Update our document cache with the latest content.
        self.document_map
            .lock()
            .unwrap()
            .insert(uri.clone(), content.clone());

        let mut diagnostics = Vec::new();

        // We use our existing nom parser from the core library for validation!
        // This is where a full AST parse happens.
        if let Err(e) = parser::parse_rule(content) {
            let diagnostic = Diagnostic {
                range: Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX)), // Underline the whole document for simplicity
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("Parse Error: {}", e),
                ..Default::default()
            };
            diagnostics.push(diagnostic);
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    // --- Feature 2: Autocompletion (Unchanged) ---
    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let dictionary = self.dictionary.lock().unwrap();
        let mut items = Vec::new();

        for model in &dictionary.canonical_models {
            for attr in &model.attributes {
                items.push(CompletionItem {
                    label: format!("{}.{}", model.entity_name, attr.name),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some(attr.description.clone()),
                    ..Default::default()
                });
            }
        }
        
        let functions = vec!["CONCAT", "REGEX_MATCH", "CAST", "UPPER", "LOWER"];
        for func in functions {
            items.push(CompletionItem {
                label: func.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!("The {} function.", func)),
                ..Default::default()
            });
        }
        Ok(Some(CompletionResponse::Array(items)))
    }

    // --- Feature 3: Hover Information (Now with real logic!) ---
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let dictionary = self.dictionary.lock().unwrap();
        let document_map = self.document_map.lock().unwrap();

        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get the document content from our cache.
        let document = match document_map.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Get the specific word under the user's cursor.
        let word = match get_word_at_position(document, position) {
            Some(w) => w,
            None => return Ok(None),
        };

        let mut hover_content = None;

        // Check if the word is a known function.
        let functions = vec!["CONCAT", "REGEX_MATCH", "CAST", "UPPER", "LOWER"];
        if functions.contains(&word.as_str()) {
            hover_content = Some(format!(
                "**Function:** `{}`\n\nA built-in data transformation function.",
                word
            ));
        }

        // If not a function, check if it's a known attribute from our dictionary.
        if hover_content.is_none() {
            for model in &dictionary.canonical_models {
                let fqn_prefix = format!("{}.", model.entity_name);
                if word.starts_with(&fqn_prefix) {
                    let attr_name = word.strip_prefix(&fqn_prefix).unwrap_or_default();
                    if let Some(attr) = model.attributes.iter().find(|a| a.name == attr_name) {
                        hover_content = Some(format!(
                            "**Attribute:** `{}`\n\n*{}*",
                            word, attr.description
                        ));
                        break;
                    }
                }
            }
        }

        // If we found content, create and return the Hover response.
        if let Some(content) = hover_content {
            return Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
                    language: "markdown".to_string(),
                    value: content,
                })),
                range: None,
            }));
        }

        Ok(None)
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

// --- Main entry point for the LSP binary ---
#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let dictionary = manager::load_dictionary().expect("Could not load data dictionary.");

    let (service, socket) = LspService::new(|client| Backend {
        client,
        dictionary: Mutex::new(dictionary),
        document_map: Mutex::new(HashMap::new()), // Initialize the document map
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
