use anyhow::Result;
// use serde::{Deserialize, Serialize}; // Unused, commented out
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;
// use uuid::Uuid; // Unused, commented out
// use chrono::{DateTime, Utc}; // Unused, commented out


// Database connection pool
pub type DbPool = Pool<Postgres>;

// Initialize database connection
pub async fn init_db() -> Result<DbPool> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://adamtc007@localhost/data_designer".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

// Common database operation patterns
pub struct DbOperations;

impl DbOperations {
    // Simplified query helper for common cases
    pub async fn query_count(
        pool: &DbPool,
        query: &str,
        param: &str,
    ) -> Result<i64, String> {
        let row: (i64,) = sqlx::query_as(query)
            .bind(param)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database query error: {}", e))?;
        Ok(row.0)
    }

    // Consistent pattern for transactions
    pub async fn begin_transaction(pool: &DbPool) -> Result<sqlx::Transaction<'_, Postgres>, String> {
        pool.begin()
            .await
            .map_err(|e| format!("Failed to start transaction: {}", e))
    }

    // Query helper that returns raw rows - no params version
    pub async fn query_raw_all_no_params(
        pool: &DbPool,
        query: &str,
    ) -> Result<Vec<sqlx::postgres::PgRow>, String> {
        sqlx::query(query)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Database query error: {}", e))
    }

    // Query helper that returns raw rows - one string param
    pub async fn query_raw_all_one_param(
        pool: &DbPool,
        query: &str,
        param: &str,
    ) -> Result<Vec<sqlx::postgres::PgRow>, String> {
        sqlx::query(query)
            .bind(param)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Database query error: {}", e))
    }

    // Query helper that returns raw rows - two params
    pub async fn query_raw_all_two_params(
        pool: &DbPool,
        query: &str,
        param1: &str,
        param2: &i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, String> {
        sqlx::query(query)
            .bind(param1)
            .bind(param2)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Database query error: {}", e))
    }

    // Query helper that returns raw rows - one i32 param
    pub async fn query_raw_all_one_i32_param(
        pool: &DbPool,
        query: &str,
        param: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, String> {
        sqlx::query(query)
            .bind(param)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Database query error: {}", e))
    }
}

// Database entity modules
pub mod rules;
pub mod attributes;
pub mod schema;

// Re-export all database entities and operations
pub use rules::*;
pub use attributes::*;

// Legacy compatibility
pub use crate::database::CreateRuleRequest;