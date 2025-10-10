use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::db::{DbPool, EmbeddingOperations, SimilarRule};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct OpenAIEmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct OpenAIEmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct AnthropicEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct AnthropicEmbeddingResponse {
    embeddings: Vec<Vec<f32>>,
}

// === WRAPPER FUNCTIONS THAT DELEGATE TO CENTRALIZED OPERATIONS ===

/// Generate embedding for a text string (placeholder implementation)
pub async fn generate_embedding(text: &str, api_key: Option<&str>) -> Result<Vec<f32>> {
    // Placeholder implementation - would use actual OpenAI/Anthropic API
    let _ = api_key; // Suppress unused warning

    // Generate a simple hash-based embedding for now
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();

    // Convert to 1536-dimensional vector (OpenAI embedding size)
    let mut embedding = vec![0.0f32; 1536];
    for i in 0..1536 {
        embedding[i] = ((hash.wrapping_mul(i as u64)) as f32) / u64::MAX as f32;
    }

    Ok(embedding)
}

/// Update embedding for a rule
pub async fn update_rule_embedding(
    pool: &DbPool,
    rule_id: &str,
    dsl_text: &str,
) -> Result<()> {
    // Use centralized operations
    EmbeddingOperations::update_rule_embedding(pool, rule_id, dsl_text)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to update rule embedding: {}", e))?;
    Ok(())
}

/// Find similar rules using vector similarity
pub async fn find_similar_rules(
    pool: &DbPool,
    dsl_text: &str,
    limit: i32,
) -> Result<Vec<SimilarRule>> {
    // Use centralized operations
    EmbeddingOperations::find_similar_rules(pool, dsl_text, limit)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to find similar rules: {}", e))
}

/// Generate embeddings for all rules (batch operation)
pub async fn generate_all_embeddings(pool: &DbPool) -> Result<()> {
    // Use centralized operations
    EmbeddingOperations::generate_all_embeddings(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to generate all embeddings: {}", e))?;
    Ok(())
}

/// Search for similar rules by semantic similarity
pub async fn semantic_search_rules(
    pool: &DbPool,
    query_text: &str,
    limit: Option<i32>,
) -> Result<Vec<SimilarRule>> {
    let search_limit = limit.unwrap_or(10);
    find_similar_rules(pool, query_text, search_limit).await
}

/// Update embeddings for multiple rules in batch
pub async fn batch_update_embeddings(
    pool: &DbPool,
    rule_updates: Vec<(String, String)>, // (rule_id, dsl_text) pairs
) -> Result<()> {
    for (rule_id, dsl_text) in rule_updates {
        update_rule_embedding(pool, &rule_id, &dsl_text).await?;
    }
    Ok(())
}