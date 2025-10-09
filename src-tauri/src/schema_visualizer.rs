use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::{Row, Column};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
    pub foreign_table: Option<String>,
    pub foreign_column: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelationshipInfo {
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    pub relationship_type: String, // "one-to-many", "many-to-one", etc.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub tables: Vec<TableInfo>,
    pub relationships: Vec<RelationshipInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SqlQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
}

pub async fn get_schema_info(pool: &PgPool) -> Result<SchemaInfo, String> {
    println!("Getting schema info from database...");

    // Query for all tables and their columns
    let tables_query = r#"
        SELECT
            t.table_name,
            c.column_name,
            c.data_type,
            c.is_nullable,
            CASE
                WHEN pk.column_name IS NOT NULL THEN true
                ELSE false
            END as is_primary_key
        FROM information_schema.tables t
        JOIN information_schema.columns c ON t.table_name = c.table_name
        LEFT JOIN (
            SELECT kcu.table_name, kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name
            WHERE tc.constraint_type = 'PRIMARY KEY'
        ) pk ON c.table_name = pk.table_name AND c.column_name = pk.column_name
        WHERE t.table_schema = 'public'
            AND t.table_type = 'BASE TABLE'
        ORDER BY t.table_name, c.ordinal_position
    "#;

    let rows = sqlx::query(tables_query)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch schema info: {}", e))?;

    let mut tables_map: HashMap<String, TableInfo> = HashMap::new();

    for row in rows {
        let table_name: String = row.get("table_name");
        let column = ColumnInfo {
            name: row.get("column_name"),
            data_type: row.get("data_type"),
            is_nullable: row.get::<String, _>("is_nullable") == "YES",
            is_primary_key: row.get("is_primary_key"),
            is_foreign_key: false, // Will be updated below
            foreign_table: None,
            foreign_column: None,
        };

        tables_map.entry(table_name.clone())
            .or_insert_with(|| TableInfo {
                name: table_name,
                columns: Vec::new(),
            })
            .columns.push(column);
    }

    // Query for foreign key relationships
    let fk_query = r#"
        SELECT
            kcu.table_name as from_table,
            kcu.column_name as from_column,
            ccu.table_name as to_table,
            ccu.column_name as to_column
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
        JOIN information_schema.constraint_column_usage ccu
            ON tc.constraint_name = ccu.constraint_name
        WHERE tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_schema = 'public'
    "#;

    let fk_rows = sqlx::query(fk_query)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch foreign keys: {}", e))?;

    let mut relationships = Vec::new();

    for row in fk_rows {
        let from_table: String = row.get("from_table");
        let from_column: String = row.get("from_column");
        let to_table: String = row.get("to_table");
        let to_column: String = row.get("to_column");

        // Update the foreign key info in the column
        if let Some(table) = tables_map.get_mut(&from_table) {
            if let Some(column) = table.columns.iter_mut().find(|c| c.name == from_column) {
                column.is_foreign_key = true;
                column.foreign_table = Some(to_table.clone());
                column.foreign_column = Some(to_column.clone());
            }
        }

        relationships.push(RelationshipInfo {
            from_table: from_table.clone(),
            from_column: from_column.clone(),
            to_table: to_table.clone(),
            to_column: to_column.clone(),
            relationship_type: "many-to-one".to_string(), // Default, can be enhanced
        });
    }

    let tables: Vec<TableInfo> = tables_map.into_values().collect();

    Ok(SchemaInfo {
        tables,
        relationships,
    })
}

pub async fn get_table_relationships(
    pool: &PgPool,
    table_name: &str,
) -> Result<Vec<RelationshipInfo>, String> {
    let query = r#"
        SELECT
            kcu.table_name as from_table,
            kcu.column_name as from_column,
            ccu.table_name as to_table,
            ccu.column_name as to_column
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
        JOIN information_schema.constraint_column_usage ccu
            ON tc.constraint_name = ccu.constraint_name
        WHERE tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_schema = 'public'
            AND (kcu.table_name = $1 OR ccu.table_name = $1)
    "#;

    let rows = sqlx::query(query)
        .bind(table_name)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch relationships: {}", e))?;

    let mut relationships = Vec::new();

    for row in rows {
        relationships.push(RelationshipInfo {
            from_table: row.get("from_table"),
            from_column: row.get("from_column"),
            to_table: row.get("to_table"),
            to_column: row.get("to_column"),
            relationship_type: "many-to-one".to_string(),
        });
    }

    Ok(relationships)
}

pub async fn execute_sql_query(pool: &PgPool, query: &str) -> Result<SqlQueryResult, String> {
    // Basic SQL injection prevention - only allow SELECT queries
    let trimmed = query.trim().to_lowercase();
    if !trimmed.starts_with("select") {
        return Err("Only SELECT queries are allowed".to_string());
    }

    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Query execution failed: {}", e))?;

    if rows.is_empty() {
        return Ok(SqlQueryResult {
            columns: vec![],
            rows: vec![],
            row_count: 0,
        });
    }

    // Get column names from the first row
    let first_row = &rows[0];
    let columns: Vec<String> = first_row
        .columns()
        .iter()
        .map(|col| col.name().to_string())
        .collect();

    // Convert rows to JSON values
    let mut result_rows = Vec::new();
    for row in rows.iter() {
        let mut row_values = Vec::new();
        for col in first_row.columns() {
            let value: serde_json::Value = if let Ok(v) = row.try_get::<String, _>(col.name()) {
                serde_json::Value::String(v)
            } else if let Ok(v) = row.try_get::<i32, _>(col.name()) {
                serde_json::Value::Number(v.into())
            } else if let Ok(v) = row.try_get::<i64, _>(col.name()) {
                serde_json::Value::Number(v.into())
            } else if let Ok(v) = row.try_get::<f64, _>(col.name()) {
                serde_json::Number::from_f64(v)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else if let Ok(v) = row.try_get::<bool, _>(col.name()) {
                serde_json::Value::Bool(v)
            } else {
                serde_json::Value::Null
            };
            row_values.push(value);
        }
        result_rows.push(row_values);
    }

    Ok(SqlQueryResult {
        columns,
        row_count: result_rows.len(),
        rows: result_rows,
    })
}