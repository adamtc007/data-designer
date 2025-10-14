use super::{DbPool, DbOperations};
use serde::{Deserialize, Serialize};
use sqlx::Row;

// Schema-related DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub tables: Vec<TableInfo>,
    pub relationships: Vec<RelationshipInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub table_name: String,
    pub schema_name: String,
    pub table_type: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
    pub is_primary_key: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelationshipInfo {
    pub source_table: String,
    pub source_column: String,
    pub target_table: String,
    pub target_column: String,
    pub constraint_name: String,
}

// Schema database operations
pub struct SchemaOperations;

impl SchemaOperations {
    // Get comprehensive schema information
    pub async fn get_schema_info(pool: &DbPool) -> Result<SchemaInfo, String> {
        let tables = Self::get_tables_with_columns(pool).await?;
        let relationships = Self::get_table_relationships(pool).await?;

        Ok(SchemaInfo {
            tables,
            relationships,
        })
    }

    // Get tables with their columns
    async fn get_tables_with_columns(pool: &DbPool) -> Result<Vec<TableInfo>, String> {
        let table_query = "
            SELECT
                t.table_name,
                t.table_schema,
                t.table_type
            FROM information_schema.tables t
            WHERE t.table_schema = 'public'
            ORDER BY t.table_name
        ";

        let table_rows = DbOperations::query_raw_all_no_params(pool, table_query).await?;
        let mut tables = Vec::new();

        for table_row in table_rows {
            let table_name = table_row.get::<&str, _>("table_name");
            let schema_name = table_row.get::<&str, _>("table_schema");
            let table_type = table_row.get::<&str, _>("table_type");

            let columns = Self::get_table_columns(pool, table_name).await?;

            tables.push(TableInfo {
                table_name: table_name.to_string(),
                schema_name: schema_name.to_string(),
                table_type: table_type.to_string(),
                columns,
            });
        }

        Ok(tables)
    }

    // Get columns for a specific table
    async fn get_table_columns(
        pool: &DbPool,
        table_name: &str,
    ) -> Result<Vec<ColumnInfo>, String> {
        let column_query = "
            SELECT
                c.column_name,
                c.data_type,
                c.is_nullable = 'YES' as is_nullable,
                c.column_default,
                CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_primary_key
            FROM information_schema.columns c
            LEFT JOIN (
                SELECT ku.column_name
                FROM information_schema.key_column_usage ku
                JOIN information_schema.table_constraints tc
                    ON ku.constraint_name = tc.constraint_name
                WHERE tc.constraint_type = 'PRIMARY KEY'
                    AND ku.table_name = $1
                    AND ku.table_schema = 'public'
            ) pk ON c.column_name = pk.column_name
            WHERE c.table_name = $1
                AND c.table_schema = 'public'
            ORDER BY c.ordinal_position
        ";

        let column_rows = DbOperations::query_raw_all_one_param(pool, column_query, table_name).await?;
        let mut columns = Vec::new();

        for row in column_rows {
            columns.push(ColumnInfo {
                column_name: row.get::<&str, _>("column_name").to_string(),
                data_type: row.get::<&str, _>("data_type").to_string(),
                is_nullable: row.get::<bool, _>("is_nullable"),
                column_default: row.get::<Option<&str>, _>("column_default").map(|s| s.to_string()),
                is_primary_key: row.get::<bool, _>("is_primary_key"),
            });
        }

        Ok(columns)
    }

    // Get table relationships (foreign keys)
    pub async fn get_table_relationships(pool: &DbPool) -> Result<Vec<RelationshipInfo>, String> {
        let relationship_query = "
            SELECT
                tc.table_name as source_table,
                kcu.column_name as source_column,
                ccu.table_name as target_table,
                ccu.column_name as target_column,
                tc.constraint_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage ccu
                ON ccu.constraint_name = tc.constraint_name
                AND ccu.table_schema = tc.table_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
                AND tc.table_schema = 'public'
            ORDER BY tc.table_name, tc.constraint_name
        ";

        let rows = DbOperations::query_raw_all_no_params(pool, relationship_query).await?;
        let mut relationships = Vec::new();

        for row in rows {
            relationships.push(RelationshipInfo {
                source_table: row.get::<&str, _>("source_table").to_string(),
                source_column: row.get::<&str, _>("source_column").to_string(),
                target_table: row.get::<&str, _>("target_table").to_string(),
                target_column: row.get::<&str, _>("target_column").to_string(),
                constraint_name: row.get::<&str, _>("constraint_name").to_string(),
            });
        }

        Ok(relationships)
    }

    // Execute safe SQL queries (SELECT only)
    pub async fn execute_sql(
        pool: &DbPool,
        sql: &str,
    ) -> Result<Vec<serde_json::Value>, String> {
        // Security check: only allow SELECT statements
        let trimmed_sql = sql.trim().to_lowercase();
        if !trimmed_sql.starts_with("select") {
            return Err("Only SELECT statements are allowed".to_string());
        }

        // Additional security: prevent dangerous keywords
        let dangerous_keywords = ["drop", "delete", "insert", "update", "alter", "create", "truncate"];
        for keyword in dangerous_keywords {
            if trimmed_sql.contains(keyword) {
                return Err(format!("Keyword '{}' is not allowed", keyword));
            }
        }

        let rows = DbOperations::query_raw_all_no_params(pool, sql).await?;
        let mut results = Vec::new();

        for row in rows {
            // Convert PostgreSQL row to JSON
            let mut json_obj = serde_json::Map::new();

            // Note: This is a simplified approach. In a real implementation,
            // you'd want to introspect the row columns dynamically
            for i in 0..row.len() {
                if let Ok(column_name) = row.try_get::<&str, usize>(i) {
                    json_obj.insert(
                        format!("column_{}", i),
                        serde_json::Value::String(column_name.to_string())
                    );
                }
            }

            results.push(serde_json::Value::Object(json_obj));
        }

        Ok(results)
    }

    // Get table statistics
    pub async fn get_table_stats(pool: &DbPool) -> Result<Vec<serde_json::Value>, String> {
        let stats_query = "
            SELECT
                schemaname,
                tablename,
                attname as column_name,
                n_distinct,
                most_common_vals,
                most_common_freqs,
                null_frac
            FROM pg_stats
            WHERE schemaname = 'public'
            ORDER BY tablename, attname
        ";

        let rows = DbOperations::query_raw_all_no_params(pool, stats_query).await?;
        let mut stats = Vec::new();

        for row in rows {
            let stat = serde_json::json!({
                "schema_name": row.get::<&str, _>("schemaname"),
                "table_name": row.get::<&str, _>("tablename"),
                "column_name": row.get::<&str, _>("column_name"),
                "n_distinct": row.get::<Option<f32>, _>("n_distinct"),
                "null_frac": row.get::<Option<f32>, _>("null_frac")
            });
            stats.push(stat);
        }

        Ok(stats)
    }
}