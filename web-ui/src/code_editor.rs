use eframe::egui;
use std::collections::HashMap;

/// Token types for syntax highlighting
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Keyword,      // WORKFLOW, STEP, PHASE, IF, THEN, ELSE, etc.
    Command,      // VERIFY_IDENTITY, ASSESS_RISK, LOG, SET, etc.
    String,       // "string literals"
    Number,       // 123, 45.67
    Identifier,   // variable names, function names
    Operator,     // =, ==, +, -, AND, OR, etc.
    Comment,      // # comments
    Whitespace,   // spaces, tabs, newlines
    Unknown,      // anything else
}

/// Token with position information
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub text: String,
    pub start: usize,
    pub end: usize,
}

/// Custom code editor for DSL with syntax highlighting and validation
#[derive(Debug, Clone)]
pub struct CodeEditor {
    /// The DSL code content
    pub content: String,
    /// Result of the last parse attempt for validation
    pub last_parse_result: Result<String, String>,
    /// Whether to show syntax validation UI
    pub show_validation: bool,
    /// Cached tokens for syntax highlighting
    pub tokens: Vec<Token>,
    /// Keywords for highlighting
    pub keywords: HashMap<String, TokenType>,
}

impl CodeEditor {
    pub fn new(content: String) -> Self {
        let mut editor = Self {
            content: content.clone(),
            last_parse_result: Ok("Ready".to_string()),
            show_validation: true,
            tokens: Vec::new(),
            keywords: Self::build_keywords(),
        };
        editor.tokenize();
        editor
    }

    /// Build the keywords map for syntax highlighting
    fn build_keywords() -> HashMap<String, TokenType> {
        let mut keywords = HashMap::new();

        // Control flow keywords
        let control_keywords = vec![
            "WORKFLOW", "STEP", "PHASE", "PROCEED_TO", "IF", "THEN", "ELSE", "END_IF",
            "FOR_EACH", "IN", "END_FOR", "AND", "OR", "NOT", "true", "false"
        ];

        for keyword in control_keywords {
            keywords.insert(keyword.to_string(), TokenType::Keyword);
        }

        // Command keywords - from actual templates
        let command_keywords = vec![
            "LOG", "SET", "DERIVE_REGULATORY_CONTEXT", "ASSESS_RISK", "SCREEN_ENTITY",
            "STORE_RESULTS", "COLLECT_DOCUMENT", "FLAG_FOR_REVIEW", "REJECT_CASE",
            "APPROVE_CASE", "FOR_JURISDICTION", "WITH_PRODUCTS", "USING_FACTORS",
            "OUTPUT", "AGAINST", "THRESHOLD", "FROM", "REQUIRED", "AS", "TO",
            "PRIORITY", "WITH_CONDITIONS"
        ];

        for keyword in command_keywords {
            keywords.insert(keyword.to_string(), TokenType::Command);
        }

        keywords
    }

    /// Tokenize the content for syntax highlighting
    fn tokenize(&mut self) {
        self.tokens.clear();
        let chars: Vec<char> = self.content.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let start = i;

            // Skip whitespace
            if chars[i].is_whitespace() {
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                self.tokens.push(Token {
                    token_type: TokenType::Whitespace,
                    text: chars[start..i].iter().collect(),
                    start,
                    end: i,
                });
                continue;
            }

            // Comments
            if chars[i] == '#' {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                self.tokens.push(Token {
                    token_type: TokenType::Comment,
                    text: chars[start..i].iter().collect(),
                    start,
                    end: i,
                });
                continue;
            }

            // String literals
            if chars[i] == '"' {
                i += 1; // Skip opening quote
                while i < chars.len() && chars[i] != '"' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1; // Skip closing quote
                }
                self.tokens.push(Token {
                    token_type: TokenType::String,
                    text: chars[start..i].iter().collect(),
                    start,
                    end: i,
                });
                continue;
            }

            // Numbers
            if chars[i].is_ascii_digit() || (chars[i] == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
                if chars[i] == '-' {
                    i += 1;
                }
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                self.tokens.push(Token {
                    token_type: TokenType::Number,
                    text: chars[start..i].iter().collect(),
                    start,
                    end: i,
                });
                continue;
            }

            // Operators
            if self.is_operator_char(chars[i]) {
                let mut op_text = String::new();

                // Handle multi-character operators
                if i + 1 < chars.len() {
                    let two_char = format!("{}{}", chars[i], chars[i + 1]);
                    if matches!(two_char.as_str(), "==" | "!=" | "<=" | ">=") {
                        op_text = two_char;
                        i += 2;
                    } else {
                        op_text.push(chars[i]);
                        i += 1;
                    }
                } else {
                    op_text.push(chars[i]);
                    i += 1;
                }

                self.tokens.push(Token {
                    token_type: TokenType::Operator,
                    text: op_text,
                    start,
                    end: i,
                });
                continue;
            }

            // Identifiers and keywords
            if chars[i].is_alphabetic() || chars[i] == '_' {
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '.') {
                    i += 1;
                }

                let text: String = chars[start..i].iter().collect();
                let token_type = self.keywords.get(&text)
                    .cloned()
                    .unwrap_or(TokenType::Identifier);

                self.tokens.push(Token {
                    token_type,
                    text,
                    start,
                    end: i,
                });
                continue;
            }

            // Unknown character
            i += 1;
            self.tokens.push(Token {
                token_type: TokenType::Unknown,
                text: chars[start..i].iter().collect(),
                start,
                end: i,
            });
        }
    }

    /// Check if character is an operator
    fn is_operator_char(&self, c: char) -> bool {
        matches!(c, '=' | '!' | '<' | '>' | '+' | '-' | '*' | '/' | '%' | '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';')
    }

    /// Main rendering method for the code editor
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            // Header with title and validation status
            ui.horizontal(|ui| {
                ui.label("📝 DSL Code Editor");
                ui.separator();
                self.show_validation_status(ui);
            });

            ui.separator();

            // Main code editor area with syntax highlighting
            let response = self.show_syntax_highlighted_editor(ui);

            // Parse and validate on content change
            if response.changed() {
                self.tokenize();
                self.validate_syntax();
            }

            response
        }).inner
    }

    /// Render syntax-highlighted editor
    fn show_syntax_highlighted_editor(&mut self, ui: &mut egui::Ui) -> egui::Response {
        // Use a ScrollArea to contain the highlighted text display and editor (full available height)
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Show syntax highlighted version (read-only)
                    ui.label("🎨 Syntax Highlighted View:");
                    ui.separator();

                    self.render_highlighted_text(ui);

                    ui.add_space(10.0);
                    ui.separator();
                    ui.label("✏️ Editor:");

                    // Regular editable text area
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut self.content)
                            .font(egui::TextStyle::Monospace)
                            .desired_rows(15)
                            .desired_width(f32::INFINITY)
                            .code_editor()
                    );

                    // Show syntax analysis in collapsible section
                    ui.collapsing("🔍 Token Analysis", |ui| {
                        self.show_token_analysis(ui);
                    });

                    response
                })
            }).inner.inner
    }

    /// Render the DSL content with syntax highlighting colors
    fn render_highlighted_text(&self, ui: &mut egui::Ui) {
        if self.tokens.is_empty() {
            ui.label("No content to highlight");
            return;
        }

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for token in &self.tokens {
                        if token.text.trim().is_empty() {
                            // Handle whitespace/newlines
                            if token.text.contains('\n') {
                                ui.end_row();
                            } else {
                                ui.add_space(token.text.len() as f32 * 6.0); // Approximate space width
                            }
                            continue;
                        }

                        let color = match token.token_type {
                            TokenType::Keyword => egui::Color32::LIGHT_BLUE,      // Blue for WORKFLOW, STEP, IF, etc.
                            TokenType::Command => egui::Color32::YELLOW,          // Yellow for LOG, SET, ASSESS_RISK, etc.
                            TokenType::String => egui::Color32::GREEN,            // Green for "string literals"
                            TokenType::Number => egui::Color32::LIGHT_RED,        // Red for numbers
                            TokenType::Identifier => egui::Color32::LIGHT_GRAY,   // Gray for variables
                            TokenType::Operator => egui::Color32::WHITE,          // White for operators
                            TokenType::Comment => egui::Color32::DARK_GRAY,       // Dark gray for comments
                            _ => egui::Color32::RED,                               // Red for unknown tokens
                        };

                        // Use RichText for colored display
                        ui.add(egui::Label::new(
                            egui::RichText::new(&token.text)
                                .color(color)
                                .font(egui::FontId::monospace(12.0))
                        ));
                    }
                });
            });
    }

    /// Show token analysis for debugging and visualization
    fn show_token_analysis(&self, ui: &mut egui::Ui) {
        ui.label(format!("Tokens found: {}", self.tokens.len()));

        let mut keyword_count = 0;
        let mut command_count = 0;
        let mut string_count = 0;
        let mut identifier_count = 0;

        for token in &self.tokens {
            match token.token_type {
                TokenType::Keyword => keyword_count += 1,
                TokenType::Command => command_count += 1,
                TokenType::String => string_count += 1,
                TokenType::Identifier => identifier_count += 1,
                _ => {}
            }
        }

        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::LIGHT_BLUE, format!("Keywords: {}", keyword_count));
            ui.colored_label(egui::Color32::YELLOW, format!("Commands: {}", command_count));
            ui.colored_label(egui::Color32::GREEN, format!("Strings: {}", string_count));
            ui.colored_label(egui::Color32::LIGHT_GRAY, format!("Identifiers: {}", identifier_count));
        });

        // Show first few tokens for debugging
        ui.collapsing("🔍 Token Details", |ui| {
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    for (i, token) in self.tokens.iter().take(20).enumerate() {
                        if token.token_type == TokenType::Whitespace {
                            continue; // Skip whitespace for readability
                        }

                        let color = match token.token_type {
                            TokenType::Keyword => egui::Color32::LIGHT_BLUE,
                            TokenType::Command => egui::Color32::YELLOW,
                            TokenType::String => egui::Color32::GREEN,
                            TokenType::Number => egui::Color32::LIGHT_RED,
                            TokenType::Identifier => egui::Color32::LIGHT_GRAY,
                            TokenType::Operator => egui::Color32::WHITE,
                            TokenType::Comment => egui::Color32::DARK_GRAY,
                            _ => egui::Color32::RED,
                        };

                        ui.horizontal(|ui| {
                            ui.small(format!("{}:", i));
                            ui.colored_label(color, format!("{:?}", token.token_type));
                            ui.small(format!("\"{}\"", token.text.replace('\n', "\\n")));
                        });
                    }

                    if self.tokens.len() > 20 {
                        ui.small(format!("... and {} more tokens", self.tokens.len() - 20));
                    }
                });
        });
    }

    /// Display syntax validation status
    fn show_validation_status(&mut self, ui: &mut egui::Ui) {
        if !self.show_validation {
            return;
        }

        match &self.last_parse_result {
            Ok(msg) => {
                ui.colored_label(egui::Color32::GREEN, "✅");
                ui.small(format!("Syntax OK - {}", msg));
            }
            Err(error) => {
                ui.colored_label(egui::Color32::RED, "❌");
                ui.small(format!("Syntax Error: {}", error));
            }
        }
    }

    /// Validate DSL syntax using token analysis
    fn validate_syntax(&mut self) {
        if self.content.trim().is_empty() {
            self.last_parse_result = Err("Empty DSL content".to_string());
            return;
        }

        // Analyze tokens for validation
        let mut has_workflow = false;
        let mut has_steps_or_phases = false;
        let mut errors = Vec::new();
        let mut open_blocks = Vec::new(); // Track IF/FOR_EACH blocks

        for token in &self.tokens {
            match &token.token_type {
                TokenType::Keyword => {
                    match token.text.as_str() {
                        "WORKFLOW" => has_workflow = true,
                        "STEP" | "PHASE" => has_steps_or_phases = true,
                        "IF" => open_blocks.push("IF"),
                        "FOR_EACH" => open_blocks.push("FOR_EACH"),
                        "END_IF" => {
                            if let Some(last) = open_blocks.last() {
                                if *last == "IF" {
                                    open_blocks.pop();
                                } else {
                                    errors.push(format!("Mismatched END_IF - expected END_{}", last));
                                }
                            } else {
                                errors.push("END_IF without matching IF".to_string());
                            }
                        },
                        "END_FOR" => {
                            if let Some(last) = open_blocks.last() {
                                if *last == "FOR_EACH" {
                                    open_blocks.pop();
                                } else {
                                    errors.push(format!("Mismatched END_FOR - expected END_{}", last));
                                }
                            } else {
                                errors.push("END_FOR without matching FOR_EACH".to_string());
                            }
                        },
                        _ => {}
                    }
                },
                TokenType::Unknown => {
                    errors.push(format!("Unknown token: '{}'", token.text));
                },
                _ => {}
            }
        }

        // Check for unclosed blocks
        for open_block in &open_blocks {
            errors.push(format!("Unclosed {} block", open_block));
        }

        // Validate structure
        if !has_workflow {
            errors.push("Missing WORKFLOW declaration".to_string());
        }

        if has_workflow && !has_steps_or_phases {
            errors.push("WORKFLOW requires at least one STEP or PHASE".to_string());
        }

        // Set validation result
        if errors.is_empty() {
            self.last_parse_result = Ok(format!(
                "Valid DSL structure ({} tokens, {} keywords, {} commands)",
                self.tokens.len(),
                self.tokens.iter().filter(|t| t.token_type == TokenType::Keyword).count(),
                self.tokens.iter().filter(|t| t.token_type == TokenType::Command).count()
            ));
        } else {
            self.last_parse_result = Err(errors.join("; "));
        }
    }

    /// Get the current content
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// Set new content
    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.tokenize();
        self.validate_syntax();
    }
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self::new(String::new())
    }
}