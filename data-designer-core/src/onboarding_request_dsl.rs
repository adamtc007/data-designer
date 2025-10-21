// Onboarding Request DSL for CRUD operations with Deal Record integration
// Grammar:
// CREATE ONBOARDING REQUEST '<deal_id>' ; '<description>' ; '<onboarding_id>' WITH
//   CBU '<cbu_id>' AND
//   PRODUCT '<product_id_1>' [AND PRODUCT '<product_id_2>' ...] [AND PRODUCT '<product_id_n>']
//
// UPDATE ONBOARDING REQUEST '<onboarding_id>' SET <field> = '<value>'
// DELETE ONBOARDING REQUEST '<onboarding_id>'
// QUERY ONBOARDING REQUEST [WHERE <condition>]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingRequestDslCommand {
    pub operation: OnboardingOperation,
    pub deal_id: Option<String>,
    pub description: Option<String>,
    pub onboarding_id: Option<String>,
    pub cbu_id: Option<String>,
    pub product_ids: Vec<String>,
    pub update_fields: HashMap<String, String>,
    pub query_conditions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnboardingOperation {
    Create,
    Update,
    Delete,
    Query,
}

#[derive(Debug, Clone)]
pub struct OnboardingRequestDslParser {
    pub pool: Option<PgPool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingRequestDslResult {
    pub success: bool,
    pub message: String,
    pub onboarding_id: Option<String>,
    pub deal_id: Option<String>,
    pub validation_errors: Vec<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum OnboardingRequestDslError {
    ParseError(String),
    ValidationError(String),
    DatabaseError(String),
    DealRecordNotFound(String),
    CbuNotFound(String),
    ProductNotFound(String),
}

impl std::fmt::Display for OnboardingRequestDslError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OnboardingRequestDslError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            OnboardingRequestDslError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            OnboardingRequestDslError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            OnboardingRequestDslError::DealRecordNotFound(msg) => write!(f, "Deal Record Not Found: {}", msg),
            OnboardingRequestDslError::CbuNotFound(msg) => write!(f, "CBU Not Found: {}", msg),
            OnboardingRequestDslError::ProductNotFound(msg) => write!(f, "Product Not Found: {}", msg),
        }
    }
}

impl std::error::Error for OnboardingRequestDslError {}

impl OnboardingRequestDslParser {
    pub fn new(pool: Option<PgPool>) -> Self {
        Self { pool }
    }

    /// Parse Onboarding Request DSL command into structured format
    pub fn parse_onboarding_request_dsl(&self, dsl_text: &str) -> Result<OnboardingRequestDslCommand, OnboardingRequestDslError> {
        let dsl_text = dsl_text.trim();

        if dsl_text.to_uppercase().starts_with("CREATE ONBOARDING REQUEST") {
            self.parse_create_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("UPDATE ONBOARDING REQUEST") {
            self.parse_update_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("DELETE ONBOARDING REQUEST") {
            self.parse_delete_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("QUERY ONBOARDING REQUEST") {
            self.parse_query_command(dsl_text)
        } else {
            Err(OnboardingRequestDslError::ParseError(
                "Command must start with CREATE ONBOARDING REQUEST, UPDATE ONBOARDING REQUEST, DELETE ONBOARDING REQUEST, or QUERY ONBOARDING REQUEST".to_string()
            ))
        }
    }

    /// Parse CREATE ONBOARDING REQUEST command
    fn parse_create_command(&self, dsl_text: &str) -> Result<OnboardingRequestDslCommand, OnboardingRequestDslError> {
        // CREATE ONBOARDING REQUEST 'DEAL001' ; 'Client Alpha Onboarding' ; 'OBR001' WITH
        //   CBU 'CBU001' AND
        //   PRODUCT 'PROD001' AND PRODUCT 'PROD002' AND PRODUCT 'PROD003'

        let parts: Vec<&str> = dsl_text.splitn(2, " WITH ").collect();
        if parts.len() != 2 {
            return Err(OnboardingRequestDslError::ParseError(
                "CREATE ONBOARDING REQUEST command must include ' WITH ' clause for CBU and products".to_string()
            ));
        }

        // Parse onboarding request parameters
        let request_part = parts[0].trim();
        let resources_part = parts[1].trim();

        // Extract deal_id, description, and onboarding_id from: CREATE ONBOARDING REQUEST 'deal_id' ; 'description' ; 'onboarding_id'
        let request_content = request_part.strip_prefix("CREATE ONBOARDING REQUEST").unwrap().trim();
        let request_parts: Vec<&str> = request_content.splitn(3, " ; ").collect();

        if request_parts.len() != 3 {
            return Err(OnboardingRequestDslError::ParseError(
                "CREATE ONBOARDING REQUEST must have format: CREATE ONBOARDING REQUEST 'deal_id' ; 'description' ; 'onboarding_id'".to_string()
            ));
        }

        let deal_id = self.extract_quoted_string(request_parts[0].trim())?;
        let description = self.extract_quoted_string(request_parts[1].trim())?;
        let onboarding_id = self.extract_quoted_string(request_parts[2].trim())?;

        // Parse CBU and products
        let (cbu_id, product_ids) = self.parse_cbu_and_products(resources_part)?;

        // Validate we have at least one product
        if product_ids.is_empty() {
            return Err(OnboardingRequestDslError::ValidationError(
                "At least one PRODUCT must be specified".to_string()
            ));
        }

        Ok(OnboardingRequestDslCommand {
            operation: OnboardingOperation::Create,
            deal_id: Some(deal_id),
            description: Some(description),
            onboarding_id: Some(onboarding_id),
            cbu_id: Some(cbu_id),
            product_ids,
            update_fields: HashMap::new(),
            query_conditions: None,
        })
    }

    /// Parse UPDATE ONBOARDING REQUEST command
    fn parse_update_command(&self, dsl_text: &str) -> Result<OnboardingRequestDslCommand, OnboardingRequestDslError> {
        // UPDATE ONBOARDING REQUEST 'OBR001' SET description = 'Updated description'
        let content = dsl_text.strip_prefix("UPDATE ONBOARDING REQUEST").unwrap().trim();
        let parts: Vec<&str> = content.splitn(2, " SET ").collect();

        if parts.len() != 2 {
            return Err(OnboardingRequestDslError::ParseError(
                "UPDATE ONBOARDING REQUEST command must include ' SET ' clause".to_string()
            ));
        }

        let onboarding_id = self.extract_quoted_string(parts[0].trim())?;
        let set_clause = parts[1].trim();

        // Parse SET clause: field = 'value'
        let mut update_fields = HashMap::new();
        let assignments: Vec<&str> = set_clause.split(" AND ").collect();

        for assignment in assignments {
            let assign_parts: Vec<&str> = assignment.splitn(2, " = ").collect();
            if assign_parts.len() != 2 {
                return Err(OnboardingRequestDslError::ParseError(
                    "SET clause must have format: field = 'value'".to_string()
                ));
            }

            let field = assign_parts[0].trim().to_string();
            let value = self.extract_quoted_string(assign_parts[1].trim())?;
            update_fields.insert(field, value);
        }

        Ok(OnboardingRequestDslCommand {
            operation: OnboardingOperation::Update,
            deal_id: None,
            description: None,
            onboarding_id: Some(onboarding_id),
            cbu_id: None,
            product_ids: Vec::new(),
            update_fields,
            query_conditions: None,
        })
    }

    /// Parse DELETE ONBOARDING REQUEST command
    fn parse_delete_command(&self, dsl_text: &str) -> Result<OnboardingRequestDslCommand, OnboardingRequestDslError> {
        // DELETE ONBOARDING REQUEST 'OBR001'
        let content = dsl_text.strip_prefix("DELETE ONBOARDING REQUEST").unwrap().trim();
        let onboarding_id = self.extract_quoted_string(content)?;

        Ok(OnboardingRequestDslCommand {
            operation: OnboardingOperation::Delete,
            deal_id: None,
            description: None,
            onboarding_id: Some(onboarding_id),
            cbu_id: None,
            product_ids: Vec::new(),
            update_fields: HashMap::new(),
            query_conditions: None,
        })
    }

    /// Parse QUERY ONBOARDING REQUEST command
    fn parse_query_command(&self, dsl_text: &str) -> Result<OnboardingRequestDslCommand, OnboardingRequestDslError> {
        // QUERY ONBOARDING REQUEST WHERE status = 'active'
        let content = dsl_text.strip_prefix("QUERY ONBOARDING REQUEST").unwrap().trim();

        let query_conditions = if content.is_empty() {
            None
        } else if content.to_uppercase().starts_with("WHERE ") {
            Some(content.strip_prefix("WHERE ").unwrap().trim().to_string())
        } else {
            return Err(OnboardingRequestDslError::ParseError(
                "QUERY ONBOARDING REQUEST can be used alone or with WHERE clause".to_string()
            ));
        };

        Ok(OnboardingRequestDslCommand {
            operation: OnboardingOperation::Query,
            deal_id: None,
            description: None,
            onboarding_id: None,
            cbu_id: None,
            product_ids: Vec::new(),
            update_fields: HashMap::new(),
            query_conditions,
        })
    }

    /// Parse CBU and products from the WITH clause
    fn parse_cbu_and_products(&self, resources_text: &str) -> Result<(String, Vec<String>), OnboardingRequestDslError> {
        // CBU 'CBU001' AND PRODUCT 'PROD001' AND PRODUCT 'PROD002' AND PRODUCT 'PROD003'

        let parts: Vec<&str> = resources_text.split(" AND ").collect();
        let mut cbu_id = None;
        let mut product_ids = Vec::new();

        for part in parts {
            let part = part.trim();

            if part.to_uppercase().starts_with("CBU ") {
                let cbu_content = part.strip_prefix("CBU ").unwrap().trim();
                cbu_id = Some(self.extract_quoted_string(cbu_content)?);
            } else if part.to_uppercase().starts_with("PRODUCT ") {
                let product_content = part.strip_prefix("PRODUCT ").unwrap().trim();
                let product_id = self.extract_quoted_string(product_content)?;
                product_ids.push(product_id);
            } else {
                return Err(OnboardingRequestDslError::ParseError(
                    format!("Invalid resource specification: {}. Must be 'CBU <id>' or 'PRODUCT <id>'", part)
                ));
            }
        }

        let cbu_id = cbu_id.ok_or_else(|| OnboardingRequestDslError::ParseError(
            "CBU must be specified in WITH clause".to_string()
        ))?;

        Ok((cbu_id, product_ids))
    }

    /// Extract string from quotes
    fn extract_quoted_string(&self, text: &str) -> Result<String, OnboardingRequestDslError> {
        let text = text.trim();
        if text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2 {
            Ok(text[1..text.len()-1].to_string())
        } else if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
            Ok(text[1..text.len()-1].to_string())
        } else {
            Err(OnboardingRequestDslError::ParseError(
                format!("String must be quoted: {}", text)
            ))
        }
    }

    /// Execute Onboarding Request DSL command
    pub async fn execute_onboarding_request_dsl(&self, command: OnboardingRequestDslCommand) -> Result<OnboardingRequestDslResult, OnboardingRequestDslError> {
        match command.operation {
            OnboardingOperation::Create => self.execute_create(command).await,
            OnboardingOperation::Update => self.execute_update(command).await,
            OnboardingOperation::Delete => self.execute_delete(command).await,
            OnboardingOperation::Query => self.execute_query(command).await,
        }
    }

    /// Validate that Deal Record, CBU, and Products exist
    async fn validate_references(&self, command: &OnboardingRequestDslCommand) -> Result<Vec<String>, OnboardingRequestDslError> {
        let Some(pool) = &self.pool else {
            return Err(OnboardingRequestDslError::DatabaseError("No database connection available".to_string()));
        };

        let mut validation_errors = Vec::new();

        // Validate Deal Record exists (Deal ID is primary key)
        if let Some(deal_id) = &command.deal_id {
            let deal_query = "SELECT COUNT(*) as count FROM deal_records WHERE deal_id = $1";
            match sqlx::query(deal_query)
                .bind(deal_id)
                .fetch_one(pool)
                .await
            {
                Ok(row) => {
                    let count: i64 = row.get("count");
                    if count == 0 {
                        validation_errors.push(format!("Deal Record '{}' not found", deal_id));
                    }
                }
                Err(e) => {
                    return Err(OnboardingRequestDslError::DatabaseError(format!("Failed to validate deal record: {}", e)));
                }
            }
        }

        // Validate CBU exists
        if let Some(cbu_id) = &command.cbu_id {
            let cbu_query = "SELECT COUNT(*) as count FROM cbu WHERE cbu_id = $1";
            match sqlx::query(cbu_query)
                .bind(cbu_id)
                .fetch_one(pool)
                .await
            {
                Ok(row) => {
                    let count: i64 = row.get("count");
                    if count == 0 {
                        validation_errors.push(format!("CBU '{}' not found", cbu_id));
                    }
                }
                Err(e) => {
                    return Err(OnboardingRequestDslError::DatabaseError(format!("Failed to validate CBU: {}", e)));
                }
            }
        }

        // Validate all products exist
        for product_id in &command.product_ids {
            let product_query = "SELECT COUNT(*) as count FROM products WHERE product_id = $1";
            match sqlx::query(product_query)
                .bind(product_id)
                .fetch_one(pool)
                .await
            {
                Ok(row) => {
                    let count: i64 = row.get("count");
                    if count == 0 {
                        validation_errors.push(format!("Product '{}' not found", product_id));
                    }
                }
                Err(e) => {
                    return Err(OnboardingRequestDslError::DatabaseError(format!("Failed to validate product: {}", e)));
                }
            }
        }

        Ok(validation_errors)
    }

    /// Execute CREATE ONBOARDING REQUEST command
    async fn execute_create(&self, command: OnboardingRequestDslCommand) -> Result<OnboardingRequestDslResult, OnboardingRequestDslError> {
        let Some(pool) = &self.pool else {
            return Err(OnboardingRequestDslError::DatabaseError("No database connection available".to_string()));
        };

        // Validate references exist
        let validation_errors = self.validate_references(&command).await?;
        if !validation_errors.is_empty() {
            return Ok(OnboardingRequestDslResult {
                success: false,
                message: "Reference validation failed".to_string(),
                onboarding_id: None,
                deal_id: None,
                validation_errors,
                data: None,
            });
        }

        // Insert onboarding request into database
        let insert_query = r#"
            INSERT INTO onboarding_requests (
                onboarding_id, deal_id, cbu_id, description, status, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            RETURNING id
        "#;

        let onboarding_id = command.onboarding_id.as_ref().unwrap();
        let deal_id = command.deal_id.as_ref().unwrap();
        let cbu_id = command.cbu_id.as_ref().unwrap();
        let description = command.description.as_ref().unwrap();

        match sqlx::query(insert_query)
            .bind(onboarding_id)
            .bind(deal_id)
            .bind(cbu_id)
            .bind(description)
            .bind("pending")
            .fetch_one(pool)
            .await
        {
            Ok(_) => {
                // Insert product associations
                for product_id in &command.product_ids {
                    let product_query = r#"
                        INSERT INTO onboarding_request_products (onboarding_id, product_id, created_at)
                        VALUES ($1, $2, NOW())
                    "#;

                    if let Err(e) = sqlx::query(product_query)
                        .bind(onboarding_id)
                        .bind(product_id)
                        .execute(pool)
                        .await
                    {
                        return Err(OnboardingRequestDslError::DatabaseError(
                            format!("Failed to create product association: {}", e)
                        ));
                    }
                }

                Ok(OnboardingRequestDslResult {
                    success: true,
                    message: format!("Onboarding Request '{}' created successfully", onboarding_id),
                    onboarding_id: Some(onboarding_id.clone()),
                    deal_id: Some(deal_id.clone()),
                    validation_errors: Vec::new(),
                    data: None,
                })
            }
            Err(e) => Err(OnboardingRequestDslError::DatabaseError(format!("Failed to create onboarding request: {}", e)))
        }
    }

    /// Execute UPDATE ONBOARDING REQUEST command
    async fn execute_update(&self, command: OnboardingRequestDslCommand) -> Result<OnboardingRequestDslResult, OnboardingRequestDslError> {
        let Some(pool) = &self.pool else {
            return Err(OnboardingRequestDslError::DatabaseError("No database connection available".to_string()));
        };

        let onboarding_id = command.onboarding_id.as_ref().unwrap();

        // Build dynamic UPDATE query
        let mut set_clauses = Vec::new();
        let mut values: Vec<&str> = Vec::new();

        for (field, value) in &command.update_fields {
            set_clauses.push(format!("{} = ${}", field, values.len() + 2));
            values.push(value);
        }

        if set_clauses.is_empty() {
            return Err(OnboardingRequestDslError::ValidationError("No fields to update".to_string()));
        }

        let update_query = format!(
            "UPDATE onboarding_requests SET {}, updated_at = NOW() WHERE onboarding_id = $1",
            set_clauses.join(", ")
        );

        let mut query = sqlx::query(&update_query).bind(onboarding_id);
        for value in values {
            query = query.bind(value);
        }

        match query.execute(pool).await {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    Ok(OnboardingRequestDslResult {
                        success: true,
                        message: format!("Onboarding Request '{}' updated successfully", onboarding_id),
                        onboarding_id: Some(onboarding_id.clone()),
                        deal_id: None,
                        validation_errors: Vec::new(),
                        data: None,
                    })
                } else {
                    Err(OnboardingRequestDslError::ValidationError(format!("Onboarding Request '{}' not found", onboarding_id)))
                }
            }
            Err(e) => Err(OnboardingRequestDslError::DatabaseError(format!("Failed to update onboarding request: {}", e)))
        }
    }

    /// Execute DELETE ONBOARDING REQUEST command
    async fn execute_delete(&self, command: OnboardingRequestDslCommand) -> Result<OnboardingRequestDslResult, OnboardingRequestDslError> {
        let Some(pool) = &self.pool else {
            return Err(OnboardingRequestDslError::DatabaseError("No database connection available".to_string()));
        };

        let onboarding_id = command.onboarding_id.as_ref().unwrap();

        // Delete product associations first
        let delete_products_query = "DELETE FROM onboarding_request_products WHERE onboarding_id = $1";
        if let Err(e) = sqlx::query(delete_products_query)
            .bind(onboarding_id)
            .execute(pool)
            .await
        {
            return Err(OnboardingRequestDslError::DatabaseError(
                format!("Failed to delete product associations: {}", e)
            ));
        }

        // Delete onboarding request
        let delete_query = "DELETE FROM onboarding_requests WHERE onboarding_id = $1";
        match sqlx::query(delete_query)
            .bind(onboarding_id)
            .execute(pool)
            .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    Ok(OnboardingRequestDslResult {
                        success: true,
                        message: format!("Onboarding Request '{}' deleted successfully", onboarding_id),
                        onboarding_id: Some(onboarding_id.clone()),
                        deal_id: None,
                        validation_errors: Vec::new(),
                        data: None,
                    })
                } else {
                    Err(OnboardingRequestDslError::ValidationError(format!("Onboarding Request '{}' not found", onboarding_id)))
                }
            }
            Err(e) => Err(OnboardingRequestDslError::DatabaseError(format!("Failed to delete onboarding request: {}", e)))
        }
    }

    /// Execute QUERY ONBOARDING REQUEST command
    async fn execute_query(&self, command: OnboardingRequestDslCommand) -> Result<OnboardingRequestDslResult, OnboardingRequestDslError> {
        let Some(pool) = &self.pool else {
            return Err(OnboardingRequestDslError::DatabaseError("No database connection available".to_string()));
        };

        let base_query = r#"
            SELECT
                or.*,
                array_agg(orp.product_id) as product_ids
            FROM onboarding_requests or
            LEFT JOIN onboarding_request_products orp ON or.onboarding_id = orp.onboarding_id
        "#;

        let (final_query, _where_clause) = if let Some(conditions) = &command.query_conditions {
            (format!("{} WHERE {} GROUP BY or.id", base_query, conditions), true)
        } else {
            (format!("{} GROUP BY or.id", base_query), false)
        };

        match sqlx::query(&final_query).fetch_all(pool).await {
            Ok(rows) => {
                let mut results = Vec::new();
                for row in rows {
                    let mut request_data = serde_json::Map::new();
                    request_data.insert("onboarding_id".to_string(), serde_json::Value::String(row.get("onboarding_id")));
                    request_data.insert("deal_id".to_string(), serde_json::Value::String(row.get("deal_id")));
                    request_data.insert("cbu_id".to_string(), serde_json::Value::String(row.get("cbu_id")));
                    request_data.insert("description".to_string(), serde_json::Value::String(row.get("description")));
                    request_data.insert("status".to_string(), serde_json::Value::String(row.get("status")));

                    let product_ids: Vec<String> = row.get("product_ids");
                    request_data.insert("product_ids".to_string(), serde_json::Value::Array(
                        product_ids.into_iter().map(serde_json::Value::String).collect()
                    ));

                    results.push(serde_json::Value::Object(request_data));
                }

                Ok(OnboardingRequestDslResult {
                    success: true,
                    message: format!("Found {} onboarding request(s)", results.len()),
                    onboarding_id: None,
                    deal_id: None,
                    validation_errors: Vec::new(),
                    data: Some(serde_json::Value::Array(results)),
                })
            }
            Err(e) => Err(OnboardingRequestDslError::DatabaseError(format!("Failed to query onboarding requests: {}", e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_onboarding_request_command() {
        let parser = OnboardingRequestDslParser::new(None);
        let dsl = r#"CREATE ONBOARDING REQUEST 'DEAL001' ; 'Client Alpha Onboarding' ; 'OBR001' WITH
            CBU 'CBU001' AND
            PRODUCT 'PROD001' AND PRODUCT 'PROD002' AND PRODUCT 'PROD003'"#;

        let result = parser.parse_onboarding_request_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, OnboardingOperation::Create));
        assert_eq!(command.deal_id, Some("DEAL001".to_string()));
        assert_eq!(command.description, Some("Client Alpha Onboarding".to_string()));
        assert_eq!(command.onboarding_id, Some("OBR001".to_string()));
        assert_eq!(command.cbu_id, Some("CBU001".to_string()));
        assert_eq!(command.product_ids, vec!["PROD001", "PROD002", "PROD003"]);
    }

    #[test]
    fn test_parse_update_onboarding_request_command() {
        let parser = OnboardingRequestDslParser::new(None);
        let dsl = "UPDATE ONBOARDING REQUEST 'OBR001' SET description = 'Updated description'";

        let result = parser.parse_onboarding_request_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, OnboardingOperation::Update));
        assert_eq!(command.onboarding_id, Some("OBR001".to_string()));
        assert_eq!(command.update_fields.get("description"), Some(&"Updated description".to_string()));
    }

    #[test]
    fn test_parse_delete_onboarding_request_command() {
        let parser = OnboardingRequestDslParser::new(None);
        let dsl = "DELETE ONBOARDING REQUEST 'OBR001'";

        let result = parser.parse_onboarding_request_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, OnboardingOperation::Delete));
        assert_eq!(command.onboarding_id, Some("OBR001".to_string()));
    }

    #[test]
    fn test_parse_query_onboarding_request_command() {
        let parser = OnboardingRequestDslParser::new(None);
        let dsl = "QUERY ONBOARDING REQUEST WHERE status = 'pending'";

        let result = parser.parse_onboarding_request_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, OnboardingOperation::Query));
        assert_eq!(command.query_conditions, Some("status = 'pending'".to_string()));
    }

    #[test]
    fn test_single_product() {
        let parser = OnboardingRequestDslParser::new(None);
        let dsl = r#"CREATE ONBOARDING REQUEST 'DEAL002' ; 'Simple Onboarding' ; 'OBR002' WITH
            CBU 'CBU002' AND
            PRODUCT 'PROD004'"#;

        let result = parser.parse_onboarding_request_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert_eq!(command.product_ids, vec!["PROD004"]);
    }
}