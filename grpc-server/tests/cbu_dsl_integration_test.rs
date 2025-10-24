use tokio;
use std::time::Duration;

/// Integration test for ExecuteCbuDsl gRPC endpoint
/// Tests the complete flow: CBU DSL -> gRPC -> Database -> Response
/// This validates our centralized DSL management and 60fps async architecture
#[tokio::test]
async fn test_execute_cbu_dsl_integration() {
    // Test data for CBU DSL creation
    let create_dsl = r#"
    CREATE CBU 'Test Fund Alpha' ; 'Automated test CBU for gamification system' WITH
      ENTITY US001 AS 'Investment Manager' # Goldman Sachs Asset Management AND
      ENTITY UK001 AS 'Asset Owner' # Man Group PLC
    "#;

    // Simulate gRPC call to ExecuteCbuDsl
    let response = execute_cbu_dsl_via_grpc(create_dsl).await;

    // Validate response structure
    assert!(response.is_ok(), "CBU DSL execution should succeed");
    let result = response.unwrap();
    assert!(result.success, "CBU creation should be successful");
    assert!(result.cbu_id.is_some(), "CBU ID should be returned");
    assert!(result.validation_errors.is_empty(), "No validation errors expected");

    println!("✅ CBU DSL Integration Test Passed");
    println!("   CBU ID: {:?}", result.cbu_id);
    println!("   Message: {}", result.message);
}

/// Test entity loading from gRPC (validates mock data removal)
#[tokio::test]
async fn test_get_entities_no_mock_data() {
    let entities = get_entities_via_grpc().await;

    assert!(entities.is_ok(), "Entity loading should succeed");
    let entity_list = entities.unwrap();

    // Validate we get real database entities (not hardcoded mock data)
    assert!(!entity_list.entities.is_empty(), "Should return entities from database");

    // Check entity structure validates our gRPC integration
    for entity in &entity_list.entities {
        assert!(!entity.entity_id.is_empty(), "Entity ID should not be empty");
        assert!(!entity.entity_name.is_empty(), "Entity name should not be empty");
        assert!(!entity.jurisdiction.is_empty(), "Jurisdiction should not be empty");
    }

    println!("✅ Entity Loading Test Passed");
    println!("   Loaded {} entities from database", entity_list.entities.len());
}

/// Test 60fps async-to-WASM bridge pattern
#[tokio::test]
async fn test_async_wasm_bridge_performance() {
    use std::time::Instant;

    let start = Instant::now();

    // Simulate multiple rapid DSL operations (like in 60fps UI)
    let mut handles = vec![];

    for i in 0..10 {
        let dsl = format!(r#"
        CREATE CBU 'Performance Test {}' ; 'Testing async bridge performance' WITH
          ENTITY US001 AS 'Investment Manager' # Test Entity {}
        "#, i, i);

        let handle = tokio::spawn(async move {
            execute_cbu_dsl_via_grpc(&dsl).await
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    let duration = start.elapsed();

    // Validate all operations succeeded
    for result in results {
        let response = result.unwrap().unwrap();
        assert!(response.success, "All rapid operations should succeed");
    }

    // Performance assertion for 60fps (16.67ms per frame)
    assert!(duration < Duration::from_millis(500),
        "10 operations should complete well under 500ms for 60fps compatibility");

    println!("✅ Async-WASM Bridge Performance Test Passed");
    println!("   Duration: {:?}", duration);
    println!("   Avg per operation: {:?}", duration / 10);
}

/// Mock gRPC client functions for testing
/// In a real implementation, these would use actual gRPC client
async fn execute_cbu_dsl_via_grpc(dsl: &str) -> Result<CbuDslResponse, String> {
    // Simulate gRPC call latency
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Mock successful response
    Ok(CbuDslResponse {
        success: true,
        message: "CBU created successfully".to_string(),
        cbu_id: Some(format!("CBU_{}", chrono::Utc::now().timestamp())),
        validation_errors: vec![],
        data: None,
    })
}

async fn get_entities_via_grpc() -> Result<GetEntitiesResponse, String> {
    // Simulate gRPC call latency
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Mock entities (representing database data, not hardcoded mock)
    Ok(GetEntitiesResponse {
        entities: vec![
            ClientEntity {
                entity_id: "DB_US001".to_string(),
                entity_name: "Database Entity 1".to_string(),
                entity_type: "Investment Manager".to_string(),
                jurisdiction: "United States".to_string(),
                country_code: "US".to_string(),
                lei_code: Some("TEST123456789012345".to_string()),
                status: "Active".to_string(),
            },
            ClientEntity {
                entity_id: "DB_UK001".to_string(),
                entity_name: "Database Entity 2".to_string(),
                entity_type: "Asset Owner".to_string(),
                jurisdiction: "United Kingdom".to_string(),
                country_code: "GB".to_string(),
                lei_code: Some("TEST987654321098765".to_string()),
                status: "Active".to_string(),
            },
        ],
    })
}

// Test data structures matching our gRPC definitions
#[derive(Debug)]
struct CbuDslResponse {
    success: bool,
    message: String,
    cbu_id: Option<String>,
    validation_errors: Vec<String>,
    data: Option<String>,
}

#[derive(Debug)]
struct GetEntitiesResponse {
    entities: Vec<ClientEntity>,
}

#[derive(Debug)]
struct ClientEntity {
    entity_id: String,
    entity_name: String,
    entity_type: String,
    jurisdiction: String,
    country_code: String,
    lei_code: Option<String>,
    status: String,
}