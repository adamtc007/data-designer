use egui::{Color32, Response, Ui, Widget, Pos2, Rect, Vec2};
use std::collections::HashMap;

/// Enhanced code editor with syntax highlighting for DSL
pub struct DslCodeEditor {
    pub text: String,
    pub language: DslLanguage,
    pub show_line_numbers: bool,
    pub highlight_current_line: bool,
    pub auto_indent: bool,
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub syntax_errors: Vec<SyntaxError>,
    pub desired_rows: usize,
    pub font_size: f32,
    // Autocomplete support
    pub autocomplete_suggestions: Vec<String>,
    pub show_autocomplete: bool,
    pub selected_suggestion: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DslLanguage {
    Dsl,
    Rust,
    Sql,
    JavaScript,
    Python,
}

#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub message: String,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword,        // IF, THEN, ELSE, AND, OR
    Operator,       // +, -, *, /, ==, !=
    Number,         // 123, 45.67
    String,         // "hello", 'world'
    Boolean,        // true, false
    Identifier,     // variable names
    Function,       // CONCAT, UPPER, etc
    Comment,        // // comments
    Punctuation,    // (, ), [, ], {, }
    Whitespace,
    Error,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Default for DslCodeEditor {
    fn default() -> Self {
        Self {
            text: String::new(),
            language: DslLanguage::Dsl,
            show_line_numbers: true,
            highlight_current_line: true,
            auto_indent: true,
            cursor_position: 0,
            selection_start: None,
            selection_end: None,
            syntax_errors: Vec::new(),
            desired_rows: 10,
            font_size: 14.0,
            autocomplete_suggestions: Vec::new(),
            show_autocomplete: false,
            selected_suggestion: 0,
        }
    }
}

impl DslCodeEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn with_language(mut self, language: DslLanguage) -> Self {
        self.language = language;
        self
    }

    pub fn with_rows(mut self, rows: usize) -> Self {
        self.desired_rows = rows;
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.cursor_position = 0;
        self.selection_start = None;
        self.selection_end = None;
        self.validate_syntax();
    }

    /// Tokenize the text for syntax highlighting
    fn tokenize(&self) -> Vec<Token> {
        match self.language {
            DslLanguage::Dsl => self.tokenize_dsl(),
            DslLanguage::Rust => self.tokenize_rust(),
            DslLanguage::Sql => self.tokenize_sql(),
            DslLanguage::JavaScript => self.tokenize_javascript(),
            DslLanguage::Python => self.tokenize_python(),
        }
    }

    fn tokenize_dsl(&self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = self.text.char_indices().peekable();
        let mut line = 1;
        let mut column = 1;

        while let Some((start_idx, ch)) = chars.next() {
            let token_start = start_idx;
            let token_line = line;
            let token_column = column;

            match ch {
                // Whitespace
                c if c.is_whitespace() => {
                    if c == '\n' {
                        line += 1;
                        column = 1;
                    } else {
                        column += 1;
                    }
                    continue;
                }

                // Comments
                '/' if chars.peek().map(|(_, c)| *c) == Some('/') => {
                    chars.next(); // consume second /
                    let mut comment = String::from("//");
                    while let Some((_, c)) = chars.peek() {
                        if *c == '\n' { break; }
                        let (_, c) = chars.next().unwrap();
                        comment.push(c);
                        column += 1;
                    }
                    tokens.push(Token {
                        token_type: TokenType::Comment,
                        text: comment.clone(),
                        start: token_start,
                        end: token_start + comment.len(),
                        line: token_line,
                        column: token_column,
                    });
                    column += comment.len();
                }

                // String literals
                '"' => {
                    let mut string_content = String::from("\"");
                    column += 1;
                    let mut escaped = false;

                    while let Some((_, c)) = chars.next() {
                        string_content.push(c);
                        column += 1;

                        if escaped {
                            escaped = false;
                            continue;
                        }

                        match c {
                            '\\' => escaped = true,
                            '"' => break,
                            '\n' => {
                                line += 1;
                                column = 1;
                            }
                            _ => {}
                        }
                    }

                    tokens.push(Token {
                        token_type: TokenType::String,
                        text: string_content.clone(),
                        start: token_start,
                        end: token_start + string_content.len(),
                        line: token_line,
                        column: token_column,
                    });
                }

                '\'' => {
                    let mut string_content = String::from("'");
                    column += 1;

                    while let Some((_, c)) = chars.next() {
                        string_content.push(c);
                        column += 1;
                        if c == '\'' { break; }
                        if c == '\n' {
                            line += 1;
                            column = 1;
                        }
                    }

                    tokens.push(Token {
                        token_type: TokenType::String,
                        text: string_content.clone(),
                        start: token_start,
                        end: token_start + string_content.len(),
                        line: token_line,
                        column: token_column,
                    });
                }

                // Numbers
                c if c.is_ascii_digit() => {
                    let mut number = String::new();
                    number.push(c);

                    while let Some((_, c)) = chars.peek() {
                        if c.is_ascii_digit() || *c == '.' {
                            let (_, c) = chars.next().unwrap();
                            number.push(c);
                            column += 1;
                        } else {
                            break;
                        }
                    }

                    tokens.push(Token {
                        token_type: TokenType::Number,
                        text: number.clone(),
                        start: token_start,
                        end: token_start + number.len(),
                        line: token_line,
                        column: token_column,
                    });
                    column += 1;
                }

                // Operators and punctuation
                '+' | '-' | '*' | '/' | '%' => {
                    tokens.push(Token {
                        token_type: TokenType::Operator,
                        text: ch.to_string(),
                        start: token_start,
                        end: token_start + 1,
                        line: token_line,
                        column: token_column,
                    });
                    column += 1;
                }

                '=' => {
                    if chars.peek().map(|(_, c)| *c) == Some('=') {
                        chars.next();
                        tokens.push(Token {
                            token_type: TokenType::Operator,
                            text: "==".to_string(),
                            start: token_start,
                            end: token_start + 2,
                            line: token_line,
                            column: token_column,
                        });
                        column += 2;
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Operator,
                            text: "=".to_string(),
                            start: token_start,
                            end: token_start + 1,
                            line: token_line,
                            column: token_column,
                        });
                        column += 1;
                    }
                }

                '!' => {
                    if chars.peek().map(|(_, c)| *c) == Some('=') {
                        chars.next();
                        tokens.push(Token {
                            token_type: TokenType::Operator,
                            text: "!=".to_string(),
                            start: token_start,
                            end: token_start + 2,
                            line: token_line,
                            column: token_column,
                        });
                        column += 2;
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Operator,
                            text: "!".to_string(),
                            start: token_start,
                            end: token_start + 1,
                            line: token_line,
                            column: token_column,
                        });
                        column += 1;
                    }
                }

                '<' | '>' => {
                    if chars.peek().map(|(_, c)| *c) == Some('=') {
                        chars.next();
                        tokens.push(Token {
                            token_type: TokenType::Operator,
                            text: format!("{}=", ch),
                            start: token_start,
                            end: token_start + 2,
                            line: token_line,
                            column: token_column,
                        });
                        column += 2;
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Operator,
                            text: ch.to_string(),
                            start: token_start,
                            end: token_start + 1,
                            line: token_line,
                            column: token_column,
                        });
                        column += 1;
                    }
                }

                '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' => {
                    tokens.push(Token {
                        token_type: TokenType::Punctuation,
                        text: ch.to_string(),
                        start: token_start,
                        end: token_start + 1,
                        line: token_line,
                        column: token_column,
                    });
                    column += 1;
                }

                // Identifiers and keywords
                c if c.is_alphabetic() || c == '_' => {
                    let mut identifier = String::new();
                    identifier.push(c);

                    while let Some((_, c)) = chars.peek() {
                        if c.is_alphanumeric() || *c == '_' || *c == '.' {
                            let (_, c) = chars.next().unwrap();
                            identifier.push(c);
                            column += 1;
                        } else {
                            break;
                        }
                    }

                    let token_type = match identifier.to_uppercase().as_str() {
                        "IF" | "THEN" | "ELSE" | "AND" | "OR" | "NOT" | "IN" | "MATCHES" | "TRUE" | "FALSE" => {
                            if identifier == "true" || identifier == "false" {
                                TokenType::Boolean
                            } else {
                                TokenType::Keyword
                            }
                        }
                        "CONCAT" | "UPPER" | "LOWER" | "LENGTH" | "TRIM" | "SUBSTRING" | "LOOKUP" |
                        "ABS" | "ROUND" | "FLOOR" | "CEIL" | "MIN" | "MAX" | "SUM" | "AVG" | "COUNT" |
                        "IS_EMAIL" | "IS_LEI" | "IS_SWIFT" | "IS_PHONE" | "VALIDATE" | "EXTRACT" => {
                            TokenType::Function
                        }
                        _ => TokenType::Identifier,
                    };

                    tokens.push(Token {
                        token_type,
                        text: identifier.clone(),
                        start: token_start,
                        end: token_start + identifier.len(),
                        line: token_line,
                        column: token_column,
                    });
                    column += 1;
                }

                // Unknown character
                _ => {
                    tokens.push(Token {
                        token_type: TokenType::Error,
                        text: ch.to_string(),
                        start: token_start,
                        end: token_start + 1,
                        line: token_line,
                        column: token_column,
                    });
                    column += 1;
                }
            }
        }

        tokens
    }

    fn tokenize_rust(&self) -> Vec<Token> {
        // Simplified Rust tokenizer - can be expanded
        let keywords = ["fn", "let", "mut", "if", "else", "match", "struct", "enum", "impl", "pub", "use", "mod"];
        self.generic_tokenize(&keywords)
    }

    fn tokenize_sql(&self) -> Vec<Token> {
        let keywords = ["SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "TABLE", "INDEX"];
        self.generic_tokenize(&keywords)
    }

    fn tokenize_javascript(&self) -> Vec<Token> {
        let keywords = ["function", "var", "let", "const", "if", "else", "for", "while", "return", "class"];
        self.generic_tokenize(&keywords)
    }

    fn tokenize_python(&self) -> Vec<Token> {
        let keywords = ["def", "class", "if", "else", "elif", "for", "while", "return", "import", "from"];
        self.generic_tokenize(&keywords)
    }

    fn generic_tokenize(&self, keywords: &[&str]) -> Vec<Token> {
        // Basic tokenizer for other languages - simplified version
        let mut tokens = Vec::new();
        let lines: Vec<&str> = self.text.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let words: Vec<&str> = line.split_whitespace().collect();
            let mut col = 0;

            for word in words {
                let token_type = if keywords.contains(&word) {
                    TokenType::Keyword
                } else if word.parse::<f64>().is_ok() {
                    TokenType::Number
                } else if word.starts_with('"') || word.starts_with('\'') {
                    TokenType::String
                } else {
                    TokenType::Identifier
                };

                tokens.push(Token {
                    token_type,
                    text: word.to_string(),
                    start: col,
                    end: col + word.len(),
                    line: line_num + 1,
                    column: col + 1,
                });

                col += word.len() + 1; // +1 for space
            }
        }

        tokens
    }

    /// Get color for token type
    fn get_token_color(&self, token_type: &TokenType, ui: &Ui) -> Color32 {
        match token_type {
            TokenType::Keyword => Color32::from_rgb(86, 156, 214),     // Blue
            TokenType::Operator => Color32::from_rgb(212, 212, 212),   // Light gray
            TokenType::Number => Color32::from_rgb(181, 206, 168),     // Light green
            TokenType::String => Color32::from_rgb(206, 145, 120),     // Orange
            TokenType::Boolean => Color32::from_rgb(86, 156, 214),     // Blue
            TokenType::Identifier => Color32::from_rgb(156, 220, 254), // Light blue
            TokenType::Function => Color32::from_rgb(220, 220, 170),   // Yellow
            TokenType::Comment => Color32::from_rgb(106, 153, 85),     // Green
            TokenType::Punctuation => Color32::from_rgb(212, 212, 212), // Light gray
            TokenType::Error => Color32::from_rgb(244, 71, 71),        // Red
            TokenType::Whitespace => ui.visuals().text_color(),        // Default
        }
    }

    /// Render the editor with syntax highlighting - ENHANCED WITH AUTOCOMPLETE
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        // Validate syntax in real-time
        self.validate_syntax();

        let editor_response = ui.vertical(|ui| {
            // Text editor
            let text_edit_response = ui.add(egui::TextEdit::multiline(&mut self.text)
                .desired_rows(self.desired_rows)
                .code_editor()
                .font(egui::FontId::monospace(self.font_size)));

            // Track cursor position from the response
            if text_edit_response.changed() {
                // Update cursor position estimation based on text length changes
                let new_len = self.text.len();
                if new_len != self.cursor_position {
                    self.cursor_position = new_len.min(self.cursor_position);
                }
            }

            // Simple status line
            ui.horizontal(|ui| {
                if !self.syntax_errors.is_empty() {
                    ui.colored_label(egui::Color32::RED, "❌ Syntax Error");
                    // Show only the first error to keep it clean
                    if let Some(error) = self.syntax_errors.first() {
                        ui.label(format!("- {}", error.message));
                    }
                } else if !self.text.trim().is_empty() {
                    ui.colored_label(egui::Color32::GREEN, "✅ Valid");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "Enter DSL code...");
                }

                // Autocomplete status
                if self.show_autocomplete {
                    ui.separator();
                    ui.colored_label(egui::Color32::YELLOW,
                        format!("💡 {} suggestions", self.autocomplete_suggestions.len()));
                }
            });

            text_edit_response
        }).inner;

        // Render autocomplete popup after the main editor
        if self.show_autocomplete && !self.autocomplete_suggestions.is_empty() {
            self.render_autocomplete_popup(ui, editor_response.rect);
        }

        editor_response
    }

    /// Render floating autocomplete popup
    fn render_autocomplete_popup(&mut self, ui: &mut Ui, editor_rect: Rect) {
        let cursor_pos = self.get_cursor_screen_position(ui, editor_rect);

        // Position popup below cursor
        let popup_pos = Pos2::new(
            cursor_pos.x,
            cursor_pos.y + ui.text_style_height(&egui::TextStyle::Monospace)
        );

        // Use fixed positioning relative to the editor
        let popup_size = Vec2::new(200.0, 150.0);
        let adjusted_pos = popup_pos; // Simplified - let egui handle bounds

        egui::Window::new("autocomplete_popup")
            .id(egui::Id::new("autocomplete_window"))
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .current_pos(adjusted_pos)
            .fixed_size(popup_size)
            .frame(egui::Frame::popup(ui.style()))
            .show(ui.ctx(), |ui| {
                ui.set_min_height(150.0);
                ui.vertical(|ui| {
                    ui.label("💡 Suggestions:");
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .max_height(120.0)
                        .show(ui, |ui| {
                            for (i, suggestion) in self.autocomplete_suggestions.iter().enumerate() {
                                let is_selected = i == self.selected_suggestion;

                                let response = ui.selectable_label(is_selected, suggestion);

                                if response.clicked() {
                                    self.selected_suggestion = i;
                                    // Apply suggestion (handled by parent)
                                }

                                if is_selected {
                                    response.scroll_to_me(Some(egui::Align::Center));
                                }
                            }
                        });

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.small("↑↓ Navigate");
                        ui.separator();
                        ui.small("Enter Accept");
                        ui.separator();
                        ui.small("Esc Cancel");
                    });
                });
            });
    }

    /// Enhanced version with syntax preview (kept for future use)
    pub fn show_with_highlighting(&mut self, ui: &mut Ui) -> Response {
        // Validate syntax in real-time
        self.validate_syntax();

        ui.vertical(|ui| {
            // Clean syntax highlighted preview
            if !self.text.trim().is_empty() {
                ui.label("📖 Preview:");
                ui.separator();

                ui.group(|ui| {
                    self.render_simple_highlighted_preview(ui);
                });

                ui.separator();
            }

            // Editable text section
            let response = ui.add(egui::TextEdit::multiline(&mut self.text)
                .desired_rows(self.desired_rows)
                .code_editor()
                .font(egui::FontId::monospace(self.font_size)));

            // Simple status
            ui.horizontal(|ui| {
                if !self.syntax_errors.is_empty() {
                    ui.colored_label(egui::Color32::RED, "❌ Syntax Error");
                } else if !self.text.trim().is_empty() {
                    ui.colored_label(egui::Color32::GREEN, "✅ Valid");
                }
            });

            response
        }).response
    }

    /// Simple, clean highlighted preview without LayoutJob
    fn render_simple_highlighted_preview(&self, ui: &mut Ui) {
        if self.text.trim().is_empty() {
            ui.colored_label(egui::Color32::GRAY, "Enter DSL code to see preview...");
            return;
        }

        // Simple approach: just show the text with basic syntax coloring
        ui.horizontal_wrapped(|ui| {
            let tokens = self.tokenize();

            for token in &tokens {
                let color = match token.token_type {
                    TokenType::Keyword => egui::Color32::from_rgb(86, 156, 214),     // Blue
                    TokenType::Function => egui::Color32::from_rgb(220, 220, 170),   // Yellow
                    TokenType::String => egui::Color32::from_rgb(206, 145, 120),     // Orange
                    TokenType::Number => egui::Color32::from_rgb(181, 206, 168),     // Light green
                    TokenType::Comment => egui::Color32::from_rgb(106, 153, 85),     // Green
                    _ => ui.visuals().text_color(),
                };

                ui.colored_label(color, &token.text);
            }
        });
    }

    /// Render a beautiful syntax highlighted preview using LayoutJob
    fn render_syntax_highlighted_preview(&self, ui: &mut Ui) {
        if self.text.trim().is_empty() {
            ui.colored_label(egui::Color32::GRAY, "Enter DSL code to see syntax highlighting...");
            return;
        }

        let tokens = self.tokenize();

        // Create a LayoutJob for advanced text rendering
        let mut job = egui::text::LayoutJob::default();

        for token in &tokens {
            let color = self.get_token_color(&token.token_type, ui);

            job.append(
                &token.text,
                0.0, // No extra spacing
                egui::TextFormat {
                    font_id: egui::FontId::monospace(self.font_size),
                    color,
                    background: egui::Color32::TRANSPARENT,
                    italics: matches!(token.token_type, TokenType::Comment),
                    underline: egui::Stroke::NONE,
                    strikethrough: egui::Stroke::NONE,
                    valign: egui::Align::BOTTOM,
                    expand_bg: 0.0,
                    extra_letter_spacing: 0.0,
                    line_height: None,
                },
            );
        }

        // Render the highlighted text
        ui.label(job);

        // Show token breakdown for debugging/learning
        if ui.collapsing("🔍 Token Analysis", |ui| {
            egui::Grid::new("token_grid").striped(true).show(ui, |ui| {
                ui.label("Token");
                ui.label("Type");
                ui.label("Color Preview");
                ui.end_row();

                for token in &tokens {
                    if token.token_type == TokenType::Whitespace {
                        continue; // Skip whitespace tokens in display
                    }

                    let color = self.get_token_color(&token.token_type, ui);

                    ui.label(&token.text);
                    ui.label(format!("{:?}", token.token_type));
                    ui.colored_label(color, "●●●");
                    ui.end_row();
                }
            });
        }).header_response.clicked() {
            // Collapsing was clicked
        }
    }

    pub fn show_old(&mut self, ui: &mut Ui) -> Response {
        let available_rect = ui.available_rect_before_wrap();
        let line_height = ui.text_style_height(&egui::TextStyle::Monospace);
        let total_rows = self.text.lines().count().max(self.desired_rows);
        let desired_height = line_height * total_rows as f32;

        ui.allocate_ui_with_layout(
            egui::vec2(available_rect.width(), desired_height.min(available_rect.height())),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                let tokens = self.tokenize();

                egui::ScrollArea::vertical()
                    .max_height(desired_height)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Line numbers
                            if self.show_line_numbers {
                                let line_count = self.text.lines().count();
                                let line_number_width = (line_count.to_string().len() as f32 + 1.0) * 8.0;

                                ui.allocate_ui_with_layout(
                                    egui::vec2(line_number_width, desired_height),
                                    egui::Layout::top_down(egui::Align::RIGHT),
                                    |ui| {
                                        ui.style_mut().visuals.extreme_bg_color = Color32::from_gray(30);

                                        for line_num in 1..=line_count.max(self.desired_rows) {
                                            ui.colored_label(
                                                Color32::from_gray(100),
                                                format!("{:>3}", line_num)
                                            );
                                        }
                                    }
                                );

                                ui.separator();
                            }

                            // Code content with syntax highlighting
                            ui.vertical(|ui| {
                                if tokens.is_empty() {
                                    // Fallback to simple text editor if no tokens
                                    ui.add(egui::TextEdit::multiline(&mut self.text)
                                        .desired_rows(self.desired_rows)
                                        .code_editor()
                                        .font(egui::FontId::monospace(self.font_size)));
                                } else {
                                    self.render_highlighted_text(ui, &tokens);
                                }
                            });
                        });

                        // Syntax errors display
                        if !self.syntax_errors.is_empty() {
                            ui.separator();
                            ui.heading("Syntax Errors:");
                            for error in &self.syntax_errors {
                                let color = match error.severity {
                                    ErrorSeverity::Error => Color32::RED,
                                    ErrorSeverity::Warning => Color32::YELLOW,
                                    ErrorSeverity::Info => Color32::LIGHT_BLUE,
                                };
                                ui.colored_label(color, format!("Line {}: {}", error.line, error.message));
                            }
                        }
                    })
            }
        ).response
    }

    fn render_highlighted_text(&mut self, ui: &mut Ui, tokens: &[Token]) {
        // Group tokens by line for proper rendering
        let mut lines: HashMap<usize, Vec<&Token>> = HashMap::new();
        for token in tokens {
            lines.entry(token.line).or_insert_with(Vec::new).push(token);
        }

        let line_count = self.text.lines().count().max(1);

        for line_num in 1..=line_count {
            ui.horizontal(|ui| {
                if let Some(line_tokens) = lines.get(&line_num) {
                    for token in line_tokens {
                        let color = self.get_token_color(&token.token_type, ui);
                        ui.label(egui::RichText::new(&token.text)
                            .color(color)
                            .font(egui::FontId::monospace(self.font_size)));
                    }
                } else {
                    // Empty line
                    ui.label(" ");
                }
            });
        }

        // Add a text edit for actual editing (this is a simplified approach)
        ui.add_space(10.0);
        ui.add(egui::TextEdit::multiline(&mut self.text)
            .desired_rows(self.desired_rows)
            .code_editor()
            .font(egui::FontId::monospace(self.font_size)));
    }

    /// Set syntax errors for highlighting
    pub fn set_syntax_errors(&mut self, errors: Vec<SyntaxError>) {
        self.syntax_errors = errors;
    }

    /// Validate syntax using the DSL parser
    pub fn validate_syntax(&mut self) -> bool {
        self.syntax_errors.clear();

        if self.text.trim().is_empty() {
            return true; // Empty is valid
        }

        // Try parsing the entire text as a DSL expression
        match data_designer_core::parser::parse_expression(&self.text) {
            Ok(_) => {
                // Also try parsing as a rule if expression parsing succeeds
                match data_designer_core::parser::parse_rule(&self.text) {
                    Ok(_) => true,
                    Err(e) => {
                        // Expression is valid but not a complete rule - this is okay
                        true // Don't mark as error for expressions
                    }
                }
            }
            Err(e) => {
                // Try parsing as a rule instead
                match data_designer_core::parser::parse_rule(&self.text) {
                    Ok(_) => true,
                    Err(rule_err) => {
                        // Both failed, show the more informative error
                        let error_msg = if self.text.contains("IF") || self.text.contains("WHEN") {
                            format!("Rule parse error: {}", rule_err)
                        } else {
                            format!("Expression parse error: {}", e)
                        };

                        self.syntax_errors.push(SyntaxError {
                            line: 1, // TODO: Extract line/column from parser error
                            column: 1,
                            length: self.text.len(),
                            message: error_msg,
                            severity: ErrorSeverity::Error,
                        });
                        false
                    }
                }
            }
        }
    }

    /// Add a helper method to get suggestions for the current cursor position
    pub fn get_completion_suggestions(&self, cursor_pos: usize) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Get the word at cursor position
        let (word_start, current_word) = self.get_word_at_position(cursor_pos);

        // DSL keywords
        let keywords = vec![
            "IF", "THEN", "ELSE", "WHEN", "AND", "OR", "NOT", "IN", "MATCHES",
            "CONTAINS", "STARTS_WITH", "ENDS_WITH", "true", "false", "null"
        ];

        // DSL functions
        let functions = vec![
            "CONCAT", "SUBSTRING", "UPPER", "LOWER", "LENGTH", "TRIM",
            "ABS", "ROUND", "FLOOR", "CEIL", "MIN", "MAX", "SUM", "AVG", "COUNT",
            "IS_EMAIL", "IS_LEI", "IS_SWIFT", "IS_PHONE", "VALIDATE", "EXTRACT",
            "HAS", "IS_NULL", "IS_EMPTY", "TO_STRING", "TO_NUMBER", "TO_BOOLEAN",
            "FIRST", "LAST", "GET", "LOOKUP"
        ];

        // Add matching keywords
        for keyword in &keywords {
            if keyword.to_lowercase().starts_with(&current_word.to_lowercase()) {
                suggestions.push(keyword.to_string());
            }
        }

        // Add matching functions
        for function in &functions {
            if function.to_lowercase().starts_with(&current_word.to_lowercase()) {
                suggestions.push(format!("{}()", function));
            }
        }

        suggestions.sort();
        suggestions.dedup();
        suggestions.truncate(10); // Limit suggestions
        suggestions
    }

    /// Helper to extract word at cursor position
    pub fn get_word_at_position(&self, pos: usize) -> (usize, String) {
        let chars: Vec<char> = self.text.chars().collect();
        if pos > chars.len() {
            return (pos, String::new());
        }

        // Find word start
        let mut start = pos;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Find word end
        let mut end = pos;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        let word: String = chars[start..end].iter().collect();
        (start, word)
    }

    /// Set autocomplete suggestions and show popup
    pub fn set_autocomplete_suggestions(&mut self, suggestions: Vec<String>) {
        self.autocomplete_suggestions = suggestions;
        self.show_autocomplete = !self.autocomplete_suggestions.is_empty();
        self.selected_suggestion = 0;
    }

    /// Hide autocomplete popup
    pub fn hide_autocomplete(&mut self) {
        self.show_autocomplete = false;
        self.autocomplete_suggestions.clear();
        self.selected_suggestion = 0;
    }

    /// Navigate autocomplete suggestions
    pub fn navigate_autocomplete(&mut self, direction: i32) {
        if !self.show_autocomplete || self.autocomplete_suggestions.is_empty() {
            return;
        }

        let len = self.autocomplete_suggestions.len();
        if direction > 0 {
            self.selected_suggestion = (self.selected_suggestion + 1) % len;
        } else if direction < 0 {
            self.selected_suggestion = if self.selected_suggestion == 0 {
                len - 1
            } else {
                self.selected_suggestion - 1
            };
        }
    }

    /// Accept the currently selected autocomplete suggestion
    pub fn accept_autocomplete(&mut self) -> Option<String> {
        if !self.show_autocomplete || self.autocomplete_suggestions.is_empty() {
            return None;
        }

        let suggestion = self.autocomplete_suggestions[self.selected_suggestion].clone();
        self.hide_autocomplete();
        Some(suggestion)
    }

    /// Calculate cursor screen position for autocomplete popup placement
    fn get_cursor_screen_position(&self, ui: &Ui, text_rect: Rect) -> Pos2 {
        let lines: Vec<&str> = self.text.lines().collect();
        let chars: Vec<char> = self.text.chars().collect();

        // Find line and column of cursor
        let mut current_pos = 0;
        let mut line_num = 0;
        let mut col_num = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            let line_end = current_pos + line.len();
            if self.cursor_position <= line_end {
                line_num = line_idx;
                col_num = self.cursor_position - current_pos;
                break;
            }
            current_pos = line_end + 1; // +1 for newline
        }

        // Calculate approximate screen position
        let line_height = ui.text_style_height(&egui::TextStyle::Monospace);
        let char_width = 8.0; // Approximate monospace character width

        let x = text_rect.left() + (col_num as f32 * char_width);
        let y = text_rect.top() + (line_num as f32 * line_height);

        Pos2::new(x, y)
    }
}

impl Widget for &mut DslCodeEditor {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}