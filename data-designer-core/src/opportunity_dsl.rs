// Opportunity DSL - Commercial negotiation entity with revenue stream modeling
// Represents initial client negotiations and revenue projections
//
// Grammar:
// CREATE OPPORTUNITY '<opportunity_id>' ; '<client_name>' ; '<description>' WITH
//   CBU '<cbu_id_1>' [AND CBU '<cbu_id_2>' ...] AND
//   PRODUCT '<product_id_1>' [AND PRODUCT '<product_id_2>' ...] AND
//   REVENUE_STREAM '<stream_type>' '<amount_per_annum>' [AND REVENUE_STREAM '<type>' '<amount>' ...]
//
// Examples:
// CREATE OPPORTUNITY 'OPP001' ; 'Alpha Corporation' ; 'Multi-product custody and fund accounting' WITH
//   CBU 'CBU001' AND CBU 'CBU002' AND
//   PRODUCT 'CUSTODY_SERVICES' AND PRODUCT 'FUND_ACCOUNTING' AND
//   REVENUE_STREAM 'custody' '$1000000' AND REVENUE_STREAM 'fund_accounting' '$250000'
//
// UPDATE OPPORTUNITY '<opportunity_id>' SET <field> = '<value>'
// DELETE OPPORTUNITY '<opportunity_id>'
// QUERY OPPORTUNITY [WHERE <condition>]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sqlx::{PgPool, Row};
use crate::dsl_utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityDslCommand {
    pub operation: OpportunityOperation,
    pub opportunity_id: Option<String>,
    pub client_name: Option<String>,
    pub description: Option<String>,
    pub cbu_ids: Vec<String>,
    pub product_ids: Vec<String>,
    pub revenue_streams: Vec<RevenueStream>,
    pub update_fields: HashMap<String, String>,
    pub query_conditions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityOperation {
    Create,
    Update,
    Delete,
    Query,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueStream {
    pub stream_type: String,
    pub amount_per_annum: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub opportunity_id: String,
    pub client_name: String,
    pub description: String,
    pub status: String,
    pub total_revenue_projection: f64,
    pub cbu_count: i32,
    pub product_count: i32,
    pub revenue_stream_count: i32,
    pub probability_percentage: Option<f32>,
    pub expected_close_date: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct OpportunityDslParser {
    pub pool: Option<PgPool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityDslResult {
    pub success: bool,
    pub message: String,
    pub opportunity_id: Option<String>,
    pub validation_errors: Vec<String>,
    pub data: Option<serde_json::Value>,
    pub revenue_summary: Option<RevenueSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueSummary {
    pub opportunity_id: String,
    pub client_name: String,
    pub total_annual_revenue: f64,
    pub revenue_breakdown: Vec<RevenueBreakdown>,
    pub business_scope: BusinessScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueBreakdown {
    pub stream_type: String,
    pub annual_amount: f64,
    pub percentage_of_total: f32,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessScope {
    pub total_cbus: i32,
    pub total_products: i32,
    pub key_services: Vec<String>,
    pub business_value_tier: String, // "High", "Medium", "Low" based on revenue
}

#[derive(Debug)]
pub enum OpportunityDslError {
    ParseError(String),
    ValidationError(String),
    DatabaseError(String),
    ResourceNotFound(String, String),
    InvalidRevenueFormat(String),
}

impl std::fmt::Display for OpportunityDslError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpportunityDslError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            OpportunityDslError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            OpportunityDslError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            OpportunityDslError::ResourceNotFound(resource_type, resource_id) => {
                write!(f, "{} '{}' not found", resource_type, resource_id)
            },
            OpportunityDslError::InvalidRevenueFormat(msg) => write!(f, "Invalid Revenue Format: {}", msg),
        }
    }
}

impl std::error::Error for OpportunityDslError {}

impl OpportunityDslParser {
    pub fn new(pool: Option<PgPool>) -> Self {
        Self { pool }
    }

    /// Parse Opportunity DSL command into structured format
    pub fn parse_opportunity_dsl(&self, dsl_text: &str) -> Result<OpportunityDslCommand, OpportunityDslError> {
        let cleaned_text = dsl_utils::strip_comments(dsl_text);
        let dsl_text = cleaned_text.trim();

        if dsl_text.to_uppercase().starts_with("CREATE OPPORTUNITY") {
            self.parse_create_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("UPDATE OPPORTUNITY") {
            self.parse_update_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("DELETE OPPORTUNITY") {
            self.parse_delete_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("QUERY OPPORTUNITY") {
            self.parse_query_command(dsl_text)
        } else {
            Err(OpportunityDslError::ParseError(
                "Command must start with CREATE OPPORTUNITY, UPDATE OPPORTUNITY, DELETE OPPORTUNITY, or QUERY OPPORTUNITY".to_string()
            ))
        }
    }

    /// Parse CREATE OPPORTUNITY command
    fn parse_create_command(&self, dsl_text: &str) -> Result<OpportunityDslCommand, OpportunityDslError> {
        // CREATE OPPORTUNITY 'OPP001' ; 'Alpha Corporation' ; 'Multi-product custody and fund accounting' WITH
        //   CBU 'CBU001' AND CBU 'CBU002' AND
        //   PRODUCT 'CUSTODY_SERVICES' AND PRODUCT 'FUND_ACCOUNTING' AND
        //   REVENUE_STREAM 'custody' '$1000000' AND REVENUE_STREAM 'fund_accounting' '$250000'

        let parts: Vec<&str> = dsl_text.splitn(2, " WITH ").collect();
        if parts.len() != 2 {
            return Err(OpportunityDslError::ParseError(
                "CREATE OPPORTUNITY command must include ' WITH ' clause".to_string()
            ));
        }

        // Parse opportunity parameters
        let opportunity_part = parts[0].trim();
        let resources_part = parts[1].trim();

        // Extract opportunity_id, client_name, and description
        let opportunity_content = opportunity_part.strip_prefix("CREATE OPPORTUNITY").unwrap().trim();
        let opportunity_parts: Vec<&str> = opportunity_content.splitn(3, " ; ").collect();

        if opportunity_parts.len() != 3 {
            return Err(OpportunityDslError::ParseError(
                "CREATE OPPORTUNITY must have format: CREATE OPPORTUNITY 'opportunity_id' ; 'client_name' ; 'description'".to_string()
            ));
        }

        let opportunity_id = self.extract_quoted_string(opportunity_parts[0].trim())?;
        let client_name = self.extract_quoted_string(opportunity_parts[1].trim())?;
        let description = self.extract_quoted_string(opportunity_parts[2].trim())?;

        // Parse CBUs, products, and revenue streams
        let (cbu_ids, product_ids, revenue_streams) = self.parse_opportunity_resources(resources_part)?;

        Ok(OpportunityDslCommand {
            operation: OpportunityOperation::Create,
            opportunity_id: Some(opportunity_id),
            client_name: Some(client_name),
            description: Some(description),
            cbu_ids,
            product_ids,
            revenue_streams,
            update_fields: HashMap::new(),
            query_conditions: None,
        })
    }

    /// Parse UPDATE OPPORTUNITY command
    fn parse_update_command(&self, dsl_text: &str) -> Result<OpportunityDslCommand, OpportunityDslError> {
        let content = dsl_text.strip_prefix("UPDATE OPPORTUNITY").unwrap().trim();
        let parts: Vec<&str> = content.splitn(2, " SET ").collect();

        if parts.len() != 2 {
            return Err(OpportunityDslError::ParseError(
                "UPDATE OPPORTUNITY command must include ' SET ' clause".to_string()
            ));
        }

        let opportunity_id = self.extract_quoted_string(parts[0].trim())?;
        let set_clause = parts[1].trim();

        let mut update_fields = HashMap::new();
        let assignments: Vec<&str> = set_clause.split(" AND ").collect();

        for assignment in assignments {
            let assign_parts: Vec<&str> = assignment.splitn(2, " = ").collect();
            if assign_parts.len() != 2 {
                return Err(OpportunityDslError::ParseError(
                    "SET clause must have format: field = 'value'".to_string()
                ));
            }

            let field = assign_parts[0].trim().to_string();
            let value = self.extract_quoted_string(assign_parts[1].trim())?;
            update_fields.insert(field, value);
        }

        Ok(OpportunityDslCommand {
            operation: OpportunityOperation::Update,
            opportunity_id: Some(opportunity_id),
            client_name: None,
            description: None,
            cbu_ids: Vec::new(),
            product_ids: Vec::new(),
            revenue_streams: Vec::new(),
            update_fields,
            query_conditions: None,
        })
    }

    /// Parse DELETE OPPORTUNITY command
    fn parse_delete_command(&self, dsl_text: &str) -> Result<OpportunityDslCommand, OpportunityDslError> {
        let content = dsl_text.strip_prefix("DELETE OPPORTUNITY").unwrap().trim();
        let opportunity_id = self.extract_quoted_string(content)?;

        Ok(OpportunityDslCommand {
            operation: OpportunityOperation::Delete,
            opportunity_id: Some(opportunity_id),
            client_name: None,
            description: None,
            cbu_ids: Vec::new(),
            product_ids: Vec::new(),
            revenue_streams: Vec::new(),
            update_fields: HashMap::new(),
            query_conditions: None,
        })
    }

    /// Parse QUERY OPPORTUNITY command
    fn parse_query_command(&self, dsl_text: &str) -> Result<OpportunityDslCommand, OpportunityDslError> {
        let content = dsl_text.strip_prefix("QUERY OPPORTUNITY").unwrap().trim();

        let query_conditions = if content.is_empty() {
            None
        } else if content.to_uppercase().starts_with("WHERE ") {
            Some(content.strip_prefix("WHERE ").unwrap().trim().to_string())
        } else {
            return Err(OpportunityDslError::ParseError(
                "QUERY OPPORTUNITY can be used alone or with WHERE clause".to_string()
            ));
        };

        Ok(OpportunityDslCommand {
            operation: OpportunityOperation::Query,
            opportunity_id: None,
            client_name: None,
            description: None,
            cbu_ids: Vec::new(),
            product_ids: Vec::new(),
            revenue_streams: Vec::new(),
            update_fields: HashMap::new(),
            query_conditions,
        })
    }

    /// Parse opportunity resources: CBUs, products, and revenue streams
    fn parse_opportunity_resources(&self, resources_text: &str) -> Result<(Vec<String>, Vec<String>, Vec<RevenueStream>), OpportunityDslError> {
        let mut cbu_ids = Vec::new();
        let mut product_ids = Vec::new();
        let mut revenue_streams = Vec::new();

        // Split by AND to get individual resource declarations
        let resource_declarations: Vec<&str> = resources_text.split(" AND ").collect();

        for declaration in resource_declarations {
            let declaration = declaration.trim();

            if declaration.to_uppercase().starts_with("CBU ") {
                let resource_id = self.extract_quoted_string(declaration.strip_prefix("CBU ").unwrap().trim())?;
                cbu_ids.push(resource_id);
            } else if declaration.to_uppercase().starts_with("PRODUCT ") {
                let resource_id = self.extract_quoted_string(declaration.strip_prefix("PRODUCT ").unwrap().trim())?;
                product_ids.push(resource_id);
            } else if declaration.to_uppercase().starts_with("REVENUE_STREAM ") {
                let revenue_content = declaration.strip_prefix("REVENUE_STREAM ").unwrap().trim();
                let revenue_stream = self.parse_revenue_stream(revenue_content)?;
                revenue_streams.push(revenue_stream);
            } else {
                return Err(OpportunityDslError::ParseError(
                    format!("Invalid resource specification: {}. Must be CBU, PRODUCT, or REVENUE_STREAM", declaration)
                ));
            }
        }

        Ok((cbu_ids, product_ids, revenue_streams))
    }

    /// Parse revenue stream: 'stream_type' '$amount'
    fn parse_revenue_stream(&self, revenue_text: &str) -> Result<RevenueStream, OpportunityDslError> {
        // REVENUE_STREAM 'custody' '$1000000'
        let parts: Vec<&str> = revenue_text.splitn(2, " ").collect();
        if parts.len() != 2 {
            return Err(OpportunityDslError::InvalidRevenueFormat(
                "REVENUE_STREAM must have format: 'stream_type' '$amount'".to_string()
            ));
        }

        let stream_type = self.extract_quoted_string(parts[0].trim())?;
        let amount_str = self.extract_quoted_string(parts[1].trim())?;

        // Parse amount (remove $ and convert to float)
        let (currency, amount) = if let Some(stripped) = amount_str.strip_prefix('$') {
            ("USD".to_string(), stripped.replace(',', ""))
        } else if let Some(stripped) = amount_str.strip_prefix('£') {
            ("GBP".to_string(), stripped.replace(',', ""))
        } else if let Some(stripped) = amount_str.strip_prefix('€') {
            ("EUR".to_string(), stripped.replace(',', ""))
        } else {
            ("USD".to_string(), amount_str.replace(",", ""))
        };

        let amount_per_annum = amount.parse::<f64>()
            .map_err(|_| OpportunityDslError::InvalidRevenueFormat(
                format!("Invalid amount format: {}", amount_str)
            ))?;

        Ok(RevenueStream {
            stream_type,
            amount_per_annum,
            currency,
        })
    }

    /// Extract string from quotes
    fn extract_quoted_string(&self, text: &str) -> Result<String, OpportunityDslError> {
        let text = text.trim();
        if (text.starts_with('\'') && text.ends_with('\'') ||
            text.starts_with('"') && text.ends_with('"')) && text.len() >= 2 {
            Ok(text[1..text.len()-1].to_string())
        } else {
            Err(OpportunityDslError::ParseError(
                format!("String must be quoted: {}", text)
            ))
        }
    }

    /// Execute Opportunity DSL command
    pub async fn execute_opportunity_dsl(&self, command: OpportunityDslCommand) -> Result<OpportunityDslResult, OpportunityDslError> {
        match command.operation {
            OpportunityOperation::Create => self.execute_create(command).await,
            OpportunityOperation::Update => self.execute_update(command).await,
            OpportunityOperation::Delete => self.execute_delete(command).await,
            OpportunityOperation::Query => self.execute_query(command).await,
        }
    }

    /// Validate that all referenced resources exist
    async fn validate_opportunity_resources(&self, command: &OpportunityDslCommand) -> Result<Vec<String>, OpportunityDslError> {
        let Some(pool) = &self.pool else {
            return Err(OpportunityDslError::DatabaseError("No database connection available".to_string()));
        };

        let mut validation_errors = Vec::new();

        // Validate CBUs
        for cbu_id in &command.cbu_ids {
            if self.validate_resource_exists(pool, "cbu", "cbu_id", cbu_id).await.is_err() {
                validation_errors.push(format!("CBU '{}' not found", cbu_id));
            }
        }

        // Validate Products
        for product_id in &command.product_ids {
            if self.validate_resource_exists(pool, "products", "product_id", product_id).await.is_err() {
                validation_errors.push(format!("Product '{}' not found", product_id));
            }
        }

        // Validate revenue streams (basic validation)
        for revenue_stream in &command.revenue_streams {
            if revenue_stream.amount_per_annum <= 0.0 {
                validation_errors.push(format!("Revenue stream '{}' must have positive amount", revenue_stream.stream_type));
            }
        }

        Ok(validation_errors)
    }

    /// Generic resource existence validator
    async fn validate_resource_exists(&self, pool: &PgPool, table: &str, id_column: &str, id_value: &str) -> Result<(), OpportunityDslError> {
        let query = format!("SELECT COUNT(*) as count FROM {} WHERE {} = $1", table, id_column);

        match sqlx::query(&query)
            .bind(id_value)
            .fetch_one(pool)
            .await
        {
            Ok(row) => {
                let count: i64 = row.get("count");
                if count > 0 {
                    Ok(())
                } else {
                    Err(OpportunityDslError::ResourceNotFound(table.to_string(), id_value.to_string()))
                }
            }
            Err(e) => Err(OpportunityDslError::DatabaseError(format!("Failed to validate {}: {}", table, e)))
        }
    }

    /// Execute CREATE OPPORTUNITY command
    async fn execute_create(&self, command: OpportunityDslCommand) -> Result<OpportunityDslResult, OpportunityDslError> {
        let Some(pool) = &self.pool else {
            return Err(OpportunityDslError::DatabaseError("No database connection available".to_string()));
        };

        // Validate resources
        let validation_errors = self.validate_opportunity_resources(&command).await?;
        if !validation_errors.is_empty() {
            return Ok(OpportunityDslResult {
                success: false,
                message: "Resource validation failed".to_string(),
                opportunity_id: None,
                validation_errors,
                data: None,
                revenue_summary: None,
            });
        }

        let opportunity_id = command.opportunity_id.as_ref().unwrap();
        let client_name = command.client_name.as_ref().unwrap();
        let description = command.description.as_ref().unwrap();

        // Calculate total revenue projection
        let total_revenue_projection: f64 = command.revenue_streams.iter()
            .map(|rs| rs.amount_per_annum)
            .sum();

        // Insert opportunity record
        let insert_query = r#"
            INSERT INTO opportunities (
                opportunity_id, client_name, description, total_revenue_projection,
                status, probability_percentage, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            RETURNING id
        "#;

        match sqlx::query(insert_query)
            .bind(opportunity_id)
            .bind(client_name)
            .bind(description)
            .bind(total_revenue_projection)
            .bind("prospecting") // Default status
            .bind(50.0) // Default 50% probability
            .fetch_one(pool)
            .await
        {
            Ok(_) => {
                // Create resource associations and revenue streams
                self.create_opportunity_associations(pool, opportunity_id, &command).await?;

                let revenue_summary = self.generate_revenue_summary(pool, opportunity_id).await?;

                Ok(OpportunityDslResult {
                    success: true,
                    message: format!("Opportunity '{}' created successfully with total revenue projection of ${:.2}",
                        opportunity_id, total_revenue_projection),
                    opportunity_id: Some(opportunity_id.clone()),
                    validation_errors: Vec::new(),
                    data: None,
                    revenue_summary: Some(revenue_summary),
                })
            }
            Err(e) => Err(OpportunityDslError::DatabaseError(format!("Failed to create opportunity: {}", e)))
        }
    }

    /// Create all resource associations for an opportunity
    async fn create_opportunity_associations(&self, pool: &PgPool, opportunity_id: &str, command: &OpportunityDslCommand) -> Result<(), OpportunityDslError> {
        // Create CBU associations
        for cbu_id in &command.cbu_ids {
            self.create_opportunity_association(pool, opportunity_id, "CBU", cbu_id).await?;
        }

        // Create Product associations
        for product_id in &command.product_ids {
            self.create_opportunity_association(pool, opportunity_id, "PRODUCT", product_id).await?;
        }

        // Create Revenue Stream records
        for revenue_stream in &command.revenue_streams {
            self.create_revenue_stream(pool, opportunity_id, revenue_stream).await?;
        }

        Ok(())
    }

    /// Create individual opportunity-resource association
    async fn create_opportunity_association(&self, pool: &PgPool, opportunity_id: &str, resource_type: &str, resource_id: &str) -> Result<(), OpportunityDslError> {
        let insert_query = r#"
            INSERT INTO opportunity_resource_associations (opportunity_id, resource_type, resource_id, created_at)
            VALUES ($1, $2, $3, NOW())
        "#;

        sqlx::query(insert_query)
            .bind(opportunity_id)
            .bind(resource_type)
            .bind(resource_id)
            .execute(pool)
            .await
            .map_err(|e| OpportunityDslError::DatabaseError(format!("Failed to create {} association: {}", resource_type, e)))?;

        Ok(())
    }

    /// Create revenue stream record
    async fn create_revenue_stream(&self, pool: &PgPool, opportunity_id: &str, revenue_stream: &RevenueStream) -> Result<(), OpportunityDslError> {
        let insert_query = r#"
            INSERT INTO opportunity_revenue_streams (
                opportunity_id, stream_type, amount_per_annum, currency, created_at
            ) VALUES ($1, $2, $3, $4, NOW())
        "#;

        sqlx::query(insert_query)
            .bind(opportunity_id)
            .bind(&revenue_stream.stream_type)
            .bind(revenue_stream.amount_per_annum)
            .bind(&revenue_stream.currency)
            .execute(pool)
            .await
            .map_err(|e| OpportunityDslError::DatabaseError(format!("Failed to create revenue stream: {}", e)))?;

        Ok(())
    }

    /// Generate comprehensive revenue summary
    async fn generate_revenue_summary(&self, pool: &PgPool, opportunity_id: &str) -> Result<RevenueSummary, OpportunityDslError> {
        // Get opportunity basic info
        let opportunity_query = "SELECT client_name FROM opportunities WHERE opportunity_id = $1";
        let opportunity_row = sqlx::query(opportunity_query)
            .bind(opportunity_id)
            .fetch_one(pool)
            .await
            .map_err(|e| OpportunityDslError::DatabaseError(format!("Failed to get opportunity info: {}", e)))?;

        let client_name: String = opportunity_row.get("client_name");

        // Get revenue streams
        let revenue_query = r#"
            SELECT stream_type, amount_per_annum, currency
            FROM opportunity_revenue_streams
            WHERE opportunity_id = $1
        "#;

        let revenue_rows = sqlx::query(revenue_query)
            .bind(opportunity_id)
            .fetch_all(pool)
            .await
            .map_err(|e| OpportunityDslError::DatabaseError(format!("Failed to get revenue streams: {}", e)))?;

        let mut revenue_breakdown = Vec::new();
        let mut total_annual_revenue = 0.0;

        for row in &revenue_rows {
            let amount: f64 = row.get("amount_per_annum");
            total_annual_revenue += amount;

            revenue_breakdown.push(RevenueBreakdown {
                stream_type: row.get("stream_type"),
                annual_amount: amount,
                percentage_of_total: 0.0, // Will calculate after total is known
                currency: row.get("currency"),
            });
        }

        // Calculate percentages
        for breakdown in &mut revenue_breakdown {
            breakdown.percentage_of_total = if total_annual_revenue > 0.0 {
                (breakdown.annual_amount / total_annual_revenue * 100.0) as f32
            } else {
                0.0
            };
        }

        // Count resources
        let resource_query = r#"
            SELECT resource_type, COUNT(*) as count
            FROM opportunity_resource_associations
            WHERE opportunity_id = $1
            GROUP BY resource_type
        "#;

        let resource_rows = sqlx::query(resource_query)
            .bind(opportunity_id)
            .fetch_all(pool)
            .await
            .map_err(|e| OpportunityDslError::DatabaseError(format!("Failed to count resources: {}", e)))?;

        let mut total_cbus = 0;
        let mut total_products = 0;
        let mut key_services = Vec::new();

        for row in resource_rows {
            let resource_type: String = row.get("resource_type");
            let count: i64 = row.get("count");

            match resource_type.as_str() {
                "CBU" => total_cbus = count as i32,
                "PRODUCT" => total_products = count as i32,
                _ => {}
            }
        }

        // Extract key services from revenue streams
        for breakdown in &revenue_breakdown {
            key_services.push(format!("{} (${:.0}K/year)",
                breakdown.stream_type.replace("_", " ").to_uppercase(),
                breakdown.annual_amount / 1000.0
            ));
        }

        // Determine business value tier
        let business_value_tier = if total_annual_revenue >= 1000000.0 {
            "High"
        } else if total_annual_revenue >= 250000.0 {
            "Medium"
        } else {
            "Low"
        };

        let business_scope = BusinessScope {
            total_cbus,
            total_products,
            key_services,
            business_value_tier: business_value_tier.to_string(),
        };

        Ok(RevenueSummary {
            opportunity_id: opportunity_id.to_string(),
            client_name,
            total_annual_revenue,
            revenue_breakdown,
            business_scope,
        })
    }

    /// Execute UPDATE OPPORTUNITY command
    async fn execute_update(&self, command: OpportunityDslCommand) -> Result<OpportunityDslResult, OpportunityDslError> {
        let Some(pool) = &self.pool else {
            return Err(OpportunityDslError::DatabaseError("No database connection available".to_string()));
        };

        let opportunity_id = command.opportunity_id.as_ref().unwrap();

        // Build dynamic UPDATE query
        let mut set_clauses = Vec::new();
        let mut values: Vec<&str> = Vec::new();

        for (field, value) in &command.update_fields {
            set_clauses.push(format!("{} = ${}", field, values.len() + 2));
            values.push(value);
        }

        if set_clauses.is_empty() {
            return Err(OpportunityDslError::ValidationError("No fields to update".to_string()));
        }

        let update_query = format!(
            "UPDATE opportunities SET {}, updated_at = NOW() WHERE opportunity_id = $1",
            set_clauses.join(", ")
        );

        let mut query = sqlx::query(&update_query).bind(opportunity_id);
        for value in values {
            query = query.bind(value);
        }

        match query.execute(pool).await {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    Ok(OpportunityDslResult {
                        success: true,
                        message: format!("Opportunity '{}' updated successfully", opportunity_id),
                        opportunity_id: Some(opportunity_id.clone()),
                        validation_errors: Vec::new(),
                        data: None,
                        revenue_summary: None,
                    })
                } else {
                    Err(OpportunityDslError::ValidationError(format!("Opportunity '{}' not found", opportunity_id)))
                }
            }
            Err(e) => Err(OpportunityDslError::DatabaseError(format!("Failed to update opportunity: {}", e)))
        }
    }

    /// Execute DELETE OPPORTUNITY command
    async fn execute_delete(&self, command: OpportunityDslCommand) -> Result<OpportunityDslResult, OpportunityDslError> {
        let Some(pool) = &self.pool else {
            return Err(OpportunityDslError::DatabaseError("No database connection available".to_string()));
        };

        let opportunity_id = command.opportunity_id.as_ref().unwrap();

        // Delete associations and revenue streams first
        let delete_associations_query = "DELETE FROM opportunity_resource_associations WHERE opportunity_id = $1";
        sqlx::query(delete_associations_query)
            .bind(opportunity_id)
            .execute(pool)
            .await
            .map_err(|e| OpportunityDslError::DatabaseError(format!("Failed to delete associations: {}", e)))?;

        let delete_revenue_query = "DELETE FROM opportunity_revenue_streams WHERE opportunity_id = $1";
        sqlx::query(delete_revenue_query)
            .bind(opportunity_id)
            .execute(pool)
            .await
            .map_err(|e| OpportunityDslError::DatabaseError(format!("Failed to delete revenue streams: {}", e)))?;

        // Delete opportunity
        let delete_query = "DELETE FROM opportunities WHERE opportunity_id = $1";
        match sqlx::query(delete_query)
            .bind(opportunity_id)
            .execute(pool)
            .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    Ok(OpportunityDslResult {
                        success: true,
                        message: format!("Opportunity '{}' and all associations deleted successfully", opportunity_id),
                        opportunity_id: Some(opportunity_id.clone()),
                        validation_errors: Vec::new(),
                        data: None,
                        revenue_summary: None,
                    })
                } else {
                    Err(OpportunityDslError::ValidationError(format!("Opportunity '{}' not found", opportunity_id)))
                }
            }
            Err(e) => Err(OpportunityDslError::DatabaseError(format!("Failed to delete opportunity: {}", e)))
        }
    }

    /// Execute QUERY OPPORTUNITY command
    async fn execute_query(&self, command: OpportunityDslCommand) -> Result<OpportunityDslResult, OpportunityDslError> {
        let Some(pool) = &self.pool else {
            return Err(OpportunityDslError::DatabaseError("No database connection available".to_string()));
        };

        let base_query = r#"
            SELECT
                o.*,
                COUNT(DISTINCT CASE WHEN ora.resource_type = 'CBU' THEN ora.resource_id END) as cbu_count,
                COUNT(DISTINCT CASE WHEN ora.resource_type = 'PRODUCT' THEN ora.resource_id END) as product_count,
                COUNT(DISTINCT ors.stream_type) as revenue_stream_count
            FROM opportunities o
            LEFT JOIN opportunity_resource_associations ora ON o.opportunity_id = ora.opportunity_id
            LEFT JOIN opportunity_revenue_streams ors ON o.opportunity_id = ors.opportunity_id
        "#;

        let (final_query, _where_clause) = if let Some(conditions) = &command.query_conditions {
            (format!("{} WHERE {} GROUP BY o.opportunity_id", base_query, conditions), true)
        } else {
            (format!("{} GROUP BY o.opportunity_id", base_query), false)
        };

        match sqlx::query(&final_query).fetch_all(pool).await {
            Ok(rows) => {
                let mut results = Vec::new();
                for row in rows {
                    let mut opportunity_data = serde_json::Map::new();
                    opportunity_data.insert("opportunity_id".to_string(), serde_json::Value::String(row.get("opportunity_id")));
                    opportunity_data.insert("client_name".to_string(), serde_json::Value::String(row.get("client_name")));
                    opportunity_data.insert("description".to_string(), serde_json::Value::String(row.get("description")));
                    opportunity_data.insert("status".to_string(), serde_json::Value::String(row.get("status")));

                    let total_revenue: f64 = row.get("total_revenue_projection");
                    let cbu_count: i64 = row.get("cbu_count");
                    let product_count: i64 = row.get("product_count");
                    let revenue_stream_count: i64 = row.get("revenue_stream_count");

                    opportunity_data.insert("commercial_summary".to_string(), serde_json::Value::Object({
                        let mut summary = serde_json::Map::new();
                        summary.insert("total_revenue_projection".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(total_revenue).unwrap()));
                        summary.insert("total_cbus".to_string(), serde_json::Value::Number(serde_json::Number::from(cbu_count)));
                        summary.insert("total_products".to_string(), serde_json::Value::Number(serde_json::Number::from(product_count)));
                        summary.insert("revenue_streams".to_string(), serde_json::Value::Number(serde_json::Number::from(revenue_stream_count)));
                        summary.insert("business_tier".to_string(), serde_json::Value::String(
                            if total_revenue >= 1000000.0 { "High" } else if total_revenue >= 250000.0 { "Medium" } else { "Low" }.to_string()
                        ));
                        summary
                    }));

                    results.push(serde_json::Value::Object(opportunity_data));
                }

                Ok(OpportunityDslResult {
                    success: true,
                    message: format!("Found {} opportunity/opportunities", results.len()),
                    opportunity_id: None,
                    validation_errors: Vec::new(),
                    data: Some(serde_json::Value::Array(results)),
                    revenue_summary: None,
                })
            }
            Err(e) => Err(OpportunityDslError::DatabaseError(format!("Failed to query opportunities: {}", e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_opportunity_command() {
        let parser = OpportunityDslParser::new(None);
        let dsl = r#"CREATE OPPORTUNITY 'OPP001' ; 'Alpha Corporation' ; 'Multi-product custody and fund accounting' WITH
            CBU 'CBU001' AND CBU 'CBU002' AND
            PRODUCT 'CUSTODY_SERVICES' AND PRODUCT 'FUND_ACCOUNTING' AND
            REVENUE_STREAM 'custody' '$1000000' AND REVENUE_STREAM 'fund_accounting' '$250000'"#;

        let result = parser.parse_opportunity_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, OpportunityOperation::Create));
        assert_eq!(command.opportunity_id, Some("OPP001".to_string()));
        assert_eq!(command.client_name, Some("Alpha Corporation".to_string()));
        assert_eq!(command.cbu_ids, vec!["CBU001", "CBU002"]);
        assert_eq!(command.product_ids, vec!["CUSTODY_SERVICES", "FUND_ACCOUNTING"]);
        assert_eq!(command.revenue_streams.len(), 2);
        assert_eq!(command.revenue_streams[0].stream_type, "custody");
        assert_eq!(command.revenue_streams[0].amount_per_annum, 1000000.0);
        assert_eq!(command.revenue_streams[1].stream_type, "fund_accounting");
        assert_eq!(command.revenue_streams[1].amount_per_annum, 250000.0);
    }

    #[test]
    fn test_parse_revenue_stream() {
        let parser = OpportunityDslParser::new(None);
        let result = parser.parse_revenue_stream("'custody' '$1,500,000'");

        assert!(result.is_ok());
        let revenue_stream = result.unwrap();
        assert_eq!(revenue_stream.stream_type, "custody");
        assert_eq!(revenue_stream.amount_per_annum, 1500000.0);
        assert_eq!(revenue_stream.currency, "USD");
    }
}