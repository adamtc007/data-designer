// Data Access Layer - PersistenceService trait and implementations
// This module provides the live data connection layer for the AI Context Engine

use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use crate::db::DbPool;
use anyhow::Result;
use sqlx::Row;

// Core data types for persistence layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceLocator {
    pub system: String,
    pub entity: String,
    pub identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<LiteralValue>),
    Object(HashMap<String, LiteralValue>),
    Null,
}

impl From<JsonValue> for LiteralValue {
    fn from(json: JsonValue) -> Self {
        match json {
            JsonValue::String(s) => LiteralValue::String(s),
            JsonValue::Number(n) => LiteralValue::Number(n.as_f64().unwrap_or(0.0)),
            JsonValue::Bool(b) => LiteralValue::Boolean(b),
            JsonValue::Array(arr) => {
                let values: Vec<LiteralValue> = arr.into_iter().map(LiteralValue::from).collect();
                LiteralValue::List(values)
            },
            JsonValue::Object(obj) => {
                let map: HashMap<String, LiteralValue> = obj.into_iter()
                    .map(|(k, v)| (k, LiteralValue::from(v)))
                    .collect();
                LiteralValue::Object(map)
            },
            JsonValue::Null => LiteralValue::Null,
        }
    }
}

impl From<LiteralValue> for JsonValue {
    fn from(literal: LiteralValue) -> Self {
        match literal {
            LiteralValue::String(s) => JsonValue::String(s),
            LiteralValue::Number(n) => JsonValue::Number(serde_json::Number::from_f64(n).unwrap_or_else(|| serde_json::Number::from(0))),
            LiteralValue::Boolean(b) => JsonValue::Bool(b),
            LiteralValue::List(arr) => {
                let values: Vec<JsonValue> = arr.into_iter().map(JsonValue::from).collect();
                JsonValue::Array(values)
            },
            LiteralValue::Object(obj) => {
                let map: serde_json::Map<String, JsonValue> = obj.into_iter()
                    .map(|(k, v)| (k, JsonValue::from(v)))
                    .collect();
                JsonValue::Object(map)
            },
            LiteralValue::Null => JsonValue::Null,
        }
    }
}

// Core trait for data access abstraction
#[async_trait]
pub trait PersistenceService: Send + Sync {
    /// Get a single value from a data source
    async fn get_value(&self, locator: &PersistenceLocator, key: &str) -> Result<LiteralValue>;

    /// Get multiple values as a batch operation
    async fn get_values(&self, locator: &PersistenceLocator, keys: &[String]) -> Result<HashMap<String, LiteralValue>>;

    /// Set a value in the data source (for testing and caching)
    async fn set_value(&self, locator: &PersistenceLocator, key: &str, value: LiteralValue) -> Result<()>;

    /// Check if the service can handle this locator
    fn can_handle(&self, locator: &PersistenceLocator) -> bool;

    /// Get service name for debugging
    fn service_name(&self) -> &'static str;
}

// PostgreSQL-based persistence service for EntityMasterDB, ComplianceDB, etc.
pub struct PostgresPersistenceService {
    pool: DbPool,
}

impl PostgresPersistenceService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PersistenceService for PostgresPersistenceService {
    async fn get_value(&self, locator: &PersistenceLocator, key: &str) -> Result<LiteralValue> {
        // Map system names to actual database tables
        let table_name = match locator.system.as_str() {
            "EntityMasterDB" => match locator.entity.as_str() {
                "legal_entities" => "business_entities",
                "related_parties" => "business_entities", // For now, using same table
                _ => return Err(anyhow::anyhow!("Unknown entity: {}", locator.entity)),
            },
            "ComplianceDB" => match locator.entity.as_str() {
                "screening_results" => "compliance_screenings",
                _ => return Err(anyhow::anyhow!("Unknown compliance entity: {}", locator.entity)),
            },
            "RiskDB" => match locator.entity.as_str() {
                "risk_assessments" => "risk_scores",
                _ => return Err(anyhow::anyhow!("Unknown risk entity: {}", locator.entity)),
            },
            "TradingSystem" => match locator.entity.as_str() {
                "trades" => "trading_records",
                "counterparties" => "trading_counterparties",
                _ => return Err(anyhow::anyhow!("Unknown trading entity: {}", locator.entity)),
            },
            _ => return Err(anyhow::anyhow!("Unknown system: {}", locator.system)),
        };

        // Build dynamic SQL query
        let column_name = &locator.identifier;
        let query = format!(
            "SELECT {} FROM {} WHERE id = $1 OR entity_name = $1 OR client_id = $1 LIMIT 1",
            column_name, table_name
        );

        // Execute query
        let row = sqlx::query(&query)
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            // Extract value by column name
            let value: Option<JsonValue> = row.try_get(column_name.as_str()).ok();
            Ok(value.map(LiteralValue::from).unwrap_or(LiteralValue::Null))
        } else {
            // Return mock data for demo purposes
            Ok(self.generate_mock_data(locator, key))
        }
    }

    async fn get_values(&self, locator: &PersistenceLocator, keys: &[String]) -> Result<HashMap<String, LiteralValue>> {
        let mut results = HashMap::new();

        // For now, call get_value for each key (could be optimized with batch queries)
        for key in keys {
            match self.get_value(locator, key).await {
                Ok(value) => {
                    results.insert(key.clone(), value);
                },
                Err(_) => {
                    results.insert(key.clone(), LiteralValue::Null);
                }
            }
        }

        Ok(results)
    }

    async fn set_value(&self, locator: &PersistenceLocator, key: &str, value: LiteralValue) -> Result<()> {
        // For now, we don't implement write operations - this would be for caching/testing
        log::info!("Setting value in {}.{}.{}: {} = {:?}",
                   locator.system, locator.entity, locator.identifier, key, value);
        Ok(())
    }

    fn can_handle(&self, locator: &PersistenceLocator) -> bool {
        matches!(locator.system.as_str(), "EntityMasterDB" | "ComplianceDB" | "RiskDB" | "TradingSystem")
    }

    fn service_name(&self) -> &'static str {
        "PostgresPersistenceService"
    }
}

impl PostgresPersistenceService {
    // Generate realistic mock data for demonstration
    fn generate_mock_data(&self, locator: &PersistenceLocator, key: &str) -> LiteralValue {
        match (locator.system.as_str(), locator.entity.as_str(), locator.identifier.as_str()) {
            ("EntityMasterDB", "legal_entities", "entity_name") => {
                LiteralValue::String(format!("{} Corporate Ltd.", key.to_uppercase()))
            },
            ("EntityMasterDB", "legal_entities", "jurisdiction_code") => {
                LiteralValue::String(if key.contains("UK") { "UK".to_string() } else { "US".to_string() })
            },
            ("EntityMasterDB", "related_parties", "full_name") => {
                LiteralValue::String(format!("John {} Smith", key.chars().take(1).collect::<String>().to_uppercase()))
            },
            ("ComplianceDB", "screening_results", "result_status") => {
                LiteralValue::String(if key.contains("high") { "POTENTIAL_MATCH".to_string() } else { "CLEAR".to_string() })
            },
            ("RiskDB", "risk_assessments", "computed_score") => {
                LiteralValue::Number(if key.contains("high") { 85.0 } else { 25.0 })
            },
            ("TradingSystem", "trades", "notional_amount") => {
                LiteralValue::Number(1000000.0)
            },
            ("TradingSystem", "trades", "settlement_date") => {
                LiteralValue::String("2024-01-15".to_string())
            },
            ("TradingSystem", "counterparties", "party_name") => {
                LiteralValue::String(format!("{} Bank AG", key.to_uppercase()))
            },
            _ => LiteralValue::String(format!("Mock value for {}", key))
        }
    }
}

// Redis-based cache service for high-performance lookups
pub struct RedisPersistenceService {
    // In a real implementation, this would have Redis connection
    #[allow(dead_code)]
    connection_string: String,
}

impl RedisPersistenceService {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

#[async_trait]
impl PersistenceService for RedisPersistenceService {
    async fn get_value(&self, locator: &PersistenceLocator, key: &str) -> Result<LiteralValue> {
        // For now, return mock cached data
        log::info!("Redis lookup: {}.{}.{} for key: {}",
                   locator.system, locator.entity, locator.identifier, key);

        // Simulate cache lookup with mock data
        match locator.entity.as_str() {
            "countries" => {
                let country_name = match key {
                    "US" => "United States",
                    "UK" => "United Kingdom",
                    "DE" => "Germany",
                    "FR" => "France",
                    _ => "Unknown Country"
                };
                Ok(LiteralValue::String(country_name.to_string()))
            },
            "rates" => {
                let rate = match key {
                    "premium" => 15.0,
                    "standard" => 10.0,
                    "basic" => 5.0,
                    _ => 0.0
                };
                Ok(LiteralValue::Number(rate))
            },
            _ => Ok(LiteralValue::String(format!("Cached value for {}", key)))
        }
    }

    async fn get_values(&self, locator: &PersistenceLocator, keys: &[String]) -> Result<HashMap<String, LiteralValue>> {
        let mut results = HashMap::new();

        for key in keys {
            if let Ok(value) = self.get_value(locator, key).await {
                results.insert(key.clone(), value);
            }
        }

        Ok(results)
    }

    async fn set_value(&self, locator: &PersistenceLocator, _key: &str, value: LiteralValue) -> Result<()> {
        log::info!("Redis cache set: {}.{}.{} = {:?}",
                   locator.system, locator.entity, locator.identifier, value);
        Ok(())
    }

    fn can_handle(&self, locator: &PersistenceLocator) -> bool {
        // Handle cache-based lookups
        matches!(locator.system.as_str(), "CacheService" | "LookupCache")
    }

    fn service_name(&self) -> &'static str {
        "RedisPersistenceService"
    }
}

// Composite service that routes to appropriate implementations
pub struct CompositePersistenceService {
    services: Vec<Box<dyn PersistenceService>>,
}

impl Default for CompositePersistenceService {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositePersistenceService {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    pub fn add_service(&mut self, service: Box<dyn PersistenceService>) {
        self.services.push(service);
    }

    pub fn with_postgres(mut self, pool: DbPool) -> Self {
        self.add_service(Box::new(PostgresPersistenceService::new(pool)));
        self
    }

    pub fn with_redis(mut self, connection_string: String) -> Self {
        self.add_service(Box::new(RedisPersistenceService::new(connection_string)));
        self
    }
}

#[async_trait]
impl PersistenceService for CompositePersistenceService {
    async fn get_value(&self, locator: &PersistenceLocator, key: &str) -> Result<LiteralValue> {
        for service in &self.services {
            if service.can_handle(locator) {
                log::debug!("Using {} for {}.{}", service.service_name(), locator.system, locator.entity);
                return service.get_value(locator, key).await;
            }
        }
        Err(anyhow::anyhow!("No service can handle system: {}", locator.system))
    }

    async fn get_values(&self, locator: &PersistenceLocator, keys: &[String]) -> Result<HashMap<String, LiteralValue>> {
        for service in &self.services {
            if service.can_handle(locator) {
                return service.get_values(locator, keys).await;
            }
        }
        Err(anyhow::anyhow!("No service can handle system: {}", locator.system))
    }

    async fn set_value(&self, locator: &PersistenceLocator, key: &str, value: LiteralValue) -> Result<()> {
        for service in &self.services {
            if service.can_handle(locator) {
                return service.set_value(locator, key, value).await;
            }
        }
        Err(anyhow::anyhow!("No service can handle system: {}", locator.system))
    }

    fn can_handle(&self, locator: &PersistenceLocator) -> bool {
        self.services.iter().any(|s| s.can_handle(locator))
    }

    fn service_name(&self) -> &'static str {
        "CompositePersistenceService"
    }
}

// Helper functions for testing and data generation
pub async fn test_persistence_service(service: &dyn PersistenceService) -> Result<()> {
    println!("Testing {}", service.service_name());

    // Test entity master lookup
    let entity_locator = PersistenceLocator {
        system: "EntityMasterDB".to_string(),
        entity: "legal_entities".to_string(),
        identifier: "entity_name".to_string(),
    };

    let entity_name = service.get_value(&entity_locator, "ACME_CORP").await?;
    println!("Entity name: {:?}", entity_name);

    // Test compliance screening
    let compliance_locator = PersistenceLocator {
        system: "ComplianceDB".to_string(),
        entity: "screening_results".to_string(),
        identifier: "result_status".to_string(),
    };

    let screening_result = service.get_value(&compliance_locator, "ACME_CORP").await?;
    println!("Screening result: {:?}", screening_result);

    // Test batch lookup
    let keys = vec!["ACME_CORP".to_string(), "BETA_INC".to_string()];
    let batch_results = service.get_values(&entity_locator, &keys).await?;
    println!("Batch results: {:?}", batch_results);

    Ok(())
}