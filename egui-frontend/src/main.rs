use eframe::egui;
use data_designer_core::db::{
    init_db, DbPool,
    ClientBusinessUnit, CreateCbuRequest, CbuMemberDetail,
    DbOperations, DataDictionaryResponse, EmbeddingOperations, SimilarRule
};

mod code_editor;
use code_editor::{DslCodeEditor, DslLanguage};

mod ai_assistant;
use ai_assistant::{AiAssistant, AiProvider, AiSuggestion, SuggestionType};
use data_designer_core::{parser, evaluator, models::{Value, DataDictionary, ViewerState}, transpiler::{Transpiler, TranspilerOptions, TargetLanguage, DslTranspiler, DslRule, TranspileError}};
use std::collections::HashMap;
use tokio::runtime::Runtime;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use rust_decimal::prelude::*;
use sqlx::Row;

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

    // Attribute Filters
    filter_business: bool,
    filter_derived: bool,
    filter_system: bool,
    filter_required: bool,
    filter_optional: bool,
    filter_key: bool,

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

    // Enhanced DSL Transpiler
    dsl_transpiler: DslTranspiler,
    parsed_rules: Vec<DslRule>,
    transpile_errors: Vec<TranspileError>,
    multi_rule_mode: bool,
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

    // CBU Expansion State
    expanded_cbus: std::collections::HashSet<String>, // Set of expanded CBU IDs
    cbu_members: std::collections::HashMap<String, Vec<CbuMemberDetail>>, // CBU ID -> Members

    // Taxonomy Data
    products: Vec<Product>,
    product_options: Vec<ProductOption>,
    services: Vec<Service>,
    resources: Vec<ResourceObject>,
    service_resource_hierarchy: Vec<ServiceResourceHierarchy>,
    investment_mandates: Vec<InvestmentMandate>,
    mandate_instruments: Vec<MandateInstrument>,
    instruction_formats: Vec<InstructionFormat>,
    cbu_mandate_structure: Vec<CbuInvestmentMandateStructure>,
    cbu_member_roles: Vec<CbuMemberInvestmentRole>,
    taxonomy_hierarchy: Vec<TaxonomyHierarchyItem>,
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
    // New taxonomy tabs
    ProductTaxonomy,   // Products → Options → Services → Resources
    InvestmentMandates, // Investment mandates with CBU roles
    TaxonomyHierarchy, // Complete hierarchy view
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

// Taxonomy Database Structs
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: i32,
    product_id: String,
    product_name: String,
    line_of_business: String,
    description: Option<String>,
    status: String,
    contract_type: Option<String>,
    commercial_status: Option<String>,
    pricing_model: Option<String>,
    target_market: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProductOption {
    id: i32,
    option_id: String,
    product_id: i32,
    option_name: String,
    option_category: String,
    option_type: String,
    option_value: serde_json::Value,
    display_name: Option<String>,
    description: Option<String>,
    pricing_impact: Option<rust_decimal::Decimal>,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Service {
    id: i32,
    service_id: String,
    service_name: String,
    service_category: Option<String>,
    description: Option<String>,
    service_type: Option<String>,
    delivery_model: Option<String>,
    billable: Option<bool>,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InvestmentMandate {
    mandate_id: String,
    cbu_id: String,
    asset_owner_name: String,
    asset_owner_lei: String,
    investment_manager_name: String,
    investment_manager_lei: String,
    base_currency: String,
    effective_date: String,
    expiry_date: Option<String>,
    gross_exposure_pct: Option<f64>,
    net_exposure_pct: Option<f64>,
    leverage_max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MandateInstrument {
    id: i32,
    mandate_id: String,
    instrument_family: String,
    subtype: Option<String>,
    cfi_code: Option<String>,
    exposure_pct: Option<f64>,
    short_allowed: Option<bool>,
    issuer_max_pct: Option<f64>,
    rating_floor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstructionFormat {
    id: i32,
    format_id: String,
    format_name: String,
    format_category: Option<String>,
    message_standard: Option<String>,
    message_type: Option<String>,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CbuInvestmentMandateStructure {
    cbu_id: String,
    cbu_name: String,
    mandate_id: Option<String>,
    asset_owner_name: Option<String>,
    investment_manager_name: Option<String>,
    base_currency: Option<String>,
    total_instruments: Option<i64>,
    families: Option<String>,
    total_exposure_pct: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CbuMemberInvestmentRole {
    cbu_id: String,
    cbu_name: String,
    entity_name: String,
    entity_lei: Option<String>,
    role_name: String,
    role_code: String,
    investment_responsibility: String,
    mandate_id: Option<String>,
    has_trading_authority: Option<bool>,
    has_settlement_authority: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResourceObject {
    id: i32,
    resource_name: String,
    description: Option<String>,
    version: String,
    category: Option<String>,
    resource_type: Option<String>,
    criticality_level: Option<String>,
    operational_status: Option<String>,
    owner_team: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceResourceHierarchy {
    // Service Information
    service_id: i32,
    service_code: String,
    service_name: String,
    service_category: Option<String>,
    service_type: Option<String>,
    delivery_model: Option<String>,
    billable: Option<bool>,
    service_description: Option<String>,
    service_status: Option<String>,

    // Resource Information
    resource_id: i32,
    resource_name: String,
    resource_description: Option<String>,
    resource_version: String,
    resource_category: Option<String>,
    resource_type: Option<String>,
    criticality_level: Option<String>,
    operational_status: Option<String>,
    owner_team: Option<String>,

    // Mapping Details
    usage_type: String,
    resource_role: Option<String>,
    cost_allocation_percentage: Option<rust_decimal::Decimal>,
    dependency_level: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaxonomyHierarchyItem {
    level: i32,
    item_type: String,
    item_id: i32,
    item_name: String,
    item_description: Option<String>,
    parent_id: Option<i32>,
    configuration: Option<serde_json::Value>,
    metadata: Option<serde_json::Value>,
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

            // Initialize attribute filters (all enabled by default)
            filter_business: true,
            filter_derived: true,
            filter_system: true,
            filter_required: true,
            filter_optional: true,
            filter_key: true,
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

            // Enhanced DSL Transpiler
            dsl_transpiler: DslTranspiler::new(),
            parsed_rules: Vec::new(),
            transpile_errors: Vec::new(),
            multi_rule_mode: false,

            ai_assistant,
            ai_suggestions: Vec::new(),
            ai_query: String::new(),
            show_ai_panel: true,
            ai_loading: false,
            show_cbu_form: false,
            cbu_form: CbuForm::default(),
            status_message: "Initializing...".to_string(),
            loading: false,
            expanded_cbus: std::collections::HashSet::new(),
            cbu_members: std::collections::HashMap::new(),

            // Initialize taxonomy data
            products: Vec::new(),
            product_options: Vec::new(),
            services: Vec::new(),
            resources: Vec::new(),
            service_resource_hierarchy: Vec::new(),
            investment_mandates: Vec::new(),
            mandate_instruments: Vec::new(),
            instruction_formats: Vec::new(),
            cbu_mandate_structure: Vec::new(),
            cbu_member_roles: Vec::new(),
            taxonomy_hierarchy: Vec::new(),
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

    fn load_cbu_members(&mut self, cbu_id: &str) {
        if let Some(ref _pool) = self.db_pool {
            let rt = self.runtime.clone();
            let cbu_id_owned = cbu_id.to_string();

            match rt.block_on(async {
                DbOperations::get_cbu_members(&cbu_id_owned).await
            }) {
                Ok(members) => {
                    self.cbu_members.insert(cbu_id_owned, members);
                    self.status_message = format!("✅ Loaded members for CBU {}", cbu_id);
                }
                Err(e) => {
                    eprintln!("Failed to load CBU members: {}", e);
                    self.status_message = format!("❌ Failed to load members: {}", e);
                }
            }
        }
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
                ui.separator();
                ui.selectable_value(&mut self.current_tab, Tab::ProductTaxonomy, "📦 Product Taxonomy");
                ui.selectable_value(&mut self.current_tab, Tab::InvestmentMandates, "🎯 Investment Mandates");
                ui.selectable_value(&mut self.current_tab, Tab::TaxonomyHierarchy, "🏗️ Taxonomy Hierarchy");
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
                Tab::ProductTaxonomy => self.show_product_taxonomy_tab(ui),
                Tab::InvestmentMandates => self.show_investment_mandates_tab(ui),
                Tab::TaxonomyHierarchy => self.show_taxonomy_hierarchy_tab(ui),
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

        // Expandable CBU list
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Clone the CBUs to avoid borrowing issues
            let cbus_list = self.cbus.clone();
            let mut cbus_to_expand: Option<String> = None;
            let mut cbus_to_collapse: Option<String> = None;

            for cbu in cbus_list.iter() {
                let is_expanded = self.expanded_cbus.contains(&cbu.cbu_id);

                ui.push_id(&cbu.cbu_id, |ui| {
                    // CBU header row
                    ui.horizontal(|ui| {
                        // Expand/collapse button
                        let expand_button = if is_expanded { "🔽" } else { "▶️" };
                        if ui.button(expand_button).clicked() {
                            if is_expanded {
                                cbus_to_collapse = Some(cbu.cbu_id.clone());
                            } else {
                                cbus_to_expand = Some(cbu.cbu_id.clone());
                            }
                        }

                        // CBU basic info
                        ui.separator();
                        ui.label(format!("🏢 {}", cbu.cbu_name));
                        ui.separator();
                        ui.label(format!("ID: {}", cbu.cbu_id));
                        ui.separator();
                        ui.label(format!("Status: {}", cbu.status));
                        ui.separator();
                        ui.label(format!("Type: {}", cbu.business_type.as_ref().unwrap_or(&"N/A".to_string())));
                    });

                    // Description row
                    if let Some(description) = &cbu.description {
                        ui.indent("desc", |ui| {
                            ui.label(format!("📝 {}", description));
                        });
                    }

                    // Expanded members section
                    if is_expanded {
                        ui.separator();

                        if let Some(members) = self.cbu_members.get(&cbu.cbu_id) {
                            ui.indent("members", |ui| {
                                ui.heading("👥 Entity Members & Roles");

                                if members.is_empty() {
                                    ui.label("No members found for this CBU");
                                } else {
                                    // Members table
                                    egui::Grid::new(format!("members_grid_{}", cbu.cbu_id))
                                        .striped(true)
                                        .show(ui, |ui| {
                                            // Header
                                            ui.label("🏗️ Entity");
                                            ui.label("🎭 Role");
                                            ui.label("🆔 LEI");
                                            ui.label("📧 Contact");
                                            ui.label("⭐ Primary");
                                            ui.label("🛡️ Authority");
                                            ui.end_row();

                                            // Member rows
                                            for member in members {
                                                ui.label(&member.entity_name);
                                                ui.label(&member.role_name);
                                                ui.label(member.entity_lei.as_ref().unwrap_or(&"N/A".to_string()));
                                                ui.label(member.contact_email.as_ref().unwrap_or(&"N/A".to_string()));
                                                ui.label(if member.is_primary { "⭐ Yes" } else { "No" });

                                                let authority = match (member.has_trading_authority, member.has_settlement_authority) {
                                                    (true, true) => "Trading + Settlement",
                                                    (true, false) => "Trading Only",
                                                    (false, true) => "Settlement Only",
                                                    (false, false) => "None"
                                                };
                                                ui.label(authority);
                                                ui.end_row();
                                            }
                                        });
                                }
                            });
                        } else {
                            ui.indent("loading", |ui| {
                                ui.label("🔄 Loading entity members...");
                            });
                        }
                    }

                    ui.separator();
                });
            }

            // Handle expansion/collapse after the loop to avoid borrowing issues
            if let Some(cbu_id) = cbus_to_expand {
                self.expanded_cbus.insert(cbu_id.clone());
                // Load members if not already loaded
                if !self.cbu_members.contains_key(&cbu_id) {
                    self.load_cbu_members(&cbu_id);
                }
            }
            if let Some(cbu_id) = cbus_to_collapse {
                self.expanded_cbus.remove(&cbu_id);
            }
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
            // Statistics row
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

            // Filter buttons
            ui.label("🔍 Filters:");
            ui.horizontal_wrapped(|ui| {
                // Attribute type filters
                let business_button = ui.selectable_label(self.filter_business, "🏢 Business");
                if business_button.clicked() {
                    self.filter_business = !self.filter_business;
                }

                let derived_button = ui.selectable_label(self.filter_derived, "⚙️ Derived");
                if derived_button.clicked() {
                    self.filter_derived = !self.filter_derived;
                }

                let system_button = ui.selectable_label(self.filter_system, "🔧 System");
                if system_button.clicked() {
                    self.filter_system = !self.filter_system;
                }

                ui.separator();

                // Nullability filters
                let required_button = ui.selectable_label(self.filter_required, "🔒 Required");
                if required_button.clicked() {
                    self.filter_required = !self.filter_required;
                }

                let optional_button = ui.selectable_label(self.filter_optional, "📋 Optional");
                if optional_button.clicked() {
                    self.filter_optional = !self.filter_optional;
                }

                ui.separator();

                // Key filter
                let key_button = ui.selectable_label(self.filter_key, "🔑 Keys");
                if key_button.clicked() {
                    self.filter_key = !self.filter_key;
                }

                ui.separator();

                // Clear all / Select all buttons
                if ui.button("🚫 Clear All").clicked() {
                    self.filter_business = false;
                    self.filter_derived = false;
                    self.filter_system = false;
                    self.filter_required = false;
                    self.filter_optional = false;
                    self.filter_key = false;
                }

                if ui.button("✅ Select All").clicked() {
                    self.filter_business = true;
                    self.filter_derived = true;
                    self.filter_system = true;
                    self.filter_required = true;
                    self.filter_optional = true;
                    self.filter_key = true;
                }
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

                    // Filter and display attributes
                    let mut displayed_count = 0;
                    for attr in &dictionary.attributes {
                        let attr_type = attr.get("attribute_type").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let is_nullable = attr.get("is_nullable").and_then(|v| v.as_bool()).unwrap_or(false);
                        let is_key = attr.get("is_key").and_then(|v| v.as_bool()).unwrap_or(false);

                        // Apply filters
                        let mut show_attribute = false;

                        // Check attribute type filters
                        match attr_type {
                            "business" if self.filter_business => show_attribute = true,
                            "derived" if self.filter_derived => show_attribute = true,
                            "system" if self.filter_system => show_attribute = true,
                            _ => {}
                        }

                        // If attribute type doesn't match, skip regardless of other filters
                        if !show_attribute {
                            continue;
                        }

                        // Apply nullability filters (both required and optional must be checked for non-nullable and nullable respectively)
                        let nullability_matches =
                            (!is_nullable && self.filter_required) ||
                            (is_nullable && self.filter_optional);

                        if !nullability_matches {
                            continue;
                        }

                        // Apply key filter - when disabled, only show non-key attributes
                        if !self.filter_key && is_key {
                            continue;
                        }

                        // If we get here, show the attribute
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
                        ui.label(if is_key { "🔑" } else { "" });
                        ui.label(if is_nullable { "✓" } else { "✗" });
                        ui.end_row();

                        displayed_count += 1;
                    }

                    // Show count of displayed vs total attributes
                    if displayed_count < dictionary.attributes.len() {
                        ui.end_row();
                        ui.colored_label(egui::Color32::GRAY, format!("Showing {} of {} attributes", displayed_count, dictionary.attributes.len()));
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.label("");
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

                    // Input area with mode controls
                    ui.horizontal(|ui| {
                        ui.label("DSL Rule:");
                        if ui.button("🔄 Clear").clicked() {
                            self.transpiler_input.clear();
                            self.dsl_editor.text.clear();
                        }
                        ui.separator();
                        ui.checkbox(&mut self.multi_rule_mode, "🔀 Multi-Rule Mode");
                        if ui.small_button("❓").on_hover_text("Enable to parse multiple rules separated by line breaks").clicked() {
                            // Help clicked
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

                        // Show detailed DSL parsing errors if available
                        if !self.transpile_errors.is_empty() {
                            ui.add_space(5.0);
                            ui.collapsing("🔍 Detailed Error Report", |ui| {
                                for (i, parse_error) in self.transpile_errors.iter().enumerate() {
                                    ui.separator();
                                    ui.label(format!("Error {}:", i + 1));
                                    ui.colored_label(egui::Color32::LIGHT_RED, &parse_error.message);

                                    if let Some(line) = parse_error.line {
                                        ui.label(format!("Line: {}", line));
                                    }
                                    if let Some(column) = parse_error.column {
                                        ui.label(format!("Column: {}", column));
                                    }
                                    if let Some(rule_name) = &parse_error.rule_name {
                                        ui.label(format!("Rule: {}", rule_name));
                                    }
                                    ui.label(format!("Type: {:?}", parse_error.error_type));
                                }
                            });
                        }

                        // Show successfully parsed rules if any
                        if !self.parsed_rules.is_empty() {
                            ui.add_space(5.0);
                            ui.collapsing("✅ Successfully Parsed Rules", |ui| {
                                for (i, rule) in self.parsed_rules.iter().enumerate() {
                                    ui.separator();
                                    ui.label(format!("Rule {}: {}", i + 1, rule.name));
                                    ui.label(format!("  Line: {}", rule.line_number));
                                    if !rule.dependencies.is_empty() {
                                        ui.label(format!("  Dependencies: {}", rule.dependencies.join(", ")));
                                    }
                                }
                            });
                        }
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
        self.transpile_errors.clear();
        self.parsed_rules.clear();

        if self.transpiler_input.trim().is_empty() {
            self.transpiler_error = Some("Input expression is empty".to_string());
            self.transpiler_output = String::new();
            return;
        }

        if self.multi_rule_mode {
            // Use enhanced DSL transpiler for multi-rule mode
            match self.dsl_transpiler.transpile_dsl_to_rules(&self.transpiler_input) {
                Ok(rules) => {
                    self.parsed_rules = rules;

                    // Generate summary output
                    let mut output = String::new();
                    output.push_str(&format!("Successfully parsed {} rule(s):\n\n", self.parsed_rules.len()));

                    for (i, rule) in self.parsed_rules.iter().enumerate() {
                        output.push_str(&format!("Rule {}: {}\n", i + 1, rule.name));
                        output.push_str(&format!("  Definition: {}\n", rule.definition));
                        if !rule.dependencies.is_empty() {
                            output.push_str(&format!("  Dependencies: {}\n", rule.dependencies.join(", ")));
                        }
                        output.push_str("\n");
                    }

                    self.transpiler_output = output;
                    self.output_editor.text = self.transpiler_output.clone();
                    self.output_editor.language = DslLanguage::Rust; // Summary is text
                }
                Err(errors) => {
                    self.transpile_errors = errors;

                    // Generate error report
                    let mut error_output = String::new();
                    error_output.push_str(&format!("Found {} error(s):\n\n", self.transpile_errors.len()));

                    for error in &self.transpile_errors {
                        error_output.push_str(&format!("{}\n", error));
                    }

                    self.transpiler_error = Some(format!("DSL parsing failed with {} errors", self.transpile_errors.len()));
                    self.transpiler_output = error_output;
                    self.output_editor.text = self.transpiler_output.clone();

                    // Trigger AI error analysis for first error
                    if let Some(first_error) = self.transpile_errors.first() {
                        let error_message = first_error.message.clone();
                        self.trigger_error_analysis(&error_message);
                    }
                }
            }
        } else {
            // Single expression mode - use original transpiler
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

    // New Taxonomy UI Methods
    fn show_product_taxonomy_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📦 Product Taxonomy");
        ui.separator();

        // Refresh button
        if ui.button("🔄 Refresh Data").clicked() {
            self.load_product_taxonomy();
        }

        ui.separator();

        // Products section
        ui.collapsing("🏪 Products", |ui| {
            if self.products.is_empty() {
                ui.label("No products loaded. Click refresh to load data.");
            } else {
                for product in &self.products {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&product.product_name);
                            ui.label(format!("({})", product.product_id));
                        });
                        ui.label(format!("Business: {}", product.line_of_business));
                        if let Some(desc) = &product.description {
                            ui.label(format!("Description: {}", desc));
                        }
                        ui.horizontal(|ui| {
                            ui.label("Status:");
                            match product.status.as_str() {
                                "active" => ui.colored_label(egui::Color32::GREEN, "✅ Active"),
                                _ => ui.colored_label(egui::Color32::YELLOW, &product.status),
                            };
                        });
                    });
                }
            }
        });

        // Product Options section
        ui.collapsing("⚙️ Product Options", |ui| {
            if self.product_options.is_empty() {
                ui.label("No product options loaded.");
            } else {
                for option in &self.product_options {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&option.option_name);
                            ui.label(format!("({})", option.option_category));
                        });
                        if let Some(display_name) = &option.display_name {
                            ui.label(format!("Display: {}", display_name));
                        }
                        if let Some(pricing_impact) = option.pricing_impact {
                            ui.label(format!("Pricing Impact: ${:.2}", pricing_impact.to_f64().unwrap_or(0.0)));
                        }
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            match option.option_type.as_str() {
                                "required" => ui.colored_label(egui::Color32::RED, "Required"),
                                "optional" => ui.colored_label(egui::Color32::BLUE, "Optional"),
                                "premium" => ui.colored_label(egui::Color32::GOLD, "Premium"),
                                _ => ui.label(&option.option_type),
                            };
                        });
                    });
                }
            }
        });

        // Services section
        ui.collapsing("🔧 Services", |ui| {
            if self.services.is_empty() {
                ui.label("No services loaded.");
            } else {
                for service in &self.services {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&service.service_name);
                            if let Some(category) = &service.service_category {
                                ui.label(format!("({})", category));
                            }
                        });
                        if let Some(service_type) = &service.service_type {
                            ui.label(format!("Type: {}", service_type));
                        }
                        if let Some(delivery_model) = &service.delivery_model {
                            ui.label(format!("Delivery: {}", delivery_model));
                        }
                        if let Some(billable) = service.billable {
                            ui.horizontal(|ui| {
                                ui.label("Billable:");
                                if billable {
                                    ui.colored_label(egui::Color32::GREEN, "✅ Yes");
                                } else {
                                    ui.colored_label(egui::Color32::GRAY, "❌ No");
                                }
                            });
                        }
                    });
                }
            }
        });

        // Resources section
        ui.collapsing("💻 Resources", |ui| {
            if self.resources.is_empty() {
                ui.label("No resources loaded.");
            } else {
                for resource in &self.resources {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&resource.resource_name);
                            ui.label(format!("v{}", resource.version));
                        });

                        if let Some(description) = &resource.description {
                            ui.label(format!("Description: {}", description));
                        }

                        if let Some(resource_type) = &resource.resource_type {
                            ui.horizontal(|ui| {
                                ui.label("Type:");
                                ui.strong(resource_type);
                            });
                        }

                        if let Some(category) = &resource.category {
                            ui.label(format!("Category: {}", category));
                        }

                        ui.horizontal(|ui| {
                            if let Some(criticality) = &resource.criticality_level {
                                ui.label("Criticality:");
                                match criticality.as_str() {
                                    "high" => ui.colored_label(egui::Color32::RED, "🔴 High"),
                                    "medium" => ui.colored_label(egui::Color32::YELLOW, "🟡 Medium"),
                                    "low" => ui.colored_label(egui::Color32::GREEN, "🟢 Low"),
                                    _ => ui.label(criticality),
                                };
                            }

                            if let Some(status) = &resource.operational_status {
                                ui.label("Status:");
                                match status.as_str() {
                                    "active" => ui.colored_label(egui::Color32::GREEN, "✅ Active"),
                                    "maintenance" => ui.colored_label(egui::Color32::YELLOW, "🔧 Maintenance"),
                                    "deprecated" => ui.colored_label(egui::Color32::RED, "❌ Deprecated"),
                                    _ => ui.label(status),
                                };
                            }
                        });

                        if let Some(owner_team) = &resource.owner_team {
                            ui.label(format!("Owner Team: {}", owner_team));
                        }
                    });
                }
            }
        });

        // Service-Resource Hierarchy (1 Service : n Resources)
        ui.collapsing("🔗 Service → Resources Hierarchy", |ui| {
            if self.service_resource_hierarchy.is_empty() {
                ui.label("No service-resource mappings loaded.");
            } else {
                // Group by service
                let mut services: std::collections::HashMap<String, Vec<&ServiceResourceHierarchy>> = std::collections::HashMap::new();
                for hierarchy_item in &self.service_resource_hierarchy {
                    services.entry(hierarchy_item.service_name.clone())
                        .or_insert_with(Vec::new)
                        .push(hierarchy_item);
                }

                for (service_name, resources) in services {
                    ui.collapsing(format!("🔧 {} (→ {} resources)", service_name, resources.len()), |ui| {
                        // Service details
                        if let Some(first_resource) = resources.first() {
                            ui.horizontal(|ui| {
                                ui.label("Service Type:");
                                if let Some(service_type) = &first_resource.service_type {
                                    ui.strong(service_type);
                                }
                            });

                            if let Some(category) = &first_resource.service_category {
                                ui.label(format!("Category: {}", category));
                            }

                            if let Some(delivery) = &first_resource.delivery_model {
                                ui.label(format!("Delivery Model: {}", delivery));
                            }
                        }

                        ui.separator();
                        ui.label("📦 Connected Resources:");

                        // Sort by dependency level
                        let mut sorted_resources = resources;
                        sorted_resources.sort_by_key(|r| r.dependency_level.unwrap_or(999));

                        for resource in sorted_resources {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.strong(&resource.resource_name);
                                    ui.label(format!("(Level {})", resource.dependency_level.unwrap_or(0)));

                                    // Usage type badge
                                    match resource.usage_type.as_str() {
                                        "primary" => ui.colored_label(egui::Color32::BLUE, "🎯 Primary"),
                                        "secondary" => ui.colored_label(egui::Color32::GRAY, "🔄 Secondary"),
                                        "auxiliary" => ui.colored_label(egui::Color32::GREEN, "➕ Auxiliary"),
                                        _ => ui.label(&resource.usage_type),
                                    };
                                });

                                if let Some(role) = &resource.resource_role {
                                    ui.label(format!("Role: {}", role));
                                }

                                ui.horizontal(|ui| {
                                    if let Some(cost_pct) = &resource.cost_allocation_percentage {
                                        ui.label(format!("Cost Allocation: {:.1}%", cost_pct.to_f64().unwrap_or(0.0)));
                                    }

                                    if let Some(criticality) = &resource.criticality_level {
                                        ui.label("Criticality:");
                                        match criticality.as_str() {
                                            "high" => ui.colored_label(egui::Color32::RED, "🔴 High"),
                                            "medium" => ui.colored_label(egui::Color32::YELLOW, "🟡 Medium"),
                                            "low" => ui.colored_label(egui::Color32::GREEN, "🟢 Low"),
                                            _ => ui.label(criticality),
                                        };
                                    }
                                });

                                if let Some(owner) = &resource.owner_team {
                                    ui.label(format!("Owner: {}", owner));
                                }
                            });
                        }
                    });
                }
            }
        });
    }

    fn show_investment_mandates_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎯 Investment Mandates");
        ui.separator();

        // Refresh button
        if ui.button("🔄 Refresh Data").clicked() {
            self.load_investment_mandates();
        }

        ui.separator();

        // CBU Investment Mandate Structure
        ui.collapsing("🏢 CBU Investment Structure", |ui| {
            if self.cbu_mandate_structure.is_empty() {
                ui.label("No CBU mandate structure loaded. Click refresh to load data.");
            } else {
                for structure in &self.cbu_mandate_structure {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&structure.cbu_name);
                            ui.label(format!("({})", structure.cbu_id));
                        });

                        if let Some(mandate_id) = &structure.mandate_id {
                            ui.horizontal(|ui| {
                                ui.label("📋 Mandate:");
                                ui.strong(mandate_id);
                            });

                            if let Some(asset_owner) = &structure.asset_owner_name {
                                ui.horizontal(|ui| {
                                    ui.label("💰 Asset Owner:");
                                    ui.label(asset_owner);
                                });
                            }

                            if let Some(investment_manager) = &structure.investment_manager_name {
                                ui.horizontal(|ui| {
                                    ui.label("📊 Investment Manager:");
                                    ui.label(investment_manager);
                                });
                            }

                            if let Some(currency) = &structure.base_currency {
                                ui.horizontal(|ui| {
                                    ui.label("💱 Currency:");
                                    ui.label(currency);
                                });
                            }

                            if let Some(instruments) = structure.total_instruments {
                                ui.horizontal(|ui| {
                                    ui.label("🎪 Instruments:");
                                    ui.label(format!("{}", instruments));
                                });
                            }

                            if let Some(families) = &structure.families {
                                ui.horizontal(|ui| {
                                    ui.label("📁 Families:");
                                    ui.label(families);
                                });
                            }

                            if let Some(exposure) = structure.total_exposure_pct {
                                ui.horizontal(|ui| {
                                    ui.label("📈 Total Exposure:");
                                    ui.label(format!("{:.1}%", exposure.to_f64().unwrap_or(0.0)));
                                });
                            }
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "No mandate assigned");
                        }
                    });
                }
            }
        });

        // CBU Member Investment Roles
        ui.collapsing("👥 CBU Member Roles", |ui| {
            if self.cbu_member_roles.is_empty() {
                ui.label("No CBU member roles loaded.");
            } else {
                for role in &self.cbu_member_roles {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&role.entity_name);
                            ui.label(format!("({})", role.role_name));
                        });

                        ui.horizontal(|ui| {
                            ui.label("🏢 CBU:");
                            ui.label(&role.cbu_name);
                        });

                        ui.horizontal(|ui| {
                            ui.label("🎭 Role:");
                            ui.strong(&role.role_code);
                        });

                        ui.horizontal(|ui| {
                            ui.label("💼 Responsibility:");
                            ui.label(&role.investment_responsibility);
                        });

                        if let Some(mandate_id) = &role.mandate_id {
                            ui.horizontal(|ui| {
                                ui.label("📋 Mandate:");
                                ui.label(mandate_id);
                            });
                        }

                        ui.horizontal(|ui| {
                            ui.label("Authorities:");
                            if role.has_trading_authority.unwrap_or(false) {
                                ui.colored_label(egui::Color32::GREEN, "🔄 Trading");
                            }
                            if role.has_settlement_authority.unwrap_or(false) {
                                ui.colored_label(egui::Color32::BLUE, "💱 Settlement");
                            }
                        });
                    });
                }
            }
        });
    }

    fn show_taxonomy_hierarchy_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏗️ Taxonomy Hierarchy");
        ui.separator();

        ui.label("Complete Products → Options → Services → Resources hierarchy view");
        ui.separator();

        if ui.button("🔄 Load Sample Hierarchy").clicked() {
            self.load_taxonomy_hierarchy();
        }

        if !self.taxonomy_hierarchy.is_empty() {
            ui.collapsing("📊 Hierarchy Data", |ui| {
                for item in &self.taxonomy_hierarchy {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&item.item_name);
                            ui.label(format!("Level {}: {}", item.level, item.item_type));
                        });
                        if let Some(desc) = &item.item_description {
                            ui.label(desc);
                        }
                    });
                }
            });
        }
    }

    // Database Loading Methods
    fn load_product_taxonomy(&mut self) {
        if let Some(ref pool) = self.db_pool {
            let pool = pool.clone();
            let rt = self.runtime.clone();

            // Load products
            match rt.block_on(async {
                sqlx::query(
                    "SELECT id, product_id, product_name, line_of_business, description, status,
                     contract_type, commercial_status, pricing_model, target_market
                     FROM products WHERE status = 'active' ORDER BY product_name")
                .fetch_all(&pool)
                .await
            }) {
                Ok(rows) => {
                    self.products = rows.into_iter().map(|row| Product {
                        id: row.get("id"),
                        product_id: row.get("product_id"),
                        product_name: row.get("product_name"),
                        line_of_business: row.get("line_of_business"),
                        description: row.get("description"),
                        status: row.get("status"),
                        contract_type: row.get("contract_type"),
                        commercial_status: row.get("commercial_status"),
                        pricing_model: row.get("pricing_model"),
                        target_market: row.get("target_market"),
                    }).collect();
                    self.status_message = format!("Loaded {} products", self.products.len());
                }
                Err(e) => {
                    self.status_message = format!("Error loading products: {}", e);
                }
            }

            // Load product options
            match rt.block_on(async {
                sqlx::query(
                    "SELECT id, option_id, product_id, option_name, option_category, option_type,
                     option_value, display_name, description, pricing_impact, status
                     FROM product_options WHERE status = 'active' ORDER BY option_name")
                .fetch_all(&pool)
                .await
            }) {
                Ok(rows) => {
                    self.product_options = rows.into_iter().map(|row| ProductOption {
                        id: row.get("id"),
                        option_id: row.get("option_id"),
                        product_id: row.get("product_id"),
                        option_name: row.get("option_name"),
                        option_category: row.get("option_category"),
                        option_type: row.get("option_type"),
                        option_value: row.get("option_value"),
                        display_name: row.get("display_name"),
                        description: row.get("description"),
                        pricing_impact: row.get("pricing_impact"),
                        status: row.get("status"),
                    }).collect();
                }
                Err(e) => {
                    eprintln!("Error loading product options: {}", e);
                }
            }

            // Load services
            match rt.block_on(async {
                sqlx::query(
                    "SELECT id, service_id, service_name, service_category, description,
                     service_type, delivery_model, billable, status
                     FROM services WHERE status = 'active' ORDER BY service_name")
                .fetch_all(&pool)
                .await
            }) {
                Ok(rows) => {
                    self.services = rows.into_iter().map(|row| Service {
                        id: row.get("id"),
                        service_id: row.get("service_id"),
                        service_name: row.get("service_name"),
                        service_category: row.get("service_category"),
                        description: row.get("description"),
                        service_type: row.get("service_type"),
                        delivery_model: row.get("delivery_model"),
                        billable: row.get("billable"),
                        status: row.get("status"),
                    }).collect();
                }
                Err(e) => {
                    eprintln!("Error loading services: {}", e);
                }
            }

            // Load resources
            match rt.block_on(async {
                sqlx::query(
                    "SELECT id, resource_name, description, version, category, resource_type,
                     criticality_level, operational_status, owner_team, status
                     FROM resource_objects WHERE status = 'active' ORDER BY resource_name")
                .fetch_all(&pool)
                .await
            }) {
                Ok(rows) => {
                    self.resources = rows.into_iter().map(|row| ResourceObject {
                        id: row.get("id"),
                        resource_name: row.get("resource_name"),
                        description: row.get("description"),
                        version: row.get("version"),
                        category: row.get("category"),
                        resource_type: row.get("resource_type"),
                        criticality_level: row.get("criticality_level"),
                        operational_status: row.get("operational_status"),
                        owner_team: row.get("owner_team"),
                        status: row.get("status"),
                    }).collect();
                }
                Err(e) => {
                    eprintln!("Error loading resources: {}", e);
                }
            }

            // Load service-resource hierarchy
            match rt.block_on(async {
                sqlx::query(
                    "SELECT service_id, service_code, service_name, service_category, service_type,
                     delivery_model, billable, service_description, service_status,
                     resource_id, resource_name, resource_description, resource_version,
                     resource_category, resource_type, criticality_level, operational_status,
                     owner_team, usage_type, resource_role, cost_allocation_percentage, dependency_level
                     FROM service_resources_hierarchy ORDER BY service_name, dependency_level")
                .fetch_all(&pool)
                .await
            }) {
                Ok(rows) => {
                    self.service_resource_hierarchy = rows.into_iter().map(|row| ServiceResourceHierarchy {
                        service_id: row.get("service_id"),
                        service_code: row.get("service_code"),
                        service_name: row.get("service_name"),
                        service_category: row.get("service_category"),
                        service_type: row.get("service_type"),
                        delivery_model: row.get("delivery_model"),
                        billable: row.get("billable"),
                        service_description: row.get("service_description"),
                        service_status: row.get("service_status"),
                        resource_id: row.get("resource_id"),
                        resource_name: row.get("resource_name"),
                        resource_description: row.get("resource_description"),
                        resource_version: row.get("resource_version"),
                        resource_category: row.get("resource_category"),
                        resource_type: row.get("resource_type"),
                        criticality_level: row.get("criticality_level"),
                        operational_status: row.get("operational_status"),
                        owner_team: row.get("owner_team"),
                        usage_type: row.get("usage_type"),
                        resource_role: row.get("resource_role"),
                        cost_allocation_percentage: row.get("cost_allocation_percentage"),
                        dependency_level: row.get("dependency_level"),
                    }).collect();
                }
                Err(e) => {
                    eprintln!("Error loading service-resource hierarchy: {}", e);
                }
            }
        }
    }

    fn load_investment_mandates(&mut self) {
        if let Some(ref pool) = self.db_pool {
            let pool = pool.clone();
            let rt = self.runtime.clone();

            // Load CBU investment mandate structure
            match rt.block_on(async {
                sqlx::query(
                    "SELECT cbu_id, cbu_name, mandate_id, asset_owner_name, investment_manager_name,
                     base_currency, total_instruments, families, total_exposure_pct
                     FROM cbu_investment_mandate_structure ORDER BY cbu_id")
                .fetch_all(&pool)
                .await
            }) {
                Ok(rows) => {
                    self.cbu_mandate_structure = rows.into_iter().map(|row| CbuInvestmentMandateStructure {
                        cbu_id: row.get("cbu_id"),
                        cbu_name: row.get("cbu_name"),
                        mandate_id: row.get("mandate_id"),
                        asset_owner_name: row.get("asset_owner_name"),
                        investment_manager_name: row.get("investment_manager_name"),
                        base_currency: row.get("base_currency"),
                        total_instruments: row.get("total_instruments"),
                        families: row.get("families"),
                        total_exposure_pct: row.get("total_exposure_pct"),
                    }).collect();
                    self.status_message = format!("Loaded {} CBU mandate structures", self.cbu_mandate_structure.len());
                }
                Err(e) => {
                    self.status_message = format!("Error loading CBU mandate structure: {}", e);
                }
            }

            // Load CBU member investment roles
            match rt.block_on(async {
                sqlx::query(
                    "SELECT cbu_id, cbu_name, entity_name, entity_lei, role_name, role_code,
                     investment_responsibility, mandate_id, has_trading_authority, has_settlement_authority
                     FROM cbu_member_investment_roles ORDER BY cbu_id, role_code")
                .fetch_all(&pool)
                .await
            }) {
                Ok(rows) => {
                    self.cbu_member_roles = rows.into_iter().map(|row| CbuMemberInvestmentRole {
                        cbu_id: row.get("cbu_id"),
                        cbu_name: row.get("cbu_name"),
                        entity_name: row.get("entity_name"),
                        entity_lei: row.get("entity_lei"),
                        role_name: row.get("role_name"),
                        role_code: row.get("role_code"),
                        investment_responsibility: row.get("investment_responsibility"),
                        mandate_id: row.get("mandate_id"),
                        has_trading_authority: row.get("has_trading_authority"),
                        has_settlement_authority: row.get("has_settlement_authority"),
                    }).collect();
                }
                Err(e) => {
                    eprintln!("Error loading CBU member roles: {}", e);
                }
            }
        }
    }

    fn load_taxonomy_hierarchy(&mut self) {
        // For now, create a sample hierarchy
        self.taxonomy_hierarchy = vec![
            TaxonomyHierarchyItem {
                level: 1,
                item_type: "product".to_string(),
                item_id: 1,
                item_name: "Institutional Custody Plus".to_string(),
                item_description: Some("Comprehensive custody services".to_string()),
                parent_id: None,
                configuration: None,
                metadata: None,
            },
            TaxonomyHierarchyItem {
                level: 2,
                item_type: "product_option".to_string(),
                item_id: 2,
                item_name: "US Market Settlement".to_string(),
                item_description: Some("Settlement in US markets".to_string()),
                parent_id: Some(1),
                configuration: None,
                metadata: None,
            },
        ];
        self.status_message = "Loaded sample taxonomy hierarchy".to_string();
    }

}