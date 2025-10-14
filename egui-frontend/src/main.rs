use eframe::egui;
use data_designer_core::db::{
    init_db, DbPool,
    ClientBusinessUnit, CreateCbuRequest,
    DbOperations, DataDictionaryResponse, EmbeddingOperations, SimilarRule
};

mod code_editor;
use code_editor::{DslCodeEditor, DslLanguage};

mod ai_assistant;
use ai_assistant::{AiAssistant, AiProvider, AiSuggestion, SuggestionType};
use data_designer_core::{parser, evaluator, models::{Value, DataDictionary, ViewerState}, transpiler::{Transpiler, TranspilerOptions, TargetLanguage}};
use std::collections::HashMap;
use tokio::runtime::Runtime;
use std::sync::Arc;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    println!("🚀 Starting Data Designer - Pure Rust Edition!");
    println!("🔌 Connecting to PostgreSQL database...");

    // Initialize database connection
    let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    let pool = rt.block_on(async {
        match init_db().await {
            Ok(pool) => {
                println!("✅ Database connected successfully");
                Some(pool)
            }
            Err(e) => {
                eprintln!("❌ Database connection failed: {}", e);
                eprintln!("   Continuing with offline mode...");
                None
            }
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1000.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Data Designer - Pure Rust + Database",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(DataDesignerApp::new(pool, rt)))
        }),
    )
}

struct DataDesignerApp {
    current_tab: Tab,
    db_pool: Option<DbPool>,
    runtime: Arc<Runtime>,

    // Data
    cbus: Vec<ClientBusinessUnit>,
    selected_cbu: Option<usize>,

    // Attribute Dictionary Data
    data_dictionary: Option<DataDictionaryResponse>,
    attribute_search: String,

    // Rule Engine Data
    rules: Vec<serde_json::Value>,
    selected_rule: Option<usize>,

    // Rule Testing
    rule_input: String,
    test_context: String,
    rule_result: Option<String>,
    rule_error: Option<String>,

    // Syntax highlighting
    syntax_highlighter: SyntaxHighlighter,

    // Autocomplete
    show_autocomplete: bool,
    autocomplete_suggestions: Vec<String>,
    autocomplete_position: usize,

    // Vector search
    semantic_search_query: String,
    similar_rules: Vec<SimilarRule>,
    embedding_status: String,

    // Dictionary Viewer State
    dictionary_data: Option<DataDictionary>,
    viewer_state: ViewerState,
    dictionary_loaded: bool,
    // Transpiler State
    transpiler_input: String,
    transpiler_output: String,
    target_language: String,
    optimization_enabled: bool,
    transpiler_error: Option<String>,
    // Custom Code Editor
    dsl_editor: DslCodeEditor,
    output_editor: DslCodeEditor,
    // AI Assistant
    ai_assistant: AiAssistant,
    ai_suggestions: Vec<AiSuggestion>,
    ai_query: String,
    show_ai_panel: bool,
    ai_loading: bool,

    // UI State
    show_cbu_form: bool,
    cbu_form: CbuForm,
    status_message: String,
    loading: bool,
}

#[derive(PartialEq, Default)]
enum Tab {
    #[default]
    Dashboard,
    CBUs,
    AttributeDictionary,
    DictionaryViewer,  // New tab for the JSON viewer
    RuleEngine,
    Transpiler,        // New tab for code generation
    Database,
}

#[derive(Default)]
struct CbuForm {
    cbu_name: String,
    description: String,
    primary_entity_id: String,
    primary_lei: String,
    domicile_country: String,
    business_type: String,
}

// Syntax highlighting for DSL
#[derive(Debug, Clone)]
struct SyntaxToken {
    start: usize,
    end: usize,
    token_type: TokenType,
}

#[derive(Debug, Clone, Copy)]
enum TokenType {
    Keyword,      // IF, THEN, ELSE, AND, OR, NOT
    Function,     // CONCAT, UPPER, ABS, etc.
    Operator,     // +, -, *, /, =, >, <, etc.
    String,       // "text", 'text'
    Number,       // 123, 45.67
    Boolean,      // true, false
    Identifier,   // variable names
    Comment,      // // comments
    Regex,        // /pattern/
    Delimiter,    // (, ), [, ], {, }
}

struct SyntaxHighlighter {
    keywords: Vec<&'static str>,
    functions: Vec<&'static str>,
    operators: Vec<&'static str>,
}

impl SyntaxHighlighter {
    fn new() -> Self {
        Self {
            keywords: vec![
                "IF", "THEN", "ELSE", "WHEN", "AND", "OR", "NOT", "IN", "NOT_IN",
                "MATCHES", "NOT_MATCHES", "CONTAINS", "STARTS_WITH", "ENDS_WITH",
                "true", "false", "null"
            ],
            functions: vec![
                "CONCAT", "SUBSTRING", "UPPER", "LOWER", "LENGTH", "TRIM",
                "ABS", "ROUND", "FLOOR", "CEIL", "MIN", "MAX", "SUM", "AVG", "COUNT",
                "HAS", "IS_NULL", "IS_EMPTY", "TO_STRING", "TO_NUMBER", "TO_BOOLEAN",
                "FIRST", "LAST", "GET", "LOOKUP"
            ],
            operators: vec![
                "+", "-", "*", "/", "%", "**", "&", "=", "!=", "<>", "<", "<=", ">", ">=",
                "&&", "||", "==", "MATCHES", "CONTAINS"
            ],
        }
    }

    fn tokenize(&self, text: &str) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();
        let mut chars = text.char_indices().peekable();

        while let Some((start, ch)) = chars.next() {
            match ch {
                // String literals
                '"' | '\'' => {
                    let quote = ch;
                    let mut end = start + 1;
                    let mut escaped = false;

                    while let Some((pos, ch)) = chars.next() {
                        end = pos + ch.len_utf8();
                        if !escaped && ch == quote {
                            break;
                        }
                        escaped = ch == '\\' && !escaped;
                    }

                    tokens.push(SyntaxToken {
                        start,
                        end,
                        token_type: TokenType::String,
                    });
                }

                // Regex literals
                '/' => {
                    if let Some((_, next_ch)) = chars.peek() {
                        if *next_ch != '/' && *next_ch != '=' && *next_ch != '*' {
                            // Likely a regex
                            let mut end = start + 1;
                            while let Some((pos, ch)) = chars.next() {
                                end = pos + ch.len_utf8();
                                if ch == '/' {
                                    break;
                                }
                            }

                            tokens.push(SyntaxToken {
                                start,
                                end,
                                token_type: TokenType::Regex,
                            });
                            continue;
                        }
                    }

                    // Regular operator
                    tokens.push(SyntaxToken {
                        start,
                        end: start + 1,
                        token_type: TokenType::Operator,
                    });
                }

                // Numbers
                '0'..='9' => {
                    let mut end = start;
                    let mut has_dot = false;

                    while let Some((pos, ch)) = chars.peek() {
                        if ch.is_ascii_digit() {
                            end = *pos + ch.len_utf8();
                            chars.next();
                        } else if *ch == '.' && !has_dot {
                            has_dot = true;
                            end = *pos + ch.len_utf8();
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    tokens.push(SyntaxToken {
                        start,
                        end,
                        token_type: TokenType::Number,
                    });
                }

                // Identifiers, keywords, functions
                ch if ch.is_alphabetic() || ch == '_' => {
                    let mut end = start;

                    while let Some((pos, ch)) = chars.peek() {
                        if ch.is_alphanumeric() || *ch == '_' || *ch == '.' {
                            end = *pos + ch.len_utf8();
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    let word = &text[start..end];
                    let token_type = if self.keywords.contains(&word.to_uppercase().as_str()) {
                        TokenType::Keyword
                    } else if self.functions.contains(&word.to_uppercase().as_str()) {
                        TokenType::Function
                    } else {
                        TokenType::Identifier
                    };

                    tokens.push(SyntaxToken {
                        start,
                        end,
                        token_type,
                    });
                }

                // Operators and delimiters
                '+' | '-' | '*' | '%' | '=' | '!' | '<' | '>' | '&' | '|' => {
                    let mut end = start + ch.len_utf8();

                    // Check for multi-character operators
                    if let Some((pos, next_ch)) = chars.peek() {
                        let two_char = format!("{}{}", ch, next_ch);
                        if self.operators.contains(&two_char.as_str()) {
                            end = *pos + next_ch.len_utf8();
                            chars.next();
                        }
                    }

                    tokens.push(SyntaxToken {
                        start,
                        end,
                        token_type: TokenType::Operator,
                    });
                }

                // Delimiters
                '(' | ')' | '[' | ']' | '{' | '}' | ',' => {
                    tokens.push(SyntaxToken {
                        start,
                        end: start + ch.len_utf8(),
                        token_type: TokenType::Delimiter,
                    });
                }

                // Skip whitespace
                _ if ch.is_whitespace() => continue,

                // Everything else as identifier for now
                _ => {
                    tokens.push(SyntaxToken {
                        start,
                        end: start + ch.len_utf8(),
                        token_type: TokenType::Identifier,
                    });
                }
            }
        }

        tokens
    }

    fn get_color(&self, token_type: TokenType) -> egui::Color32 {
        match token_type {
            TokenType::Keyword => egui::Color32::from_rgb(197, 134, 192),    // Purple
            TokenType::Function => egui::Color32::from_rgb(78, 201, 176),    // Teal
            TokenType::Operator => egui::Color32::from_rgb(86, 156, 214),    // Blue
            TokenType::String => egui::Color32::from_rgb(206, 145, 120),     // Orange
            TokenType::Number => egui::Color32::from_rgb(181, 206, 168),     // Green
            TokenType::Boolean => egui::Color32::from_rgb(86, 156, 214),     // Blue
            TokenType::Identifier => egui::Color32::from_rgb(220, 220, 170), // Yellow
            TokenType::Comment => egui::Color32::from_rgb(106, 153, 85),     // Dark Green
            TokenType::Regex => egui::Color32::from_rgb(215, 186, 125),      // Gold
            TokenType::Delimiter => egui::Color32::from_rgb(128, 128, 128),  // Gray
        }
    }

    fn get_autocomplete_suggestions(&self, partial_word: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let partial_upper = partial_word.to_uppercase();

        // Add matching keywords
        for &keyword in &self.keywords {
            if keyword.starts_with(&partial_upper) {
                suggestions.push(keyword.to_string());
            }
        }

        // Add matching functions
        for &function in &self.functions {
            if function.starts_with(&partial_upper) {
                suggestions.push(format!("{}()", function));
            }
        }

        // Add common attribute suggestions
        let common_attributes = vec![
            "age", "balance", "country", "email", "name", "id", "status",
            "amount", "date", "type", "value", "score", "rating", "level"
        ];

        for attr in common_attributes {
            if attr.to_uppercase().starts_with(&partial_upper) {
                suggestions.push(attr.to_string());
            }
        }

        suggestions.sort();
        suggestions.dedup();
        suggestions.truncate(10); // Limit to 10 suggestions
        suggestions
    }
}

impl DataDesignerApp {
    fn new(db_pool: Option<DbPool>, runtime: Arc<Runtime>) -> Self {
        let ai_assistant = if let Some(ref pool) = db_pool {
            AiAssistant::new()
                .with_provider(AiProvider::Offline)
                .with_env_api_keys()
                .with_database(pool.clone())
        } else {
            AiAssistant::new()
                .with_provider(AiProvider::Offline)
                .with_env_api_keys()
        };

        let mut app = Self {
            current_tab: Tab::default(),
            db_pool,
            runtime,
            cbus: Vec::new(),
            selected_cbu: None,
            data_dictionary: None,
            attribute_search: String::new(),
            rules: Vec::new(),
            selected_rule: None,
            rule_input: String::from("age > 18 AND country = \"USA\""),
            test_context: String::from("{\n  \"age\": 25,\n  \"country\": \"USA\",\n  \"balance\": 50000\n}"),
            rule_result: None,
            rule_error: None,
            syntax_highlighter: SyntaxHighlighter::new(),
            show_autocomplete: false,
            autocomplete_suggestions: Vec::new(),
            autocomplete_position: 0,
            semantic_search_query: String::new(),
            similar_rules: Vec::new(),
            embedding_status: "Ready for semantic search".to_string(),
            dictionary_data: None,
            viewer_state: ViewerState::default(),
            dictionary_loaded: false,
            transpiler_input: "price * quantity + tax".to_string(),
            transpiler_output: String::new(),
            target_language: "Rust".to_string(),
            optimization_enabled: true,
            transpiler_error: None,
            dsl_editor: DslCodeEditor::new()
                .with_text("price * quantity + tax")
                .with_language(DslLanguage::Dsl)
                .with_rows(12)
                .with_font_size(16.0)
                .show_line_numbers(true),
            output_editor: DslCodeEditor::new()
                .with_language(DslLanguage::Rust)
                .with_rows(15)
                .with_font_size(16.0)
                .show_line_numbers(true),
            ai_assistant,
            ai_suggestions: Vec::new(),
            ai_query: String::new(),
            show_ai_panel: true,
            ai_loading: false,
            show_cbu_form: false,
            cbu_form: CbuForm::default(),
            status_message: "Initializing...".to_string(),
            loading: false,
        };

        // Load initial data
        app.load_cbus();
        app.load_data_dictionary();
        app
    }

    fn load_cbus(&mut self) {
        if let Some(ref pool) = self.db_pool {
            self.loading = true;
            self.status_message = "Loading CBUs from database...".to_string();

            let _pool = pool.clone();
            let rt = self.runtime.clone();

            match rt.block_on(async {
                DbOperations::list_cbus().await
            }) {
                Ok(cbu_summaries) => {
                    // Convert summaries to full CBUs - for now just create basic ones
                    self.cbus = cbu_summaries.into_iter().map(|summary| ClientBusinessUnit {
                        id: summary.id,
                        cbu_id: summary.cbu_id,
                        cbu_name: summary.cbu_name,
                        description: summary.description,
                        primary_entity_id: None,
                        primary_lei: None,
                        domicile_country: None,
                        regulatory_jurisdiction: None,
                        business_type: None,
                        status: summary.status,
                        created_date: None,
                        last_review_date: None,
                        next_review_date: None,
                        created_by: None,
                        created_at: summary.created_at,
                        updated_by: None,
                        updated_at: summary.updated_at,
                        metadata: None,
                    }).collect();

                    self.status_message = format!("✅ Loaded {} CBUs from database", self.cbus.len());
                }
                Err(e) => {
                    eprintln!("Failed to load CBUs: {}", e);
                    self.status_message = format!("❌ Failed to load CBUs: {}", e);
                    self.load_sample_data();
                }
            }
            self.loading = false;
        } else {
            self.load_sample_data();
        }
    }

    fn load_sample_data(&mut self) {
        use chrono::Utc;

        self.cbus = vec![
            ClientBusinessUnit {
                id: 0,
                cbu_id: "OFFLINE001".to_string(),
                cbu_name: "Sample CBU (Offline Mode)".to_string(),
                description: Some("No database connection - sample data".to_string()),
                primary_entity_id: None,
                primary_lei: None,
                domicile_country: None,
                regulatory_jurisdiction: None,
                business_type: None,
                status: "Sample".to_string(),
                created_date: None,
                last_review_date: None,
                next_review_date: None,
                created_by: None,
                created_at: Utc::now(),
                updated_by: None,
                updated_at: Utc::now(),
                metadata: None,
            }
        ];
        self.status_message = "⚠️ Offline mode - using sample data".to_string();
    }

    fn create_cbu(&mut self) {
        if let Some(ref _pool) = self.db_pool {
            let request = CreateCbuRequest {
                cbu_name: self.cbu_form.cbu_name.clone(),
                description: if self.cbu_form.description.is_empty() { None } else { Some(self.cbu_form.description.clone()) },
                primary_entity_id: if self.cbu_form.primary_entity_id.is_empty() { None } else { Some(self.cbu_form.primary_entity_id.clone()) },
                primary_lei: if self.cbu_form.primary_lei.is_empty() { None } else { Some(self.cbu_form.primary_lei.clone()) },
                domicile_country: if self.cbu_form.domicile_country.is_empty() { None } else { Some(self.cbu_form.domicile_country.clone()) },
                regulatory_jurisdiction: None,
                business_type: if self.cbu_form.business_type.is_empty() { None } else { Some(self.cbu_form.business_type.clone()) },
                created_by: Some("egui-app".to_string()),
            };

            let rt = self.runtime.clone();
            match rt.block_on(async {
                DbOperations::create_cbu(request).await
            }) {
                Ok(cbu) => {
                    self.cbus.push(cbu);
                    self.status_message = "✅ CBU created successfully".to_string();
                    self.show_cbu_form = false;
                    self.cbu_form = CbuForm::default();
                }
                Err(e) => {
                    self.status_message = format!("❌ Failed to create CBU: {}", e);
                }
            }
        } else {
            self.status_message = "❌ No database connection".to_string();
        }
    }

    fn load_data_dictionary(&mut self) {
        // Load JSON dictionary for viewer
        self.load_json_dictionary();

        // Load database dictionary for existing functionality
        if let Some(ref _pool) = self.db_pool {
            let rt = self.runtime.clone();

            match rt.block_on(async {
                let pool = DbOperations::get_pool().await.map_err(|e| e.to_string())?;
                use data_designer_core::db::DataDictionaryOperations;
                DataDictionaryOperations::get_data_dictionary(&pool, None).await
            }) {
                Ok(dictionary) => {
                    self.data_dictionary = Some(dictionary);
                }
                Err(e) => {
                    eprintln!("Failed to load data dictionary: {}", e);
                }
            }
        }
    }

    fn load_json_dictionary(&mut self) {
        match std::fs::read_to_string("test_data/source_attributes.json") {
            Ok(json_content) => {
                match DataDictionary::load_from_json(&json_content) {
                    Ok(dictionary) => {
                        self.dictionary_data = Some(dictionary);
                        self.dictionary_loaded = true;
                        println!("✅ Dictionary JSON loaded successfully");
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to parse dictionary JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to read test_data/source_attributes.json: {}", e);
                eprintln!("   Make sure the file exists in the project root");
            }
        }
    }
}

impl eframe::App for DataDesignerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        ui.close();
                    }
                });
                ui.menu_button("Database", |ui| {
                    if ui.button("Refresh").clicked() {
                        self.load_cbus();
                        ui.close();
                    }
                    if ui.button("Test Connection").clicked() {
                        if self.db_pool.is_some() {
                            self.status_message = "✅ Database connected".to_string();
                        } else {
                            self.status_message = "❌ No database connection".to_string();
                        }
                        ui.close();
                    }
                });
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                let color = if self.db_pool.is_some() {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::YELLOW
                };
                ui.colored_label(color, &self.status_message);

                if self.loading {
                    ui.spinner();
                }
            });
        });

        // Tab panel
        egui::TopBottomPanel::top("tab_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Dashboard, "🏠 Dashboard");
                ui.selectable_value(&mut self.current_tab, Tab::CBUs, "🏢 CBUs");
                ui.selectable_value(&mut self.current_tab, Tab::AttributeDictionary, "📚 Attributes");
                ui.selectable_value(&mut self.current_tab, Tab::DictionaryViewer, "📋 Dictionary Viewer");
                ui.selectable_value(&mut self.current_tab, Tab::RuleEngine, "⚡ Rules");
                ui.selectable_value(&mut self.current_tab, Tab::Transpiler, "🔄 Transpiler");
                ui.selectable_value(&mut self.current_tab, Tab::Database, "🗄️ Database");
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Dashboard => self.show_dashboard(ui),
                Tab::CBUs => self.show_cbu_tab(ui),
                Tab::AttributeDictionary => self.show_attribute_dictionary_tab(ui),
                Tab::DictionaryViewer => self.show_dictionary_viewer_tab(ui),
                Tab::RuleEngine => self.show_rule_engine_tab(ui),
                Tab::Transpiler => self.show_transpiler_tab(ui),
                Tab::Database => self.show_database_tab(ui),
            }
        });

        // CBU form modal
        if self.show_cbu_form {
            egui::Window::new("Create CBU")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    self.show_cbu_form_ui(ui);
                });
        }
    }
}

impl DataDesignerApp {
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("🦀 Pure Rust Data Designer");

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Database Status:");
            if self.db_pool.is_some() {
                ui.colored_label(egui::Color32::GREEN, "✅ Connected to PostgreSQL");
            } else {
                ui.colored_label(egui::Color32::YELLOW, "⚠️ Offline mode");
            }
        });

        ui.horizontal(|ui| {
            ui.label("CBUs Loaded:");
            ui.label(format!("{}", self.cbus.len()));
        });

        ui.separator();

        if ui.button("🔄 Refresh Data").clicked() {
            self.load_cbus();
        }
    }

    fn show_cbu_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Client Business Units");

        ui.horizontal(|ui| {
            if ui.button("➕ Create CBU").clicked() {
                self.show_cbu_form = true;
                self.cbu_form = CbuForm::default();
            }

            if ui.button("🔄 Refresh").clicked() {
                self.load_cbus();
            }
        });

        ui.separator();

        // CBU table
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("cbu_grid").striped(true).show(ui, |ui| {
                ui.label("ID");
                ui.label("CBU ID");
                ui.label("Name");
                ui.label("Status");
                ui.label("Description");
                ui.label("Created");
                ui.end_row();

                for (_index, cbu) in self.cbus.iter().enumerate() {
                    ui.label(cbu.id.to_string());
                    ui.label(&cbu.cbu_id);
                    ui.label(&cbu.cbu_name);
                    ui.label(&cbu.status);
                    ui.label(cbu.description.as_ref().unwrap_or(&"N/A".to_string()));
                    ui.label(cbu.created_at.format("%Y-%m-%d").to_string());
                    ui.end_row();
                }
            });
        });
    }

    fn show_database_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🗄️ Database Management");

        ui.separator();

        if let Some(ref _pool) = self.db_pool {
            ui.colored_label(egui::Color32::GREEN, "✅ PostgreSQL Connected");
            ui.label("Connection pool active and ready");

            ui.separator();

            if ui.button("🧪 Test Query").clicked() {
                self.load_cbus();
            }
        } else {
            ui.colored_label(egui::Color32::YELLOW, "⚠️ No Database Connection");
            ui.label("The application is running in offline mode");
            ui.label("Check config.toml or DATABASE_URL environment variable");
        }
    }

    fn show_cbu_form_ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("cbu_form_grid").show(ui, |ui| {
            ui.label("CBU Name:");
            ui.text_edit_singleline(&mut self.cbu_form.cbu_name);
            ui.end_row();

            ui.label("Description:");
            ui.text_edit_multiline(&mut self.cbu_form.description);
            ui.end_row();

            ui.label("Primary Entity ID:");
            ui.text_edit_singleline(&mut self.cbu_form.primary_entity_id);
            ui.end_row();

            ui.label("Primary LEI:");
            ui.text_edit_singleline(&mut self.cbu_form.primary_lei);
            ui.end_row();

            ui.label("Domicile Country:");
            ui.text_edit_singleline(&mut self.cbu_form.domicile_country);
            ui.end_row();

            ui.label("Business Type:");
            ui.text_edit_singleline(&mut self.cbu_form.business_type);
            ui.end_row();
        });

        ui.horizontal(|ui| {
            if ui.button("💾 Create").clicked() {
                self.create_cbu();
            }

            if ui.button("❌ Cancel").clicked() {
                self.show_cbu_form = false;
                self.cbu_form = CbuForm::default();
            }
        });
    }

    fn show_attribute_dictionary_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📚 Attribute Dictionary");

        ui.horizontal(|ui| {
            ui.label("Search:");
            if ui.text_edit_singleline(&mut self.attribute_search).changed() {
                // Trigger search when text changes
                self.search_attributes();
            }
            if ui.button("🔄 Refresh").clicked() {
                self.load_data_dictionary();
            }
        });

        ui.separator();

        if let Some(ref dictionary) = self.data_dictionary {
            ui.horizontal(|ui| {
                ui.label(format!("📊 Total: {}", dictionary.total_count));
                ui.separator();
                ui.colored_label(egui::Color32::from_rgb(52, 152, 219), format!("🏢 Business: {}", dictionary.business_count));
                ui.separator();
                ui.colored_label(egui::Color32::from_rgb(155, 89, 182), format!("⚙️ Derived: {}", dictionary.derived_count));
                ui.separator();
                ui.colored_label(egui::Color32::from_rgb(231, 76, 60), format!("🔧 System: {}", dictionary.system_count));
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("attribute_grid").striped(true).show(ui, |ui| {
                    ui.label("Name");
                    ui.label("Type");
                    ui.label("Entity");
                    ui.label("Data Type");
                    ui.label("Description");
                    ui.label("Key");
                    ui.label("Nullable");
                    ui.end_row();

                    for attr in &dictionary.attributes {
                        let attr_type = attr.get("attribute_type").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let color = match attr_type {
                            "business" => egui::Color32::from_rgb(52, 152, 219),
                            "derived" => egui::Color32::from_rgb(155, 89, 182),
                            "system" => egui::Color32::from_rgb(231, 76, 60),
                            _ => egui::Color32::GRAY,
                        };

                        ui.colored_label(color, attr.get("attribute_name").and_then(|v| v.as_str()).unwrap_or(""));
                        ui.label(attr_type);
                        ui.label(attr.get("entity_name").and_then(|v| v.as_str()).unwrap_or(""));
                        ui.label(attr.get("data_type").and_then(|v| v.as_str()).unwrap_or(""));
                        ui.label(attr.get("description").and_then(|v| v.as_str()).unwrap_or("N/A"));
                        ui.label(if attr.get("is_key").and_then(|v| v.as_bool()).unwrap_or(false) { "🔑" } else { "" });
                        ui.label(if attr.get("is_nullable").and_then(|v| v.as_bool()).unwrap_or(false) { "✓" } else { "✗" });
                        ui.end_row();
                    }
                });
            });
        } else {
            ui.label("Loading attribute dictionary...");
            ui.spinner();
        }
    }

    fn show_rule_engine_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚡ Rule Engine & DSL Live Testing");

        // Live Rule Testing Section
        ui.group(|ui| {
            ui.heading("🧪 Live Rule Testing");

            ui.horizontal(|ui| {
                ui.label("Rule DSL:");
                if ui.button("📋 Paste Sample").clicked() {
                    self.rule_input = "IF age >= 18 THEN \"adult\" ELSE \"minor\"".to_string();
                }
                if ui.button("🔄 Clear").clicked() {
                    self.rule_input.clear();
                    self.rule_result = None;
                    self.rule_error = None;
                }
            });

            // Rule input with syntax highlighting
            let mut rule_input = self.rule_input.clone();
            let response = self.show_highlighted_text_edit(ui, &mut rule_input, [ui.available_width(), 120.0]);
            if response.changed() {
                self.rule_input = rule_input;
            }

            ui.horizontal(|ui| {
                ui.label("Test Context (JSON):");
                if ui.button("📋 Sample Data").clicked() {
                    self.test_context = "{\n  \"age\": 25,\n  \"country\": \"USA\",\n  \"balance\": 50000,\n  \"email\": \"test@example.com\"\n}".to_string();
                }
            });

            // Context input
            ui.add_sized([ui.available_width(), 100.0],
                egui::TextEdit::multiline(&mut self.test_context)
                    .hint_text("Enter test data as JSON...")
                    .font(egui::TextStyle::Monospace));

            ui.horizontal(|ui| {
                if ui.button("🚀 Test Rule").clicked() {
                    self.test_rule();
                }
                if ui.button("📊 Parse AST").clicked() {
                    self.parse_ast_only();
                }
                if ui.button("🔍 Validate Syntax").clicked() {
                    self.validate_syntax_only();
                }
                if ui.button("💾 Save Rule").clicked() {
                    self.status_message = "Feature coming soon: Save to Database".to_string();
                }
            });

            ui.separator();

            // Results display
            if let Some(ref result) = self.rule_result {
                ui.group(|ui| {
                    ui.heading("✅ Result");
                    ui.label(egui::RichText::new(result).color(egui::Color32::GREEN).monospace());
                });
            }

            if let Some(ref error) = self.rule_error {
                ui.group(|ui| {
                    ui.heading("❌ Error");
                    ui.label(egui::RichText::new(error).color(egui::Color32::RED).monospace());
                });
            }
        });

        ui.separator();

        // Semantic Search Section
        ui.group(|ui| {
            ui.heading("🔍 Semantic Rule Search");

            ui.horizontal(|ui| {
                ui.label("Search Query:");
                ui.text_edit_singleline(&mut self.semantic_search_query);
                if ui.button("🧠 Find Similar Rules").clicked() {
                    self.search_similar_rules();
                }
                if ui.button("⚡ Generate Embeddings").clicked() {
                    self.generate_all_embeddings();
                }
            });

            ui.label(&self.embedding_status);

            if !self.similar_rules.is_empty() {
                ui.separator();
                ui.heading("📊 Similar Rules Found:");

                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    egui::Grid::new("similar_rules_grid").striped(true).show(ui, |ui| {
                        ui.label("Similarity");
                        ui.label("Rule Name");
                        ui.label("Definition");
                        ui.label("Actions");
                        ui.end_row();

                        for similar_rule in &self.similar_rules {
                            ui.label(format!("{:.3}", 1.0 - similar_rule.similarity)); // Convert distance to similarity
                            ui.label(&similar_rule.rule_name);
                            ui.label(&similar_rule.rule_definition);
                            if ui.button("📋 Copy").clicked() {
                                self.rule_input = similar_rule.rule_definition.clone();
                            }
                            ui.end_row();
                        }
                    });
                });
            }
        });

        ui.separator();

        // DSL Reference
        ui.collapsing("📖 DSL Quick Reference", |ui| {
            ui.label("🔢 Arithmetic: +, -, *, /, %, **");
            ui.label("🔤 String: &, CONCAT(), UPPER(), LOWER(), LENGTH()");
            ui.label("🔍 Comparison: =, !=, <, <=, >, >=");
            ui.label("🎯 Pattern: MATCHES, CONTAINS, STARTS_WITH, ENDS_WITH");
            ui.label("🔗 Logical: AND, OR, NOT");
            ui.label("📋 Lists: IN, NOT_IN, [item1, item2]");
            ui.label("🎛️ Conditionals: IF...THEN...ELSE, WHEN...THEN...ELSE");
            ui.label("⚙️ Functions: ABS(), ROUND(), MIN(), MAX(), SUM(), AVG()");
            ui.label("🔧 Type Cast: TO_STRING(), TO_NUMBER(), TO_BOOLEAN()");
        });

        // Sample Rules Gallery
        ui.collapsing("🎨 Sample Rules Gallery", |ui| {
            if ui.button("Age Classification").clicked() {
                self.rule_input = "IF age < 18 THEN \"minor\" ELSE IF age < 65 THEN \"adult\" ELSE \"senior\"".to_string();
            }
            if ui.button("KYC Risk Score").clicked() {
                self.rule_input = "IF balance > 100000 AND age > 25 THEN \"low_risk\" ELSE \"high_risk\"".to_string();
            }
            if ui.button("Email Validation").clicked() {
                self.rule_input = "email MATCHES /^[\\w\\._%+-]+@[\\w\\.-]+\\.[A-Za-z]{2,}$/".to_string();
            }
            if ui.button("Complex Business Rule").clicked() {
                self.rule_input = "WHEN country IN [\"USA\", \"UK\", \"CA\"] AND balance > 50000 THEN CONCAT(\"VIP_\", UPPER(country)) ELSE \"STANDARD\"".to_string();
            }
        });

        if !self.rules.is_empty() {
            ui.separator();
            ui.heading("📝 Saved Rules");

            for (i, _rule) in self.rules.iter().enumerate() {
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.selected_rule == Some(i), format!("Rule {}", i + 1)).clicked() {
                        self.selected_rule = Some(i);
                    }
                });
            }
        }
    }

    fn test_rule(&mut self) {
        self.rule_result = None;
        self.rule_error = None;

        // Parse the rule
        let ast = match parser::parse_rule(&self.rule_input) {
            Ok((_, ast)) => ast,
            Err(e) => {
                self.rule_error = Some(format!("Parse Error: {:?}", e));
                return;
            }
        };

        // Parse the test context
        let context: HashMap<String, Value> = match serde_json::from_str(&self.test_context) {
            Ok(json_value) => {
                let mut facts = HashMap::new();
                if let serde_json::Value::Object(map) = json_value {
                    for (key, value) in map {
                        let val = match value {
                            serde_json::Value::String(s) => Value::String(s),
                            serde_json::Value::Number(n) => {
                                if let Some(i) = n.as_i64() {
                                    Value::Integer(i)
                                } else if let Some(f) = n.as_f64() {
                                    Value::Float(f)
                                } else {
                                    Value::Null
                                }
                            },
                            serde_json::Value::Bool(b) => Value::Boolean(b),
                            serde_json::Value::Array(arr) => {
                                let list: Vec<Value> = arr.into_iter().map(|v| match v {
                                    serde_json::Value::String(s) => Value::String(s),
                                    serde_json::Value::Number(n) => {
                                        if let Some(i) = n.as_i64() {
                                            Value::Integer(i)
                                        } else {
                                            Value::Float(n.as_f64().unwrap_or(0.0))
                                        }
                                    },
                                    serde_json::Value::Bool(b) => Value::Boolean(b),
                                    _ => Value::Null,
                                }).collect();
                                Value::List(list)
                            },
                            serde_json::Value::Null => Value::Null,
                            _ => Value::Null,
                        };
                        facts.insert(key, val);
                    }
                }
                facts
            },
            Err(e) => {
                self.rule_error = Some(format!("JSON Parse Error: {}", e));
                return;
            }
        };

        // Evaluate the rule
        match evaluator::evaluate(&ast, &context) {
            Ok(result) => {
                let result_str = match result {
                    Value::String(s) => format!("\"{}\"", s),
                    Value::Integer(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::Boolean(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    Value::List(list) => {
                        let items: Vec<String> = list.iter().map(|v| match v {
                            Value::String(s) => format!("\"{}\"", s),
                            Value::Integer(i) => i.to_string(),
                            Value::Float(f) => f.to_string(),
                            Value::Boolean(b) => b.to_string(),
                            Value::Null => "null".to_string(),
                            _ => "complex".to_string(),
                        }).collect();
                        format!("[{}]", items.join(", "))
                    },
                    Value::Regex(pattern) => format!("/{}/", pattern),
                };
                self.rule_result = Some(format!("Result: {}", result_str));
            },
            Err(e) => {
                self.rule_error = Some(format!("Evaluation Error: {}", e));
            }
        }
    }

    fn parse_ast_only(&mut self) {
        self.rule_result = None;
        self.rule_error = None;

        match parser::parse_rule(&self.rule_input) {
            Ok((_, ast)) => {
                self.rule_result = Some(format!("AST: {:#?}", ast));
            },
            Err(e) => {
                self.rule_error = Some(format!("Parse Error: {:?}", e));
            }
        }
    }

    fn validate_syntax_only(&mut self) {
        self.rule_result = None;
        self.rule_error = None;

        match parser::parse_rule(&self.rule_input) {
            Ok((remaining, _ast)) => {
                if remaining.trim().is_empty() {
                    self.rule_result = Some("✅ Syntax is valid!".to_string());
                } else {
                    self.rule_error = Some(format!("⚠️ Unexpected remaining input: '{}'", remaining));
                }
            },
            Err(e) => {
                self.rule_error = Some(format!("❌ Syntax Error: {:?}", e));
            }
        }
    }

    fn search_similar_rules(&mut self) {
        if self.semantic_search_query.trim().is_empty() {
            self.embedding_status = "❌ Please enter a search query".to_string();
            return;
        }

        if let Some(ref _pool) = self.db_pool {
            let rt = self.runtime.clone();
            let query = self.semantic_search_query.clone();

            self.embedding_status = "🔄 Searching for similar rules...".to_string();

            match rt.block_on(async {
                let pool = DbOperations::get_pool().await.map_err(|e| e.to_string())?;
                EmbeddingOperations::find_similar_rules(&pool, &query, 5).await
            }) {
                Ok(rules) => {
                    self.similar_rules = rules;
                    self.embedding_status = format!("✅ Found {} similar rules", self.similar_rules.len());
                }
                Err(e) => {
                    self.embedding_status = format!("❌ Search failed: {}", e);
                    self.similar_rules.clear();
                }
            }
        } else {
            self.embedding_status = "❌ No database connection".to_string();
        }
    }

    fn generate_all_embeddings(&mut self) {
        if let Some(ref _pool) = self.db_pool {
            let rt = self.runtime.clone();

            self.embedding_status = "🔄 Generating embeddings for all rules...".to_string();

            match rt.block_on(async {
                let pool = DbOperations::get_pool().await.map_err(|e| e.to_string())?;
                EmbeddingOperations::generate_all_embeddings(&pool).await
            }) {
                Ok(_) => {
                    self.embedding_status = "✅ All embeddings generated successfully".to_string();
                }
                Err(e) => {
                    self.embedding_status = format!("❌ Embedding generation failed: {}", e);
                }
            }
        } else {
            self.embedding_status = "❌ No database connection".to_string();
        }
    }

    fn search_attributes(&mut self) {
        if self.attribute_search.len() >= 2 {
            if let Some(ref _pool) = self.db_pool {
                let rt = self.runtime.clone();
                let search_term = self.attribute_search.clone();

                match rt.block_on(async {
                    let pool = DbOperations::get_pool().await.map_err(|e| e.to_string())?;
                    use data_designer_core::db::DataDictionaryOperations;
                    DataDictionaryOperations::get_data_dictionary(&pool, Some(&search_term)).await
                }) {
                    Ok(dictionary) => {
                        self.data_dictionary = Some(dictionary);
                    }
                    Err(e) => {
                        eprintln!("Failed to search attributes: {}", e);
                    }
                }
            }
        } else if self.attribute_search.is_empty() {
            self.load_data_dictionary();
        }
    }

    fn show_highlighted_text_edit(&mut self, ui: &mut egui::Ui, text: &mut String, size: [f32; 2]) -> egui::Response {
        // For now, use a simple approach with colored text display
        ui.group(|ui| {
            ui.vertical(|ui| {
                // Show syntax highlighted preview
                if !text.is_empty() {
                    ui.label("📖 Syntax Highlighted Preview:");
                    ui.separator();

                    let tokens = self.syntax_highlighter.tokenize(text);

                    ui.horizontal_wrapped(|ui| {
                        let mut last_end = 0;

                        for token in tokens {
                            // Add any whitespace between tokens
                            if token.start > last_end {
                                let whitespace = &text[last_end..token.start];
                                if !whitespace.trim().is_empty() {
                                    ui.label(whitespace);
                                }
                            }

                            // Add the colored token
                            let token_text = &text[token.start..token.end];
                            let color = self.syntax_highlighter.get_color(token.token_type);
                            ui.label(egui::RichText::new(token_text).color(color).monospace());

                            last_end = token.end;
                        }

                        // Add any remaining text
                        if last_end < text.len() {
                            ui.label(&text[last_end..]);
                        }
                    });

                    ui.separator();
                }

                // Regular text editor
                ui.add_sized(size, egui::TextEdit::multiline(text)
                    .hint_text("Enter your DSL code here...")
                    .font(egui::TextStyle::Monospace))
            }).inner
        }).inner
    }

    fn show_dictionary_viewer_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Dictionary Viewer - JSON Data Explorer");

        if !self.dictionary_loaded {
            ui.colored_label(egui::Color32::YELLOW, "⚠️ Dictionary not loaded");
            if ui.button("🔄 Reload Dictionary").clicked() {
                self.load_json_dictionary();
            }
            return;
        }

        if let Some(ref dictionary) = self.dictionary_data {
            // Statistics and overview
            ui.horizontal(|ui| {
                let stats = dictionary.get_statistics();
                ui.label(format!("📊 {} Datasets", stats.total_datasets));
                ui.separator();
                ui.label(format!("🏷️ {} Attributes", stats.total_attributes));
                ui.separator();
                ui.label(format!("📚 {} Lookup Tables", stats.lookup_tables_count));
                ui.separator();
                ui.label(format!("🔗 {} Lookup Entries", stats.total_lookup_entries));
            });

            ui.separator();

            // Search functionality
            ui.horizontal(|ui| {
                ui.label("🔍 Search:");
                ui.text_edit_singleline(&mut self.viewer_state.search_query);
                if ui.button("❌ Clear").clicked() {
                    self.viewer_state.clear_search();
                }
                if self.viewer_state.has_active_filters() {
                    ui.colored_label(egui::Color32::GREEN, "Filters active");
                }
            });

            ui.separator();

            // Two-panel layout
            ui.horizontal(|ui| {
                // Left panel - Dataset list and groups
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.heading("📁 Datasets");
                        ui.set_width(300.0);

                        egui::ScrollArea::vertical()
                            .max_height(500.0)
                            .show(ui, |ui| {
                                for (i, dataset) in dictionary.datasets.iter().enumerate() {
                                    let group_name = &dataset.name;
                                    let expanded = self.viewer_state.is_group_expanded(group_name);

                                    ui.horizontal(|ui| {
                                        let expand_icon = if expanded { "📂" } else { "📁" };
                                        if ui.button(format!("{} {}", expand_icon, dataset.name)).clicked() {
                                            self.viewer_state.toggle_group(group_name);
                                        }
                                        ui.label(format!("({} attrs)", dataset.attributes.len()));
                                    });

                                    if expanded {
                                        ui.indent("dataset_attrs", |ui| {
                                            ui.label(&dataset.description);
                                            ui.separator();

                                            // Show filtered attributes
                                            for (attr_name, attr_value) in &dataset.attributes {
                                                if self.viewer_state.search_query.is_empty()
                                                   || attr_name.to_lowercase().contains(&self.viewer_state.search_query.to_lowercase()) {

                                                    if ui.selectable_label(false, format!("🏷️ {}", attr_name)).clicked() {
                                                        self.viewer_state.selected_dataset = Some(format!("{}.{}", dataset.id, attr_name));
                                                    }
                                                }
                                            }
                                        });
                                    }
                                    ui.separator();
                                }
                            });
                    });
                });

                ui.separator();

                // Right panel - Attribute details
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.heading("🔍 Attribute Details");

                        if let Some(ref selected) = self.viewer_state.selected_dataset {
                            if let Some((dataset_id, attr_name)) = selected.split_once('.') {
                                if let Some(dataset) = dictionary.get_dataset_by_id(dataset_id) {
                                    if let Some(attr_value) = dataset.attributes.get(attr_name) {
                                        ui.label(format!("Dataset: {}", dataset.name));
                                        ui.label(format!("Attribute: {}", attr_name));

                                        ui.separator();

                                        ui.label("Value:");
                                        ui.code(format!("{:#}", attr_value));

                                        ui.separator();

                                        // Type information
                                        match attr_value {
                                            serde_json::Value::String(s) => {
                                                ui.label(format!("Type: String (length: {})", s.len()));
                                                if s.contains('@') {
                                                    ui.colored_label(egui::Color32::BLUE, "📧 Looks like email");
                                                }
                                                if s.contains("http") {
                                                    ui.colored_label(egui::Color32::BLUE, "🔗 Looks like URL");
                                                }
                                            }
                                            serde_json::Value::Number(n) => {
                                                if n.is_i64() {
                                                    ui.label("Type: Integer");
                                                } else {
                                                    ui.label("Type: Decimal");
                                                }
                                            }
                                            serde_json::Value::Bool(_) => {
                                                ui.label("Type: Boolean");
                                            }
                                            serde_json::Value::Array(arr) => {
                                                ui.label(format!("Type: Array (length: {})", arr.len()));
                                            }
                                            serde_json::Value::Object(obj) => {
                                                ui.label(format!("Type: Object (keys: {})", obj.len()));
                                            }
                                            serde_json::Value::Null => {
                                                ui.label("Type: Null");
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            ui.label("Select an attribute to view details");
                        }
                    });
                });
            });

            ui.separator();

            // Lookup tables section
            ui.collapsing("📚 Lookup Tables", |ui| {
                for (table_name, table_data) in &dictionary.lookup_tables {
                    ui.collapsing(format!("📋 {} ({} entries)", table_name, table_data.len()), |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for (key, value) in table_data {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("🔑 {}", key));
                                        ui.separator();
                                        ui.label(format!("{}", value));
                                    });
                                }
                            });
                    });
                }
            });

        } else {
            ui.colored_label(egui::Color32::RED, "❌ No dictionary data available");
        }
    }

    fn show_transpiler_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔄 DSL Transpiler & Code Generation");

        // AI Assistant Toggle
        ui.horizontal(|ui| {
            if ui.button(if self.show_ai_panel { "🤖 Hide AI Assistant" } else { "🤖 Show AI Assistant" }).clicked() {
                self.show_ai_panel = !self.show_ai_panel;
            }
            ui.separator();
            ui.label("💡 Get intelligent suggestions and help while coding");
        });

        ui.add_space(10.0);

        // Main layout with optional AI panel
        if self.show_ai_panel {
            ui.horizontal(|ui| {
                // Left side: Editor and transpiler
                ui.vertical(|ui| {
                    self.show_editor_panel(ui);
                });

                ui.separator();

                // Right side: AI Assistant
                ui.vertical(|ui| {
                    self.show_ai_assistant_panel(ui);
                });
            });
        } else {
            self.show_editor_panel(ui);
        }
    }

    fn show_editor_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("📝 Input DSL Expression");
                    ui.add_space(10.0);

                    // Input area
                    ui.horizontal(|ui| {
                        ui.label("DSL Rule:");
                        if ui.button("🔄 Clear").clicked() {
                            self.transpiler_input.clear();
                            self.dsl_editor.text.clear();
                        }
                    });

                    // Enhanced DSL Editor with syntax highlighting and real-time completion
                    let editor_response = ui.add(&mut self.dsl_editor);

                    // Sync with transpiler input for now
                    self.transpiler_input = self.dsl_editor.text.clone();

                    // Trigger intelligent code completion on typing
                    if editor_response.changed() || self.should_trigger_completion() {
                        self.trigger_code_completion();
                    }

                    ui.add_space(10.0);

                    // Configuration
                    ui.horizontal(|ui| {
                        ui.label("Target Language:");
                        egui::ComboBox::from_id_salt("target_language")
                            .selected_text(&self.target_language)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.target_language, "Rust".to_string(), "🦀 Rust");
                                ui.selectable_value(&mut self.target_language, "SQL".to_string(), "🗄️ SQL");
                                ui.selectable_value(&mut self.target_language, "JavaScript".to_string(), "📱 JavaScript");
                                ui.selectable_value(&mut self.target_language, "Python".to_string(), "🐍 Python");
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.optimization_enabled, "Enable Optimizations");
                        ui.label("(constant folding, dead code elimination)");
                    });

                    ui.add_space(10.0);

                    // Real-time syntax validation
                    ui.horizontal(|ui| {
                        if ui.button("🔍 Validate Syntax").clicked() {
                            self.dsl_editor.validate_syntax();
                        }

                        // Transpile button
                        if ui.add(egui::Button::new("🚀 Transpile").min_size(egui::vec2(100.0, 30.0))).clicked() {
                            self.transpile_expression();
                        }
                    });
                });
            });

            ui.separator();

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("💻 Generated Code");
                    ui.add_space(10.0);

                    // Output area
                    ui.horizontal(|ui| {
                        ui.label(format!("Output ({})", self.target_language));
                        if ui.button("📋 Copy").clicked() {
                            ui.ctx().copy_text(self.transpiler_output.clone());
                        }
                    });

                    let output_color = if self.transpiler_error.is_some() {
                        egui::Color32::LIGHT_RED
                    } else {
                        ui.visuals().text_color()
                    };

                    // Enhanced output editor with syntax highlighting
                    ui.add(&mut self.output_editor);

                    // Error display
                    if let Some(error) = &self.transpiler_error {
                        ui.add_space(10.0);
                        ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
                    }

                    ui.add_space(10.0);

                    // Info panel
                    ui.collapsing("ℹ️ Transpiler Features", |ui| {
                        ui.label("🔧 Optimizations:");
                        ui.label("  • Constant folding (2 + 3 → 5)");
                        ui.label("  • Dead code elimination");
                        ui.label("  • Function inlining");
                        ui.separator();
                        ui.label("🎯 Target Languages:");
                        ui.label("  • Rust: Direct Value enum mapping");
                        ui.label("  • SQL: CASE/WHEN expressions");
                        ui.label("  • JavaScript: Ternary operators");
                        ui.label("  • Python: Native expressions");
                        ui.separator();
                        ui.label("✅ Supported Features:");
                        ui.label("  • Arithmetic operations");
                        ui.label("  • Logical operations");
                        ui.label("  • Function calls");
                        ui.label("  • Conditional expressions");
                        ui.label("  • String operations");
                    });
                });
            });
        });
    }

    fn transpile_expression(&mut self) {
        self.transpiler_error = None;

        if self.transpiler_input.trim().is_empty() {
            self.transpiler_error = Some("Input expression is empty".to_string());
            self.transpiler_output = String::new();
            return;
        }

        // Parse the expression
        match parser::parse_expression(&self.transpiler_input) {
            Ok((_, ast)) => {
                // Determine target language
                let target = match self.target_language.as_str() {
                    "Rust" => TargetLanguage::Rust,
                    "SQL" => TargetLanguage::SQL,
                    "JavaScript" => TargetLanguage::JavaScript,
                    "Python" => TargetLanguage::Python,
                    _ => TargetLanguage::Rust,
                };

                // Create transpiler with options
                let options = TranspilerOptions {
                    target,
                    optimize: self.optimization_enabled,
                    inline_functions: self.optimization_enabled,
                    constant_folding: self.optimization_enabled,
                    dead_code_elimination: self.optimization_enabled,
                };

                let transpiler = Transpiler::new(options);

                // Transpile
                match transpiler.transpile(&ast) {
                    Ok(code) => {
                        self.transpiler_output = code.clone();
                        self.output_editor.text = code;

                        // Update output editor language based on target
                        self.output_editor.language = match self.target_language.as_str() {
                            "Rust" => DslLanguage::Rust,
                            "SQL" => DslLanguage::Sql,
                            "JavaScript" => DslLanguage::JavaScript,
                            "Python" => DslLanguage::Python,
                            _ => DslLanguage::Rust,
                        };
                    }
                    Err(e) => {
                        self.transpiler_error = Some(e.to_string());
                        self.transpiler_output = String::new();
                        self.output_editor.text = String::new();

                        // Trigger AI-powered error analysis
                        self.trigger_error_analysis(&e.to_string());
                    }
                }
            }
            Err(e) => {
                self.transpiler_error = Some(format!("Parse error: {}", e));
                self.transpiler_output = String::new();
            }
        }
    }

    fn show_ai_assistant_panel(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.heading("🤖 AI Assistant");
                ui.add_space(10.0);

                // Provider selection
                ui.horizontal(|ui| {
                    ui.label("Provider:");
                    egui::ComboBox::from_id_salt("ai_provider")
                        .selected_text(&self.ai_assistant.get_provider_status())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.ai_assistant.provider, AiProvider::OpenAI { api_key: None }, "🔮 OpenAI");
                            ui.selectable_value(&mut self.ai_assistant.provider, AiProvider::Anthropic { api_key: None }, "🧠 Anthropic");
                            ui.selectable_value(&mut self.ai_assistant.provider, AiProvider::Offline, "🔒 Offline");
                        });
                });

                ui.add_space(10.0);

                // Query input
                ui.horizontal(|ui| {
                    ui.label("Ask AI:");
                    if ui.button("🧹 Clear").clicked() {
                        self.ai_query.clear();
                    }
                });

                ui.add(egui::TextEdit::multiline(&mut self.ai_query)
                    .hint_text("Ask for help with DSL syntax, patterns, or optimizations...")
                    .desired_rows(3));

                ui.add_space(5.0);

                // Generate suggestion button
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("✨ Get Suggestions").min_size(egui::vec2(120.0, 25.0))).clicked() {
                        self.generate_ai_suggestions();
                    }

                    if ui.button("🔄 Refresh Context").clicked() {
                        self.update_ai_context();
                    }
                });

                ui.add_space(10.0);

                // Context display
                ui.collapsing("📋 Current Context", |ui| {
                    ui.label(format!("Input: {}", if self.ai_assistant.context.current_rule.is_empty() {
                        "No input provided"
                    } else {
                        &self.ai_assistant.context.current_rule
                    }));

                    ui.label(format!("Target: {}", if self.ai_assistant.context.target_language.is_empty() {
                        "Not set"
                    } else {
                        &self.ai_assistant.context.target_language
                    }));

                    if !self.ai_assistant.context.available_attributes.is_empty() {
                        ui.label(format!("Attributes: {} available", self.ai_assistant.context.available_attributes.len()));
                    }

                    if !self.ai_assistant.context.recent_errors.is_empty() {
                        ui.label(format!("Recent errors: {}", self.ai_assistant.context.recent_errors.len()));
                    }
                });

                ui.add_space(10.0);

                // AI Suggestions display
                ui.separator();
                ui.label("💡 AI Suggestions:");

                let mut apply_suggestion_index = None;

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        if self.ai_suggestions.is_empty() {
                            ui.label("No suggestions yet. Ask a question or request help!");
                        } else {
                            for (i, suggestion) in self.ai_suggestions.iter().enumerate() {
                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                        // Suggestion header
                                        ui.horizontal(|ui| {
                                            let icon = match suggestion.suggestion_type {
                                                SuggestionType::CodeCompletion => "💻",
                                                SuggestionType::ErrorFix => "🔧",
                                                SuggestionType::Optimization => "⚡",
                                                SuggestionType::Alternative => "🔄",
                                                SuggestionType::Documentation => "📖",
                                                SuggestionType::FunctionUsage => "🔧",
                                                SuggestionType::BestPractice => "⭐",
                                                SuggestionType::SimilarPattern => "🔍",
                                                SuggestionType::PatternMatch => "🎯",
                                                SuggestionType::AutoComplete => "🏃",
                                                SuggestionType::SnippetCompletion => "📝",
                                                SuggestionType::ErrorAnalysis => "🔍",
                                                SuggestionType::QuickFix => "⚡",
                                            };
                                            ui.label(format!("{} {}", icon, suggestion.title));

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("📋 Apply").clicked() {
                                                    apply_suggestion_index = Some(i);
                                                }
                                                ui.label(format!("Confidence: {:.0}%", suggestion.confidence * 100.0));
                                            });
                                        });

                                        // Suggestion content
                                        ui.label(&suggestion.description);

                                        if let Some(code) = &suggestion.code_snippet {
                                            ui.add_space(5.0);
                                            ui.monospace(code);
                                        }
                                    });
                                });
                                ui.add_space(5.0);
                            }
                        }
                    });

                // Apply suggestion outside the borrow
                if let Some(index) = apply_suggestion_index {
                    self.apply_ai_suggestion(index);
                }

                ui.add_space(10.0);

                // Quick actions
                ui.separator();
                ui.label("🚀 Quick Actions:");
                ui.horizontal_wrapped(|ui| {
                    if ui.small_button("🔍 Analyze Input").clicked() {
                        self.ai_query = "Analyze my current DSL expression and suggest improvements".to_string();
                        self.generate_ai_suggestions();
                    }
                    if ui.small_button("💡 Suggest Optimizations").clicked() {
                        self.ai_query = "What optimizations can be applied to this expression?".to_string();
                        self.generate_ai_suggestions();
                    }
                    if ui.small_button("🐛 Debug Errors").clicked() {
                        self.ai_query = "Help me debug any errors in my expression".to_string();
                        self.generate_ai_suggestions();
                    }
                    if ui.small_button("📚 Explain Syntax").clicked() {
                        self.ai_query = "Explain the DSL syntax I'm using".to_string();
                        self.generate_ai_suggestions();
                    }
                });
            });
        });
    }

    fn generate_ai_suggestions(&mut self) {
        // Update context with current state
        self.update_ai_context();

        // Use offline suggestions for now (can be enhanced later for async RAG)
        let query = self.ai_query.clone();
        let suggestions = self.ai_assistant.get_offline_suggestions(&query);

        // Add manual RAG suggestions if database is available
        if let Some(_) = &self.db_pool {
            // Note: RAG suggestions require async, which would need a different architecture
            // For now, we'll use the offline suggestions and enhance later
        }

        self.ai_suggestions = suggestions;
    }

    fn should_trigger_completion(&self) -> bool {
        // Trigger completion on certain characters or after a delay
        let text = &self.dsl_editor.text;
        let cursor = self.dsl_editor.cursor_position;

        if cursor == 0 || cursor > text.len() {
            return false;
        }

        // Get the character before cursor
        let chars: Vec<char> = text.chars().collect();
        if cursor > 0 && cursor <= chars.len() {
            let prev_char = chars[cursor - 1];
            // Trigger on letters, dots, and after spaces following keywords
            prev_char.is_alphabetic() || prev_char == '.' || prev_char == '_'
        } else {
            false
        }
    }

    fn trigger_code_completion(&mut self) {
        // Update AI context for real-time completion
        self.update_ai_context();

        // Get intelligent code completions based on cursor position
        let completions = self.ai_assistant.get_code_completions(
            &self.dsl_editor.text,
            self.dsl_editor.cursor_position
        );

        // Mix completion suggestions with existing suggestions
        if !completions.is_empty() {
            // Keep existing suggestions but prioritize completions
            let mut mixed_suggestions = completions;

            // Add some existing suggestions if we have room
            for suggestion in &self.ai_suggestions {
                if mixed_suggestions.len() < 10 &&
                   !mixed_suggestions.iter().any(|s| s.title == suggestion.title) {
                    mixed_suggestions.push(suggestion.clone());
                }
            }

            self.ai_suggestions = mixed_suggestions;
        }
    }

    fn update_ai_context(&mut self) {
        // Update AI context with current application state
        self.ai_assistant.context.current_rule = self.transpiler_input.clone();

        // Set target language
        self.ai_assistant.context.target_language = self.target_language.clone();

        // Add recent error if present
        if let Some(error) = &self.transpiler_error {
            self.ai_assistant.context.recent_errors.push(error.clone());
            // Keep only last 5 errors
            if self.ai_assistant.context.recent_errors.len() > 5 {
                self.ai_assistant.context.recent_errors.remove(0);
            }
        }

        // Add available attributes from dictionary
        if let Some(dictionary) = &self.data_dictionary {
            let mut attributes = Vec::new();
            for attr in &dictionary.attributes {
                if let Some(attr_name) = attr.get("attribute_name").and_then(|v| v.as_str()) {
                    if let Some(entity_name) = attr.get("entity_name").and_then(|v| v.as_str()) {
                        attributes.push(format!("{}.{}", entity_name, attr_name));
                    } else {
                        attributes.push(attr_name.to_string());
                    }
                }
            }
            self.ai_assistant.context.available_attributes = attributes;
        }
    }

    fn apply_ai_suggestion(&mut self, suggestion_index: usize) {
        if let Some(suggestion) = self.ai_suggestions.get(suggestion_index) {
            match suggestion.suggestion_type {
                SuggestionType::CodeCompletion | SuggestionType::Alternative |
                SuggestionType::AutoComplete | SuggestionType::SnippetCompletion => {
                    // Replace current input with suggested code
                    if let Some(code) = &suggestion.code_snippet {
                        self.transpiler_input = code.clone();
                        self.dsl_editor.text = code.clone();

                        // Auto-transpile the new code
                        self.transpile_expression();
                    }
                }
                SuggestionType::ErrorFix | SuggestionType::QuickFix => {
                    // Apply the fix to current input
                    if let Some(code) = &suggestion.code_snippet {
                        self.transpiler_input = code.clone();
                        self.dsl_editor.text = code.clone();
                        self.transpile_expression();
                    }
                }
                SuggestionType::ErrorAnalysis => {
                    // For error analysis, just display the information
                    // The analysis text is already shown in the UI
                }
                SuggestionType::Optimization => {
                    // Apply optimization and retranspile
                    if let Some(code) = &suggestion.code_snippet {
                        self.transpiler_input = code.clone();
                        self.dsl_editor.text = code.clone();
                        self.optimization_enabled = true; // Enable optimizations
                        self.transpile_expression();
                    }
                }
                _ => {
                    // For explanations, just show in a popup or status
                    // Note: Clipboard access requires UI context, handled in the UI layer
                }
            }
        }
    }

    fn trigger_error_analysis(&mut self, error_message: &str) {
        let suggestions = self.ai_assistant.analyze_error(error_message, &self.transpiler_input);

        // Add error analysis suggestions to the AI suggestions list
        for suggestion in suggestions {
            self.ai_suggestions.push(suggestion);
        }

        // Ensure AI panel is visible
        self.show_ai_panel = true;
    }

}