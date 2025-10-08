use serde::{Deserialize, Serialize};

// --- Top-level Dictionary Structure ---
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct DataDictionary {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_models: Vec<CanonicalModel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_attributes: Vec<DerivedAttribute>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub solicitation_packs: Vec<SolicitationPack>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub axes: Vec<Axis>,
}

// ... (Rest of the model definitions are unchanged) ...

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Axis {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalModel {
    pub entity_name: String,
    pub description: String,
    pub attributes: Vec<CanonicalAttribute>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalAttribute {
    pub name: String,
    pub data_type: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    #[serde(default)]
    pub governance: Governance,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Governance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorized_source: Option<AuthorizedSource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consumers: Vec<Consumer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizedSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub lineage_uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Consumer {
    pub name: String,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DerivedAttribute {
    pub name: String,
    #[serde(rename = "type")]
    pub attribute_type: String,
    pub visibility: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    pub dependencies: Vec<String>,
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub governance: Governance,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub description: String,
    #[serde(rename = "if", default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(rename = "then", default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(rename = "otherwise", default, skip_serializing_if = "Option::is_none")]
    pub otherwise_value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SolicitationPack {
    pub name: String,
    pub description: String,
    pub process: String,
    pub audience: String,
    pub attributes: Vec<String>,
}


// --- AST Definitions for the Rules DSL ---

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Regex(String),
    Null,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum BinaryOperator {
    // Arithmetic operators
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,

    // String operations
    Concat,

    // Comparison operators
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    // Pattern matching
    Matches,
    NotMatches,
    Contains,
    StartsWith,
    EndsWith,

    // Logical operators
    And,
    Or,

    // Set operations
    In,
    NotIn,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum UnaryOperator {
    Not,
    Minus,
    Plus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Value),
    Identifier(String),
    Assignment {
        target: String,
        value: Box<Expression>,
    },
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    Cast {
        expr: Box<Expression>,
        data_type: String,
    },
    List(Vec<Expression>),
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Option<Box<Expression>>,
    },
}
