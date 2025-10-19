use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::TestEvent;

/// Unique identifier for tracing a request through the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceId {
    pub id: String,
    pub test_name: String,
    pub test_run_id: String,
    pub created_at: DateTime<Utc>,
}

impl TraceId {
    /// Create a new trace ID
    pub fn new(test_name: &str, test_run_id: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            test_name: test_name.to_string(),
            test_run_id: test_run_id.to_string(),
            created_at: Utc::now(),
        }
    }

    /// Get the trace ID as a string
    pub fn as_str(&self) -> &str {
        &self.id
    }
}

/// Complete trace of a request through the system
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestTrace {
    pub trace_id: String,
    pub events: Vec<TestEvent>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl RequestTrace {
    /// Create a new request trace
    pub fn new(trace_id: String, mut events: Vec<TestEvent>) -> Self {
        // Sort events by timestamp
        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let started_at = events.first().map(|e| e.timestamp);
        let completed_at = events.last().map(|e| e.timestamp);

        Self {
            trace_id,
            events,
            started_at,
            completed_at,
        }
    }

    /// Get the total duration of the trace
    pub fn total_duration(&self) -> chrono::Duration {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => end - start,
            _ => chrono::Duration::zero(),
        }
    }

    /// Get events for a specific component
    pub fn events_for_component(&self, component: &str) -> Vec<&TestEvent> {
        self.events.iter()
            .filter(|e| e.component == component)
            .collect()
    }

    /// Get all error events in the trace
    pub fn error_events(&self) -> Vec<&TestEvent> {
        self.events.iter()
            .filter(|e| matches!(e.level, crate::LogLevel::Error))
            .collect()
    }

    /// Get the data flow through components
    pub fn component_flow(&self) -> Vec<ComponentStep> {
        let mut flow = Vec::new();
        let mut current_component = None;
        let mut step_start = None;

        for event in &self.events {
            if Some(&event.component) != current_component.as_ref() {
                // Component changed, record the previous step
                if let (Some(comp), Some(start)) = (current_component.take(), step_start.take()) {
                    flow.push(ComponentStep {
                        component: comp,
                        started_at: start,
                        completed_at: event.timestamp,
                        duration: event.timestamp - start,
                        event_count: 1,
                    });
                }

                // Start new step
                current_component = Some(event.component.clone());
                step_start = Some(event.timestamp);
            }
        }

        // Record the final step
        if let (Some(comp), Some(start)) = (current_component, step_start) {
            let end = self.completed_at.unwrap_or(Utc::now());
            flow.push(ComponentStep {
                component: comp,
                started_at: start,
                completed_at: end,
                duration: end - start,
                event_count: 1,
            });
        }

        flow
    }

    /// Get performance summary
    pub fn performance_summary(&self) -> PerformanceSummary {
        let component_flow = self.component_flow();
        let total_duration = self.total_duration();

        let slowest_component = component_flow.iter()
            .max_by_key(|step| step.duration.num_milliseconds())
            .map(|step| step.component.clone());

        let component_breakdown = component_flow.iter()
            .map(|step| (step.component.clone(), step.duration.num_milliseconds() as u64))
            .collect();

        PerformanceSummary {
            total_duration_ms: total_duration.num_milliseconds() as u64,
            component_count: component_flow.len(),
            slowest_component,
            component_breakdown,
            error_count: self.error_events().len(),
        }
    }

    /// Check if the trace has any errors
    pub fn has_errors(&self) -> bool {
        !self.error_events().is_empty()
    }

    /// Get the first error in the trace
    pub fn first_error(&self) -> Option<&TestEvent> {
        self.error_events().first().copied()
    }

    /// Format trace as human-readable timeline
    pub fn format_timeline(&self) -> String {
        let mut timeline = String::new();
        timeline.push_str(&format!("Trace: {}\n", self.trace_id));
        timeline.push_str(&format!("Duration: {}ms\n", self.total_duration().num_milliseconds()));
        timeline.push_str("Timeline:\n");

        for event in &self.events {
            let relative_time = if let Some(start) = self.started_at {
                (event.timestamp - start).num_milliseconds()
            } else {
                0
            };

            timeline.push_str(&format!(
                "  +{}ms [{}] {}: {}\n",
                relative_time,
                event.component,
                format!("{:?}", event.level),
                event.message
            ));
        }

        timeline
    }
}

/// A step in the component flow
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentStep {
    pub component: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration: chrono::Duration,
    pub event_count: usize,
}

/// Performance summary for a trace
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_duration_ms: u64,
    pub component_count: usize,
    pub slowest_component: Option<String>,
    pub component_breakdown: Vec<(String, u64)>,
    pub error_count: usize,
}

/// Trace correlation utilities
pub struct TraceCorrelation;

impl TraceCorrelation {
    /// Find related traces (e.g., same test run, similar patterns)
    pub fn find_related_traces(traces: &[RequestTrace], target_trace: &RequestTrace) -> Vec<String> {
        let mut related = Vec::new();

        for trace in traces {
            if trace.trace_id == target_trace.trace_id {
                continue;
            }

            // Check for similar component flow
            let target_components: Vec<String> = target_trace.component_flow()
                .iter()
                .map(|step| step.component.clone())
                .collect();

            let trace_components: Vec<String> = trace.component_flow()
                .iter()
                .map(|step| step.component.clone())
                .collect();

            // If component flows are similar, consider them related
            if Self::similarity_score(&target_components, &trace_components) > 0.7 {
                related.push(trace.trace_id.clone());
            }
        }

        related
    }

    /// Calculate similarity score between two component flows
    fn similarity_score(flow1: &[String], flow2: &[String]) -> f64 {
        if flow1.is_empty() && flow2.is_empty() {
            return 1.0;
        }

        if flow1.is_empty() || flow2.is_empty() {
            return 0.0;
        }

        let set1: std::collections::HashSet<&String> = flow1.iter().collect();
        let set2: std::collections::HashSet<&String> = flow2.iter().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Analyze common error patterns across traces
    pub fn analyze_error_patterns(traces: &[RequestTrace]) -> Vec<ErrorPattern> {
        let mut patterns: std::collections::HashMap<String, ErrorPattern> = std::collections::HashMap::new();

        for trace in traces {
            for error in trace.error_events() {
                let key = format!("{}:{}", error.component, error.message);
                let pattern = patterns.entry(key.clone()).or_insert(ErrorPattern {
                    component: error.component.clone(),
                    error_message: error.message.clone(),
                    occurrences: 0,
                    first_seen: error.timestamp,
                    last_seen: error.timestamp,
                    affected_traces: Vec::new(),
                });

                pattern.occurrences += 1;
                pattern.last_seen = error.timestamp;
                if !pattern.affected_traces.contains(&trace.trace_id) {
                    pattern.affected_traces.push(trace.trace_id.clone());
                }
            }
        }

        patterns.into_values().collect()
    }
}

/// Error pattern analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub component: String,
    pub error_message: String,
    pub occurrences: usize,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub affected_traces: Vec<String>,
}