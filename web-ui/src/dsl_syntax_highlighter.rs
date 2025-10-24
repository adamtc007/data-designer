//! Advanced DSL Syntax Highlighter for CBU DSL Editor
//! Provides professional syntax highlighting with semantic understanding

use egui::{Color32, RichText};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Keyword,      // Functions like create-cbu, entity
    EntityRole,   // asset-owner, investment-manager, etc.
    String,       // "quoted strings"
    Comment,      // ; comments
    Number,       // 123, 3.14
    Delimiter,    // ( ) [ ]
    Operator,     // =, +, -, etc.
    Identifier,   // variable names
    Boolean,      // true, false, nil
    Error,        // Invalid syntax
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub token_type: TokenType,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    pub keyword_color: Color32,
    pub entity_role_color: Color32,
    pub string_color: Color32,
    pub comment_color: Color32,
    pub number_color: Color32,
    pub delimiter_color: Color32,
    pub operator_color: Color32,
    pub identifier_color: Color32,
    pub boolean_color: Color32,
    pub error_color: Color32,
    pub background_color: Color32,
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self::dark_theme()
    }
}

impl SyntaxTheme {
    pub fn dark_theme() -> Self {
        Self {
            keyword_color: Color32::from_rgb(86, 156, 214),      // Blue
            entity_role_color: Color32::from_rgb(255, 180, 100), // Orange
            string_color: Color32::from_rgb(206, 145, 120),      // Light brown
            comment_color: Color32::from_rgb(106, 153, 85),      // Green
            number_color: Color32::from_rgb(181, 206, 168),      // Light green
            delimiter_color: Color32::from_rgb(255, 255, 0),     // Yellow
            operator_color: Color32::from_rgb(212, 212, 212),    // Light gray
            identifier_color: Color32::from_rgb(156, 220, 254),  // Light blue
            boolean_color: Color32::from_rgb(86, 156, 214),      // Blue
            error_color: Color32::from_rgb(244, 71, 71),         // Red
            background_color: Color32::from_rgb(30, 30, 30),     // Dark gray
        }
    }

    pub fn light_theme() -> Self {
        Self {
            keyword_color: Color32::from_rgb(0, 0, 255),         // Blue
            entity_role_color: Color32::from_rgb(255, 140, 0),   // Dark orange
            string_color: Color32::from_rgb(163, 21, 21),        // Dark red
            comment_color: Color32::from_rgb(0, 128, 0),         // Green
            number_color: Color32::from_rgb(9, 134, 88),         // Dark green
            delimiter_color: Color32::from_rgb(0, 0, 0),         // Black
            operator_color: Color32::from_rgb(0, 0, 0),          // Black
            identifier_color: Color32::from_rgb(1, 1, 1),        // Almost black
            boolean_color: Color32::from_rgb(0, 0, 255),         // Blue
            error_color: Color32::from_rgb(255, 0, 0),           // Red
            background_color: Color32::from_rgb(255, 255, 255),  // White
        }
    }
}

pub struct DslSyntaxHighlighter {
    theme: SyntaxTheme,
    keywords: HashMap<String, TokenType>,
}

impl Default for DslSyntaxHighlighter {
    fn default() -> Self {
        Self::new(SyntaxTheme::default())
    }
}

impl DslSyntaxHighlighter {
    pub fn new(theme: SyntaxTheme) -> Self {
        let mut keywords = HashMap::new();

        // S-expression functions
        let functions = vec![
            "create-cbu", "update-cbu", "delete-cbu", "query-cbu",
            "entity", "entities", "list", "quote"
        ];
        for func in functions {
            keywords.insert(func.to_string(), TokenType::Keyword);
        }

        // Entity roles
        let roles = vec![
            "asset-owner", "investment-manager", "managing-company",
            "general-partner", "limited-partner", "prime-broker",
            "administrator", "custodian"
        ];
        for role in roles {
            keywords.insert(role.to_string(), TokenType::EntityRole);
        }

        // Boolean values
        let booleans = vec!["true", "false", "nil"];
        for boolean in booleans {
            keywords.insert(boolean.to_string(), TokenType::Boolean);
        }

        Self { theme, keywords }
    }

    pub fn set_theme(&mut self, theme: SyntaxTheme) {
        self.theme = theme;
    }

    /// Tokenize DSL text into syntax tokens
    pub fn tokenize(&self, text: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = text.char_indices().peekable();

        while let Some((start_idx, ch)) = chars.next() {
            match ch {
                // Whitespace - skip
                ' ' | '\t' | '\n' | '\r' => continue,

                // Comments
                ';' => {
                    let mut end_idx = start_idx + 1;
                    let mut comment_text = String::from(";");

                    // Read until end of line
                    while let Some((idx, ch)) = chars.peek() {
                        if *ch == '\n' || *ch == '\r' {
                            break;
                        }
                        comment_text.push(*ch);
                        end_idx = *idx + ch.len_utf8();
                        chars.next();
                    }

                    tokens.push(Token {
                        text: comment_text,
                        token_type: TokenType::Comment,
                        start: start_idx,
                        end: end_idx,
                    });
                }

                // String literals
                '"' => {
                    let mut end_idx = start_idx + 1;
                    let mut string_content = String::from("\"");
                    let mut escaped = false;

                    while let Some((idx, ch)) = chars.next() {
                        string_content.push(ch);
                        end_idx = idx + ch.len_utf8();

                        if escaped {
                            escaped = false;
                            continue;
                        }

                        match ch {
                            '\\' => escaped = true,
                            '"' => break,
                            _ => {}
                        }
                    }

                    tokens.push(Token {
                        text: string_content,
                        token_type: TokenType::String,
                        start: start_idx,
                        end: end_idx,
                    });
                }

                // Numbers
                '0'..='9' => {
                    let mut end_idx = start_idx;
                    let mut number_text = String::new();
                    let mut has_dot = false;

                    // Current character
                    number_text.push(ch);

                    // Continue reading digits and dots
                    while let Some((idx, next_ch)) = chars.peek() {
                        match next_ch {
                            '0'..='9' => {
                                number_text.push(*next_ch);
                                end_idx = *idx + next_ch.len_utf8();
                                chars.next();
                            }
                            '.' if !has_dot => {
                                number_text.push(*next_ch);
                                has_dot = true;
                                end_idx = *idx + next_ch.len_utf8();
                                chars.next();
                            }
                            _ => break,
                        }
                    }

                    tokens.push(Token {
                        text: number_text,
                        token_type: TokenType::Number,
                        start: start_idx,
                        end: end_idx + 1,
                    });
                }

                // Delimiters
                '(' | ')' | '[' | ']' | '{' | '}' => {
                    tokens.push(Token {
                        text: ch.to_string(),
                        token_type: TokenType::Delimiter,
                        start: start_idx,
                        end: start_idx + ch.len_utf8(),
                    });
                }

                // Operators
                '=' | '+' | '-' | '*' | '/' | '<' | '>' | '!' | '&' | '|' => {
                    let mut end_idx = start_idx;
                    let mut operator_text = String::from(ch);

                    // Check for multi-character operators
                    if let Some((idx, next_ch)) = chars.peek() {
                        match (ch, *next_ch) {
                            ('=', '=') | ('!', '=') | ('<', '=') | ('>', '=') |
                            ('&', '&') | ('|', '|') | ('<', '>') => {
                                operator_text.push(*next_ch);
                                end_idx = *idx + next_ch.len_utf8();
                                chars.next();
                            }
                            _ => {}
                        }
                    }

                    tokens.push(Token {
                        text: operator_text,
                        token_type: TokenType::Operator,
                        start: start_idx,
                        end: end_idx + 1,
                    });
                }

                // Identifiers and keywords
                _ if ch.is_alphabetic() || ch == '_' => {
                    let mut end_idx = start_idx;
                    let mut identifier_text = String::from(ch);

                    // Continue reading alphanumeric, underscores, and hyphens
                    while let Some((idx, next_ch)) = chars.peek() {
                        if next_ch.is_alphanumeric() || *next_ch == '_' || *next_ch == '-' {
                            identifier_text.push(*next_ch);
                            end_idx = *idx + next_ch.len_utf8();
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    // Determine token type based on keywords
                    let token_type = self.keywords.get(&identifier_text)
                        .copied()
                        .unwrap_or(TokenType::Identifier);

                    tokens.push(Token {
                        text: identifier_text,
                        token_type,
                        start: start_idx,
                        end: end_idx + 1,
                    });
                }

                // Unknown character
                _ => {
                    tokens.push(Token {
                        text: ch.to_string(),
                        token_type: TokenType::Error,
                        start: start_idx,
                        end: start_idx + ch.len_utf8(),
                    });
                }
            }
        }

        tokens
    }

    /// Render syntax-highlighted DSL text in egui
    pub fn render_highlighted_text(&self, ui: &mut egui::Ui, text: &str) {
        let tokens = self.tokenize(text);

        if tokens.is_empty() {
            ui.label(text);
            return;
        }

        ui.horizontal_wrapped(|ui| {
            for token in tokens {
                let color = self.get_token_color(token.token_type);
                let rich_text = RichText::new(&token.text).color(color);

                // Add font styling for certain token types
                let final_text = match token.token_type {
                    TokenType::Keyword => rich_text.strong(),
                    TokenType::Comment => rich_text.italics(),
                    TokenType::String => rich_text,
                    TokenType::Error => rich_text.underline(),
                    _ => rich_text,
                };

                ui.label(final_text);
            }
        });
    }

    /// Render syntax-highlighted text line by line
    pub fn render_highlighted_lines(&self, ui: &mut egui::Ui, text: &str) {
        for line in text.lines() {
            self.render_highlighted_text(ui, line);
        }
    }

    /// Render DSL text with line numbers and syntax highlighting
    pub fn render_with_line_numbers(&self, ui: &mut egui::Ui, text: &str) {
        let lines: Vec<&str> = text.lines().collect();
        let max_line_digits = lines.len().to_string().len();

        for (line_num, line) in lines.iter().enumerate() {
            ui.horizontal(|ui| {
                // Line number
                let line_number_text = format!("{:width$}", line_num + 1, width = max_line_digits);
                ui.label(
                    RichText::new(line_number_text)
                        .color(Color32::from_rgb(128, 128, 128))
                        .monospace()
                );

                ui.separator();

                // Syntax highlighted line
                if line.trim().is_empty() {
                    ui.label(" "); // Empty line placeholder
                } else {
                    self.render_highlighted_text(ui, line);
                }
            });
        }
    }

    /// Get color for a specific token type
    fn get_token_color(&self, token_type: TokenType) -> Color32 {
        match token_type {
            TokenType::Keyword => self.theme.keyword_color,
            TokenType::EntityRole => self.theme.entity_role_color,
            TokenType::String => self.theme.string_color,
            TokenType::Comment => self.theme.comment_color,
            TokenType::Number => self.theme.number_color,
            TokenType::Delimiter => self.theme.delimiter_color,
            TokenType::Operator => self.theme.operator_color,
            TokenType::Identifier => self.theme.identifier_color,
            TokenType::Boolean => self.theme.boolean_color,
            TokenType::Error => self.theme.error_color,
        }
    }

    /// Validate DSL syntax and return errors
    pub fn validate_syntax(&self, text: &str) -> Vec<String> {
        let mut errors = Vec::new();
        let mut paren_stack = Vec::new();
        let mut in_string = false;
        let mut escaped = false;

        for (line_num, line) in text.lines().enumerate() {
            for (char_num, ch) in line.char_indices() {
                if escaped {
                    escaped = false;
                    continue;
                }

                match ch {
                    '"' if !in_string => in_string = true,
                    '"' if in_string => in_string = false,
                    '\\' if in_string => escaped = true,
                    '(' if !in_string => paren_stack.push((line_num, char_num, ch)),
                    ')' if !in_string => {
                        if let Some((_, _, '(')) = paren_stack.last() {
                            paren_stack.pop();
                        } else {
                            errors.push(format!(
                                "Line {}: Unmatched closing parenthesis at column {}",
                                line_num + 1, char_num + 1
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check for unmatched opening parentheses
        for (line_num, char_num, _) in paren_stack {
            errors.push(format!(
                "Line {}: Unmatched opening parenthesis at column {}",
                line_num + 1, char_num + 1
            ));
        }

        // Check for unclosed strings
        if in_string {
            errors.push("Unclosed string literal".to_string());
        }

        errors
    }

    /// Get completion suggestions for a given position
    pub fn get_completions(&self, text: &str, cursor_pos: usize) -> Vec<String> {
        let mut completions = Vec::new();

        // Get the word at cursor
        let word_at_cursor = self.get_word_at_position(text, cursor_pos);

        // S-expression functions
        let functions = vec![
            "create-cbu", "update-cbu", "delete-cbu", "query-cbu",
            "entity", "entities", "list", "quote"
        ];

        // Entity roles
        let roles = vec![
            "asset-owner", "investment-manager", "managing-company",
            "general-partner", "limited-partner", "prime-broker",
            "administrator", "custodian"
        ];

        // Add matching completions
        for func in functions {
            if func.starts_with(&word_at_cursor.to_lowercase()) {
                completions.push(func.to_string());
            }
        }

        for role in roles {
            if role.starts_with(&word_at_cursor.to_lowercase()) {
                completions.push(role.to_string());
            }
        }

        // Boolean values
        for boolean in ["true", "false", "nil"] {
            if boolean.starts_with(&word_at_cursor.to_lowercase()) {
                completions.push(boolean.to_string());
            }
        }

        completions.sort();
        completions.dedup();
        completions
    }

    /// Get the word at a specific position in the text
    fn get_word_at_position(&self, text: &str, pos: usize) -> String {
        let chars: Vec<char> = text.chars().collect();
        if pos >= chars.len() {
            return String::new();
        }

        let mut start = pos;
        let mut end = pos;

        // Find word boundaries
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '-' || chars[start - 1] == '_') {
            start -= 1;
        }

        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '-' || chars[end] == '_') {
            end += 1;
        }

        chars[start..end].iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenization() {
        let highlighter = DslSyntaxHighlighter::default();
        let tokens = highlighter.tokenize(r#"(create-cbu "Test Fund" "Description")"#);

        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].token_type, TokenType::Delimiter); // (
        assert_eq!(tokens[1].token_type, TokenType::Keyword);   // create-cbu
        assert_eq!(tokens[2].token_type, TokenType::String);    // "Test Fund"
        assert_eq!(tokens[3].token_type, TokenType::String);    // "Description"
        assert_eq!(tokens[4].token_type, TokenType::Delimiter); // )
    }

    #[test]
    fn test_syntax_validation() {
        let highlighter = DslSyntaxHighlighter::default();

        // Valid syntax
        let valid_dsl = r#"(create-cbu "Test" "Description")"#;
        let errors = highlighter.validate_syntax(valid_dsl);
        assert!(errors.is_empty());

        // Invalid syntax - unmatched parenthesis
        let invalid_dsl = r#"(create-cbu "Test" "Description""#;
        let errors = highlighter.validate_syntax(invalid_dsl);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_completions() {
        let highlighter = DslSyntaxHighlighter::default();
        let completions = highlighter.get_completions("create", 6);

        assert!(completions.contains(&"create-cbu".to_string()));
    }
}