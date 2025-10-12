use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use chrono::{NaiveDate, DateTime, Utc};

use super::DbOperations;

// Core CBU structures

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClientBusinessUnit {
    pub id: i32,
    pub cbu_id: String,
    pub cbu_name: String,
    pub description: Option<String>,
    pub primary_entity_id: Option<String>,
    pub primary_lei: Option<String>,
    pub domicile_country: Option<String>,
    pub regulatory_jurisdiction: Option<String>,
    pub business_type: Option<String>,
    pub status: String,
    pub created_date: Option<NaiveDate>,
    pub last_review_date: Option<NaiveDate>,
    pub next_review_date: Option<NaiveDate>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CbuRole {
    pub id: i32,
    pub role_code: String,
    pub role_name: String,
    pub description: Option<String>,
    pub role_category: Option<String>,
    pub display_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CbuMember {
    pub id: i32,
    pub cbu_id: i32,
    pub role_id: i32,
    pub entity_id: String,
    pub entity_name: String,
    pub entity_lei: Option<String>,
    pub is_primary: bool,
    pub effective_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub authorized_persons: Option<serde_json::Value>,
    pub is_active: bool,
    pub receives_notifications: bool,
    pub has_trading_authority: bool,
    pub has_settlement_authority: bool,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// Request/Response structures for API

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCbuRequest {
    pub cbu_name: String,
    pub description: Option<String>,
    pub primary_entity_id: Option<String>,
    pub primary_lei: Option<String>,
    pub domicile_country: Option<String>,
    pub regulatory_jurisdiction: Option<String>,
    pub business_type: Option<String>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCbuMemberRequest {
    pub cbu_id: String, // External CBU ID
    pub role_code: String,
    pub entity_id: String,
    pub entity_name: String,
    pub entity_lei: Option<String>,
    pub is_primary: Option<bool>,
    pub effective_date: Option<NaiveDate>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub has_trading_authority: Option<bool>,
    pub has_settlement_authority: Option<bool>,
    pub notes: Option<String>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CbuSummary {
    pub id: i32,
    pub cbu_id: String,
    pub cbu_name: String,
    pub description: Option<String>,
    pub primary_lei: Option<String>,
    pub domicile_country: Option<String>,
    pub business_type: Option<String>,
    pub status: String,
    pub created_date: Option<NaiveDate>,
    pub member_count: Option<i64>,
    pub role_count: Option<i64>,
    pub roles: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CbuMemberDetail {
    pub id: i32,
    pub cbu_id: String,
    pub cbu_name: String,
    pub role_code: String,
    pub role_name: String,
    pub role_category: Option<String>,
    pub entity_id: String,
    pub entity_name: String,
    pub entity_lei: Option<String>,
    pub is_primary: bool,
    pub effective_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub contact_email: Option<String>,
    pub is_active: bool,
    pub has_trading_authority: bool,
    pub has_settlement_authority: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DbOperations {
    // === CBU MANAGEMENT ===

    /// Create a new Client Business Unit
    pub async fn create_cbu(request: CreateCbuRequest) -> Result<ClientBusinessUnit, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        // Generate unique CBU ID (could be customized based on business rules)
        let cbu_id = format!("CBU-{:06}", chrono::Utc::now().timestamp_millis() % 1000000);

        let query = r#"
            INSERT INTO client_business_units (
                cbu_id, cbu_name, description, primary_entity_id, primary_lei,
                domicile_country, regulatory_jurisdiction, business_type, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
        "#;

        sqlx::query_as::<_, ClientBusinessUnit>(query)
            .bind(&cbu_id)
            .bind(&request.cbu_name)
            .bind(&request.description)
            .bind(&request.primary_entity_id)
            .bind(&request.primary_lei)
            .bind(&request.domicile_country)
            .bind(&request.regulatory_jurisdiction)
            .bind(&request.business_type)
            .bind(&request.created_by)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to create CBU: {}", e))
    }

    /// Get CBU by external ID
    pub async fn get_cbu_by_id(cbu_id: &str) -> Result<Option<ClientBusinessUnit>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = "SELECT * FROM client_business_units WHERE cbu_id = $1";

        match sqlx::query_as::<_, ClientBusinessUnit>(query)
            .bind(cbu_id)
            .fetch_optional(&pool)
            .await
        {
            Ok(result) => Ok(result),
            Err(e) => Err(format!("Failed to get CBU: {}", e)),
        }
    }

    /// List all CBUs with summary information
    pub async fn list_cbus() -> Result<Vec<CbuSummary>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = "SELECT * FROM v_cbu_summary ORDER BY cbu_name";

        sqlx::query_as::<_, CbuSummary>(query)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to list CBUs: {}", e))
    }

    /// Get all available CBU roles
    pub async fn get_cbu_roles() -> Result<Vec<CbuRole>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = r#"
            SELECT * FROM cbu_roles
            WHERE is_active = true
            ORDER BY role_category, display_order, role_name
        "#;

        sqlx::query_as::<_, CbuRole>(query)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to get CBU roles: {}", e))
    }

    /// Add a member to a CBU with a specific role
    pub async fn add_cbu_member(request: AddCbuMemberRequest) -> Result<CbuMember, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        // Get CBU internal ID
        let cbu = Self::get_cbu_by_id(&request.cbu_id).await?
            .ok_or_else(|| format!("CBU not found: {}", request.cbu_id))?;

        // Get role ID
        let role_query = "SELECT id FROM cbu_roles WHERE role_code = $1 AND is_active = true";
        let role_id: (i32,) = sqlx::query_as(role_query)
            .bind(&request.role_code)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Role not found or inactive: {}", e))?;

        // Insert member
        let insert_query = r#"
            INSERT INTO cbu_members (
                cbu_id, role_id, entity_id, entity_name, entity_lei, is_primary,
                effective_date, contact_email, contact_phone,
                has_trading_authority, has_settlement_authority, notes, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
        "#;

        sqlx::query_as::<_, CbuMember>(insert_query)
            .bind(cbu.id)
            .bind(role_id.0)
            .bind(&request.entity_id)
            .bind(&request.entity_name)
            .bind(&request.entity_lei)
            .bind(request.is_primary.unwrap_or(false))
            .bind(request.effective_date)
            .bind(&request.contact_email)
            .bind(&request.contact_phone)
            .bind(request.has_trading_authority.unwrap_or(false))
            .bind(request.has_settlement_authority.unwrap_or(false))
            .bind(&request.notes)
            .bind(&request.created_by)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to add CBU member: {}", e))
    }

    /// Get all members of a CBU with role details
    pub async fn get_cbu_members(cbu_id: &str) -> Result<Vec<CbuMemberDetail>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = r#"
            SELECT * FROM v_cbu_members_detail
            WHERE cbu_id = $1 AND is_active = true
            ORDER BY role_code, entity_name
        "#;

        sqlx::query_as::<_, CbuMemberDetail>(query)
            .bind(cbu_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to get CBU members: {}", e))
    }

    /// Update CBU member status
    pub async fn update_cbu_member_status(
        member_id: i32,
        is_active: bool,
        updated_by: Option<String>
    ) -> Result<(), String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = r#"
            UPDATE cbu_members
            SET is_active = $1, updated_by = $2, updated_at = CURRENT_TIMESTAMP
            WHERE id = $3
        "#;

        sqlx::query(query)
            .bind(is_active)
            .bind(updated_by)
            .bind(member_id)
            .execute(&pool)
            .await
            .map_err(|e| format!("Failed to update member status: {}", e))?;

        Ok(())
    }

    /// Remove a member from a CBU (soft delete)
    pub async fn remove_cbu_member(
        cbu_id: &str,
        entity_id: &str,
        role_code: &str,
        updated_by: Option<String>
    ) -> Result<(), String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = r#"
            UPDATE cbu_members
            SET is_active = false, updated_by = $4, updated_at = CURRENT_TIMESTAMP
            WHERE cbu_id = (SELECT id FROM client_business_units WHERE cbu_id = $1)
              AND entity_id = $2
              AND role_id = (SELECT id FROM cbu_roles WHERE role_code = $3)
        "#;

        sqlx::query(query)
            .bind(cbu_id)
            .bind(entity_id)
            .bind(role_code)
            .bind(updated_by)
            .execute(&pool)
            .await
            .map_err(|e| format!("Failed to remove CBU member: {}", e))?;

        Ok(())
    }

    /// Search CBUs by name or entity
    pub async fn search_cbus(search_term: &str) -> Result<Vec<CbuSummary>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = r#"
            SELECT * FROM v_cbu_summary
            WHERE cbu_name ILIKE $1
               OR description ILIKE $1
               OR primary_lei ILIKE $1
            ORDER BY cbu_name
        "#;

        let search_pattern = format!("%{}%", search_term);

        sqlx::query_as::<_, CbuSummary>(query)
            .bind(&search_pattern)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to search CBUs: {}", e))
    }

    /// Get CBU roles grouped by category
    pub async fn get_cbu_roles_by_category() -> Result<HashMap<String, Vec<CbuRole>>, String> {
        let roles = Self::get_cbu_roles().await?;

        let mut grouped = HashMap::new();
        for role in roles {
            let category = role.role_category.clone().unwrap_or_else(|| "Other".to_string());
            grouped.entry(category).or_insert_with(Vec::new).push(role);
        }

        Ok(grouped)
    }

    /// Update CBU basic information
    pub async fn update_cbu(
        cbu_id: &str,
        cbu_name: Option<String>,
        description: Option<String>,
        business_type: Option<String>,
        updated_by: Option<String>
    ) -> Result<ClientBusinessUnit, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = r#"
            UPDATE client_business_units
            SET cbu_name = COALESCE($2, cbu_name),
                description = COALESCE($3, description),
                business_type = COALESCE($4, business_type),
                updated_by = $5,
                updated_at = CURRENT_TIMESTAMP
            WHERE cbu_id = $1
            RETURNING *
        "#;

        sqlx::query_as::<_, ClientBusinessUnit>(query)
            .bind(cbu_id)
            .bind(cbu_name)
            .bind(description)
            .bind(business_type)
            .bind(updated_by)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to update CBU: {}", e))
    }
}