use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Core expression evaluation types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Number(f64),
    Integer(i64),  // Added for parser compatibility
    Float(f64),    // Added for parser compatibility
    Boolean(bool),
    Null,
    Regex(String), // Added for regex support
    List(Vec<Value>), // Added for list support
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Value),
    Variable(String),
    Identifier(String), // Added for parser compatibility
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator, // Changed from operator to op for parser compatibility
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator, // Changed from operator to op for parser compatibility
        operand: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Option<Box<Expression>>, // Made optional for parser compatibility
    },
    Assignment {
        target: String,
        value: Box<Expression>,
    },
    List(Vec<Expression>), // Added for list support
    Cast {
        expr: Box<Expression>,
        data_type: String,
    }, // Added for type casting
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,      // Added for parser compatibility
    Modulo,     // Added for parser compatibility
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
    Matches,
    NotMatches, // Added for negative pattern matching
    Concat,     // Added for string concatenation
    Contains,   // Added for string operations
    StartsWith, // Added for string operations
    EndsWith,   // Added for string operations
    In,         // Added for list operations
    NotIn,      // Added for list operations
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Minus,
    Plus, // Added for parser compatibility
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDictionary {
    pub datasets: Vec<Dataset>,
    pub lookup_tables: HashMap<String, HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeGroup {
    pub name: String,
    pub description: String,
    pub attributes: Vec<AttributeInfo>,
    pub expanded: bool, // For UI state
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeInfo {
    pub name: String,
    pub value: serde_json::Value,
    pub data_type: String,
    pub description: Option<String>,
    pub required: bool,
    pub validation_rules: Vec<String>,
}

impl DataDictionary {
    pub fn load_from_json(json_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(json_str)?)
    }

    pub fn get_attribute_groups(&self) -> Vec<AttributeGroup> {
        self.datasets.iter().map(|dataset| {
            let attributes = dataset.attributes.iter().map(|(name, value)| {
                AttributeInfo {
                    name: name.clone(),
                    value: value.clone(),
                    data_type: infer_data_type(value),
                    description: None,
                    required: true, // Default assumption
                    validation_rules: vec![],
                }
            }).collect();

            AttributeGroup {
                name: dataset.name.clone(),
                description: dataset.description.clone(),
                attributes,
                expanded: false,
            }
        }).collect()
    }

    pub fn get_lookup_table_names(&self) -> Vec<String> {
        self.lookup_tables.keys().cloned().collect()
    }

    pub fn search_attributes(&self, query: &str) -> Vec<(String, String, serde_json::Value)> {
        let mut results = vec![];
        let query_lower = query.to_lowercase();

        for dataset in &self.datasets {
            for (attr_name, attr_value) in &dataset.attributes {
                if attr_name.to_lowercase().contains(&query_lower)
                   || dataset.name.to_lowercase().contains(&query_lower)
                   || attr_value.to_string().to_lowercase().contains(&query_lower) {
                    results.push((dataset.name.clone(), attr_name.clone(), attr_value.clone()));
                }
            }
        }

        results
    }

    pub fn get_dataset_by_id(&self, id: &str) -> Option<&Dataset> {
        self.datasets.iter().find(|d| d.id == id)
    }

    pub fn get_statistics(&self) -> DataDictionaryStats {
        let total_datasets = self.datasets.len();
        let total_attributes = self.datasets.iter()
            .map(|d| d.attributes.len())
            .sum();
        let lookup_tables_count = self.lookup_tables.len();
        let total_lookup_entries = self.lookup_tables.iter()
            .map(|(_, table)| table.len())
            .sum();

        DataDictionaryStats {
            total_datasets,
            total_attributes,
            lookup_tables_count,
            total_lookup_entries,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataDictionaryStats {
    pub total_datasets: usize,
    pub total_attributes: usize,
    pub lookup_tables_count: usize,
    pub total_lookup_entries: usize,
}

fn infer_data_type(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) => "boolean".to_string(),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                "integer".to_string()
            } else if n.is_f64() {
                "decimal".to_string()
            } else {
                "number".to_string()
            }
        },
        serde_json::Value::String(s) => {
            // Try to infer more specific types
            if s.contains('@') {
                "email".to_string()
            } else if s.contains('-') && s.len() == 10 {
                "date".to_string()
            } else if s.starts_with("http") {
                "url".to_string()
            } else if s.len() > 50 {
                "text".to_string()
            } else {
                "string".to_string()
            }
        },
        serde_json::Value::Array(_) => "array".to_string(),
        serde_json::Value::Object(_) => "object".to_string(),
    }
}

// UI State management
#[derive(Debug, Clone)]
pub struct ViewerState {
    pub selected_dataset: Option<String>,
    pub selected_lookup_table: Option<String>,
    pub search_query: String,
    pub expanded_groups: HashMap<String, bool>,
    pub show_statistics: bool,
    pub filter_by_type: Option<String>,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            selected_dataset: None,
            selected_lookup_table: None,
            search_query: String::new(),
            expanded_groups: HashMap::new(),
            show_statistics: true,
            filter_by_type: None,
        }
    }
}

impl ViewerState {
    pub fn is_group_expanded(&self, group_name: &str) -> bool {
        self.expanded_groups.get(group_name).copied().unwrap_or(false)
    }

    pub fn toggle_group(&mut self, group_name: &str) {
        let expanded = self.is_group_expanded(group_name);
        self.expanded_groups.insert(group_name.to_string(), !expanded);
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
    }

    pub fn has_active_filters(&self) -> bool {
        !self.search_query.is_empty() || self.filter_by_type.is_some()
    }
}