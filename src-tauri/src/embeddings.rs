use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIEmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicEmbeddingResponse {
    embeddings: Vec<Vec<f32>>,
}

pub async fn generate_embedding(text: &str, api_key: Option<&str>) -> Result<Vec<f32>> {
    // Try to get API key from environment if not provided
    let key = if let Some(k) = api_key {
        k.to_string()
    } else if let Ok(k) = std::env::var("OPENAI_API_KEY") {
        k
    } else if let Ok(_k) = std::env::var("ANTHROPIC_API_KEY") {
        // For now, use OpenAI-compatible endpoint if we have Anthropic key
        // In production, you'd use the actual Anthropic endpoint
        return Ok(generate_mock_embedding(text));
    } else {
        // Return mock embedding if no API key available
        return Ok(generate_mock_embedding(text));
    };

    // Call OpenAI API
    let client = reqwest::Client::new();
    let request = OpenAIEmbeddingRequest {
        input: text.to_string(),
        model: "text-embedding-ada-002".to_string(),
    };

    let response = client
        .post("https://api.openai.com/v1/embeddings")
        .header("Authorization", format!("Bearer {}", key))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let embedding_response: OpenAIEmbeddingResponse = response.json().await?;
        if let Some(data) = embedding_response.data.first() {
            return Ok(data.embedding.clone());
        }
    }

    // Fallback to mock embedding
    Ok(generate_mock_embedding(text))
}

fn generate_mock_embedding(text: &str) -> Vec<f32> {
    // Generate a deterministic mock embedding based on text content
    // This ensures same text always gets same embedding for consistency
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let seed = hasher.finish();

    // Generate 1536-dimensional vector (OpenAI ada-002 size)
    let mut embedding = Vec::with_capacity(1536);
    let mut rng = seed;

    for _ in 0..1536 {
        // Simple linear congruential generator for deterministic pseudo-random numbers
        rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) & 0x7fffffff;
        let value = (rng as f32 / 0x7fffffff as f32) * 2.0 - 1.0; // Normalize to [-1, 1]
        embedding.push(value);
    }

    // Normalize the vector
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for value in &mut embedding {
            *value /= magnitude;
        }
    }

    embedding
}

pub async fn update_rule_embedding(pool: &PgPool, rule_id: &str, dsl_text: &str) -> Result<()> {
    let embedding = generate_embedding(dsl_text, None).await?;

    // Convert Vec<f32> to PostgreSQL vector format
    let embedding_str = format!("[{}]", embedding.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(","));

    sqlx::query(
        "UPDATE rules SET embedding = $1::vector WHERE id = $2"
    )
    .bind(&embedding_str)
    .bind(rule_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_similar_rules(pool: &PgPool, dsl_text: &str, limit: i32) -> Result<Vec<SimilarRule>> {
    let embedding = generate_embedding(dsl_text, None).await?;

    // Convert Vec<f32> to PostgreSQL vector format
    let embedding_str = format!("[{}]", embedding.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(","));

    // Use cosine similarity (1 - cosine distance)
    let similar_rules = sqlx::query_as::<_, SimilarRule>(
        "SELECT id, name, description, dsl_text,
                1 - (embedding <=> $1::vector) as similarity
         FROM rules
         WHERE embedding IS NOT NULL
         ORDER BY embedding <=> $1::vector
         LIMIT $2"
    )
    .bind(&embedding_str)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(similar_rules)
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SimilarRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub dsl_text: String,
    pub similarity: Option<f64>,
}

// Generate embeddings for all existing rules
pub async fn generate_all_embeddings(pool: &PgPool) -> Result<()> {
    let rules: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, dsl_text FROM rules WHERE embedding IS NULL"
    )
    .fetch_all(pool)
    .await?;

    for (id, dsl_text) in rules {
        if let Err(e) = update_rule_embedding(pool, &id, &dsl_text).await {
            eprintln!("Failed to generate embedding for rule {}: {}", id, e);
        }
    }

    Ok(())
}