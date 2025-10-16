use axum::{
    body::Body,
    extract::{Request, MatchedPath},
    http::{StatusCode, HeaderMap, Method, Uri},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Utc};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Instant;
use tracing::{info, error, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub matched_path: Option<String>,
    pub query_string: Option<String>,
    pub status_code: u16,
    pub duration_ms: u64,
    pub request_headers: Value,
    pub response_headers: Value,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub service: String,
    pub version: String,
}

#[derive(Clone)]
pub struct ElasticsearchLogger {
    client: reqwest::Client,
    elasticsearch_url: String,
    index: String,
    buffer: std::sync::Arc<tokio::sync::Mutex<Vec<ApiLogEntry>>>,
    batch_size: usize,
}

impl ElasticsearchLogger {
    pub async fn new(elasticsearch_url: &str, index: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        // Test connection
        let health_url = format!("{}/_cluster/health", elasticsearch_url);
        match client.get(&health_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("✅ Connected to Elasticsearch at {}", elasticsearch_url);
                } else {
                    warn!("⚠️  Elasticsearch health check failed with status: {}", response.status());
                }
            },
            Err(e) => {
                warn!("⚠️  Elasticsearch connection failed: {}. Logging will continue with fallback to console.", e);
            }
        }

        // Create index mapping for API logs
        let mapping = json!({
            "mappings": {
                "properties": {
                    "id": { "type": "keyword" },
                    "timestamp": { "type": "date" },
                    "method": { "type": "keyword" },
                    "path": { "type": "text", "fields": { "keyword": { "type": "keyword" } } },
                    "matched_path": { "type": "keyword" },
                    "query_string": { "type": "text" },
                    "status_code": { "type": "integer" },
                    "duration_ms": { "type": "long" },
                    "request_headers": { "type": "object", "enabled": false },
                    "response_headers": { "type": "object", "enabled": false },
                    "request_body": { "type": "text" },
                    "response_body": { "type": "text" },
                    "client_ip": { "type": "ip" },
                    "user_agent": { "type": "text", "fields": { "keyword": { "type": "keyword" } } },
                    "service": { "type": "keyword" },
                    "version": { "type": "keyword" }
                }
            },
            "settings": {
                "number_of_shards": 1,
                "number_of_replicas": 0
            }
        });

        // Create index if it doesn't exist
        let index_url = format!("{}/{}", elasticsearch_url, index);
        match client.put(&index_url).json(&mapping).send().await {
            Ok(response) => {
                if response.status().is_success() || response.status() == reqwest::StatusCode::BAD_REQUEST {
                    info!("📊 Index '{}' created or already exists", index);
                } else {
                    warn!("⚠️  Failed to create index: {}", response.status());
                }
            },
            Err(e) => {
                warn!("⚠️  Index creation failed: {}", e);
            }
        }

        Ok(Self {
            client,
            elasticsearch_url: elasticsearch_url.to_string(),
            index: index.to_string(),
            buffer: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
            batch_size: 10, // Batch every 10 requests
        })
    }

    pub async fn log_api_call(&self, log_entry: ApiLogEntry) {
        // Log to console first
        info!(
            "📡 API Call: {} {} -> {} ({} ms)",
            log_entry.method,
            log_entry.path,
            log_entry.status_code,
            log_entry.duration_ms
        );

        // Add to buffer for batch processing
        {
            let mut buffer = self.buffer.lock().await;
            buffer.push(log_entry);

            // Flush buffer if it reaches batch size
            if buffer.len() >= self.batch_size {
                let entries = buffer.drain(..).collect::<Vec<_>>();
                let client = self.client.clone();
                let index = self.index.clone();

                // Send to Elasticsearch in background
                let elasticsearch_url = self.elasticsearch_url.clone();
                tokio::spawn(async move {
                    Self::send_batch_to_elasticsearch(&client, &elasticsearch_url, &index, entries).await;
                });
            }
        }
    }

    async fn send_batch_to_elasticsearch(
        client: &reqwest::Client,
        elasticsearch_url: &str,
        index: &str,
        entries: Vec<ApiLogEntry>,
    ) {
        if entries.is_empty() {
            return;
        }

        let mut body = String::new();
        for entry in &entries {
            // Add index action line
            let index_line = json!({
                "index": {
                    "_index": index,
                    "_id": entry.id
                }
            });
            body.push_str(&serde_json::to_string(&index_line).unwrap_or_default());
            body.push('\n');

            // Add document line
            body.push_str(&serde_json::to_string(entry).unwrap_or_default());
            body.push('\n');
        }

        let bulk_url = format!("{}/_bulk", elasticsearch_url);
        match client
            .post(&bulk_url)
            .header("Content-Type", "application/x-ndjson")
            .body(body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("📊 Logged {} API calls to Elasticsearch", entries.len());
                } else {
                    error!("❌ Failed to log to Elasticsearch: {}", response.status());
                }
            }
            Err(e) => {
                error!("❌ Elasticsearch logging error: {}", e);
                // Fallback: log to console
                for entry in entries {
                    error!("📋 Fallback log: {:?}", entry);
                }
            }
        }
    }

    // Manual flush for shutdown
    pub async fn flush(&self) {
        let mut buffer = self.buffer.lock().await;
        if !buffer.is_empty() {
            let entries = buffer.drain(..).collect::<Vec<_>>();
            Self::send_batch_to_elasticsearch(&self.client, &self.elasticsearch_url, &self.index, entries).await;
        }
    }
}

// Middleware to capture and log API requests/responses
pub async fn api_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();

    // Extract request information
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    let query_string = uri.query().map(|q| q.to_string());
    let matched_path = request.extensions().get::<MatchedPath>()
        .map(|mp| mp.as_str().to_string());

    // Extract headers (be careful with sensitive data)
    let request_headers = headers_to_json(request.headers());
    let client_ip = extract_client_ip(request.headers());
    let user_agent = request.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // For now, we'll skip request body logging to avoid complexity
    // In production, you might want to selectively log request bodies

    // Process the request
    let response = next.run(request).await;

    // Calculate duration
    let duration = start_time.elapsed();

    // Extract response information
    let status_code = response.status().as_u16();
    let response_headers = headers_to_json(response.headers());

    // Create log entry
    let log_entry = ApiLogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        method: method.to_string(),
        path,
        matched_path,
        query_string,
        status_code,
        duration_ms: duration.as_millis() as u64,
        request_headers,
        response_headers,
        request_body: None, // Skip for now
        response_body: None, // Skip for now
        client_ip,
        user_agent,
        service: "template-server".to_string(),
        version: "0.1.0".to_string(),
    };

    // For now, just log to console and attempt Elasticsearch
    info!(
        "📡 API Call: {} {} -> {} ({} ms)",
        method,
        uri.path(),
        status_code,
        duration.as_millis()
    );

    // Send to Elasticsearch in background
    let log_entry_json = serde_json::to_string(&log_entry).unwrap_or_default();
    info!("📊 Detailed Log: {}", log_entry_json);

    // Send to Elasticsearch asynchronously
    tokio::spawn(async move {
        send_to_elasticsearch(log_entry).await;
    });

    response
}

// Simple function to send logs to Elasticsearch
async fn send_to_elasticsearch(log_entry: ApiLogEntry) {
    let client = reqwest::Client::new();
    let elasticsearch_url = "http://localhost:9200";
    let index = "api-logs";

    // Create document URL with the log entry ID
    let doc_url = format!("{}/{}/_doc/{}", elasticsearch_url, index, log_entry.id);

    match client.put(&doc_url).json(&log_entry).send().await {
        Ok(response) => {
            if response.status().is_success() {
                info!("✅ Logged API call to Elasticsearch: {}", log_entry.id);
            } else {
                error!("❌ Failed to log to Elasticsearch: {} - {}", response.status(), log_entry.id);
            }
        }
        Err(e) => {
            error!("❌ Elasticsearch logging error: {} - {}", e, log_entry.id);
        }
    }
}

fn headers_to_json(headers: &HeaderMap) -> Value {
    let mut map = serde_json::Map::new();

    for (name, value) in headers.iter() {
        // Skip sensitive headers
        let name_str = name.as_str();
        if name_str.to_lowercase().contains("authorization")
            || name_str.to_lowercase().contains("cookie")
            || name_str.to_lowercase().contains("token") {
            continue;
        }

        if let Ok(value_str) = value.to_str() {
            map.insert(name_str.to_string(), Value::String(value_str.to_string()));
        }
    }

    Value::Object(map)
}

fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Try various headers that might contain the real client IP
    let ip_headers = ["x-forwarded-for", "x-real-ip", "cf-connecting-ip", "x-client-ip"];

    for header_name in &ip_headers {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(ip_str) = header_value.to_str() {
                // Take the first IP if there are multiple (comma-separated)
                let first_ip = ip_str.split(',').next().unwrap_or("").trim();
                if !first_ip.is_empty() {
                    return Some(first_ip.to_string());
                }
            }
        }
    }

    None
}

// Extension trait to make the logger available in middleware
pub trait RequestLoggerExt {
    fn with_logger(self, logger: ElasticsearchLogger) -> Self;
}

impl RequestLoggerExt for Request {
    fn with_logger(mut self, logger: ElasticsearchLogger) -> Self {
        self.extensions_mut().insert(logger);
        self
    }
}