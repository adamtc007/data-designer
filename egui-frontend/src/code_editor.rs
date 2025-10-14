use egui::{Color32, Response, Ui, Widget};
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

    /// Render the editor with syntax highlighting
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        // Simplified, reliable approach - just use the enhanced text editor
        ui.add(egui::TextEdit::multiline(&mut self.text)
            .desired_rows(self.desired_rows)
            .code_editor()
            .font(egui::FontId::monospace(self.font_size)))
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

        // This would integrate with your actual parser
        match data_designer_core::parser::parse_expression(&self.text) {
            Ok(_) => true,
            Err(e) => {
                self.syntax_errors.push(SyntaxError {
                    line: 1, // Parser would provide actual line/column
                    column: 1,
                    length: self.text.len(),
                    message: format!("Parse error: {}", e),
                    severity: ErrorSeverity::Error,
                });
                false
            }
        }
    }
}

impl Widget for &mut DslCodeEditor {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}