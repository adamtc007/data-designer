// CBU DSL for CRUD operations with entity validation
// Grammar:
// CREATE CBU '<name>' ; '<description>' WITH
//   ENTITY ('<name>', '<id>') AS 'Asset Owner' AND
//   ENTITY ('<name>', '<id>') AS 'Investment Manager' AND
//   ENTITY ('<name>', '<id>') AS 'Managing Company'
//
// UPDATE CBU '<cbu_id>' SET <field> = '<value>'
// DELETE CBU '<cbu_id>'
// QUERY CBU [WHERE <condition>]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbuEntity {
    pub name: String,
    pub entity_id: String,
    pub role: EntityRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityRole {
    AssetOwner,
    InvestmentManager,
    ManagingCompany,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbuDslCommand {
    pub operation: CbuOperation,
    pub cbu_name: Option<String>,
    pub cbu_id: Option<String>,
    pub description: Option<String>,
    pub entities: Vec<CbuEntity>,
    pub update_fields: HashMap<String, String>,
    pub query_conditions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CbuOperation {
    Create,
    Update,
    Delete,
    Query,
}

#[derive(Debug, Clone)]
pub struct CbuDslParser {
    pub pool: Option<PgPool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbuDslResult {
    pub success: bool,
    pub message: String,
    pub cbu_id: Option<String>,
    pub validation_errors: Vec<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum CbuDslError {
    ParseError(String),
    ValidationError(String),
    DatabaseError(String),
    EntityNotFound(String),
}

impl std::fmt::Display for CbuDslError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CbuDslError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            CbuDslError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            CbuDslError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            CbuDslError::EntityNotFound(msg) => write!(f, "Entity Not Found: {}", msg),
        }
    }
}

impl std::error::Error for CbuDslError {}

impl CbuDslParser {
    pub fn new(pool: Option<PgPool>) -> Self {
        Self { pool }
    }

    /// Parse CBU DSL command into structured format
    pub fn parse_cbu_dsl(&self, dsl_text: &str) -> Result<CbuDslCommand, CbuDslError> {
        let dsl_text = dsl_text.trim();

        if dsl_text.to_uppercase().starts_with("CREATE CBU") {
            self.parse_create_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("UPDATE CBU") {
            self.parse_update_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("DELETE CBU") {
            self.parse_delete_command(dsl_text)
        } else if dsl_text.to_uppercase().starts_with("QUERY CBU") {
            self.parse_query_command(dsl_text)
        } else {
            Err(CbuDslError::ParseError(
                "Command must start with CREATE CBU, UPDATE CBU, DELETE CBU, or QUERY CBU".to_string()
            ))
        }
    }

    /// Parse CREATE CBU command
    fn parse_create_command(&self, dsl_text: &str) -> Result<CbuDslCommand, CbuDslError> {
        // CREATE CBU 'Growth Fund Alpha' ; 'A diversified growth-focused investment fund' WITH
        //   ENTITY ('Alpha Capital', 'AC001') AS 'Asset Owner' AND
        //   ENTITY ('Beta Management', 'BM002') AS 'Investment Manager' AND
        //   ENTITY ('Gamma Services', 'GS003') AS 'Managing Company'

        let parts: Vec<&str> = dsl_text.splitn(2, " WITH ").collect();
        if parts.len() != 2 {
            return Err(CbuDslError::ParseError(
                "CREATE CBU command must include ' WITH ' clause for entities".to_string()
            ));
        }

        // Parse CBU name and description
        let cbu_part = parts[0].trim();
        let entities_part = parts[1].trim();

        // Extract CBU name and description from: CREATE CBU 'name' ; 'description'
        let cbu_content = cbu_part.strip_prefix("CREATE CBU").unwrap().trim();
        let cbu_parts: Vec<&str> = cbu_content.splitn(2, " ; ").collect();

        if cbu_parts.len() != 2 {
            return Err(CbuDslError::ParseError(
                "CREATE CBU must have format: CREATE CBU 'name' ; 'description'".to_string()
            ));
        }

        let cbu_name = self.extract_quoted_string(cbu_parts[0].trim())?;
        let description = self.extract_quoted_string(cbu_parts[1].trim())?;

        // Parse entities
        let entities = self.parse_entities(entities_part)?;

        // Validate we have all required roles
        self.validate_required_roles(&entities)?;

        Ok(CbuDslCommand {
            operation: CbuOperation::Create,
            cbu_name: Some(cbu_name),
            cbu_id: None,
            description: Some(description),
            entities,
            update_fields: HashMap::new(),
            query_conditions: None,
        })
    }

    /// Parse UPDATE CBU command
    fn parse_update_command(&self, dsl_text: &str) -> Result<CbuDslCommand, CbuDslError> {
        // UPDATE CBU 'CBU001' SET description = 'Updated description'
        let content = dsl_text.strip_prefix("UPDATE CBU").unwrap().trim();
        let parts: Vec<&str> = content.splitn(2, " SET ").collect();

        if parts.len() != 2 {
            return Err(CbuDslError::ParseError(
                "UPDATE CBU command must include ' SET ' clause".to_string()
            ));
        }

        let cbu_id = self.extract_quoted_string(parts[0].trim())?;
        let set_clause = parts[1].trim();

        // Parse SET clause: field = 'value'
        let mut update_fields = HashMap::new();
        let assignments: Vec<&str> = set_clause.split(" AND ").collect();

        for assignment in assignments {
            let assign_parts: Vec<&str> = assignment.splitn(2, " = ").collect();
            if assign_parts.len() != 2 {
                return Err(CbuDslError::ParseError(
                    "SET clause must have format: field = 'value'".to_string()
                ));
            }

            let field = assign_parts[0].trim().to_string();
            let value = self.extract_quoted_string(assign_parts[1].trim())?;
            update_fields.insert(field, value);
        }

        Ok(CbuDslCommand {
            operation: CbuOperation::Update,
            cbu_name: None,
            cbu_id: Some(cbu_id),
            description: None,
            entities: Vec::new(),
            update_fields,
            query_conditions: None,
        })
    }

    /// Parse DELETE CBU command
    fn parse_delete_command(&self, dsl_text: &str) -> Result<CbuDslCommand, CbuDslError> {
        // DELETE CBU 'CBU001'
        let content = dsl_text.strip_prefix("DELETE CBU").unwrap().trim();
        let cbu_id = self.extract_quoted_string(content)?;

        Ok(CbuDslCommand {
            operation: CbuOperation::Delete,
            cbu_name: None,
            cbu_id: Some(cbu_id),
            description: None,
            entities: Vec::new(),
            update_fields: HashMap::new(),
            query_conditions: None,
        })
    }

    /// Parse QUERY CBU command
    fn parse_query_command(&self, dsl_text: &str) -> Result<CbuDslCommand, CbuDslError> {
        // QUERY CBU WHERE status = 'active'
        let content = dsl_text.strip_prefix("QUERY CBU").unwrap().trim();

        let query_conditions = if content.is_empty() {
            None
        } else if content.to_uppercase().starts_with("WHERE ") {
            Some(content.strip_prefix("WHERE ").unwrap().trim().to_string())
        } else {
            return Err(CbuDslError::ParseError(
                "QUERY CBU can be used alone or with WHERE clause".to_string()
            ));
        };

        Ok(CbuDslCommand {
            operation: CbuOperation::Query,
            cbu_name: None,
            cbu_id: None,
            description: None,
            entities: Vec::new(),
            update_fields: HashMap::new(),
            query_conditions,
        })
    }

    /// Parse entities from the WITH clause
    fn parse_entities(&self, entities_text: &str) -> Result<Vec<CbuEntity>, CbuDslError> {
        let mut entities = Vec::new();

        // Split by AND to get individual entity declarations
        let entity_declarations: Vec<&str> = entities_text.split(" AND ").collect();

        for declaration in entity_declarations {
            let declaration = declaration.trim();

            // Parse: ENTITY ('name', 'id') AS 'role'
            let parts: Vec<&str> = declaration.splitn(2, " AS ").collect();
            if parts.len() != 2 {
                return Err(CbuDslError::ParseError(
                    "Entity declaration must have format: ENTITY ('name', 'id') AS 'role'".to_string()
                ));
            }

            let entity_part = parts[0].trim();
            let role_part = parts[1].trim();

            // Extract entity info
            if !entity_part.to_uppercase().starts_with("ENTITY") {
                return Err(CbuDslError::ParseError(
                    "Entity declaration must start with 'ENTITY'".to_string()
                ));
            }

            let entity_content = entity_part.strip_prefix("ENTITY").unwrap().trim();
            if !entity_content.starts_with('(') || !entity_content.ends_with(')') {
                return Err(CbuDslError::ParseError(
                    "Entity must be specified as ('name', 'id')".to_string()
                ));
            }

            let entity_inner = &entity_content[1..entity_content.len()-1];
            let entity_parts: Vec<&str> = entity_inner.splitn(2, ", ").collect();

            if entity_parts.len() != 2 {
                return Err(CbuDslError::ParseError(
                    "Entity must have format ('name', 'id')".to_string()
                ));
            }

            let name = self.extract_quoted_string(entity_parts[0].trim())?;
            let entity_id = self.extract_quoted_string(entity_parts[1].trim())?;

            // Extract role
            let role_str = self.extract_quoted_string(role_part)?;
            let role = match role_str.as_str() {
                "Asset Owner" => EntityRole::AssetOwner,
                "Investment Manager" => EntityRole::InvestmentManager,
                "Managing Company" => EntityRole::ManagingCompany,
                _ => return Err(CbuDslError::ParseError(
                    format!("Invalid role '{}'. Must be 'Asset Owner', 'Investment Manager', or 'Managing Company'", role_str)
                )),
            };

            entities.push(CbuEntity {
                name,
                entity_id,
                role,
            });
        }

        Ok(entities)
    }

    /// Extract string from quotes
    fn extract_quoted_string(&self, text: &str) -> Result<String, CbuDslError> {
        let text = text.trim();
        if text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2 {
            Ok(text[1..text.len()-1].to_string())
        } else if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
            Ok(text[1..text.len()-1].to_string())
        } else {
            Err(CbuDslError::ParseError(
                format!("String must be quoted: {}", text)
            ))
        }
    }

    /// Validate that all required roles are present
    fn validate_required_roles(&self, entities: &[CbuEntity]) -> Result<(), CbuDslError> {
        let mut has_asset_owner = false;
        let mut has_investment_manager = false;
        let mut has_managing_company = false;

        for entity in entities {
            match entity.role {
                EntityRole::AssetOwner => has_asset_owner = true,
                EntityRole::InvestmentManager => has_investment_manager = true,
                EntityRole::ManagingCompany => has_managing_company = true,
            }
        }

        let mut missing_roles = Vec::new();
        if !has_asset_owner {
            missing_roles.push("Asset Owner");
        }
        if !has_investment_manager {
            missing_roles.push("Investment Manager");
        }
        if !has_managing_company {
            missing_roles.push("Managing Company");
        }

        if !missing_roles.is_empty() {
            return Err(CbuDslError::ValidationError(
                format!("Missing required roles: {}", missing_roles.join(", "))
            ));
        }

        Ok(())
    }

    /// Execute CBU DSL command
    pub async fn execute_cbu_dsl(&self, command: CbuDslCommand) -> Result<CbuDslResult, CbuDslError> {
        match command.operation {
            CbuOperation::Create => self.execute_create(command).await,
            CbuOperation::Update => self.execute_update(command).await,
            CbuOperation::Delete => self.execute_delete(command).await,
            CbuOperation::Query => self.execute_query(command).await,
        }
    }

    /// Validate that all referenced entities exist in the client entity table
    async fn validate_entities(&self, entities: &[CbuEntity]) -> Result<Vec<String>, CbuDslError> {
        let Some(pool) = &self.pool else {
            return Err(CbuDslError::DatabaseError("No database connection available".to_string()));
        };

        let mut validation_errors = Vec::new();

        for entity in entities {
            // Check if entity exists in client_entities table
            let query = "SELECT COUNT(*) as count FROM client_entities WHERE entity_id = $1 AND entity_name = $2";

            match sqlx::query(query)
                .bind(&entity.entity_id)
                .bind(&entity.name)
                .fetch_one(pool)
                .await
            {
                Ok(row) => {
                    let count: i64 = row.get("count");
                    if count == 0 {
                        validation_errors.push(format!(
                            "Entity '{}' with ID '{}' not found in client entities table",
                            entity.name, entity.entity_id
                        ));
                    }
                }
                Err(e) => {
                    return Err(CbuDslError::DatabaseError(format!("Failed to validate entity: {}", e)));
                }
            }
        }

        Ok(validation_errors)
    }

    /// Execute CREATE CBU command
    async fn execute_create(&self, command: CbuDslCommand) -> Result<CbuDslResult, CbuDslError> {
        let Some(pool) = &self.pool else {
            return Err(CbuDslError::DatabaseError("No database connection available".to_string()));
        };

        // Validate entities exist
        let validation_errors = self.validate_entities(&command.entities).await?;
        if !validation_errors.is_empty() {
            return Ok(CbuDslResult {
                success: false,
                message: "Entity validation failed".to_string(),
                cbu_id: None,
                validation_errors,
                data: None,
            });
        }

        // Generate CBU ID (simplified for demo)
        let cbu_id = format!("CBU{:06}", 123456);

        // Insert CBU into database
        let insert_query = r#"
            INSERT INTO cbu (cbu_id, cbu_name, description, legal_entity_name, jurisdiction, business_model, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            RETURNING id
        "#;

        match sqlx::query(insert_query)
            .bind(&cbu_id)
            .bind(command.cbu_name.as_ref().unwrap())
            .bind(command.description.as_ref().unwrap())
            .bind(command.cbu_name.as_ref().unwrap()) // Use CBU name as legal entity name
            .bind("US") // Default jurisdiction
            .bind("Investment Management") // Default business model
            .bind("active")
            .fetch_one(pool)
            .await
        {
            Ok(_) => {
                // Insert entity relationships
                for entity in &command.entities {
                    let role_name = match entity.role {
                        EntityRole::AssetOwner => "Asset Owner",
                        EntityRole::InvestmentManager => "Investment Manager",
                        EntityRole::ManagingCompany => "Managing Company",
                    };

                    let relationship_query = r#"
                        INSERT INTO cbu_entity_relationships (cbu_id, entity_id, entity_name, role_name, created_at)
                        VALUES ($1, $2, $3, $4, NOW())
                    "#;

                    if let Err(e) = sqlx::query(relationship_query)
                        .bind(&cbu_id)
                        .bind(&entity.entity_id)
                        .bind(&entity.name)
                        .bind(role_name)
                        .execute(pool)
                        .await
                    {
                        return Err(CbuDslError::DatabaseError(
                            format!("Failed to create entity relationship: {}", e)
                        ));
                    }
                }

                Ok(CbuDslResult {
                    success: true,
                    message: format!("CBU '{}' created successfully", command.cbu_name.unwrap()),
                    cbu_id: Some(cbu_id),
                    validation_errors: Vec::new(),
                    data: None,
                })
            }
            Err(e) => Err(CbuDslError::DatabaseError(format!("Failed to create CBU: {}", e)))
        }
    }

    /// Execute UPDATE CBU command
    async fn execute_update(&self, command: CbuDslCommand) -> Result<CbuDslResult, CbuDslError> {
        let Some(pool) = &self.pool else {
            return Err(CbuDslError::DatabaseError("No database connection available".to_string()));
        };

        let cbu_id = command.cbu_id.as_ref().unwrap();

        // Build dynamic UPDATE query
        let mut set_clauses = Vec::new();
        let mut values: Vec<&str> = Vec::new();

        for (field, value) in &command.update_fields {
            set_clauses.push(format!("{} = ${}", field, values.len() + 2));
            values.push(value);
        }

        if set_clauses.is_empty() {
            return Err(CbuDslError::ValidationError("No fields to update".to_string()));
        }

        let update_query = format!(
            "UPDATE cbu SET {}, updated_at = NOW() WHERE cbu_id = $1",
            set_clauses.join(", ")
        );

        let mut query = sqlx::query(&update_query).bind(cbu_id);
        for value in values {
            query = query.bind(value);
        }

        match query.execute(pool).await {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    Ok(CbuDslResult {
                        success: true,
                        message: format!("CBU '{}' updated successfully", cbu_id),
                        cbu_id: Some(cbu_id.clone()),
                        validation_errors: Vec::new(),
                        data: None,
                    })
                } else {
                    Err(CbuDslError::ValidationError(format!("CBU '{}' not found", cbu_id)))
                }
            }
            Err(e) => Err(CbuDslError::DatabaseError(format!("Failed to update CBU: {}", e)))
        }
    }

    /// Execute DELETE CBU command
    async fn execute_delete(&self, command: CbuDslCommand) -> Result<CbuDslResult, CbuDslError> {
        let Some(pool) = &self.pool else {
            return Err(CbuDslError::DatabaseError("No database connection available".to_string()));
        };

        let cbu_id = command.cbu_id.as_ref().unwrap();

        // Delete entity relationships first
        let delete_relationships_query = "DELETE FROM cbu_entity_relationships WHERE cbu_id = $1";
        if let Err(e) = sqlx::query(delete_relationships_query)
            .bind(cbu_id)
            .execute(pool)
            .await
        {
            return Err(CbuDslError::DatabaseError(
                format!("Failed to delete CBU relationships: {}", e)
            ));
        }

        // Delete CBU
        let delete_query = "DELETE FROM cbu WHERE cbu_id = $1";
        match sqlx::query(delete_query)
            .bind(cbu_id)
            .execute(pool)
            .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    Ok(CbuDslResult {
                        success: true,
                        message: format!("CBU '{}' deleted successfully", cbu_id),
                        cbu_id: Some(cbu_id.clone()),
                        validation_errors: Vec::new(),
                        data: None,
                    })
                } else {
                    Err(CbuDslError::ValidationError(format!("CBU '{}' not found", cbu_id)))
                }
            }
            Err(e) => Err(CbuDslError::DatabaseError(format!("Failed to delete CBU: {}", e)))
        }
    }

    /// Execute QUERY CBU command
    async fn execute_query(&self, command: CbuDslCommand) -> Result<CbuDslResult, CbuDslError> {
        let Some(pool) = &self.pool else {
            return Err(CbuDslError::DatabaseError("No database connection available".to_string()));
        };

        let base_query = r#"
            SELECT c.*,
                   array_agg(cer.entity_name || ' (' || cer.role_name || ')') as entities
            FROM cbu c
            LEFT JOIN cbu_entity_relationships cer ON c.cbu_id = cer.cbu_id
        "#;

        let (final_query, _where_clause) = if let Some(conditions) = &command.query_conditions {
            (format!("{} WHERE {} GROUP BY c.id", base_query, conditions), true)
        } else {
            (format!("{} GROUP BY c.id", base_query), false)
        };

        match sqlx::query(&final_query).fetch_all(pool).await {
            Ok(rows) => {
                let mut results = Vec::new();
                for row in rows {
                    let mut cbu_data = serde_json::Map::new();
                    cbu_data.insert("cbu_id".to_string(), serde_json::Value::String(row.get("cbu_id")));
                    cbu_data.insert("cbu_name".to_string(), serde_json::Value::String(row.get("cbu_name")));
                    cbu_data.insert("description".to_string(), serde_json::Value::String(row.get::<Option<String>, _>("description").unwrap_or_default()));
                    cbu_data.insert("status".to_string(), serde_json::Value::String(row.get("status")));

                    let entities: Vec<String> = row.get("entities");
                    cbu_data.insert("entities".to_string(), serde_json::Value::Array(
                        entities.into_iter().map(serde_json::Value::String).collect()
                    ));

                    results.push(serde_json::Value::Object(cbu_data));
                }

                Ok(CbuDslResult {
                    success: true,
                    message: format!("Found {} CBU(s)", results.len()),
                    cbu_id: None,
                    validation_errors: Vec::new(),
                    data: Some(serde_json::Value::Array(results)),
                })
            }
            Err(e) => Err(CbuDslError::DatabaseError(format!("Failed to query CBUs: {}", e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_cbu_command() {
        let parser = CbuDslParser::new(None);
        let dsl = r#"CREATE CBU 'Growth Fund Alpha' ; 'A diversified growth-focused investment fund' WITH
            ENTITY ('Alpha Capital', 'AC001') AS 'Asset Owner' AND
            ENTITY ('Beta Management', 'BM002') AS 'Investment Manager' AND
            ENTITY ('Gamma Services', 'GS003') AS 'Managing Company'"#;

        let result = parser.parse_cbu_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, CbuOperation::Create));
        assert_eq!(command.cbu_name, Some("Growth Fund Alpha".to_string()));
        assert_eq!(command.description, Some("A diversified growth-focused investment fund".to_string()));
        assert_eq!(command.entities.len(), 3);
    }

    #[test]
    fn test_parse_update_cbu_command() {
        let parser = CbuDslParser::new(None);
        let dsl = "UPDATE CBU 'CBU001' SET description = 'Updated description'";

        let result = parser.parse_cbu_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, CbuOperation::Update));
        assert_eq!(command.cbu_id, Some("CBU001".to_string()));
        assert_eq!(command.update_fields.get("description"), Some(&"Updated description".to_string()));
    }

    #[test]
    fn test_parse_delete_cbu_command() {
        let parser = CbuDslParser::new(None);
        let dsl = "DELETE CBU 'CBU001'";

        let result = parser.parse_cbu_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, CbuOperation::Delete));
        assert_eq!(command.cbu_id, Some("CBU001".to_string()));
    }

    #[test]
    fn test_parse_query_cbu_command() {
        let parser = CbuDslParser::new(None);
        let dsl = "QUERY CBU WHERE status = 'active'";

        let result = parser.parse_cbu_dsl(dsl);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert!(matches!(command.operation, CbuOperation::Query));
        assert_eq!(command.query_conditions, Some("status = 'active'".to_string()));
    }
}