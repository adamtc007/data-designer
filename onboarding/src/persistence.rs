//! Database persistence layer for onboarding workflow instances
//!
//! Provides CRUD operations for:
//! - OnboardingInstance lifecycle management
//! - Plan persistence and retrieval
//! - Event sourcing support

use crate::{OnboardingInstance, Plan, Idd, Bindings, InstanceState};
use anyhow::Result;

#[cfg(feature = "sqlx")]
use sqlx::PgPool;

/// Database repository for onboarding instances
pub struct OnboardingRepository {
    #[cfg(feature = "sqlx")]
    pool: PgPool,
}

impl OnboardingRepository {
    #[cfg(feature = "sqlx")]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[cfg(not(feature = "sqlx"))]
    pub fn new() -> Self {
        Self {}
    }

    /// Create a new onboarding instance
    pub async fn create_instance(&self, instance: &OnboardingInstance) -> Result<()> {
        #[cfg(feature = "sqlx")]
        {
            sqlx::query!(
                r#"
                INSERT INTO onboarding_instance (instance_id, state, cbu_id, products, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                instance.id,
                instance.state.to_string(),
                instance.cbu_id,
                &instance.products,
                instance.created_at,
                instance.updated_at
            )
            .execute(&self.pool)
            .await?;
        }

        #[cfg(not(feature = "sqlx"))]
        {
            // Mock implementation for non-database builds
            tracing::info!("Mock: Creating instance {}", instance.id);
        }

        Ok(())
    }

    /// Load instance by ID
    pub async fn get_instance(&self, instance_id: &str) -> Result<Option<OnboardingInstance>> {
        #[cfg(feature = "sqlx")]
        {
            let row = sqlx::query!(
                "SELECT instance_id, state, cbu_id, products, created_at, updated_at FROM onboarding_instance WHERE instance_id = $1",
                instance_id
            )
            .fetch_optional(&self.pool)
            .await?;

            if let Some(row) = row {
                Ok(Some(OnboardingInstance {
                    id: row.instance_id,
                    state: row.state.parse().unwrap_or(InstanceState::Draft),
                    cbu_id: row.cbu_id,
                    products: row.products.unwrap_or_default(),
                    created_at: row.created_at.unwrap_or_else(|| chrono::Utc::now()),
                    updated_at: row.updated_at.unwrap_or_else(|| chrono::Utc::now()),
                }))
            } else {
                Ok(None)
            }
        }

        #[cfg(not(feature = "sqlx"))]
        {
            tracing::info!("Mock: Getting instance {}", instance_id);
            Ok(None)
        }
    }

    /// Update instance state and metadata
    pub async fn update_instance(&self, instance: &OnboardingInstance) -> Result<()> {
        #[cfg(feature = "sqlx")]
        {
            sqlx::query!(
                r#"
                UPDATE onboarding_instance
                SET state = $2, cbu_id = $3, products = $4, updated_at = $5
                WHERE instance_id = $1
                "#,
                instance.id,
                instance.state.to_string(),
                instance.cbu_id,
                &instance.products,
                instance.updated_at
            )
            .execute(&self.pool)
            .await?;
        }

        #[cfg(not(feature = "sqlx"))]
        {
            tracing::info!("Mock: Updating instance {}", instance.id);
        }

        Ok(())
    }

    /// Save compiled plan for instance
    pub async fn save_plan(&self, instance_id: &str, _plan: &Plan, _idd: &Idd, _bindings: &Bindings) -> Result<()> {
        #[cfg(feature = "sqlx")]
        {
            sqlx::query!(
                r#"
                INSERT INTO instance_plan (instance_id, plan_json, idd_json, bindings_json, created_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (instance_id) DO UPDATE SET
                    plan_json = EXCLUDED.plan_json,
                    idd_json = EXCLUDED.idd_json,
                    bindings_json = EXCLUDED.bindings_json,
                    created_at = EXCLUDED.created_at
                "#,
                instance_id,
                serde_json::to_value(_plan)?,
                serde_json::to_value(_idd)?,
                serde_json::to_value(_bindings)?,
                chrono::Utc::now()
            )
            .execute(&self.pool)
            .await?;
        }

        #[cfg(not(feature = "sqlx"))]
        {
            tracing::info!("Mock: Saving plan for instance {}", instance_id);
        }

        Ok(())
    }
}

impl std::fmt::Display for InstanceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceState::Draft => write!(f, "Draft"),
            InstanceState::ReadyToCompile => write!(f, "ReadyToCompile"),
            InstanceState::Compiled => write!(f, "Compiled"),
            InstanceState::Executing => write!(f, "Executing"),
            InstanceState::Completed => write!(f, "Completed"),
            InstanceState::Failed => write!(f, "Failed"),
        }
    }
}

impl Default for OnboardingRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for InstanceState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Draft" => Ok(InstanceState::Draft),
            "ReadyToCompile" => Ok(InstanceState::ReadyToCompile),
            "Compiled" => Ok(InstanceState::Compiled),
            "Executing" => Ok(InstanceState::Executing),
            "Completed" => Ok(InstanceState::Completed),
            "Failed" => Ok(InstanceState::Failed),
            _ => Err(anyhow::anyhow!("Invalid instance state: {}", s)),
        }
    }
}