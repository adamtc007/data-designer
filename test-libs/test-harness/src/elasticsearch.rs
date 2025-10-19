use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use elasticsearch::{Elasticsearch, http::transport::Transport, IndexParts, SearchParts};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{TestEvent, TestMetrics, ComponentMetrics, trace::RequestTrace};

/// Elasticsearch client for test logging and debugging
pub struct ElasticsearchTestClient {
    client: Elasticsearch,
    test_run_id: String,
    index_prefix: String,
}

impl ElasticsearchTestClient {
    /// Create new Elasticsearch test client
    pub async fn new(test_run_id: &str) -> Result<Self> {
        let transport = Transport::single_node("http://localhost:9200")?;
        let client = Elasticsearch::new(transport);

        let index_prefix = format!("test-logs-{}", chrono::Utc::now().format("%Y-%m"));

        let instance = Self {
            client,
            test_run_id: test_run_id.to_string(),
            index_prefix,
        };

        // Ensure index exists with proper mapping
        instance.ensure_index_exists().await?;

        Ok(instance)
    }

    /// Ensure the test index exists with proper mapping
    async fn ensure_index_exists(&self) -> Result<()> {
        let index_name = &self.index_prefix;

        // Create index if it doesn't exist
        let mapping = json!({
            "mappings": {
                "properties": {
                    "timestamp": { "type": "date" },
                    "test_run_id": { "type": "keyword" },
                    "trace_id": { "type": "keyword" },
                    "test_name": { "type": "keyword" },
                    "component": { "type": "keyword" },
                    "event_type": { "type": "keyword" },
                    "level": { "type": "keyword" },
                    "message": { "type": "text" },
                    "data": { "type": "object" },
                    "duration_ms": { "type": "long" }
                }
            }
        });

        // Check if index exists
        let exists_response = self.client
            .indices()
            .exists(elasticsearch::indices::IndicesExistsParts::Index(&[index_name]))
            .send()
            .await?;

        if exists_response.status_code() == 404 {
            // Index doesn't exist, create it
            let create_response = self.client
                .indices()
                .create(elasticsearch::indices::IndicesCreateParts::Index(index_name))
                .body(mapping)
                .send()
                .await?;

            if !create_response.status_code().is_success() {
                return Err(anyhow::anyhow!("Failed to create Elasticsearch index: {}", index_name));
            }

            tracing::info!("Created Elasticsearch index: {}", index_name);
        }

        Ok(())
    }

    /// Log a test event
    pub async fn log_event(&self, event: TestEvent) -> Result<()> {
        let index_name = &self.index_prefix;

        let response = self.client
            .index(IndexParts::Index(index_name))
            .body(&event)
            .send()
            .await
            .context("Failed to index test event")?;

        if !response.status_code().is_success() {
            return Err(anyhow::anyhow!("Failed to log event to Elasticsearch"));
        }

        Ok(())
    }

    /// Get all events for a specific trace
    pub async fn get_request_trace(&self, trace_id: &str) -> Result<RequestTrace> {
        let query = json!({
            "query": {
                "bool": {
                    "must": [
                        { "term": { "trace_id": trace_id } },
                        { "term": { "test_run_id": self.test_run_id } }
                    ]
                }
            },
            "sort": [
                { "timestamp": { "order": "asc" } }
            ],
            "size": 1000
        });

        let response = self.client
            .search(SearchParts::Index(&[&self.index_prefix]))
            .body(query)
            .send()
            .await
            .context("Failed to search for trace events")?;

        let response_body: Value = response.json().await?;
        let hits = response_body["hits"]["hits"].as_array()
            .context("Invalid Elasticsearch response")?;

        let mut events = Vec::new();
        for hit in hits {
            let event: TestEvent = serde_json::from_value(hit["_source"].clone())?;
            events.push(event);
        }

        Ok(RequestTrace::new(trace_id.to_string(), events))
    }

    /// Get events for a specific component
    pub async fn get_component_events(&self, component: &str) -> Result<Vec<TestEvent>> {
        let query = json!({
            "query": {
                "bool": {
                    "must": [
                        { "term": { "component": component } },
                        { "term": { "test_run_id": self.test_run_id } }
                    ]
                }
            },
            "sort": [
                { "timestamp": { "order": "asc" } }
            ],
            "size": 1000
        });

        let response = self.client
            .search(SearchParts::Index(&[&self.index_prefix]))
            .body(query)
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        let hits = response_body["hits"]["hits"].as_array()
            .context("Invalid Elasticsearch response")?;

        let mut events = Vec::new();
        for hit in hits {
            let event: TestEvent = serde_json::from_value(hit["_source"].clone())?;
            events.push(event);
        }

        Ok(events)
    }

    /// Get error events for a component
    pub async fn get_error_logs_for_component(&self, component: &str) -> Result<Vec<TestEvent>> {
        let query = json!({
            "query": {
                "bool": {
                    "must": [
                        { "term": { "component": component } },
                        { "term": { "level": "Error" } },
                        { "term": { "test_run_id": self.test_run_id } }
                    ]
                }
            },
            "sort": [
                { "timestamp": { "order": "asc" } }
            ],
            "size": 100
        });

        let response = self.client
            .search(SearchParts::Index(&[&self.index_prefix]))
            .body(query)
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        let hits = response_body["hits"]["hits"].as_array()
            .context("Invalid Elasticsearch response")?;

        let mut events = Vec::new();
        for hit in hits {
            let event: TestEvent = serde_json::from_value(hit["_source"].clone())?;
            events.push(event);
        }

        Ok(events)
    }

    /// Get test metrics for the current test run
    pub async fn get_test_metrics(&self, test_run_id: &str) -> Result<TestMetrics> {
        // Aggregation query to get metrics by component
        let query = json!({
            "query": {
                "term": { "test_run_id": test_run_id }
            },
            "aggs": {
                "components": {
                    "terms": { "field": "component" },
                    "aggs": {
                        "avg_duration": {
                            "avg": { "field": "duration_ms" }
                        },
                        "max_duration": {
                            "max": { "field": "duration_ms" }
                        },
                        "error_count": {
                            "filter": {
                                "term": { "level": "Error" }
                            }
                        }
                    }
                },
                "test_summary": {
                    "terms": { "field": "event_type" }
                }
            },
            "size": 0
        });

        let response = self.client
            .search(SearchParts::Index(&[&self.index_prefix]))
            .body(query)
            .send()
            .await?;

        let response_body: Value = response.json().await?;

        // Parse aggregation results
        let mut component_metrics = HashMap::new();
        if let Some(components) = response_body["aggregations"]["components"]["buckets"].as_array() {
            for bucket in components {
                let component = bucket["key"].as_str().unwrap_or("unknown").to_string();
                let total_calls = bucket["doc_count"].as_u64().unwrap_or(0) as usize;
                let avg_duration_ms = bucket["avg_duration"]["value"].as_f64().unwrap_or(0.0);
                let slowest_operation_ms = bucket["max_duration"]["value"].as_u64().unwrap_or(0);
                let error_count = bucket["error_count"]["doc_count"].as_u64().unwrap_or(0) as usize;

                component_metrics.insert(component.clone(), ComponentMetrics {
                    component,
                    total_calls,
                    avg_duration_ms,
                    error_count,
                    slowest_operation_ms,
                });
            }
        }

        // Count test results
        let mut total_tests = 0;
        let mut passed_tests = 0;
        let mut failed_tests = 0;

        if let Some(test_summary) = response_body["aggregations"]["test_summary"]["buckets"].as_array() {
            for bucket in test_summary {
                if let Some(event_type) = bucket["key"].as_str() {
                    let count = bucket["doc_count"].as_u64().unwrap_or(0) as usize;
                    match event_type {
                        "TestStart" => total_tests = count,
                        "TestEnd" => passed_tests = count,
                        "Error" => failed_tests += count,
                        _ => {}
                    }
                }
            }
        }

        Ok(TestMetrics {
            test_run_id: test_run_id.to_string(),
            total_tests,
            passed_tests,
            failed_tests,
            total_duration: chrono::Duration::seconds(0), // TODO: Calculate from first/last events
            component_metrics,
        })
    }

    /// Search logs with custom query
    pub async fn search_logs(&self, query: &str) -> Result<Vec<TestEvent>> {
        let search_query = json!({
            "query": {
                "bool": {
                    "must": [
                        {
                            "query_string": {
                                "query": query
                            }
                        },
                        { "term": { "test_run_id": self.test_run_id } }
                    ]
                }
            },
            "sort": [
                { "timestamp": { "order": "asc" } }
            ],
            "size": 1000
        });

        let response = self.client
            .search(SearchParts::Index(&[&self.index_prefix]))
            .body(search_query)
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        let hits = response_body["hits"]["hits"].as_array()
            .context("Invalid Elasticsearch response")?;

        let mut events = Vec::new();
        for hit in hits {
            let event: TestEvent = serde_json::from_value(hit["_source"].clone())?;
            events.push(event);
        }

        Ok(events)
    }

    /// Clean up test indices
    pub async fn cleanup(&self) -> Result<()> {
        // Delete test run specific documents
        let delete_query = json!({
            "query": {
                "term": { "test_run_id": self.test_run_id }
            }
        });

        let response = self.client
            .delete_by_query(elasticsearch::DeleteByQueryParts::Index(&[&self.index_prefix]))
            .body(delete_query)
            .send()
            .await?;

        if response.status_code().is_success() {
            tracing::info!("Cleaned up Elasticsearch logs for test run: {}", self.test_run_id);
        }

        Ok(())
    }

    /// Get logs for a trace with real-time streaming
    pub async fn stream_logs_for_trace(&self, trace_id: &str) -> Result<tokio::sync::mpsc::Receiver<TestEvent>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // This would typically use Elasticsearch's scroll API or real-time features
        // For now, we'll implement a simple polling mechanism
        let client = self.client.clone();
        let index_prefix = self.index_prefix.clone();
        let test_run_id = self.test_run_id.clone();
        let trace_id = trace_id.to_string();

        tokio::spawn(async move {
            let mut last_timestamp = chrono::Utc::now() - chrono::Duration::minutes(1);

            loop {
                let query = json!({
                    "query": {
                        "bool": {
                            "must": [
                                { "term": { "trace_id": trace_id } },
                                { "term": { "test_run_id": test_run_id } },
                                { "range": { "timestamp": { "gt": last_timestamp.to_rfc3339() } } }
                            ]
                        }
                    },
                    "sort": [
                        { "timestamp": { "order": "asc" } }
                    ],
                    "size": 100
                });

                if let Ok(response) = client
                    .search(SearchParts::Index(&[&index_prefix]))
                    .body(query)
                    .send()
                    .await
                {
                    if let Ok(response_body) = response.json::<Value>().await {
                        if let Some(hits) = response_body["hits"]["hits"].as_array() {
                            for hit in hits {
                                if let Ok(event) = serde_json::from_value::<TestEvent>(hit["_source"].clone()) {
                                    last_timestamp = event.timestamp;
                                    if tx.send(event).await.is_err() {
                                        // Receiver dropped, exit
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        Ok(rx)
    }
}