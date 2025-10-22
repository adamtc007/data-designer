# HTTP to gRPC Reconciliation Analysis

## Overview
This document analyzes discrepancies between HTTP fallback endpoints and gRPC service definitions, providing a roadmap for full reconciliation.

## Current Status: ✅ Working but Inconsistent

The HTTP fallback mechanism is **functionally working** but has several **argument format inconsistencies** that need reconciliation for production readiness.

## Endpoint Analysis

### 1. ExecuteCbuDsl ✅ WORKING
**gRPC Definition:**
```protobuf
message ExecuteCbuDslRequest {
  string dsl_script = 1;
}
message ExecuteCbuDslResponse {
  bool success = 1;
  string message = 2;
  optional string cbu_id = 3;
  repeated string validation_errors = 4;
  optional string data = 5; // JSON string
}
```

**HTTP Implementation:** ✅ COMPATIBLE
- **Input:** `Json(request): Json<serde_json::Value>` - extracts `request["dsl_script"]`
- **Output:** Returns JSON matching gRPC response format exactly
- **Status:** Working correctly, formats match

### 2. GetEntities ⚠️ PARTIALLY COMPATIBLE
**gRPC Definition:**
```protobuf
message GetEntitiesRequest {
  optional string jurisdiction = 1;
  optional string entity_type = 2;
  optional string status = 3;
}
message GetEntitiesResponse {
  repeated EntityInfo entities = 1;
}
```

**HTTP Implementation:** ⚠️ MOCK DATA
- **Input:** `Json(_request): Json<serde_json::Value>` - ignores filter parameters
- **Output:** Returns hardcoded mock entities (not from database)
- **Issues:**
  - No filter parameter support
  - Mock data instead of database queries
  - Response format matches but data source differs

### 3. ListCbus ⚠️ PARTIALLY COMPATIBLE
**gRPC Definition:**
```protobuf
message ListCbusRequest {
  optional string status_filter = 1;
  optional int32 limit = 2;
  optional int32 offset = 3;
}
message ListCbusResponse {
  repeated Cbu cbus = 1;
  int32 total_count = 2;
}
```

**HTTP Implementation:** ⚠️ MOCK DATA
- **Input:** `Json(_request): Json<serde_json::Value>` - ignores pagination
- **Output:** Returns hardcoded mock CBUs
- **Issues:**
  - No pagination support (limit/offset)
  - No status filtering
  - Mock data instead of database queries

### 4. GetAiSuggestions ✅ COMPATIBLE
**gRPC Definition:**
```protobuf
message GetAiSuggestionsRequest {
  string query = 1;
  optional string context = 2;
  optional AiProviderConfig ai_provider = 3;
}
message GetAiSuggestionsResponse {
  repeated AiSuggestion suggestions = 1;
  string status_message = 2;
}
```

**HTTP Implementation:** ✅ TYPED
- **Input:** `Json(request): Json<GetAiSuggestionsRequest>` - uses proper gRPC types
- **Output:** Generates realistic AI suggestions
- **Status:** Properly structured and compatible

### 5. InstantiateResource ⚠️ MOCK ONLY
**gRPC Definition:**
```protobuf
message InstantiateResourceRequest {
  string template_id = 1;
  string onboarding_request_id = 2;
  optional string context = 3;
  optional string initial_data = 4;
}
message InstantiateResourceResponse {
  bool success = 1;
  string message = 2;
  optional ResourceInstance instance = 3;
}
```

**HTTP Implementation:** ⚠️ MOCK RESPONSE
- **Input:** `Json(request): Json<InstantiateResourceRequest>` - properly typed
- **Output:** Returns mock response only
- **Issues:** No actual instantiation logic

### 6. ListProducts ⚠️ BASIC MOCK
**gRPC Definition:**
```protobuf
message ListProductsRequest {
  optional string status_filter = 1;
  optional string line_of_business_filter = 2;
  optional int32 limit = 3;
  optional int32 offset = 4;
}
message ListProductsResponse {
  repeated ProductDetails products = 1;
  int32 total_count = 2;
}
```

**HTTP Implementation:** ⚠️ MOCK DATA
- **Input:** `Json(_request): Json<serde_json::Value>` - ignores all filters
- **Output:** Simple mock response
- **Issues:** No filtering, pagination, or database integration

## Web UI Client Analysis

### Request Format Compatibility ✅ GOOD
The web UI properly constructs typed request objects:

```rust
// Example: ExecuteCbuDsl
let request = ExecuteCbuDslRequest {
    dsl_script: self.dsl_script.clone(),
};
```

### Service Method Mapping ✅ CORRECT
```rust
match service_method {
    "financial_taxonomy.FinancialTaxonomyService/ExecuteCbuDsl" => "/api/execute-cbu-dsl",
    "financial_taxonomy.FinancialTaxonomyService/GetEntities" => "/api/entities",
    "financial_taxonomy.FinancialTaxonomyService/ListCbus" => "/api/list-cbus",
    "financial_taxonomy.FinancialTaxonomyService/GetAiSuggestions" => "/api/ai-suggestions",
    "financial_taxonomy.FinancialTaxonomyService/InstantiateResource" => "/api/instantiate",
    "financial_taxonomy.FinancialTaxonomyService/ListProducts" => "/api/list-products",
    // ... other mappings
}
```

## Priority Issues to Fix

### HIGH PRIORITY 🔥
1. **Mock Data Endpoints** - Replace hardcoded responses with database queries
   - `GetEntities` - Connect to entity database table
   - `ListCbus` - Connect to CBU database table
   - `ListProducts` - Connect to products database table

2. **Missing Filter Support** - Implement gRPC parameter handling
   - `GetEntities` - jurisdiction, entity_type, status filters
   - `ListCbus` - status_filter, pagination (limit/offset)
   - `ListProducts` - status_filter, line_of_business_filter, pagination

### MEDIUM PRIORITY ⚠️
3. **InstantiateResource** - Implement actual instantiation logic
4. **Parameter Validation** - Add proper request validation for HTTP endpoints
5. **Error Handling** - Standardize error response formats

### LOW PRIORITY 📝
6. **Type Safety** - Convert remaining `serde_json::Value` to typed structs
7. **Documentation** - Add OpenAPI/Swagger docs for HTTP endpoints

## Implementation Plan

### Phase 1: Database Integration (HIGH)
```rust
// Replace mock implementations with database queries
async fn get_entities_http(
    State(pool): State<PgPool>,
    Json(request): Json<GetEntitiesRequest>, // Use typed request
) -> Result<ResponseJson<GetEntitiesResponse>, StatusCode> {
    // Query database with filters
    let entities = query_entities(&pool, &request).await?;
    Ok(ResponseJson(GetEntitiesResponse { entities }))
}
```

### Phase 2: Parameter Support (HIGH)
```rust
// Add proper filter and pagination support
async fn list_cbus_http(
    State(pool): State<PgPool>,
    Json(request): Json<ListCbusRequest>,
) -> Result<ResponseJson<ListCbusResponse>, StatusCode> {
    // Apply status_filter, limit, offset
    let cbus = query_cbus_with_filters(&pool, &request).await?;
    Ok(ResponseJson(ListCbusResponse {
        cbus,
        total_count: get_total_cbu_count(&pool).await?
    }))
}
```

### Phase 3: Type Safety (MEDIUM)
- Convert all `Json<serde_json::Value>` to proper typed requests
- Add request validation middleware
- Standardize error response format

## Testing Strategy

### Unit Tests
- Test each HTTP endpoint with various parameter combinations
- Verify response format matches gRPC definition exactly
- Test error conditions and edge cases

### Integration Tests
- End-to-end tests calling HTTP endpoints and comparing with gRPC responses
- Database integration tests with real data
- Performance comparison between HTTP and gRPC

### Compatibility Tests
```rust
#[tokio::test]
async fn test_execute_cbu_dsl_http_grpc_compatibility() {
    let http_response = execute_cbu_dsl_http(request.clone()).await?;
    let grpc_response = grpc_client.execute_cbu_dsl(request).await?;
    assert_eq!(http_response, grpc_response);
}
```

## Expected Outcomes

### After Full Reconciliation ✅
1. **100% Compatible** - HTTP and gRPC endpoints return identical responses
2. **Database Integration** - All endpoints query real data
3. **Parameter Support** - Full filtering and pagination support
4. **Type Safety** - All endpoints use proper gRPC types
5. **Production Ready** - HTTP fallback can handle full production load

### Benefits
- **Reliability** - Seamless fallback when gRPC unavailable
- **Development** - Easier testing and debugging via HTTP
- **Integration** - Third-party systems can use HTTP API
- **Monitoring** - Standard HTTP tooling for observability

## Current Working Status: ✅ FUNCTIONAL

**Key Point:** The system is currently **working end-to-end** with successful DSL execution. The reconciliation is about **improving consistency and production readiness**, not fixing broken functionality.

**Evidence from logs:**
```
[2025-10-22T17:07:30] HTTP ExecuteCbuDsl called
[2025-10-22T17:07:30] Processing DSL script: (CONFIGURE_SYSTEM "test")
[2025-10-22T17:07:30] Detected LISP syntax, using LISP parser
```

The user can successfully execute DSL, and the HTTP fallback is functioning correctly for the core use case.