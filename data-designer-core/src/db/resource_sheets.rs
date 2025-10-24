use crate::db::DbPool;
use crate::resource_sheets::*;
use anyhow::Result;
use serde_json::{Value as JsonValue, json};
use sqlx::{FromRow, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Database model for resource sheets stored as JSON
#[derive(Debug, Clone, FromRow)]
pub struct ResourceSheetRecord {
    pub resource_id: String,
    pub resource_type: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub client_id: Option<String>,
    pub product_id: Option<String>,
    pub status: String,
    pub json_data: JsonValue,
    pub metadata: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub tags: JsonValue,
}

/// Resource sheet persistence operations
pub struct ResourceSheetDb;

impl ResourceSheetDb {
    /// Initialize resource sheet tables
    pub async fn init_tables(pool: &DbPool) -> Result<()> {
        let sql = r#"
            -- Create resource_sheets table for JSON persistence
            CREATE TABLE IF NOT EXISTS resource_sheets (
                resource_id TEXT PRIMARY KEY,
                resource_type TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                version TEXT NOT NULL DEFAULT '1.0.0',
                client_id TEXT,
                product_id TEXT,
                status TEXT NOT NULL DEFAULT 'Pending',
                json_data JSONB NOT NULL,
                metadata JSONB NOT NULL DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                created_by TEXT NOT NULL,
                tags JSONB DEFAULT '[]'
            );

            -- Create indexes for efficient querying
            CREATE INDEX IF NOT EXISTS idx_resource_sheets_type ON resource_sheets(resource_type);
            CREATE INDEX IF NOT EXISTS idx_resource_sheets_status ON resource_sheets(status);
            CREATE INDEX IF NOT EXISTS idx_resource_sheets_client ON resource_sheets(client_id);
            CREATE INDEX IF NOT EXISTS idx_resource_sheets_created ON resource_sheets(created_at);
            CREATE INDEX IF NOT EXISTS idx_resource_sheets_tags ON resource_sheets USING GIN(tags);
            CREATE INDEX IF NOT EXISTS idx_resource_sheets_json ON resource_sheets USING GIN(json_data);

            -- Create resource_sheet_execution_logs table for tracking execution
            CREATE TABLE IF NOT EXISTS resource_sheet_execution_logs (
                log_id SERIAL PRIMARY KEY,
                resource_id TEXT NOT NULL,
                execution_id TEXT NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                step TEXT NOT NULL,
                message TEXT NOT NULL,
                log_level TEXT NOT NULL,
                log_data JSONB DEFAULT '{}',
                FOREIGN KEY (resource_id) REFERENCES resource_sheets(resource_id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_execution_logs_resource ON resource_sheet_execution_logs(resource_id);
            CREATE INDEX IF NOT EXISTS idx_execution_logs_execution ON resource_sheet_execution_logs(execution_id);
            CREATE INDEX IF NOT EXISTS idx_execution_logs_timestamp ON resource_sheet_execution_logs(timestamp);

            -- Create resource_sheet_relationships table for orchestration dependencies
            CREATE TABLE IF NOT EXISTS resource_sheet_relationships (
                relationship_id SERIAL PRIMARY KEY,
                parent_resource_id TEXT NOT NULL,
                child_resource_id TEXT NOT NULL,
                relationship_type TEXT NOT NULL, -- 'dependency', 'sub_resource', 'parallel'
                sequence_order INTEGER DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                FOREIGN KEY (parent_resource_id) REFERENCES resource_sheets(resource_id) ON DELETE CASCADE,
                FOREIGN KEY (child_resource_id) REFERENCES resource_sheets(resource_id) ON DELETE CASCADE,
                UNIQUE(parent_resource_id, child_resource_id, relationship_type)
            );

            CREATE INDEX IF NOT EXISTS idx_relationships_parent ON resource_sheet_relationships(parent_resource_id);
            CREATE INDEX IF NOT EXISTS idx_relationships_child ON resource_sheet_relationships(child_resource_id);
        "#;

        sqlx::query(sql)
            .execute(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize resource sheet tables: {}", e))?;

        println!("✅ Resource sheet database tables initialized");
        Ok(())
    }

    /// Create a new resource sheet
    pub async fn create_resource_sheet(
        pool: &DbPool,
        resource_sheet: &dyn ResourceSheet,
        created_by: &str,
    ) -> Result<String> {
        let resource_id = Uuid::new_v4().to_string();
        let resource_type = match resource_sheet.resource_type() {
            ResourceType::Orchestrator => "Orchestrator".to_string(),
            ResourceType::Domain(domain) => format!("Domain_{:?}", domain),
        };

        let status = match resource_sheet.status() {
            ResourceStatus::Pending => "Pending",
            ResourceStatus::Discovering => "Discovering",
            ResourceStatus::Instantiating => "Instantiating",
            ResourceStatus::Executing => "Executing",
            ResourceStatus::Waiting => "Waiting",
            ResourceStatus::Complete => "Complete",
            ResourceStatus::Review => "Review",
            ResourceStatus::Failed(_) => "Failed",
        }.to_string();

        let metadata = resource_sheet.metadata();
        let json_data = Self::serialize_resource_sheet(resource_sheet)?;
        let metadata_json = json!({
            "name": metadata.name,
            "description": metadata.description,
            "version": metadata.version,
            "priority": metadata.priority,
            "estimated_duration_minutes": metadata.estimated_duration_minutes,
            "business_context": metadata.business_context,
        });

        let tags_json = json!(metadata.tags);

        let sql = r#"
            INSERT INTO resource_sheets (
                resource_id, resource_type, name, description, version,
                status, json_data, metadata, created_by, tags
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#;

        sqlx::query(sql)
            .bind(&resource_id)
            .bind(&resource_type)
            .bind(&metadata.name)
            .bind(&metadata.description)
            .bind(&metadata.version)
            .bind(&status)
            .bind(&json_data)
            .bind(&metadata_json)
            .bind(created_by)
            .bind(&tags_json)
            .execute(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create resource sheet: {}", e))?;

        println!("✅ Created resource sheet: {} ({})", metadata.name, resource_id);
        Ok(resource_id)
    }

    /// Get resource sheet by ID
    pub async fn get_resource_sheet(
        pool: &DbPool,
        resource_id: &str,
    ) -> Result<Option<ResourceSheetRecord>> {
        let sql = "SELECT * FROM resource_sheets WHERE resource_id = $1";

        match sqlx::query_as::<_, ResourceSheetRecord>(sql)
            .bind(resource_id)
            .fetch_optional(pool)
            .await
        {
            Ok(record) => Ok(record),
            Err(e) => Err(anyhow::anyhow!("Failed to get resource sheet: {}", e)),
        }
    }

    /// Update resource sheet JSON data
    pub async fn update_resource_sheet_json(
        pool: &DbPool,
        resource_id: &str,
        json_data: &JsonValue,
    ) -> Result<()> {
        let sql = r#"
            UPDATE resource_sheets
            SET json_data = $1, updated_at = NOW()
            WHERE resource_id = $2
        "#;

        let rows_affected = sqlx::query(sql)
            .bind(json_data)
            .bind(resource_id)
            .execute(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update resource sheet: {}", e))?
            .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Resource sheet not found: {}", resource_id));
        }

        println!("✅ Updated resource sheet JSON: {}", resource_id);
        Ok(())
    }

    /// Update resource sheet status
    pub async fn update_resource_sheet_status(
        pool: &DbPool,
        resource_id: &str,
        status: &ResourceStatus,
    ) -> Result<()> {
        let status_str = match status {
            ResourceStatus::Pending => "Pending",
            ResourceStatus::Discovering => "Discovering",
            ResourceStatus::Instantiating => "Instantiating",
            ResourceStatus::Executing => "Executing",
            ResourceStatus::Waiting => "Waiting",
            ResourceStatus::Complete => "Complete",
            ResourceStatus::Review => "Review",
            ResourceStatus::Failed(_) => "Failed",
        };

        let sql = r#"
            UPDATE resource_sheets
            SET status = $1, updated_at = NOW()
            WHERE resource_id = $2
        "#;

        sqlx::query(sql)
            .bind(status_str)
            .bind(resource_id)
            .execute(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update resource sheet status: {}", e))?;

        Ok(())
    }

    /// List resource sheets with optional filters
    pub async fn list_resource_sheets(
        pool: &DbPool,
        resource_type: Option<&str>,
        status: Option<&str>,
        client_id: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<ResourceSheetRecord>> {
        let mut sql = "SELECT * FROM resource_sheets WHERE 1=1".to_string();
        let mut params = Vec::new();

        if let Some(rt) = resource_type {
            params.push(rt);
            sql.push_str(&format!(" AND resource_type = ${}", params.len()));
        }

        if let Some(st) = status {
            params.push(st);
            sql.push_str(&format!(" AND status = ${}", params.len()));
        }

        if let Some(cid) = client_id {
            params.push(cid);
            sql.push_str(&format!(" AND client_id = ${}", params.len()));
        }

        sql.push_str(" ORDER BY created_at DESC");

        if let Some(lim) = limit {
            sql.push_str(&format!(" LIMIT {}", lim));
        }

        let mut query = sqlx::query_as::<_, ResourceSheetRecord>(&sql);
        for param in params {
            query = query.bind(param);
        }

        let records = query
            .fetch_all(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list resource sheets: {}", e))?;

        Ok(records)
    }

    /// Delete resource sheet
    pub async fn delete_resource_sheet(
        pool: &DbPool,
        resource_id: &str,
    ) -> Result<()> {
        let sql = "DELETE FROM resource_sheets WHERE resource_id = $1";

        let rows_affected = sqlx::query(sql)
            .bind(resource_id)
            .execute(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete resource sheet: {}", e))?
            .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Resource sheet not found: {}", resource_id));
        }

        println!("✅ Deleted resource sheet: {}", resource_id);
        Ok(())
    }

    /// Add execution log entry
    pub async fn add_execution_log(
        pool: &DbPool,
        resource_id: &str,
        execution_id: &str,
        log_entry: &ExecutionLogEntry,
    ) -> Result<()> {
        let level_str = match log_entry.level {
            LogLevel::Debug => "Debug",
            LogLevel::Info => "Info",
            LogLevel::Warning => "Warning",
            LogLevel::Error => "Error",
        };

        let sql = r#"
            INSERT INTO resource_sheet_execution_logs (
                resource_id, execution_id, timestamp, step, message, log_level, log_data
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        sqlx::query(sql)
            .bind(resource_id)
            .bind(execution_id)
            .bind(log_entry.timestamp)
            .bind(&log_entry.step)
            .bind(&log_entry.message)
            .bind(level_str)
            .bind(json!(log_entry.data))
            .execute(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add execution log: {}", e))?;

        Ok(())
    }

    /// Get execution logs for a resource
    pub async fn get_execution_logs(
        pool: &DbPool,
        resource_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<ExecutionLogEntry>> {
        let mut sql = r#"
            SELECT timestamp, step, message, log_level, log_data
            FROM resource_sheet_execution_logs
            WHERE resource_id = $1
            ORDER BY timestamp DESC
        "#.to_string();

        if let Some(lim) = limit {
            sql.push_str(&format!(" LIMIT {}", lim));
        }

        let rows = sqlx::query(&sql)
            .bind(resource_id)
            .fetch_all(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get execution logs: {}", e))?;

        let mut logs = Vec::new();
        for row in rows {
            let level = match row.get::<String, _>("log_level").as_str() {
                "Debug" => LogLevel::Debug,
                "Info" => LogLevel::Info,
                "Warning" => LogLevel::Warning,
                "Error" => LogLevel::Error,
                _ => LogLevel::Info,
            };

            let log_data_json: JsonValue = row.get("log_data");
            let log_data = if let JsonValue::Object(map) = log_data_json {
                map.into_iter()
                    .map(|(k, v)| (k, Self::json_to_value(v)))
                    .collect()
            } else {
                HashMap::new()
            };

            logs.push(ExecutionLogEntry {
                timestamp: row.get("timestamp"),
                resource_id: resource_id.to_string(),
                step: row.get("step"),
                message: row.get("message"),
                level,
                data: log_data,
            });
        }

        Ok(logs)
    }

    /// Add resource relationship
    pub async fn add_resource_relationship(
        pool: &DbPool,
        parent_resource_id: &str,
        child_resource_id: &str,
        relationship_type: &str,
        sequence_order: Option<i32>,
    ) -> Result<()> {
        let sql = r#"
            INSERT INTO resource_sheet_relationships (
                parent_resource_id, child_resource_id, relationship_type, sequence_order
            ) VALUES ($1, $2, $3, $4)
            ON CONFLICT (parent_resource_id, child_resource_id, relationship_type)
            DO UPDATE SET sequence_order = $4
        "#;

        sqlx::query(sql)
            .bind(parent_resource_id)
            .bind(child_resource_id)
            .bind(relationship_type)
            .bind(sequence_order.unwrap_or(0))
            .execute(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add resource relationship: {}", e))?;

        Ok(())
    }

    /// Get child resources
    pub async fn get_child_resources(
        pool: &DbPool,
        parent_resource_id: &str,
    ) -> Result<Vec<ResourceSheetRecord>> {
        let sql = r#"
            SELECT rs.* FROM resource_sheets rs
            JOIN resource_sheet_relationships rsr ON rs.resource_id = rsr.child_resource_id
            WHERE rsr.parent_resource_id = $1
            ORDER BY rsr.sequence_order, rs.created_at
        "#;

        let records = sqlx::query_as::<_, ResourceSheetRecord>(sql)
            .bind(parent_resource_id)
            .fetch_all(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get child resources: {}", e))?;

        Ok(records)
    }

    /// Serialize a resource sheet to JSON
    fn serialize_resource_sheet(resource_sheet: &dyn ResourceSheet) -> Result<JsonValue> {
        // For now, create a basic JSON structure
        // In a full implementation, you'd need to implement trait serialization
        let json = json!({
            "id": resource_sheet.id(),
            "resource_type": resource_sheet.resource_type(),
            "status": resource_sheet.status(),
            "metadata": resource_sheet.metadata(),
            "dsl_code": resource_sheet.dsl_code(),
            "dictionary": resource_sheet.dictionary(),
        });
        Ok(json)
    }

    /// Convert JsonValue to our Value enum
    fn json_to_value(json_val: JsonValue) -> crate::models::Value {
        match json_val {
            JsonValue::String(s) => crate::models::Value::String(s),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    crate::models::Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    crate::models::Value::Float(f)
                } else {
                    crate::models::Value::Float(0.0)
                }
            },
            JsonValue::Bool(b) => crate::models::Value::Boolean(b),
            JsonValue::Null => crate::models::Value::Null,
            JsonValue::Array(arr) => {
                let values: Vec<_> = arr.into_iter().map(Self::json_to_value).collect();
                crate::models::Value::List(values)
            },
            JsonValue::Object(_) => crate::models::Value::String(json_val.to_string()),
        }
    }

    /// Create sample KYC resource sheet for testing
    pub async fn create_sample_kyc_resource_sheet(
        pool: &DbPool,
        client_id: &str,
        product_id: &str,
    ) -> Result<String> {
        let resource_id = Uuid::new_v4().to_string();
        let case_id = format!("KYC-{}", &resource_id[..8]);

        let sample_json = json!({
            "id": resource_id,
            "case_id": case_id,
            "client_id": client_id,
            "product_id": product_id,
            "resource_type": "Domain_KYC",
            "status": "Pending",
            "metadata": {
                "name": format!("KYC Case for Client {}", client_id),
                "description": "Know Your Customer compliance check",
                "version": "1.0.0",
                "created_at": Utc::now().to_rfc3339(),
                "priority": "Normal",
                "estimated_duration_minutes": 120,
                "tags": ["KYC", "Compliance", "Onboarding"]
            },
            "dictionary": {
                "data_requirements": [
                    {
                        "field_name": "client.fullName",
                        "data_type": "string",
                        "required": true,
                        "source": "Parent"
                    },
                    {
                        "field_name": "client.dateOfBirth",
                        "data_type": "date",
                        "required": true,
                        "source": "Parent"
                    },
                    {
                        "field_name": "client.nationality",
                        "data_type": "string",
                        "required": true,
                        "source": "Parent"
                    }
                ],
                "validation_rules": [
                    {
                        "rule_name": "AgeValidation",
                        "error_message": "Client must be at least 18 years old",
                        "severity": "Error"
                    }
                ]
            },
            "business_logic_dsl": "WORKFLOW \"StandardClientKYC\"\nSTEP \"InitialRiskAssessment\"\n    DERIVE_REGULATORY_CONTEXT FOR_JURISDICTION \"US\" WITH_PRODUCTS [\"Trading\"]\n    ASSESS_RISK USING_FACTORS [\"jurisdiction\", \"product\", \"client\"] OUTPUT \"combinedRisk\"\nSTEP \"DocumentCollection\"\n    COLLECT_DOCUMENT \"PassportCopy\" FROM Client REQUIRED true\n    COLLECT_DOCUMENT \"ProofOfAddress\" FROM Client REQUIRED true\nSTEP \"Screening\"\n    SCREEN_ENTITY \"client.name\" AGAINST \"SanctionsList\" THRESHOLD 0.85\n    SCREEN_ENTITY \"client.name\" AGAINST \"PEPList\" THRESHOLD 0.90\nSTEP \"Decision\"\n    IF combinedRisk = \"High\" THEN\n        FLAG_FOR_REVIEW \"High risk client requires manual review\" PRIORITY High\n    ELSE\n        APPROVE_CASE WITH_CONDITIONS [\"Annual review required\"]",
            "risk_profile": {
                "jurisdiction_risk": "Medium",
                "product_risk": "Medium",
                "client_risk": "Medium",
                "combined_risk": "Medium",
                "risk_factors": []
            },
            "documents": [
                {
                    "document_type": "PassportCopy",
                    "required": true,
                    "collected": false,
                    "verified": false
                },
                {
                    "document_type": "ProofOfAddress",
                    "required": true,
                    "collected": false,
                    "verified": false
                }
            ],
            "screenings": [],
            "regulatory_context": {
                "applicable_regulations": ["AML", "KYC", "BSA"],
                "jurisdiction": "US",
                "policy_overrides": {},
                "exemptions": []
            },
            "clearance_decision": null
        });

        let sql = r#"
            INSERT INTO resource_sheets (
                resource_id, resource_type, name, description, version,
                client_id, product_id, status, json_data, metadata, created_by, tags
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#;

        sqlx::query(sql)
            .bind(&resource_id)
            .bind("Domain_KYC")
            .bind(format!("KYC Case for Client {}", client_id))
            .bind("Know Your Customer compliance check")
            .bind("1.0.0")
            .bind(client_id)
            .bind(product_id)
            .bind("Pending")
            .bind(&sample_json)
            .bind(json!({"priority": "Normal", "estimated_duration_minutes": 120}))
            .bind("system")
            .bind(json!(["KYC", "Compliance", "Onboarding"]))
            .execute(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create sample KYC resource sheet: {}", e))?;

        println!("✅ Created sample KYC resource sheet: {} ({})", case_id, resource_id);
        Ok(resource_id)
    }

    /// Create sample Onboarding orchestrator resource sheet
    pub async fn create_sample_onboarding_resource_sheet(
        pool: &DbPool,
        client_id: &str,
        products: &[String],
    ) -> Result<String> {
        let resource_id = Uuid::new_v4().to_string();

        let sample_json = json!({
            "id": resource_id,
            "client_id": client_id,
            "products": products,
            "resource_type": "Orchestrator",
            "status": "Pending",
            "metadata": {
                "name": format!("Onboarding for Client {}", client_id),
                "description": "Complete client onboarding orchestration",
                "version": "1.0.0",
                "created_at": Utc::now().to_rfc3339(),
                "priority": "High",
                "estimated_duration_minutes": 240,
                "tags": ["Onboarding", "Orchestration", "Master"]
            },
            "orchestration_dsl": "WORKFLOW \"ClientOnboarding\"\nPHASE \"Discovery\"\n    DISCOVER_DEPENDENCIES FOR_PRODUCTS [\"Trading\", \"Custody\"]\n    BUILD_MASTER_DICTIONARY FROM_RESOURCES [\"ProductCatalog\", \"RegulatoryRules\"]\nPHASE \"ResourceCreation\"\n    INSTANTIATE_RESOURCE \"KYC\" \"ClientKYCClearance\"\n    INSTANTIATE_RESOURCE \"AccountSetup\" \"ClientAccountSetup\"\nPHASE \"Execution\"\n    EXECUTE_RESOURCE_DSL \"ClientKYCClearance\"\n    AWAIT_RESOURCES [\"ClientKYCClearance\"] TO_BE \"Complete\"\n    EXECUTE_RESOURCE_DSL \"ClientAccountSetup\"\nPHASE \"Completion\"\n    VALIDATE_ORCHESTRATION_STATE USING [\"AllResourcesComplete\", \"NoErrors\"]\n    DERIVE_GLOBAL_STATE FROM_RESOURCES [\"ClientKYCClearance\", \"ClientAccountSetup\"]",
            "sub_resources": {},
            "execution_plan": {
                "phases": [
                    {
                        "name": "Discovery",
                        "description": "Discover required resources and build master data dictionary",
                        "resources": [],
                        "blocking": true,
                        "timeout_minutes": 30
                    },
                    {
                        "name": "ResourceCreation",
                        "description": "Instantiate required domain resources",
                        "resources": [],
                        "blocking": true,
                        "timeout_minutes": 15
                    },
                    {
                        "name": "Execution",
                        "description": "Execute domain resources in sequence",
                        "resources": ["ClientKYCClearance", "ClientAccountSetup"],
                        "blocking": true,
                        "timeout_minutes": 180
                    }
                ],
                "current_phase": 0,
                "parallel_execution": false,
                "failure_strategy": "RequireManualReview"
            },
            "master_data": {
                "client_profile": {
                    "client_id": client_id,
                    "basic_info": {},
                    "computed_attributes": {},
                    "risk_indicators": [],
                    "relationships": []
                },
                "product_catalog": {},
                "regulatory_requirements": {},
                "business_rules": {},
                "lookup_tables": {}
            }
        });

        let sql = r#"
            INSERT INTO resource_sheets (
                resource_id, resource_type, name, description, version,
                client_id, status, json_data, metadata, created_by, tags
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#;

        sqlx::query(sql)
            .bind(&resource_id)
            .bind("Orchestrator")
            .bind(format!("Onboarding for Client {}", client_id))
            .bind("Complete client onboarding orchestration")
            .bind("1.0.0")
            .bind(client_id)
            .bind("Pending")
            .bind(&sample_json)
            .bind(json!({"priority": "High", "estimated_duration_minutes": 240}))
            .bind("system")
            .bind(json!(["Onboarding", "Orchestration", "Master"]))
            .execute(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create sample onboarding resource sheet: {}", e))?;

        println!("✅ Created sample onboarding resource sheet: {} ({})",
                format!("Onboarding for Client {}", client_id), resource_id);
        Ok(resource_id)
    }
}