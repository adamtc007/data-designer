use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, FromRow};

// Re-export common types for convenience

// Database connection pool
pub type DbPool = Pool<Postgres>;

// Initialize database connection using configuration
pub async fn init_db() -> Result<DbPool> {
    init_db_with_config(None).await
}

// Initialize database connection with optional configuration
pub async fn init_db_with_config(config: Option<crate::config::Config>) -> Result<DbPool> {
    let config = config.unwrap_or_else(|| {
        crate::config::Config::load().unwrap_or_else(|e| {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Using default configuration");
            crate::config::Config::default()
        })
    });

    let database_url = config.database_url();
    println!("🔧 Connecting to database: {}@{}:{}/{}",
             config.database.username,
             config.database.host,
             config.database.port,
             config.database.database);

    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.database.acquire_timeout_seconds))
        .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout_seconds))
        .connect(&database_url)
        .await?;

    println!("✅ Database connection established");
    Ok(pool)
}

// Centralized database operations interface for the entire IDE
pub struct DbOperations;

impl DbOperations {
    // === CONNECTION MANAGEMENT ===

    /// Get a database connection pool - the ONLY way to access the database
    pub async fn get_pool() -> Result<DbPool> {
        init_db().await
    }

    /// Begin a transaction for multi-operation consistency
    pub async fn begin_transaction(pool: &DbPool) -> Result<sqlx::Transaction<'_, Postgres>, String> {
        pool.begin()
            .await
            .map_err(|e| format!("Failed to start transaction: {}", e))
    }

    // === QUERY HELPERS ===

    /// Execute a simple count query with one parameter
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


    /// Execute a query that returns multiple typed results
    pub async fn query_all<T>(
        pool: &DbPool,
        query: &str,
    ) -> Result<Vec<T>, String>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(query)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Database query error: {}", e))
    }



    /// Execute a query with two parameters
    pub async fn query_all_with_two_params<T, P1, P2>(
        pool: &DbPool,
        query: &str,
        param1: P1,
        param2: P2,
    ) -> Result<Vec<T>, String>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
        P1: for<'q> sqlx::Encode<'q, Postgres> + sqlx::Type<Postgres> + Send + Sync,
        P2: for<'q> sqlx::Encode<'q, Postgres> + sqlx::Type<Postgres> + Send + Sync,
    {
        sqlx::query_as::<_, T>(query)
            .bind(param1)
            .bind(param2)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Database query error: {}", e))
    }

    /// Execute a command (INSERT, UPDATE, DELETE) and return affected rows
    pub async fn execute(
        pool: &DbPool,
        query: &str,
    ) -> Result<u64, String> {
        sqlx::query(query)
            .execute(pool)
            .await
            .map(|result| result.rows_affected())
            .map_err(|e| format!("Database execution error: {}", e))
    }



    /// Execute a command with two parameters
    pub async fn execute_with_two_params<P1, P2>(
        pool: &DbPool,
        query: &str,
        param1: P1,
        param2: P2,
    ) -> Result<u64, String>
    where
        P1: for<'q> sqlx::Encode<'q, Postgres> + sqlx::Type<Postgres> + Send + Sync,
        P2: for<'q> sqlx::Encode<'q, Postgres> + sqlx::Type<Postgres> + Send + Sync,
    {
        sqlx::query(query)
            .bind(param1)
            .bind(param2)
            .execute(pool)
            .await
            .map(|result| result.rows_affected())
            .map_err(|e| format!("Database execution error: {}", e))
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

    /// Execute a parameterized query with one parameter
    pub async fn query_one_with_param<T>(
        pool: &DbPool,
        query: &str,
        param: &str,
    ) -> Result<T, String>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(query)
            .bind(param)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database query error: {}", e))
    }

    /// Execute a query that returns a single typed result
    pub async fn query_one<T>(
        pool: &DbPool,
        query: &str,
    ) -> Result<T, String>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(query)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database query error: {}", e))
    }

    /// Execute a parameterized query that returns multiple results
    pub async fn query_all_with_param<T>(
        pool: &DbPool,
        query: &str,
        param: &str,
    ) -> Result<Vec<T>, String>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(query)
            .bind(param)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Database query error: {}", e))
    }

    /// Execute a parameterized command
    pub async fn execute_with_param<P>(
        pool: &DbPool,
        query: &str,
        param: P,
    ) -> Result<u64, String>
    where
        P: for<'q> sqlx::Encode<'q, Postgres> + sqlx::Type<Postgres> + Send + Sync,
    {
        sqlx::query(query)
            .bind(param)
            .execute(pool)
            .await
            .map(|result| result.rows_affected())
            .map_err(|e| format!("Database execution error: {}", e))
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
pub mod embeddings;
pub mod data_dictionary;
pub mod grammar;
pub mod cbu;
pub mod products;
pub mod config_driven;
pub mod persistence;

// Re-export all database entities and operations
pub use rules::*;
pub use schema::*;
pub use persistence::*;
pub use embeddings::*;
pub use data_dictionary::*;
pub use cbu::*;
pub use products::*;
pub use config_driven::*;

// Legacy compatibility
pub use self::rules::CreateRuleRequest;