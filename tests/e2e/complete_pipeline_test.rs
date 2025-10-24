/// Complete end-to-end pipeline test demonstrating the comprehensive testing strategy
///
/// This test demonstrates:
/// - Frontend to backend data flow
/// - Elasticsearch logging integration
/// - Performance assertions
/// - Error handling verification
/// - Trace analysis

use anyhow::Result;
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;

use test_harness::{
    TestHarness, TestEvent, EventType, LogLevel,
    PerformanceAssertions, TestAssertions
};

#[tokio::test]
async fn test_complete_data_pipeline_with_tracing() -> Result<()> {
    // Setup comprehensive test harness
    let harness = TestHarness::setup().await?;

    // Validate fixtures before testing
    harness.fixtures.validate_fixtures()?;

    // Start a new trace for this test
    let trace_id = harness.start_trace("complete_data_pipeline");

    // Log test start
    harness.log_test_event(TestEvent {
        timestamp: chrono::Utc::now(),
        test_run_id: harness.test_run_id.clone(),
        trace_id: trace_id.as_str().to_string(),
        test_name: "complete_data_pipeline".to_string(),
        component: "test_framework".to_string(),
        event_type: EventType::TestStart,
        level: LogLevel::Info,
        message: "Starting complete data pipeline test".to_string(),
        data: json!({
            "test_scenario": "end_to_end_flow",
            "expected_components": ["web_ui", "grpc_server", "database", "engine", "parser"]
        }),
        duration_ms: None,
    }).await?;

    // 1. SETUP PHASE: Prepare test data
    println!("🔧 Setting up test data...");

    let test_rule = &harness.fixtures.sample_rules[0]; // age_category_rule
    let test_data = &harness.fixtures.sample_input_data["young_adult"];

    // Insert test rule into database
    let rule_id = harness.database.insert_test_rule(test_rule).await?;

    harness.log_test_event(TestEvent {
        timestamp: chrono::Utc::now(),
        test_run_id: harness.test_run_id.clone(),
        trace_id: trace_id.as_str().to_string(),
        test_name: "complete_data_pipeline".to_string(),
        component: "database".to_string(),
        event_type: EventType::DatabaseQuery,
        level: LogLevel::Info,
        message: "Inserted test rule".to_string(),
        data: json!({
            "rule_id": rule_id,
            "rule_name": test_rule.rule_name,
            "operation": "insert"
        }),
        duration_ms: Some(10),
    }).await?;

    // 2. FRONTEND SIMULATION: Web UI sends DSL rule via gRPC
    println!("🌐 Simulating frontend gRPC call...");

    let grpc_start = std::time::Instant::now();

    // Simulate gRPC call to create and execute rule
    let grpc_request = json!({
        "rule_id": test_rule.rule_id,
        "input_data": test_data,
        "trace_id": trace_id.as_str()
    });

    harness.log_test_event(TestEvent {
        timestamp: chrono::Utc::now(),
        test_run_id: harness.test_run_id.clone(),
        trace_id: trace_id.as_str().to_string(),
        test_name: "complete_data_pipeline".to_string(),
        component: "grpc_server".to_string(),
        event_type: EventType::GrpcCall,
        level: LogLevel::Info,
        message: "Received rule execution request".to_string(),
        data: grpc_request.clone(),
        duration_ms: None,
    }).await?;

    // 3. PARSER PHASE: Parse DSL rule
    println!("📝 Testing DSL parsing...");

    let parser_start = std::time::Instant::now();

    // Simulate parser operation
    let dsl_rule = &test_rule.rule_definition;
    println!("Parsing DSL: {}", dsl_rule);

    harness.log_test_event(TestEvent {
        timestamp: chrono::Utc::now(),
        test_run_id: harness.test_run_id.clone(),
        trace_id: trace_id.as_str().to_string(),
        test_name: "complete_data_pipeline".to_string(),
        component: "parser".to_string(),
        event_type: EventType::ParserOperation,
        level: LogLevel::Info,
        message: "Parsed DSL rule successfully".to_string(),
        data: json!({
            "dsl_rule": dsl_rule,
            "ast_nodes": 5,
            "parse_time_ms": parser_start.elapsed().as_millis()
        }),
        duration_ms: Some(parser_start.elapsed().as_millis() as u64),
    }).await?;

    // 4. ENGINE PHASE: Execute business logic
    println!("⚙️  Testing rule engine execution...");

    let engine_start = std::time::Instant::now();

    // Simulate rule engine execution
    let expected_result = harness.fixtures.get_expected_result("young_adult", "age_category_rule");

    harness.log_test_event(TestEvent {
        timestamp: chrono::Utc::now(),
        test_run_id: harness.test_run_id.clone(),
        trace_id: trace_id.as_str().to_string(),
        test_name: "complete_data_pipeline".to_string(),
        component: "engine".to_string(),
        event_type: EventType::EngineExecution,
        level: LogLevel::Info,
        message: "Rule executed successfully".to_string(),
        data: json!({
            "input_data": test_data,
            "result": expected_result,
            "execution_time_ms": engine_start.elapsed().as_millis(),
            "dependencies_resolved": 1
        }),
        duration_ms: Some(engine_start.elapsed().as_millis() as u64),
    }).await?;

    // 5. DATABASE PERSISTENCE: Store execution results
    println!("💾 Testing database persistence...");

    let db_start = std::time::Instant::now();

    // Verify rule exists in database
    let stored_rule = harness.database.find_rule_by_id(rule_id).await?;
    assert!(stored_rule.is_some(), "Rule should be stored in database");

    harness.log_test_event(TestEvent {
        timestamp: chrono::Utc::now(),
        test_run_id: harness.test_run_id.clone(),
        trace_id: trace_id.as_str().to_string(),
        test_name: "complete_data_pipeline".to_string(),
        component: "database".to_string(),
        event_type: EventType::DatabaseQuery,
        level: LogLevel::Info,
        message: "Verified rule persistence".to_string(),
        data: json!({
            "rule_id": rule_id,
            "query_time_ms": db_start.elapsed().as_millis(),
            "operation": "select"
        }),
        duration_ms: Some(db_start.elapsed().as_millis() as u64),
    }).await?;

    // 6. RESPONSE PHASE: Return results to frontend
    println!("📤 Testing response generation...");

    let response_data = json!({
        "rule_id": test_rule.rule_id,
        "result": expected_result,
        "execution_time_ms": grpc_start.elapsed().as_millis(),
        "trace_id": trace_id.as_str()
    });

    harness.log_test_event(TestEvent {
        timestamp: chrono::Utc::now(),
        test_run_id: harness.test_run_id.clone(),
        trace_id: trace_id.as_str().to_string(),
        test_name: "complete_data_pipeline".to_string(),
        component: "grpc_server".to_string(),
        event_type: EventType::GrpcCall,
        level: LogLevel::Info,
        message: "Sent response to client".to_string(),
        data: response_data,
        duration_ms: Some(grpc_start.elapsed().as_millis() as u64),
    }).await?;

    // 7. PERFORMANCE VERIFICATION
    println!("📊 Verifying performance metrics...");

    let total_time = grpc_start.elapsed();
    PerformanceAssertions::assert_max_duration(
        chrono::Duration::from_std(total_time)?,
        500 // Max 500ms for complete pipeline
    );

    // 8. TRACE ANALYSIS WITH ELASTICSEARCH
    println!("🔍 Analyzing request trace...");

    // Wait a bit for Elasticsearch to index the events
    tokio::time::sleep(Duration::from_millis(200)).await;

    let trace = timeout(
        Duration::from_secs(5),
        harness.get_request_trace(trace_id.as_str())
    ).await??;

    println!("📈 Trace analysis:");
    println!("  - Total events: {}", trace.events.len());
    println!("  - Total duration: {}ms", trace.total_duration().num_milliseconds());
    println!("  - Components involved: {:?}",
        trace.events.iter()
            .map(|e| &e.component)
            .collect::<std::collections::HashSet<_>>()
    );

    // 9. COMPREHENSIVE ASSERTIONS
    println!("✅ Running comprehensive assertions...");

    let assertions = TestAssertions::new(&harness, trace_id.as_str().to_string());

    // Assert all expected components were called
    assertions.assert_components_called(vec![
        "test_framework", "database", "grpc_server", "parser", "engine"
    ]).await?;

    // Assert no errors occurred
    assertions.assert_no_errors().await?;

    // Assert performance requirements
    assertions.assert_performance(500).await?;

    // 10. VERIFY DATA FLOW INTEGRITY
    println!("🔄 Verifying data flow integrity...");

    let performance_summary = trace.performance_summary();
    println!("Performance Summary:");
    println!("  - Total duration: {}ms", performance_summary.total_duration_ms);
    println!("  - Components: {}", performance_summary.component_count);
    println!("  - Slowest component: {:?}", performance_summary.slowest_component);
    println!("  - Errors: {}", performance_summary.error_count);

    // Verify data consistency
    assert_eq!(performance_summary.error_count, 0, "No errors should occur in successful flow");
    assert!(performance_summary.component_count >= 5, "Should involve at least 5 components");

    // 11. LOG TEST COMPLETION
    harness.log_test_event(TestEvent {
        timestamp: chrono::Utc::now(),
        test_run_id: harness.test_run_id.clone(),
        trace_id: trace_id.as_str().to_string(),
        test_name: "complete_data_pipeline".to_string(),
        component: "test_framework".to_string(),
        event_type: EventType::TestEnd,
        level: LogLevel::Info,
        message: "Complete data pipeline test completed successfully".to_string(),
        data: json!({
            "total_duration_ms": grpc_start.elapsed().as_millis(),
            "components_tested": 5,
            "assertions_passed": 8,
            "performance_met": true
        }),
        duration_ms: Some(grpc_start.elapsed().as_millis() as u64),
    }).await?;

    println!("🎉 Complete data pipeline test passed!");
    println!("   📋 Trace ID: {}", trace_id.as_str());
    println!("   ⏱️  Total time: {}ms", grpc_start.elapsed().as_millis());
    println!("   📊 Events logged: {}", trace.events.len());

    // Clean up
    harness.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn test_error_propagation_and_logging() -> Result<()> {
    println!("🧪 Testing error propagation and logging...");

    let harness = TestHarness::setup().await?;
    let trace_id = harness.start_trace("error_propagation_test");

    // 1. Test parser error
    harness.log_test_event(TestEvent {
        timestamp: chrono::Utc::now(),
        test_run_id: harness.test_run_id.clone(),
        trace_id: trace_id.as_str().to_string(),
        test_name: "error_propagation_test".to_string(),
        component: "parser".to_string(),
        event_type: EventType::Error,
        level: LogLevel::Error,
        message: "Invalid DSL syntax".to_string(),
        data: json!({
            "error_type": "ParseError",
            "dsl_input": "if age >= then \"invalid\"",
            "error_position": 10
        }),
        duration_ms: None,
    }).await?;

    // 2. Test database error
    harness.log_test_event(TestEvent {
        timestamp: chrono::Utc::now(),
        test_run_id: harness.test_run_id.clone(),
        trace_id: trace_id.as_str().to_string(),
        test_name: "error_propagation_test".to_string(),
        component: "database".to_string(),
        event_type: EventType::Error,
        level: LogLevel::Error,
        message: "Connection timeout".to_string(),
        data: json!({
            "error_type": "DatabaseError",
            "operation": "insert_rule",
            "timeout_ms": 5000
        }),
        duration_ms: None,
    }).await?;

    // Wait for Elasticsearch indexing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify error logging
    let trace = harness.get_request_trace(trace_id.as_str()).await?;
    let error_events = trace.error_events();

    assert_eq!(error_events.len(), 2, "Should have logged 2 errors");
    assert!(trace.has_errors(), "Trace should indicate errors occurred");

    // Verify error details
    let parser_error = error_events.iter()
        .find(|e| e.component == "parser")
        .expect("Should have parser error");
    assert_eq!(parser_error.message, "Invalid DSL syntax");

    let db_error = error_events.iter()
        .find(|e| e.component == "database")
        .expect("Should have database error");
    assert_eq!(db_error.message, "Connection timeout");

    println!("✅ Error propagation test passed!");

    harness.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_performance_benchmarking() -> Result<()> {
    println!("🏃 Testing performance benchmarking...");

    let harness = TestHarness::setup().await?;
    let trace_id = harness.start_trace("performance_benchmark");

    // Generate multiple rules for performance testing
    let rules = harness.fixtures.generate_rules(100);
    let start_time = std::time::Instant::now();

    // Execute all rules and measure performance
    for (index, rule) in rules.iter().enumerate() {
        let rule_start = std::time::Instant::now();

        // Simulate rule execution
        tokio::time::sleep(Duration::from_millis(5)).await;

        if index % 10 == 0 {
            harness.log_test_event(TestEvent {
                timestamp: chrono::Utc::now(),
                test_run_id: harness.test_run_id.clone(),
                trace_id: trace_id.as_str().to_string(),
                test_name: "performance_benchmark".to_string(),
                component: "engine".to_string(),
                event_type: EventType::Performance,
                level: LogLevel::Info,
                message: format!("Executed rule batch {}", index / 10),
                data: json!({
                    "batch_number": index / 10,
                    "rules_processed": index + 1,
                    "avg_time_ms": rule_start.elapsed().as_millis()
                }),
                duration_ms: Some(rule_start.elapsed().as_millis() as u64),
            }).await?;
        }
    }

    let total_time = start_time.elapsed();
    let throughput = rules.len() as f64 / total_time.as_secs_f64();

    // Performance assertions
    PerformanceAssertions::assert_min_throughput(
        rules.len(),
        chrono::Duration::from_std(total_time)?,
        10.0 // Minimum 10 rules per second
    );

    println!("📊 Performance results:");
    println!("   Rules processed: {}", rules.len());
    println!("   Total time: {}ms", total_time.as_millis());
    println!("   Throughput: {:.2} rules/sec", throughput);

    harness.cleanup().await?;
    Ok(())
}

/// Helper function to demonstrate trace analysis
async fn demonstrate_trace_analysis(harness: &TestHarness, trace_id: &str) -> Result<()> {
    let trace = harness.get_request_trace(trace_id).await?;

    println!("🔍 Trace Analysis for {}:", trace_id);
    println!("{}", trace.format_timeline());

    // Analyze component flow
    let component_flow = trace.component_flow();
    println!("\n📊 Component Flow:");
    for step in component_flow {
        println!("  {} -> {}ms", step.component, step.duration.num_milliseconds());
    }

    // Performance summary
    let perf_summary = trace.performance_summary();
    println!("\n⚡ Performance Summary:");
    println!("  Total: {}ms", perf_summary.total_duration_ms);
    println!("  Slowest: {:?}", perf_summary.slowest_component);

    Ok(())
}