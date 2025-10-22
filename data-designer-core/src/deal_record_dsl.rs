// Deal Record DSL - Master orchestrator for comprehensive business relationship management
// The Deal Record is the primary entity linking ALL business components under one negotiated piece
//
// Grammar:
// CREATE DEAL '<deal_id>' ; '<description>' ; '<primary_introducing_client>' WITH
//   CBU '<cbu_id_1>' [AND CBU '<cbu_id_2>' ...] AND
//   PRODUCT '<product_id_1>' [AND PRODUCT '<product_id_2>' ...] AND
//   CONTRACT '<contract_id_1>' [AND CONTRACT '<contract_id_2>' ...] AND
//   KYC '<kyc_clearance_id_1>' [AND KYC '<kyc_clearance_id_2>' ...] AND
//   SERVICE_MAP '<service_map_id_1>' [AND SERVICE_MAP '<service_map_id_2>' ...] AND
//   OPPORTUNITY '<opportunity_id_1>' [AND OPPORTUNITY '<opportunity_id_2>' ...]
//
// UPDATE DEAL '<deal_id>' SET <field> = '<value>'
// DELETE DEAL '<deal_id>'
// QUERY DEAL [WHERE <condition>]
// LINK DEAL '<deal_id>' WITH <resource_type> '<resource_id>'
// UNLINK DEAL '<deal_id>' FROM <resource_type> '<resource_id>'

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sqlx::{PgPool, Row};
use crate::dsl_utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealRecordDslCommand {
    pub operation: DealOperation,
    pub deal_id: Option<String>,
    pub description: Option<String>,
    pub primary_introducing_client: Option<String>,
    pub cbu_ids: Vec<String>,
    pub product_ids: Vec<String>,
    pub contract_ids: Vec<String>,
    pub kyc_clearance_ids: Vec<String>,
    pub service_map_ids: Vec<String>,
    pub opportunity_ids: Vec<String>,
    pub update_fields: HashMap<String, String>,
    pub query_conditions: Option<String>,
    pub link_resource_type: Option<String>,
    pub link_resource_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DealOperation {
    Create,
    Update,
    Delete,
    Query,
    Link,    // Link additional resources to existing deal
    Unlink,  // Unlink resources from existing deal
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealRecord {
    pub deal_id: String,
    pub description: String,
    pub primary_introducing_client: String,
    pub status: String,
    pub cbu_count: i32,
    pub product_count: i32,
    pub contract_count: i32,
    pub kyc_clearance_count: i32,
    pub service_map_count: i32,
    pub opportunity_count: i32,
    pub total_business_value: Option<f64>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct DealRecordDslParser {
    pub pool: Option<PgPool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealRecordDslResult {
    pub success: bool,
    pub message: String,
    pub deal_id: Option<String>,
    pub validation_errors: Vec<String>,
    pub data: Option<serde_json::Value>,
    pub summary: Option<DealSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealSummary {
    pub deal_id: String,
    pub description: String,
    pub primary_introducing_client: String,
    pub total_cbus: i32,
    pub total_products: i32,
    pub total_contracts: i32,
    pub total_kyc_clearances: i32,
    pub total_service_maps: i32,
    pub total_opportunities: i32,
    pub business_relationships: Vec<String>,
}

#[derive(Debug)]
pub enum DealRecordDslError {
    ParseError(String),
    ValidationError(String),
    DatabaseError(String),
    ResourceNotFound(String, String), // (resource_type, resource_id)
    DuplicateAssociation(String, String), // (resource_type, resource_id)
}

impl std::fmt::Display for DealRecordDslError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DealRecordDslError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            DealRecordDslError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            DealRecordDslError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            DealRecordDslError::ResourceNotFound(resource_type, resource_id) => {
                write!(f, "{} '{}' not found", resource_type, resource_id)
            },
            DealRecordDslError::DuplicateAssociation(resource_type, resource_id) => {
                write!(f, "{} '{}' already associated with this deal", resource_type, resource_id)
            },
        }
    }
}

impl std::error::Error for DealRecordDslError {}

impl DealRecordDslParser {
    pub fn new(pool: Option<PgPool>) -> Self {
        Self { pool }
    }

    /// Parse Deal Record DSL command into structured format
    pub fn parse_deal_record_dsl(&self, dsl_text: &str) -> Result<DealRecordDslCommand, DealRecordDslError> {
        let cleaned_text = dsl_utils::strip_comments(dsl_text);
        let dsl_text = cleaned_text.trim();

        if dsl_text.to_uppercase().starts_with("CREATE DEAL") {
            self.parse_create_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("UPDATE DEAL") {
            self.parse_update_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("DELETE DEAL") {
            self.parse_delete_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("QUERY DEAL") {
            self.parse_query_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("LINK DEAL") {
            self.parse_link_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("UNLINK DEAL") {
            self.parse_unlink_command(dsl_text)
        } else {
            Err(DealRecordDslError::ParseError(
                "Command must start with CREATE DEAL, UPDATE DEAL, DELETE DEAL, QUERY DEAL, LINK DEAL, or UNLINK DEAL".to_string()
            ))
        }
    }

    /// Parse CREATE DEAL command - Master business orchestrator
    fn parse_create_command(&self, dsl_text: &str) -> Result<DealRecordDslCommand, DealRecordDslError> {
        // CREATE DEAL 'DEAL001' ; 'Alpha Bank Multi-Product Onboarding' ; 'Alpha Corporation' WITH
        //   CBU 'CBU001' AND CBU 'CBU002' AND
        //   PRODUCT 'PROD001' AND PRODUCT 'PROD002' AND PRODUCT 'PROD003' AND
        //   CONTRACT 'CONTR001' AND CONTRACT 'CONTR002' AND
        //   KYC 'KYC001' AND KYC 'KYC002' AND
        //   SERVICE_MAP 'SM001' AND SERVICE_MAP 'SM002'

        let parts: Vec<&str> = dsl_text.splitn(2, " WITH ").collect();
        if parts.len() != 2 {
            return Err(DealRecordDslError::ParseError(
                "CREATE DEAL command must include ' WITH ' clause for business resources".to_string()
            ));
        }

        // Parse deal parameters
        let deal_part = parts[0].trim();
        let resources_part = parts[1].trim();

        // Extract deal_id, description, and primary_introducing_client
        let deal_content = deal_part.strip_prefix("CREATE DEAL").unwrap().trim();
        let deal_parts: Vec<&str> = deal_content.splitn(3, " ; ").collect();

        if deal_parts.len() != 3 {
            return Err(DealRecordDslError::ParseError(
                "CREATE DEAL must have format: CREATE DEAL 'deal_id' ; 'description' ; 'primary_introducing_client'".to_string()
            ));
        }

        let deal_id = self.extract_quoted_string(deal_parts[0].trim())?;
        let description = self.extract_quoted_string(deal_parts[1].trim())?;
        let primary_introducing_client = self.extract_quoted_string(deal_parts[2].trim())?;

        // Parse all business resources
        let (cbu_ids, product_ids, contract_ids, kyc_clearance_ids, service_map_ids, opportunity_ids) =
            self.parse_business_resources(resources_part)?;

        Ok(DealRecordDslCommand {
            operation: DealOperation::Create,
            deal_id: Some(deal_id),
            description: Some(description),
            primary_introducing_client: Some(primary_introducing_client),
            cbu_ids,
            product_ids,
            contract_ids,
            kyc_clearance_ids,
            service_map_ids,
            opportunity_ids,
            update_fields: HashMap::new(),
            query_conditions: None,
            link_resource_type: None,
            link_resource_id: None,
        })
    }

    /// Parse UPDATE DEAL command
    fn parse_update_command(&self, dsl_text: &str) -> Result<DealRecordDslCommand, DealRecordDslError> {
        // UPDATE DEAL 'DEAL001' SET description = 'Updated deal description'
        let content = dsl_text.strip_prefix("UPDATE DEAL").unwrap().trim();
        let parts: Vec<&str> = content.splitn(2, " SET ").collect();

        if parts.len() != 2 {
            return Err(DealRecordDslError::ParseError(
                "UPDATE DEAL command must include ' SET ' clause".to_string()
            ));
        }

        let deal_id = self.extract_quoted_string(parts[0].trim())?;
        let set_clause = parts[1].trim();

        // Parse SET clause: field = 'value'
        let mut update_fields = HashMap::new();
        let assignments: Vec<&str> = set_clause.split(" AND ").collect();

        for assignment in assignments {
            let assign_parts: Vec<&str> = assignment.splitn(2, " = ").collect();
            if assign_parts.len() != 2 {
                return Err(DealRecordDslError::ParseError(
                    "SET clause must have format: field = 'value'".to_string()
                ));
            }

            let field = assign_parts[0].trim().to_string();
            let value = self.extract_quoted_string(assign_parts[1].trim())?;
            update_fields.insert(field, value);
        }

        Ok(DealRecordDslCommand {
            operation: DealOperation::Update,
            deal_id: Some(deal_id),
            description: None,
            primary_introducing_client: None,
            cbu_ids: Vec::new(),
            product_ids: Vec::new(),
            contract_ids: Vec::new(),
            kyc_clearance_ids: Vec::new(),
            service_map_ids: Vec::new(),
            opportunity_ids: Vec::new(),
            update_fields,
            query_conditions: None,
            link_resource_type: None,
            link_resource_id: None,
        })
    }

    /// Parse DELETE DEAL command
    fn parse_delete_command(&self, dsl_text: &str) -> Result<DealRecordDslCommand, DealRecordDslError> {
        // DELETE DEAL 'DEAL001'
        let content = dsl_text.strip_prefix("DELETE DEAL").unwrap().trim();
        let deal_id = self.extract_quoted_string(content)?;

        Ok(DealRecordDslCommand {
            operation: DealOperation::Delete,
            deal_id: Some(deal_id),
            description: None,
            primary_introducing_client: None,
            cbu_ids: Vec::new(),
            product_ids: Vec::new(),
            contract_ids: Vec::new(),
            kyc_clearance_ids: Vec::new(),
            service_map_ids: Vec::new(),
            opportunity_ids: Vec::new(),
            update_fields: HashMap::new(),
            query_conditions: None,
            link_resource_type: None,
            link_resource_id: None,
        })
    }

    /// Parse QUERY DEAL command
    fn parse_query_command(&self, dsl_text: &str) -> Result<DealRecordDslCommand, DealRecordDslError> {
        // QUERY DEAL WHERE status = 'active'
        let content = dsl_text.strip_prefix("QUERY DEAL").unwrap().trim();

        let query_conditions = if content.is_empty() {
            None
        } else if content.to_uppercase().starts_with("WHERE ") {
            Some(content.strip_prefix("WHERE ").unwrap().trim().to_string())
        } else {
            return Err(DealRecordDslError::ParseError(
                "QUERY DEAL can be used alone or with WHERE clause".to_string()
            ));
        };

        Ok(DealRecordDslCommand {
            operation: DealOperation::Query,
            deal_id: None,
            description: None,
            primary_introducing_client: None,
            cbu_ids: Vec::new(),
            product_ids: Vec::new(),
            contract_ids: Vec::new(),
            kyc_clearance_ids: Vec::new(),
            service_map_ids: Vec::new(),
            opportunity_ids: Vec::new(),
            update_fields: HashMap::new(),
            query_conditions,
            link_resource_type: None,
            link_resource_id: None,
        })
    }

    /// Parse LINK DEAL command - Add resources to existing deal
    fn parse_link_command(&self, dsl_text: &str) -> Result<DealRecordDslCommand, DealRecordDslError> {
        // LINK DEAL 'DEAL001' WITH CBU 'CBU003'
        let content = dsl_text.strip_prefix("LINK DEAL").unwrap().trim();
        let parts: Vec<&str> = content.splitn(2, " WITH ").collect();

        if parts.len() != 2 {
            return Err(DealRecordDslError::ParseError(
                "LINK DEAL command must have format: LINK DEAL 'deal_id' WITH <resource_type> 'resource_id'".to_string()
            ));
        }

        let deal_id = self.extract_quoted_string(parts[0].trim())?;
        let resource_part = parts[1].trim();

        let (resource_type, resource_id) = self.parse_single_resource(resource_part)?;

        Ok(DealRecordDslCommand {
            operation: DealOperation::Link,
            deal_id: Some(deal_id),
            description: None,
            primary_introducing_client: None,
            cbu_ids: Vec::new(),
            product_ids: Vec::new(),
            contract_ids: Vec::new(),
            kyc_clearance_ids: Vec::new(),
            service_map_ids: Vec::new(),
            opportunity_ids: Vec::new(),
            update_fields: HashMap::new(),
            query_conditions: None,
            link_resource_type: Some(resource_type),
            link_resource_id: Some(resource_id),
        })
    }

    /// Parse UNLINK DEAL command - Remove resources from existing deal
    fn parse_unlink_command(&self, dsl_text: &str) -> Result<DealRecordDslCommand, DealRecordDslError> {
        // UNLINK DEAL 'DEAL001' FROM CBU 'CBU003'
        let content = dsl_text.strip_prefix("UNLINK DEAL").unwrap().trim();
        let parts: Vec<&str> = content.splitn(2, " FROM ").collect();

        if parts.len() != 2 {
            return Err(DealRecordDslError::ParseError(
                "UNLINK DEAL command must have format: UNLINK DEAL 'deal_id' FROM <resource_type> 'resource_id'".to_string()
            ));
        }

        let deal_id = self.extract_quoted_string(parts[0].trim())?;
        let resource_part = parts[1].trim();

        let (resource_type, resource_id) = self.parse_single_resource(resource_part)?;

        Ok(DealRecordDslCommand {
            operation: DealOperation::Unlink,
            deal_id: Some(deal_id),
            description: None,
            primary_introducing_client: None,
            cbu_ids: Vec::new(),
            product_ids: Vec::new(),
            contract_ids: Vec::new(),
            kyc_clearance_ids: Vec::new(),
            service_map_ids: Vec::new(),
            opportunity_ids: Vec::new(),
            update_fields: HashMap::new(),
            query_conditions: None,
            link_resource_type: Some(resource_type),
            link_resource_id: Some(resource_id),
        })
    }

    /// Parse all business resources from the WITH clause
    fn parse_business_resources(&self, resources_text: &str) -> Result<(Vec<String>, Vec<String>, Vec<String>, Vec<String>, Vec<String>, Vec<String>), DealRecordDslError> {
        let mut cbu_ids = Vec::new();
        let mut product_ids = Vec::new();
        let mut contract_ids = Vec::new();
        let mut kyc_clearance_ids = Vec::new();
        let mut service_map_ids = Vec::new();
        let mut opportunity_ids = Vec::new();

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
            } else if declaration.to_uppercase().starts_with("CONTRACT ") {
                let resource_id = self.extract_quoted_string(declaration.strip_prefix("CONTRACT ").unwrap().trim())?;
                contract_ids.push(resource_id);
            } else if declaration.to_uppercase().starts_with("KYC ") {
                let resource_id = self.extract_quoted_string(declaration.strip_prefix("KYC ").unwrap().trim())?;
                kyc_clearance_ids.push(resource_id);
            } else if declaration.to_uppercase().starts_with("SERVICE_MAP ") {
                let resource_id = self.extract_quoted_string(declaration.strip_prefix("SERVICE_MAP ").unwrap().trim())?;
                service_map_ids.push(resource_id);
            } else if declaration.to_uppercase().starts_with("OPPORTUNITY ") {
                let resource_id = self.extract_quoted_string(declaration.strip_prefix("OPPORTUNITY ").unwrap().trim())?;
                opportunity_ids.push(resource_id);
            } else {
                return Err(DealRecordDslError::ParseError(
                    format!("Invalid resource specification: {}. Must be CBU, PRODUCT, CONTRACT, KYC, SERVICE_MAP, or OPPORTUNITY", declaration)
                ));
            }
        }

        Ok((cbu_ids, product_ids, contract_ids, kyc_clearance_ids, service_map_ids, opportunity_ids))
    }

    /// Parse single resource for LINK/UNLINK operations
    fn parse_single_resource(&self, resource_text: &str) -> Result<(String, String), DealRecordDslError> {
        let resource_text = resource_text.trim();

        if resource_text.to_uppercase().starts_with("CBU ") {
            let resource_id = self.extract_quoted_string(resource_text.strip_prefix("CBU ").unwrap().trim())?;
            Ok(("CBU".to_string(), resource_id))
        } else if resource_text.to_uppercase().starts_with("PRODUCT ") {
            let resource_id = self.extract_quoted_string(resource_text.strip_prefix("PRODUCT ").unwrap().trim())?;
            Ok(("PRODUCT".to_string(), resource_id))
        } else if resource_text.to_uppercase().starts_with("CONTRACT ") {
            let resource_id = self.extract_quoted_string(resource_text.strip_prefix("CONTRACT ").unwrap().trim())?;
            Ok(("CONTRACT".to_string(), resource_id))
        } else if resource_text.to_uppercase().starts_with("KYC ") {
            let resource_id = self.extract_quoted_string(resource_text.strip_prefix("KYC ").unwrap().trim())?;
            Ok(("KYC".to_string(), resource_id))
        } else if resource_text.to_uppercase().starts_with("SERVICE_MAP ") {
            let resource_id = self.extract_quoted_string(resource_text.strip_prefix("SERVICE_MAP ").unwrap().trim())?;
            Ok(("SERVICE_MAP".to_string(), resource_id))
        } else if resource_text.to_uppercase().starts_with("OPPORTUNITY ") {
            let resource_id = self.extract_quoted_string(resource_text.strip_prefix("OPPORTUNITY ").unwrap().trim())?;
            Ok(("OPPORTUNITY".to_string(), resource_id))
        } else {
            Err(DealRecordDslError::ParseError(
                format!("Invalid resource specification: {}. Must be CBU, PRODUCT, CONTRACT, KYC, SERVICE_MAP, or OPPORTUNITY", resource_text)
            ))
        }
    }

    /// Extract string from quotes
    fn extract_quoted_string(&self, text: &str) -> Result<String, DealRecordDslError> {
        let text = text.trim();
        if text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2 {
            Ok(text[1..text.len()-1].to_string())
        } else if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
            Ok(text[1..text.len()-1].to_string())
        } else {
            Err(DealRecordDslError::ParseError(
                format!("String must be quoted: {}", text)
            ))
        }
    }

    /// Execute Deal Record DSL command
    pub async fn execute_deal_record_dsl(&self, command: DealRecordDslCommand) -> Result<DealRecordDslResult, DealRecordDslError> {
        match command.operation {
            DealOperation::Create => self.execute_create(command).await,
            DealOperation::Update => self.execute_update(command).await,
            DealOperation::Delete => self.execute_delete(command).await,
            DealOperation::Query => self.execute_query(command).await,
            DealOperation::Link => self.execute_link(command).await,
            DealOperation::Unlink => self.execute_unlink(command).await,
        }
    }

    /// Validate that all referenced resources exist
    async fn validate_all_resources(&self, command: &DealRecordDslCommand) -> Result<Vec<String>, DealRecordDslError> {
        let Some(pool) = &self.pool else {
            return Err(DealRecordDslError::DatabaseError("No database connection available".to_string()));
        };

        let mut validation_errors = Vec::new();

        // Validate CBUs
        for cbu_id in &command.cbu_ids {
            if let Err(_) = self.validate_resource_exists(pool, "cbu", "cbu_id", cbu_id).await {
                validation_errors.push(format!("CBU '{}' not found", cbu_id));
            }
        }

        // Validate Products
        for product_id in &command.product_ids {
            if let Err(_) = self.validate_resource_exists(pool, "products", "product_id", product_id).await {
                validation_errors.push(format!("Product '{}' not found", product_id));
            }
        }

        // Validate Contracts (assuming contracts table)
        for contract_id in &command.contract_ids {
            if let Err(_) = self.validate_resource_exists(pool, "contracts", "contract_id", contract_id).await {
                validation_errors.push(format!("Contract '{}' not found", contract_id));
            }
        }

        // Validate KYC Clearances (assuming kyc_clearances table)
        for kyc_id in &command.kyc_clearance_ids {
            if let Err(_) = self.validate_resource_exists(pool, "kyc_clearances", "kyc_id", kyc_id).await {
                validation_errors.push(format!("KYC Clearance '{}' not found", kyc_id));
            }
        }

        // Validate Service Maps (assuming service_maps table)
        for service_map_id in &command.service_map_ids {
            if let Err(_) = self.validate_resource_exists(pool, "service_maps", "service_map_id", service_map_id).await {
                validation_errors.push(format!("Service Map '{}' not found", service_map_id));
            }
        }

        // Validate Opportunities
        for opportunity_id in &command.opportunity_ids {
            if let Err(_) = self.validate_resource_exists(pool, "opportunities", "opportunity_id", opportunity_id).await {
                validation_errors.push(format!("Opportunity '{}' not found", opportunity_id));
            }
        }

        Ok(validation_errors)
    }

    /// Generic resource existence validator
    async fn validate_resource_exists(&self, pool: &PgPool, table: &str, id_column: &str, id_value: &str) -> Result<(), DealRecordDslError> {
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
                    Err(DealRecordDslError::ResourceNotFound(table.to_string(), id_value.to_string()))
                }
            }
            Err(e) => Err(DealRecordDslError::DatabaseError(format!("Failed to validate {}: {}", table, e)))
        }
    }

    /// Execute CREATE DEAL command - Master business orchestrator
    async fn execute_create(&self, command: DealRecordDslCommand) -> Result<DealRecordDslResult, DealRecordDslError> {
        let Some(pool) = &self.pool else {
            return Err(DealRecordDslError::DatabaseError("No database connection available".to_string()));
        };

        // Validate all referenced resources exist
        let validation_errors = self.validate_all_resources(&command).await?;
        if !validation_errors.is_empty() {
            return Ok(DealRecordDslResult {
                success: false,
                message: "Resource validation failed".to_string(),
                deal_id: None,
                validation_errors,
                data: None,
                summary: None,
            });
        }

        let deal_id = command.deal_id.as_ref().unwrap();
        let description = command.description.as_ref().unwrap();
        let primary_introducing_client = command.primary_introducing_client.as_ref().unwrap();

        // Create deal record
        let insert_query = r#"
            INSERT INTO deal_records (
                deal_id, description, primary_introducing_client, status, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING id
        "#;

        match sqlx::query(insert_query)
            .bind(deal_id)
            .bind(description)
            .bind(primary_introducing_client)
            .bind("active")
            .fetch_one(pool)
            .await
        {
            Ok(_) => {
                // Create all resource associations
                self.create_resource_associations(pool, deal_id, &command).await?;

                let summary = self.generate_deal_summary(pool, deal_id).await?;

                Ok(DealRecordDslResult {
                    success: true,
                    message: format!("Deal Record '{}' created successfully with {} total resources", deal_id,
                        command.cbu_ids.len() + command.product_ids.len() + command.contract_ids.len() +
                        command.kyc_clearance_ids.len() + command.service_map_ids.len() + command.opportunity_ids.len()),
                    deal_id: Some(deal_id.clone()),
                    validation_errors: Vec::new(),
                    data: None,
                    summary: Some(summary),
                })
            }
            Err(e) => Err(DealRecordDslError::DatabaseError(format!("Failed to create deal record: {}", e)))
        }
    }

    /// Create all resource associations for a deal
    async fn create_resource_associations(&self, pool: &PgPool, deal_id: &str, command: &DealRecordDslCommand) -> Result<(), DealRecordDslError> {
        // Create CBU associations
        for cbu_id in &command.cbu_ids {
            self.create_deal_association(pool, deal_id, "CBU", cbu_id).await?;
        }

        // Create Product associations
        for product_id in &command.product_ids {
            self.create_deal_association(pool, deal_id, "PRODUCT", product_id).await?;
        }

        // Create Contract associations
        for contract_id in &command.contract_ids {
            self.create_deal_association(pool, deal_id, "CONTRACT", contract_id).await?;
        }

        // Create KYC associations
        for kyc_id in &command.kyc_clearance_ids {
            self.create_deal_association(pool, deal_id, "KYC", kyc_id).await?;
        }

        // Create Service Map associations
        for service_map_id in &command.service_map_ids {
            self.create_deal_association(pool, deal_id, "SERVICE_MAP", service_map_id).await?;
        }

        // Create Opportunity associations
        for opportunity_id in &command.opportunity_ids {
            self.create_deal_association(pool, deal_id, "OPPORTUNITY", opportunity_id).await?;
        }

        Ok(())
    }

    /// Create individual deal-resource association
    async fn create_deal_association(&self, pool: &PgPool, deal_id: &str, resource_type: &str, resource_id: &str) -> Result<(), DealRecordDslError> {
        let insert_query = r#"
            INSERT INTO deal_resource_associations (deal_id, resource_type, resource_id, created_at)
            VALUES ($1, $2, $3, NOW())
        "#;

        sqlx::query(insert_query)
            .bind(deal_id)
            .bind(resource_type)
            .bind(resource_id)
            .execute(pool)
            .await
            .map_err(|e| DealRecordDslError::DatabaseError(format!("Failed to create {} association: {}", resource_type, e)))?;

        Ok(())
    }

    /// Generate comprehensive deal summary
    async fn generate_deal_summary(&self, pool: &PgPool, deal_id: &str) -> Result<DealSummary, DealRecordDslError> {
        // Get deal basic info
        let deal_query = "SELECT description, primary_introducing_client FROM deal_records WHERE deal_id = $1";
        let deal_row = sqlx::query(deal_query)
            .bind(deal_id)
            .fetch_one(pool)
            .await
            .map_err(|e| DealRecordDslError::DatabaseError(format!("Failed to get deal info: {}", e)))?;

        let description: String = deal_row.get("description");
        let primary_introducing_client: String = deal_row.get("primary_introducing_client");

        // Count resources by type
        let count_query = r#"
            SELECT resource_type, COUNT(*) as count
            FROM deal_resource_associations
            WHERE deal_id = $1
            GROUP BY resource_type
        "#;

        let count_rows = sqlx::query(count_query)
            .bind(deal_id)
            .fetch_all(pool)
            .await
            .map_err(|e| DealRecordDslError::DatabaseError(format!("Failed to count resources: {}", e)))?;

        let mut total_cbus = 0;
        let mut total_products = 0;
        let mut total_contracts = 0;
        let mut total_kyc_clearances = 0;
        let mut total_service_maps = 0;
        let mut total_opportunities = 0;

        for row in count_rows {
            let resource_type: String = row.get("resource_type");
            let count: i64 = row.get("count");

            match resource_type.as_str() {
                "CBU" => total_cbus = count as i32,
                "PRODUCT" => total_products = count as i32,
                "CONTRACT" => total_contracts = count as i32,
                "KYC" => total_kyc_clearances = count as i32,
                "SERVICE_MAP" => total_service_maps = count as i32,
                "OPPORTUNITY" => total_opportunities = count as i32,
                _ => {} // Ignore unknown types
            }
        }

        let business_relationships = vec![
            format!("Primary Client: {}", primary_introducing_client),
            format!("Total Business Units: {}", total_cbus),
            format!("Product Portfolio: {}", total_products),
            format!("Contractual Agreements: {}", total_contracts),
            format!("Compliance Clearances: {}", total_kyc_clearances),
            format!("Service Delivery Maps: {}", total_service_maps),
            format!("Commercial Opportunities: {}", total_opportunities),
        ];

        Ok(DealSummary {
            deal_id: deal_id.to_string(),
            description,
            primary_introducing_client,
            total_cbus,
            total_products,
            total_contracts,
            total_kyc_clearances,
            total_service_maps,
            total_opportunities,
            business_relationships,
        })
    }

    /// Execute UPDATE DEAL command
    async fn execute_update(&self, command: DealRecordDslCommand) -> Result<DealRecordDslResult, DealRecordDslError> {
        let Some(pool) = &self.pool else {
            return Err(DealRecordDslError::DatabaseError("No database connection available".to_string()));
        };

        let deal_id = command.deal_id.as_ref().unwrap();

        // Build dynamic UPDATE query
        let mut set_clauses = Vec::new();
        let mut values: Vec<&str> = Vec::new();

        for (field, value) in &command.update_fields {
            set_clauses.push(format!("{} = ${}", field, values.len() + 2));
            values.push(value);
        }

        if set_clauses.is_empty() {
            return Err(DealRecordDslError::ValidationError("No fields to update".to_string()));
        }

        let update_query = format!(
            "UPDATE deal_records SET {}, updated_at = NOW() WHERE deal_id = $1",
            set_clauses.join(", ")
        );

        let mut query = sqlx::query(&update_query).bind(deal_id);
        for value in values {
            query = query.bind(value);
        }

        match query.execute(pool).await {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    Ok(DealRecordDslResult {
                        success: true,
                        message: format!("Deal Record '{}' updated successfully", deal_id),
                        deal_id: Some(deal_id.clone()),
                        validation_errors: Vec::new(),
                        data: None,
                        summary: None,
                    })
                } else {
                    Err(DealRecordDslError::ValidationError(format!("Deal Record '{}' not found", deal_id)))
                }
            }
            Err(e) => Err(DealRecordDslError::DatabaseError(format!("Failed to update deal record: {}", e)))
        }
    }

    /// Execute DELETE DEAL command
    async fn execute_delete(&self, command: DealRecordDslCommand) -> Result<DealRecordDslResult, DealRecordDslError> {
        let Some(pool) = &self.pool else {
            return Err(DealRecordDslError::DatabaseError("No database connection available".to_string()));
        };

        let deal_id = command.deal_id.as_ref().unwrap();

        // Delete all resource associations first
        let delete_associations_query = "DELETE FROM deal_resource_associations WHERE deal_id = $1";
        sqlx::query(delete_associations_query)
            .bind(deal_id)
            .execute(pool)
            .await
            .map_err(|e| DealRecordDslError::DatabaseError(format!("Failed to delete deal associations: {}", e)))?;

        // Delete deal record
        let delete_query = "DELETE FROM deal_records WHERE deal_id = $1";
        match sqlx::query(delete_query)
            .bind(deal_id)
            .execute(pool)
            .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    Ok(DealRecordDslResult {
                        success: true,
                        message: format!("Deal Record '{}' and all associations deleted successfully", deal_id),
                        deal_id: Some(deal_id.clone()),
                        validation_errors: Vec::new(),
                        data: None,
                        summary: None,
                    })
                } else {
                    Err(DealRecordDslError::ValidationError(format!("Deal Record '{}' not found", deal_id)))
                }
            }
            Err(e) => Err(DealRecordDslError::DatabaseError(format!("Failed to delete deal record: {}", e)))
        }
    }

    /// Execute QUERY DEAL command
    async fn execute_query(&self, command: DealRecordDslCommand) -> Result<DealRecordDslResult, DealRecordDslError> {
        let Some(pool) = &self.pool else {
            return Err(DealRecordDslError::DatabaseError("No database connection available".to_string()));
        };

        let base_query = r#"
            SELECT
                dr.*,
                COUNT(DISTINCT CASE WHEN dra.resource_type = 'CBU' THEN dra.resource_id END) as cbu_count,
                COUNT(DISTINCT CASE WHEN dra.resource_type = 'PRODUCT' THEN dra.resource_id END) as product_count,
                COUNT(DISTINCT CASE WHEN dra.resource_type = 'CONTRACT' THEN dra.resource_id END) as contract_count,
                COUNT(DISTINCT CASE WHEN dra.resource_type = 'KYC' THEN dra.resource_id END) as kyc_count,
                COUNT(DISTINCT CASE WHEN dra.resource_type = 'SERVICE_MAP' THEN dra.resource_id END) as service_map_count,
                COUNT(DISTINCT CASE WHEN dra.resource_type = 'OPPORTUNITY' THEN dra.resource_id END) as opportunity_count
            FROM deal_records dr
            LEFT JOIN deal_resource_associations dra ON dr.deal_id = dra.deal_id
        "#;

        let (final_query, _where_clause) = if let Some(conditions) = &command.query_conditions {
            (format!("{} WHERE {} GROUP BY dr.deal_id", base_query, conditions), true)
        } else {
            (format!("{} GROUP BY dr.deal_id", base_query), false)
        };

        match sqlx::query(&final_query).fetch_all(pool).await {
            Ok(rows) => {
                let mut results = Vec::new();
                for row in rows {
                    let mut deal_data = serde_json::Map::new();
                    deal_data.insert("deal_id".to_string(), serde_json::Value::String(row.get("deal_id")));
                    deal_data.insert("description".to_string(), serde_json::Value::String(row.get("description")));
                    deal_data.insert("primary_introducing_client".to_string(), serde_json::Value::String(row.get("primary_introducing_client")));
                    deal_data.insert("status".to_string(), serde_json::Value::String(row.get("status")));

                    let cbu_count: i64 = row.get("cbu_count");
                    let product_count: i64 = row.get("product_count");
                    let contract_count: i64 = row.get("contract_count");
                    let kyc_count: i64 = row.get("kyc_count");
                    let service_map_count: i64 = row.get("service_map_count");
                    let opportunity_count: i64 = row.get("opportunity_count");

                    deal_data.insert("business_summary".to_string(), serde_json::Value::Object({
                        let mut summary = serde_json::Map::new();
                        summary.insert("total_cbus".to_string(), serde_json::Value::Number(serde_json::Number::from(cbu_count)));
                        summary.insert("total_products".to_string(), serde_json::Value::Number(serde_json::Number::from(product_count)));
                        summary.insert("total_contracts".to_string(), serde_json::Value::Number(serde_json::Number::from(contract_count)));
                        summary.insert("total_kyc_clearances".to_string(), serde_json::Value::Number(serde_json::Number::from(kyc_count)));
                        summary.insert("total_service_maps".to_string(), serde_json::Value::Number(serde_json::Number::from(service_map_count)));
                        summary.insert("total_opportunities".to_string(), serde_json::Value::Number(serde_json::Number::from(opportunity_count)));
                        summary.insert("total_resources".to_string(), serde_json::Value::Number(serde_json::Number::from(
                            cbu_count + product_count + contract_count + kyc_count + service_map_count + opportunity_count
                        )));
                        summary
                    }));

                    results.push(serde_json::Value::Object(deal_data));
                }

                Ok(DealRecordDslResult {
                    success: true,
                    message: format!("Found {} deal record(s)", results.len()),
                    deal_id: None,
                    validation_errors: Vec::new(),
                    data: Some(serde_json::Value::Array(results)),
                    summary: None,
                })
            }
            Err(e) => Err(DealRecordDslError::DatabaseError(format!("Failed to query deal records: {}", e)))
        }
    }

    /// Execute LINK DEAL command
    async fn execute_link(&self, command: DealRecordDslCommand) -> Result<DealRecordDslResult, DealRecordDslError> {
        let Some(pool) = &self.pool else {
            return Err(DealRecordDslError::DatabaseError("No database connection available".to_string()));
        };

        let deal_id = command.deal_id.as_ref().unwrap();
        let resource_type = command.link_resource_type.as_ref().unwrap();
        let resource_id = command.link_resource_id.as_ref().unwrap();

        // Validate resource exists
        match resource_type.as_str() {
            "CBU" => self.validate_resource_exists(pool, "cbu", "cbu_id", resource_id).await?,
            "PRODUCT" => self.validate_resource_exists(pool, "products", "product_id", resource_id).await?,
            "CONTRACT" => self.validate_resource_exists(pool, "contracts", "contract_id", resource_id).await?,
            "KYC" => self.validate_resource_exists(pool, "kyc_clearances", "kyc_id", resource_id).await?,
            "SERVICE_MAP" => self.validate_resource_exists(pool, "service_maps", "service_map_id", resource_id).await?,
            "OPPORTUNITY" => self.validate_resource_exists(pool, "opportunities", "opportunity_id", resource_id).await?,
            _ => return Err(DealRecordDslError::ValidationError(format!("Invalid resource type: {}", resource_type))),
        }

        // Create association
        self.create_deal_association(pool, deal_id, resource_type, resource_id).await?;

        Ok(DealRecordDslResult {
            success: true,
            message: format!("{} '{}' linked to Deal '{}' successfully", resource_type, resource_id, deal_id),
            deal_id: Some(deal_id.clone()),
            validation_errors: Vec::new(),
            data: None,
            summary: None,
        })
    }

    /// Execute UNLINK DEAL command
    async fn execute_unlink(&self, command: DealRecordDslCommand) -> Result<DealRecordDslResult, DealRecordDslError> {
        let Some(pool) = &self.pool else {
            return Err(DealRecordDslError::DatabaseError("No database connection available".to_string()));
        };

        let deal_id = command.deal_id.as_ref().unwrap();
        let resource_type = command.link_resource_type.as_ref().unwrap();
        let resource_id = command.link_resource_id.as_ref().unwrap();

        // Remove association
        let delete_query = r#"
            DELETE FROM deal_resource_associations
            WHERE deal_id = $1 AND resource_type = $2 AND resource_id = $3
        "#;

        match sqlx::query(delete_query)
            .bind(deal_id)
            .bind(resource_type)
            .bind(resource_id)
            .execute(pool)
            .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    Ok(DealRecordDslResult {
                        success: true,
                        message: format!("{} '{}' unlinked from Deal '{}' successfully", resource_type, resource_id, deal_id),
                        deal_id: Some(deal_id.clone()),
                        validation_errors: Vec::new(),
                        data: None,
                        summary: None,
                    })
                } else {
                    Err(DealRecordDslError::ValidationError(
                        format!("Association between Deal '{}' and {} '{}' not found", deal_id, resource_type, resource_id)
                    ))
                }
            }
            Err(e) => Err(DealRecordDslError::DatabaseError(format!("Failed to unlink resource: {}", e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_deal_command() {
        let parser = DealRecordDslParser::new(None);
        let dsl = "CREATE DEAL 'DEAL001' ; 'Alpha Bank Multi-Product Onboarding' ; 'Alpha Corporation' WITH CBU 'CBU001' AND CBU 'CBU002' AND PRODUCT 'PROD001' AND PRODUCT 'PROD002' AND PRODUCT 'PROD003' AND CONTRACT 'CONTR001' AND CONTRACT 'CONTR002' AND KYC 'KYC001' AND KYC 'KYC002' AND SERVICE_MAP 'SM001' AND SERVICE_MAP 'SM002' AND OPPORTUNITY 'OPP001' AND OPPORTUNITY 'OPP002'";

        let result = parser.parse_deal_record_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, DealOperation::Create));
        assert_eq!(command.deal_id, Some("DEAL001".to_string()));
        assert_eq!(command.description, Some("Alpha Bank Multi-Product Onboarding".to_string()));
        assert_eq!(command.primary_introducing_client, Some("Alpha Corporation".to_string()));
        assert_eq!(command.cbu_ids, vec!["CBU001", "CBU002"]);
        assert_eq!(command.product_ids, vec!["PROD001", "PROD002", "PROD003"]);
        assert_eq!(command.contract_ids, vec!["CONTR001", "CONTR002"]);
        assert_eq!(command.kyc_clearance_ids, vec!["KYC001", "KYC002"]);
        assert_eq!(command.service_map_ids, vec!["SM001", "SM002"]);
        assert_eq!(command.opportunity_ids, vec!["OPP001", "OPP002"]);
    }

    #[test]
    fn test_parse_link_deal_command() {
        let parser = DealRecordDslParser::new(None);
        let dsl = "LINK DEAL 'DEAL001' WITH CBU 'CBU003'";

        let result = parser.parse_deal_record_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, DealOperation::Link));
        assert_eq!(command.deal_id, Some("DEAL001".to_string()));
        assert_eq!(command.link_resource_type, Some("CBU".to_string()));
        assert_eq!(command.link_resource_id, Some("CBU003".to_string()));
    }

    #[test]
    fn test_parse_unlink_deal_command() {
        let parser = DealRecordDslParser::new(None);
        let dsl = "UNLINK DEAL 'DEAL001' FROM PRODUCT 'PROD005'";

        let result = parser.parse_deal_record_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, DealOperation::Unlink));
        assert_eq!(command.deal_id, Some("DEAL001".to_string()));
        assert_eq!(command.link_resource_type, Some("PRODUCT".to_string()));
        assert_eq!(command.link_resource_id, Some("PROD005".to_string()));
    }

    #[test]
    fn test_parse_link_opportunity_command() {
        let parser = DealRecordDslParser::new(None);
        let dsl = "LINK DEAL 'DEAL001' WITH OPPORTUNITY 'OPP003'";

        let result = parser.parse_deal_record_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, DealOperation::Link));
        assert_eq!(command.deal_id, Some("DEAL001".to_string()));
        assert_eq!(command.link_resource_type, Some("OPPORTUNITY".to_string()));
        assert_eq!(command.link_resource_id, Some("OPP003".to_string()));
    }

    #[test]
    fn test_parse_opportunity_in_create_deal() {
        let parser = DealRecordDslParser::new(None);
        let dsl = "CREATE DEAL 'DEAL002' ; 'Fund Management Deal' ; 'Beta Corp' WITH OPPORTUNITY 'OPP001' AND OPPORTUNITY 'OPP002' AND CBU 'CBU001'";

        let result = parser.parse_deal_record_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, DealOperation::Create));
        assert_eq!(command.deal_id, Some("DEAL002".to_string()));
        assert_eq!(command.opportunity_ids, vec!["OPP001", "OPP002"]);
        assert_eq!(command.cbu_ids, vec!["CBU001"]);
    }
}