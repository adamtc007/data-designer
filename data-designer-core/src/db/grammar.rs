use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::db::{DbPool, DbOperations};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GrammarRule {
    pub id: i32,
    pub name: String,
    pub definition: String,
    pub rule_type: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GrammarExtension {
    pub id: i32,
    pub name: String,
    pub extension_type: String, // Maps to 'type' column
    pub signature: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GrammarMetadata {
    pub id: i32,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactGrammarInfo {
    pub keywords: Vec<String>,
    pub functions: Vec<FunctionInfo>,
    pub operators: Vec<String>,
    pub kyc_attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub signature: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGrammarRuleRequest {
    pub name: String,
    pub definition: String,
    pub rule_type: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGrammarExtensionRequest {
    pub name: String,
    pub extension_type: String,
    pub signature: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
}

pub struct GrammarOperations;

impl GrammarOperations {
    /// Get all grammar rules
    pub async fn get_all_rules(pool: &DbPool) -> Result<Vec<GrammarRule>, String> {
        let query = "SELECT id, name, definition, rule_type, description, category, created_at, updated_at FROM grammar_rules ORDER BY category, name";

        DbOperations::query_all(pool, query).await
    }

    /// Get all grammar extensions
    pub async fn get_all_extensions(pool: &DbPool) -> Result<Vec<GrammarExtension>, String> {
        let query = "SELECT id, name, type as extension_type, signature, description, category, created_at, updated_at FROM grammar_extensions ORDER BY type, category, name";

        DbOperations::query_all(pool, query).await
    }

    /// Get grammar metadata
    pub async fn get_metadata(pool: &DbPool) -> Result<GrammarMetadata, String> {
        let query = "SELECT id, version, description, author, created_at, updated_at, is_active FROM grammar_metadata WHERE is_active = true ORDER BY updated_at DESC LIMIT 1";

        DbOperations::query_one(pool, query).await
    }

    /// Get compact grammar info for syntax highlighting
    pub async fn get_compact_grammar_info(pool: &DbPool) -> Result<CompactGrammarInfo, String> {
        println!("🔍 Starting get_compact_grammar_info...");

        // Get keywords
        let keywords_query = "SELECT name FROM grammar_extensions WHERE type = 'keyword' ORDER BY name";
        println!("🔍 Executing keywords query...");
        let keyword_rows: Vec<(String,)> = DbOperations::query_all(pool, keywords_query).await?;
        let mut keywords: Vec<String> = keyword_rows.into_iter().map(|(name,)| name).collect();
        println!("🔍 Keywords loaded: {} items", keywords.len());

        // Add function names from grammar rules
        let func_rules_query = "SELECT name FROM grammar_rules WHERE category = 'function' AND name LIKE '%_fn' ORDER BY name";
        println!("🔍 Executing function rules query...");
        let func_rule_rows: Vec<(String,)> = DbOperations::query_all(pool, func_rules_query).await?;
        for (rule_name,) in func_rule_rows {
            if rule_name.ends_with("_fn") {
                let func_name = rule_name.trim_end_matches("_fn").to_uppercase();
                keywords.push(func_name);
            }
        }

        keywords.sort();
        keywords.dedup();

        // Get functions with signatures
        let functions_query = "SELECT name, COALESCE(signature, '()') as signature, COALESCE(description, '') as description FROM grammar_extensions WHERE type = 'function' ORDER BY name";
        let function_rows: Vec<(String, String, String)> = DbOperations::query_all(pool, functions_query).await?;
        let functions: Vec<FunctionInfo> = function_rows.into_iter().map(|(name, signature, description)| {
            FunctionInfo { name, signature, description }
        }).collect();

        // Get operators
        let operators_query = "SELECT name FROM grammar_extensions WHERE type = 'operator' ORDER BY category, name";
        let operator_rows: Vec<(String,)> = DbOperations::query_all(pool, operators_query).await?;
        let operators: Vec<String> = operator_rows.into_iter().map(|(name,)| name).collect();

        // Get KYC attributes from business attributes table
        let kyc_attributes_query = "SELECT DISTINCT attribute_name FROM business_attributes WHERE entity_name IN ('Client', 'KYC', 'Compliance') ORDER BY attribute_name LIMIT 50";
        println!("🔍 Executing KYC attributes query...");
        let kyc_rows: Vec<(String,)> = DbOperations::query_all(pool, kyc_attributes_query).await.unwrap_or_default();
        let mut kyc_attributes: Vec<String> = kyc_rows.into_iter().map(|(name,)| name).collect();

        // Add fallback KYC attributes if none found
        if kyc_attributes.is_empty() {
            kyc_attributes = vec![
                "client_id".to_string(),
                "legal_entity_name".to_string(),
                "legal_entity_identifier".to_string(),
                "risk_rating".to_string(),
                "aum_usd".to_string(),
                "kyc_completeness".to_string(),
                "documents_received".to_string(),
                "documents_required".to_string(),
                "aml_risk_score".to_string(),
                "pep_status".to_string(),
                "sanctions_check".to_string(),
                "fatca_status".to_string(),
                "crs_reporting".to_string(),
                "entity_type".to_string(),
                "jurisdiction".to_string(),
                "regulatory_status".to_string(),
                "onboarding_date".to_string(),
                "compliance_officer".to_string(),
                "trading_authority".to_string(),
                "authority_limit_usd".to_string(),
                "background_check_status".to_string(),
            ];
        }

        println!("🔍 Grammar info compiled successfully: {} keywords, {} functions, {} operators, {} KYC attributes",
                 keywords.len(), functions.len(), operators.len(), kyc_attributes.len());

        Ok(CompactGrammarInfo {
            keywords,
            functions,
            operators,
            kyc_attributes,
        })
    }

    /// Create a new grammar rule
    pub async fn create_rule(pool: &DbPool, request: CreateGrammarRuleRequest) -> Result<i32, String> {
        let query = "INSERT INTO grammar_rules (name, definition, rule_type, description, category) VALUES ($1, $2, $3, $4, $5) RETURNING id";

        let row: (i32,) = sqlx::query_as(query)
            .bind(&request.name)
            .bind(&request.definition)
            .bind(&request.rule_type)
            .bind(request.description.as_deref().unwrap_or(""))
            .bind(request.category.as_deref().unwrap_or(""))
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database insert error: {}", e))?;

        Ok(row.0)
    }

    /// Create a new grammar extension
    pub async fn create_extension(pool: &DbPool, request: CreateGrammarExtensionRequest) -> Result<i32, String> {
        let query = "INSERT INTO grammar_extensions (name, type, signature, description, category) VALUES ($1, $2, $3, $4, $5) RETURNING id";

        let row: (i32,) = sqlx::query_as(query)
            .bind(&request.name)
            .bind(&request.extension_type)
            .bind(request.signature.as_deref().unwrap_or(""))
            .bind(request.description.as_deref().unwrap_or(""))
            .bind(request.category.as_deref().unwrap_or(""))
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database insert error: {}", e))?;

        Ok(row.0)
    }

    /// Update a grammar rule
    pub async fn update_rule(pool: &DbPool, id: i32, request: CreateGrammarRuleRequest) -> Result<(), String> {
        let query = "UPDATE grammar_rules SET name = $2, definition = $3, rule_type = $4, description = $5, category = $6, updated_at = CURRENT_TIMESTAMP WHERE id = $1";

        sqlx::query(query)
            .bind(id)
            .bind(&request.name)
            .bind(&request.definition)
            .bind(&request.rule_type)
            .bind(request.description.as_deref().unwrap_or(""))
            .bind(request.category.as_deref().unwrap_or(""))
            .execute(pool)
            .await
            .map_err(|e| format!("Database update error: {}", e))?;

        Ok(())
    }

    /// Update a grammar extension
    pub async fn update_extension(pool: &DbPool, id: i32, request: CreateGrammarExtensionRequest) -> Result<(), String> {
        let query = "UPDATE grammar_extensions SET name = $2, type = $3, signature = $4, description = $5, category = $6, updated_at = CURRENT_TIMESTAMP WHERE id = $1";

        sqlx::query(query)
            .bind(id)
            .bind(&request.name)
            .bind(&request.extension_type)
            .bind(request.signature.as_deref().unwrap_or(""))
            .bind(request.description.as_deref().unwrap_or(""))
            .bind(request.category.as_deref().unwrap_or(""))
            .execute(pool)
            .await
            .map_err(|e| format!("Database update error: {}", e))?;

        Ok(())
    }

    /// Delete a grammar rule
    pub async fn delete_rule(pool: &DbPool, id: i32) -> Result<(), String> {
        let query = "DELETE FROM grammar_rules WHERE id = $1";
        DbOperations::execute_with_param(pool, query, id).await?;
        Ok(())
    }

    /// Delete a grammar extension
    pub async fn delete_extension(pool: &DbPool, id: i32) -> Result<(), String> {
        let query = "DELETE FROM grammar_extensions WHERE id = $1";
        DbOperations::execute_with_param(pool, query, id).await?;
        Ok(())
    }

    /// Get grammar rules by category
    pub async fn get_rules_by_category(pool: &DbPool, category: &str) -> Result<Vec<GrammarRule>, String> {
        let query = "SELECT id, name, definition, rule_type, description, category, created_at, updated_at FROM grammar_rules WHERE category = $1 ORDER BY name";

        DbOperations::query_all_with_param(pool, query, category).await
    }

    /// Get grammar extensions by type
    pub async fn get_extensions_by_type(pool: &DbPool, extension_type: &str) -> Result<Vec<GrammarExtension>, String> {
        let query = "SELECT id, name, type as extension_type, signature, description, category, created_at, updated_at FROM grammar_extensions WHERE type = $1 ORDER BY name";

        DbOperations::query_all_with_param(pool, query, extension_type).await
    }

    /// Search grammar rules and extensions
    pub async fn search_grammar(pool: &DbPool, search_term: &str) -> Result<(Vec<GrammarRule>, Vec<GrammarExtension>), String> {
        let search_pattern = format!("%{}%", search_term);

        let rules_query = "SELECT id, name, definition, rule_type, description, category, created_at, updated_at FROM grammar_rules WHERE name ILIKE $1 OR description ILIKE $1 ORDER BY name";
        let rules: Vec<GrammarRule> = DbOperations::query_all_with_param(pool, rules_query, &search_pattern).await?;

        let extensions_query = "SELECT id, name, type as extension_type, signature, description, category, created_at, updated_at FROM grammar_extensions WHERE name ILIKE $1 OR description ILIKE $1 ORDER BY name";
        let extensions: Vec<GrammarExtension> = DbOperations::query_all_with_param(pool, extensions_query, &search_pattern).await?;

        Ok((rules, extensions))
    }
}