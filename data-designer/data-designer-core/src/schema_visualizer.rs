use serde::{Deserialize, Serialize};
use crate::db::{DbPool, SchemaOperations};
use std::collections::HashMap;

// Re-export types from centralized db module
pub use crate::db::{SchemaInfo, TableInfo, ColumnInfo, RelationshipInfo};

#[derive(Debug, Serialize, Deserialize)]
pub struct SqlQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
}

// === WRAPPER FUNCTIONS THAT DELEGATE TO CENTRALIZED OPERATIONS ===

/// Get comprehensive schema information
pub async fn get_schema_info(pool: &DbPool) -> Result<SchemaInfo, String> {
    // Use centralized operations
    SchemaOperations::get_schema_info(pool).await
}

/// Get table relationships (foreign keys)
pub async fn get_table_relationships(pool: &DbPool) -> Result<Vec<RelationshipInfo>, String> {
    // Use centralized operations
    SchemaOperations::get_table_relationships(pool).await
}

/// Execute a safe SQL query (SELECT only)
pub async fn execute_sql_query(pool: &DbPool, query: &str) -> Result<SqlQueryResult, String> {
    // Use centralized operations for safe SQL execution
    let results = SchemaOperations::execute_sql(pool, query).await?;

    // Convert to our response format
    let mut columns = Vec::new();
    let mut rows = Vec::new();

    if let Some(first_result) = results.first() {
        if let serde_json::Value::Object(obj) = first_result {
            columns = obj.keys().cloned().collect();
        }
    }

    for result in &results {
        if let serde_json::Value::Object(obj) = result {
            let row: Vec<serde_json::Value> = columns
                .iter()
                .map(|col| obj.get(col).unwrap_or(&serde_json::Value::Null).clone())
                .collect();
            rows.push(row);
        }
    }

    Ok(SqlQueryResult {
        columns,
        row_count: rows.len(),
        rows,
    })
}

/// Get table statistics
pub async fn get_table_stats(pool: &DbPool) -> Result<Vec<serde_json::Value>, String> {
    // Use centralized operations
    SchemaOperations::get_table_stats(pool).await
}

/// Get detailed table information with column metadata
pub async fn get_table_details(pool: &DbPool, table_name: &str) -> Result<TableInfo, String> {
    let schema_info = get_schema_info(pool).await?;

    schema_info
        .tables
        .into_iter()
        .find(|table| table.table_name == table_name)
        .ok_or_else(|| format!("Table '{}' not found", table_name))
}

/// Get schema summary with counts
pub async fn get_schema_summary(pool: &DbPool) -> Result<HashMap<String, serde_json::Value>, String> {
    let schema_info = get_schema_info(pool).await?;
    let relationships = get_table_relationships(pool).await?;

    let mut summary = HashMap::new();
    summary.insert("table_count".to_string(), serde_json::Value::Number(schema_info.tables.len().into()));
    summary.insert("relationship_count".to_string(), serde_json::Value::Number(relationships.len().into()));

    let total_columns: usize = schema_info.tables.iter().map(|t| t.columns.len()).sum();
    summary.insert("total_columns".to_string(), serde_json::Value::Number(total_columns.into()));

    let table_names: Vec<String> = schema_info.tables.iter().map(|t| t.table_name.clone()).collect();
    summary.insert("table_names".to_string(), serde_json::Value::Array(
        table_names.into_iter().map(serde_json::Value::String).collect()
    ));

    Ok(summary)
}

/// Validate SQL query before execution
pub fn validate_sql_query(query: &str) -> Result<(), String> {
    let trimmed = query.trim().to_lowercase();

    if !trimmed.starts_with("select") {
        return Err("Only SELECT statements are allowed".to_string());
    }

    let dangerous_keywords = ["drop", "delete", "insert", "update", "alter", "create", "truncate"];
    for keyword in dangerous_keywords {
        if trimmed.contains(keyword) {
            return Err(format!("Keyword '{}' is not allowed", keyword));
        }
    }

    Ok(())
}

/// Get column information for a specific table
pub async fn get_table_columns(pool: &DbPool, table_name: &str) -> Result<Vec<ColumnInfo>, String> {
    let table_details = get_table_details(pool, table_name).await?;
    Ok(table_details.columns)
}

/// Search tables and columns by name
pub async fn search_schema(pool: &DbPool, search_term: &str) -> Result<SchemaInfo, String> {
    let schema_info = get_schema_info(pool).await?;
    let search_lower = search_term.to_lowercase();

    let filtered_tables: Vec<TableInfo> = schema_info
        .tables
        .into_iter()
        .filter(|table| {
            table.table_name.to_lowercase().contains(&search_lower) ||
            table.columns.iter().any(|col| col.column_name.to_lowercase().contains(&search_lower))
        })
        .collect();

    Ok(SchemaInfo {
        tables: filtered_tables,
        relationships: schema_info.relationships,
    })
}