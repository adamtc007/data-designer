use anyhow::Result;
use financial_taxonomy_grpc_server::{
    financial_taxonomy_service_client::FinancialTaxonomyServiceClient,
    GetProductsRequest, GetProductOptionsRequest, GetServicesRequest,
    GetCbuMandateStructureRequest, GetCbuMemberRolesRequest,
    GetTaxonomyHierarchyRequest, GetAiSuggestionsRequest,
    HealthCheckRequest, GetApiKeyRequest, ListApiKeysRequest,
};
use tonic::transport::Channel;

const GRPC_SERVER_URL: &str = "http://127.0.0.1:50051";

/// Helper function to create a gRPC client
async fn create_grpc_client() -> Result<FinancialTaxonomyServiceClient<Channel>> {
    let client = FinancialTaxonomyServiceClient::connect(GRPC_SERVER_URL).await?;
    Ok(client)
}

#[tokio::test]
async fn test_grpc_server_health_check() {
    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    let request = HealthCheckRequest {
        service: "financial_taxonomy".to_string(),
    };

    let response = client.check(request).await.expect("Health check failed");
    let health = response.into_inner();

    println!("✅ Health check status: {:?}", health.status);
    assert_eq!(health.status, 1); // SERVING = 1
}

#[tokio::test]
async fn test_get_products() {
    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    let request = GetProductsRequest {
        status_filter: Some("active".to_string()),
        limit: Some(10),
        offset: None,
    };

    let response = client.get_products(request).await.expect("Failed to get products");
    let products_response = response.into_inner();

    println!("✅ Retrieved {} products", products_response.products.len());

    // Validate response structure
    assert!(!products_response.products.is_empty(), "Should have at least one product");

    // Validate first product has required fields
    let first_product = &products_response.products[0];
    assert!(!first_product.product_id.is_empty(), "Product ID should not be empty");
    assert!(!first_product.product_name.is_empty(), "Product name should not be empty");
    assert_eq!(first_product.status, "active", "Product status should be active");

    println!("✅ First product: {} ({})", first_product.product_name, first_product.product_id);
}

#[tokio::test]
async fn test_get_product_options() {
    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    let request = GetProductOptionsRequest {
        product_id: None, // Get all product options
        status_filter: Some("active".to_string()),
        limit: Some(5),
        offset: None,
    };

    let response = client.get_product_options(request).await.expect("Failed to get product options");
    let options_response = response.into_inner();

    println!("✅ Retrieved {} product options", options_response.product_options.len());

    if !options_response.product_options.is_empty() {
        let first_option = &options_response.product_options[0];
        assert!(!first_option.option_name.is_empty(), "Option name should not be empty");
        assert!(!first_option.option_category.is_empty(), "Option category should not be empty");

        println!("✅ First option: {} ({})", first_option.option_name, first_option.option_category);
    }
}

#[tokio::test]
async fn test_get_services() {
    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    let request = GetServicesRequest {
        status_filter: Some("active".to_string()),
        service_category: None,
        limit: Some(5),
        offset: None,
    };

    let response = client.get_services(request).await.expect("Failed to get services");
    let services_response = response.into_inner();

    println!("✅ Retrieved {} services", services_response.services.len());

    if !services_response.services.is_empty() {
        let first_service = &services_response.services[0];
        assert!(!first_service.service_name.is_empty(), "Service name should not be empty");

        println!("✅ First service: {}", first_service.service_name);
    }
}

#[tokio::test]
async fn test_get_cbu_mandate_structure() {
    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    let request = GetCbuMandateStructureRequest {
        cbu_id: None,
        mandate_id: None,
        limit: Some(3),
        offset: None,
    };

    let response = client.get_cbu_mandate_structure(request).await.expect("Failed to get CBU mandate structure");
    let structures_response = response.into_inner();

    println!("✅ Retrieved {} CBU mandate structures", structures_response.structures.len());

    if !structures_response.structures.is_empty() {
        let first_structure = &structures_response.structures[0];
        assert!(!first_structure.cbu_id.is_empty(), "CBU ID should not be empty");
        assert!(!first_structure.cbu_name.is_empty(), "CBU name should not be empty");

        println!("✅ First CBU: {} ({})", first_structure.cbu_name, first_structure.cbu_id);
    }
}

#[tokio::test]
async fn test_get_cbu_member_roles() {
    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    let request = GetCbuMemberRolesRequest {
        cbu_id: None,
        role_code: None,
        limit: Some(3),
        offset: None,
    };

    let response = client.get_cbu_member_roles(request).await.expect("Failed to get CBU member roles");
    let roles_response = response.into_inner();

    println!("✅ Retrieved {} CBU member roles", roles_response.roles.len());

    if !roles_response.roles.is_empty() {
        let first_role = &roles_response.roles[0];
        assert!(!first_role.entity_name.is_empty(), "Entity name should not be empty");
        assert!(!first_role.role_name.is_empty(), "Role name should not be empty");

        println!("✅ First role: {} - {}", first_role.entity_name, first_role.role_name);
    }
}

#[tokio::test]
async fn test_get_taxonomy_hierarchy() {
    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    let request = GetTaxonomyHierarchyRequest {
        max_levels: Some(3),
        item_type_filter: None,
    };

    let response = client.get_taxonomy_hierarchy(request).await.expect("Failed to get taxonomy hierarchy");
    let hierarchy_response = response.into_inner();

    println!("✅ Retrieved {} taxonomy hierarchy items", hierarchy_response.items.len());

    if !hierarchy_response.items.is_empty() {
        let first_item = &hierarchy_response.items[0];
        assert!(!first_item.item_name.is_empty(), "Item name should not be empty");
        assert!(!first_item.item_type.is_empty(), "Item type should not be empty");

        println!("✅ First hierarchy item: {} (Level {})", first_item.item_name, first_item.level);
    }
}

#[tokio::test]
async fn test_get_ai_suggestions() {
    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    let request = GetAiSuggestionsRequest {
        query: "product configuration".to_string(),
        ai_provider: None, // Skip AI provider for now
        context: Some("financial_taxonomy".to_string()),
    };

    let response = client.get_ai_suggestions(request).await.expect("Failed to get AI suggestions");
    let suggestions_response = response.into_inner();

    println!("✅ Retrieved {} AI suggestions", suggestions_response.suggestions.len());
    println!("Status: {}", suggestions_response.status_message);

    if !suggestions_response.suggestions.is_empty() {
        let first_suggestion = &suggestions_response.suggestions[0];
        assert!(!first_suggestion.title.is_empty(), "Suggestion title should not be empty");
        assert!(!first_suggestion.description.is_empty(), "Suggestion description should not be empty");
        assert!(first_suggestion.confidence >= 0.0 && first_suggestion.confidence <= 1.0, "Confidence should be between 0 and 1");

        println!("✅ First suggestion: {} (confidence: {:.1}%)",
                first_suggestion.title, first_suggestion.confidence * 100.0);
    }
}

#[tokio::test]
async fn test_error_handling_invalid_request() {
    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    // Test with invalid status filter
    let request = GetProductsRequest {
        status_filter: Some("invalid_status".to_string()),
        limit: Some(10),
        offset: None,
    };

    let response = client.get_products(request).await;

    // Should either return empty results or handle gracefully
    match response {
        Ok(resp) => {
            let products = resp.into_inner().products;
            println!("✅ Invalid status filter handled gracefully: {} products", products.len());
        }
        Err(e) => {
            println!("✅ Invalid request properly rejected: {}", e);
        }
    }
}

#[tokio::test]
async fn test_pagination() {
    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    // Get first page
    let request1 = GetProductsRequest {
        status_filter: Some("active".to_string()),
        limit: Some(2),
        offset: Some(0),
    };

    let response1 = client.get_products(request1).await.expect("Failed to get first page");
    let products1 = response1.into_inner().products;

    // Get second page
    let request2 = GetProductsRequest {
        status_filter: Some("active".to_string()),
        limit: Some(2),
        offset: Some(2),
    };

    let response2 = client.get_products(request2).await.expect("Failed to get second page");
    let products2 = response2.into_inner().products;

    println!("✅ Page 1: {} products, Page 2: {} products", products1.len(), products2.len());

    // Ensure different products (if we have enough data)
    if !products1.is_empty() && !products2.is_empty() {
        assert_ne!(products1[0].product_id, products2[0].product_id, "Pages should have different products");
        println!("✅ Pagination working correctly");
    }
}

#[tokio::test]
async fn test_concurrent_requests() {
    use tokio::time::{timeout, Duration};

    // Create separate clients to avoid borrowing issues
    let mut client1 = create_grpc_client().await.expect("Failed to connect to gRPC server");
    let mut client2 = create_grpc_client().await.expect("Failed to connect to gRPC server");
    let mut client3 = create_grpc_client().await.expect("Failed to connect to gRPC server");

    // Make multiple concurrent requests
    let products_req = GetProductsRequest {
        status_filter: Some("active".to_string()),
        limit: Some(5),
        offset: None,
    };

    let services_req = GetServicesRequest {
        status_filter: Some("active".to_string()),
        service_category: None,
        limit: Some(5),
        offset: None,
    };

    let health_req = HealthCheckRequest {
        service: "financial_taxonomy".to_string(),
    };

    // Execute all requests concurrently with timeout
    let results = timeout(Duration::from_secs(10), async {
        tokio::join!(
            client1.get_products(products_req),
            client2.get_services(services_req),
            client3.check(health_req)
        )
    }).await.expect("Concurrent requests timed out");

    // Verify all requests succeeded
    assert!(results.0.is_ok(), "Products request failed");
    assert!(results.1.is_ok(), "Services request failed");
    assert!(results.2.is_ok(), "Health check failed");

    println!("✅ Concurrent requests completed successfully");
}

#[tokio::test]
async fn test_performance_benchmark() {
    use std::time::Instant;

    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    let start = Instant::now();
    let mut total_records = 0;

    // Benchmark multiple requests
    for i in 0..5 {
        let request = GetProductsRequest {
            status_filter: Some("active".to_string()),
            limit: Some(10),
            offset: Some(i * 10),
        };

        let response = client.get_products(request).await.expect("Request failed");
        total_records += response.into_inner().products.len();
    }

    let duration = start.elapsed();
    let avg_latency = duration.as_millis() / 5;

    println!("✅ Performance test: {} records in {:?} (avg: {}ms per request)",
             total_records, duration, avg_latency);

    // Basic performance assertion (should complete within reasonable time)
    assert!(duration.as_secs() < 10, "Requests should complete within 10 seconds");
    assert!(avg_latency < 2000, "Average latency should be under 2 seconds");
}

/// Integration test that verifies the full data flow:
/// Database → gRPC Server → Client
#[tokio::test]
async fn test_end_to_end_data_flow() {
    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    println!("🧪 Starting end-to-end data flow test...");

    // 1. Health check
    let health_request = HealthCheckRequest {
        service: "financial_taxonomy".to_string(),
    };
    let health_response = client.check(health_request).await.expect("Health check failed");
    assert_eq!(health_response.into_inner().status, 1);
    println!("✅ 1. Health check passed");

    // 2. Get products
    let products_request = GetProductsRequest {
        status_filter: Some("active".to_string()),
        limit: Some(3),
        offset: None,
    };
    let products_response = client.get_products(products_request).await.expect("Products request failed");
    let products = products_response.into_inner().products;
    assert!(!products.is_empty(), "Should have products");
    println!("✅ 2. Products retrieved: {}", products.len());

    // 3. Get product options for first product
    if let Some(first_product) = products.first() {
        let options_request = GetProductOptionsRequest {
            product_id: Some(first_product.product_id.clone()),
            status_filter: Some("active".to_string()),
            limit: Some(5),
            offset: None,
        };
        let options_response = client.get_product_options(options_request).await.expect("Product options request failed");
        let options = options_response.into_inner().product_options;
        println!("✅ 3. Product options for '{}': {}", first_product.product_name, options.len());
    }

    // 4. Get services
    let services_request = GetServicesRequest {
        status_filter: Some("active".to_string()),
        service_category: None,
        limit: Some(3),
        offset: None,
    };
    let services_response = client.get_services(services_request).await.expect("Services request failed");
    let services = services_response.into_inner().services;
    println!("✅ 4. Services retrieved: {}", services.len());

    // 5. Get CBU data
    let cbu_request = GetCbuMandateStructureRequest {
        cbu_id: None,
        mandate_id: None,
        limit: Some(2),
        offset: None,
    };
    let cbu_response = client.get_cbu_mandate_structure(cbu_request).await.expect("CBU request failed");
    let cbu_structures = cbu_response.into_inner().structures;
    println!("✅ 5. CBU structures retrieved: {}", cbu_structures.len());

    println!("🎉 End-to-end test completed successfully!");
    println!("Data flow verified: Database → gRPC Server → Client ✅");
}

#[tokio::test]
async fn test_keychain_integration() {
    let mut client = create_grpc_client().await.expect("Failed to connect to gRPC server");

    println!("🔑 Testing keychain integration...");

    // Test listing stored API keys
    let list_request = ListApiKeysRequest {
        client_id: "test-client".to_string(),
    };
    let list_response = client.list_api_keys(list_request).await.expect("Failed to list API keys");
    let list_result = list_response.into_inner();

    println!("✅ Listed API keys: {:?}", list_result.providers);
    println!("Message: {}", list_result.message);

    // Test retrieving Anthropic API key (should exist from previous storage)
    let get_request = GetApiKeyRequest {
        provider: "anthropic".to_string(),
        client_id: "test-client".to_string(),
    };
    let get_response = client.get_api_key(get_request).await.expect("Failed to get Anthropic API key");
    let get_result = get_response.into_inner();

    println!("✅ Anthropic API key check:");
    println!("  - Success: {}", get_result.success);
    println!("  - Key exists: {}", get_result.key_exists);
    println!("  - Message: {}", get_result.message);

    if get_result.key_exists {
        println!("  - Key retrieved: {}***", &get_result.api_key[..std::cmp::min(8, get_result.api_key.len())]);
        assert!(!get_result.api_key.is_empty(), "API key should not be empty");
        assert!(get_result.api_key.len() > 10, "API key should be substantial length");
        println!("🎉 Keychain integration successful - previously stored Anthropic key accessible!");
    } else {
        println!("⚠️  No Anthropic API key found in keychain");
    }
}