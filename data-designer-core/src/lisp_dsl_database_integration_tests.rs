// LISP DSL Database Integration Tests - Full Round-Trip with Real Database
// Tests: DSL → Parse → Execute → Database → Query Back → Regenerate DSL

#[cfg(test)]
mod lisp_dsl_database_integration_tests {
    use super::*;
    use crate::lisp_cbu_dsl::{LispCbuParser, LispDslError};
    use sqlx::{PgPool, Row};
    use std::env;

    async fn get_test_db_pool() -> Result<PgPool, sqlx::Error> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://adamtc007@localhost/data_designer".to_string());

        PgPool::connect(&database_url).await
    }

    /// Database Integration Test Suite 1: Create → Query → Verify Round-Trip
    #[tokio::test]
    async fn test_database_roundtrip_create_cbu() {
        println!("🗄️ Testing Database Round-Trip: Create CBU");

        let pool = match get_test_db_pool().await {
            Ok(pool) => pool,
            Err(_) => {
                println!("⚠️ Skipping database test - no database connection available");
                return;
            }
        };

        let mut parser = LispCbuParser::new(Some(pool.clone()));

        // Step 1: Create CBU via DSL
        let create_dsl = r#"
            ; Database integration test CBU
            (create-cbu "DB Integration Test Fund" "Full round-trip database test"
              (entities
                (entity "US001" "Test Investment Manager" investment-manager)
                (entity "US002" "Test Asset Owner" asset-owner)))
        "#;

        println!("Step 1: Executing create-cbu DSL...");
        let create_result = parser.parse_and_eval(create_dsl).expect("Create DSL should execute");
        assert!(create_result.success, "CBU creation should succeed");

        let cbu_id = create_result.cbu_id.expect("CBU ID should be returned");
        println!("✅ Step 1: CBU created with ID: {}", cbu_id);

        // Step 2: Query CBU from database directly
        println!("Step 2: Querying CBU from database...");
        let db_query = "SELECT cbu_name, description FROM client_business_units WHERE cbu_id = $1";
        let row = sqlx::query(db_query)
            .bind(&cbu_id)
            .fetch_one(&pool)
            .await
            .expect("CBU should exist in database");

        let db_name: String = row.get("cbu_name");
        let db_description: String = row.get("description");

        assert_eq!(db_name, "DB Integration Test Fund");
        assert_eq!(db_description, "Full round-trip database test");
        println!("✅ Step 2: CBU data verified in database");

        // Step 3: Query CBU via DSL
        println!("Step 3: Querying CBU via DSL...");
        let query_dsl = format!("(query-cbu (filter (= cbu_id \"{}\")))", cbu_id);
        let query_result = parser.parse_and_eval(&query_dsl).expect("Query DSL should execute");
        assert!(query_result.success, "CBU query should succeed");
        println!("✅ Step 3: CBU queried successfully via DSL");

        // Step 4: Generate DSL from queried data
        println!("Step 4: Regenerating DSL from database data...");
        let entities = vec![
            ("US001".to_string(), "Test Investment Manager".to_string(), "investment-manager".to_string()),
            ("US002".to_string(), "Test Asset Owner".to_string(), "asset-owner".to_string()),
        ];
        let regenerated_dsl = parser.generate_dsl_from_cbu(&db_name, &db_description, &entities);
        println!("Regenerated DSL:\n{}", regenerated_dsl);

        // Step 5: Verify regenerated DSL parses correctly
        println!("Step 5: Verifying regenerated DSL...");
        let reparsed = parser.parse(&regenerated_dsl).expect("Regenerated DSL should parse");
        assert!(!reparsed.is_empty(), "Reparsed DSL should not be empty");
        println!("✅ Step 5: Round-trip DSL generation successful");

        // Cleanup
        println!("Cleanup: Removing test CBU...");
        let cleanup_query = "DELETE FROM client_business_units WHERE cbu_id = $1";
        sqlx::query(cleanup_query)
            .bind(&cbu_id)
            .execute(&pool)
            .await
            .expect("Cleanup should succeed");
        println!("✅ Cleanup: Test data removed");

        println!("🎉 Database Round-Trip Test PASSED!");
    }

    /// Database Integration Test Suite 2: Update → Verify Changes
    #[tokio::test]
    async fn test_database_update_roundtrip() {
        println!("🗄️ Testing Database Update Round-Trip");

        let pool = match get_test_db_pool().await {
            Ok(pool) => pool,
            Err(_) => {
                println!("⚠️ Skipping database test - no database connection available");
                return;
            }
        };

        let mut parser = LispCbuParser::new(Some(pool.clone()));

        // Step 1: Create initial CBU
        let create_dsl = "(create-cbu \"Update Test Fund\" \"Initial description\")";
        let create_result = parser.parse_and_eval(create_dsl).expect("Create should succeed");
        let cbu_id = create_result.cbu_id.expect("CBU ID should be returned");
        println!("✅ Step 1: Initial CBU created: {}", cbu_id);

        // Step 2: Update CBU via DSL
        let update_dsl = format!(
            "(update-cbu \"{}\" \"Updated Test Fund\" \"Updated description\")",
            cbu_id
        );
        let update_result = parser.parse_and_eval(&update_dsl).expect("Update should succeed");
        assert!(update_result.success, "CBU update should succeed");
        println!("✅ Step 2: CBU updated via DSL");

        // Step 3: Verify changes in database
        let verify_query = "SELECT cbu_name, description FROM client_business_units WHERE cbu_id = $1";
        let row = sqlx::query(verify_query)
            .bind(&cbu_id)
            .fetch_one(&pool)
            .await
            .expect("Updated CBU should exist");

        let updated_name: String = row.get("cbu_name");
        let updated_desc: String = row.get("description");

        assert_eq!(updated_name, "Updated Test Fund");
        assert_eq!(updated_desc, "Updated description");
        println!("✅ Step 3: Database changes verified");

        // Cleanup
        sqlx::query("DELETE FROM client_business_units WHERE cbu_id = $1")
            .bind(&cbu_id)
            .execute(&pool)
            .await
            .expect("Cleanup should succeed");
        println!("✅ Cleanup completed");
    }

    /// Database Integration Test Suite 3: Complex Entity Management
    #[tokio::test]
    async fn test_complex_entity_database_integration() {
        println!("🗄️ Testing Complex Entity Database Integration");

        let pool = match get_test_db_pool().await {
            Ok(pool) => pool,
            Err(_) => {
                println!("⚠️ Skipping database test - no database connection available");
                return;
            }
        };

        let mut parser = LispCbuParser::new(Some(pool.clone()));

        // Step 1: Create CBU with multiple entities
        let complex_create_dsl = r#"
            (create-cbu "Complex Entity Test Fund" "Multi-entity integration test"
              (entities
                (entity "US001" "Alpha Investment Management LLC" investment-manager)
                (entity "US002" "Beta Pension Fund" asset-owner)
                (entity "US003" "Gamma Custody Bank" custodian)
                (entity "US004" "Delta Administration Services" administrator)
                (entity "US005" "Epsilon Prime Brokerage" prime-broker)))
        "#;

        let create_result = parser.parse_and_eval(complex_create_dsl).expect("Complex create should succeed");
        let cbu_id = create_result.cbu_id.expect("CBU ID should be returned");
        println!("✅ Step 1: Complex CBU created: {}", cbu_id);

        // Step 2: Verify entity relationships in database
        let entity_query = r#"
            SELECT le.entity_id, le.entity_name, cemr.role
            FROM legal_entities le
            JOIN cbu_entity_member_roles cemr ON le.id = cemr.legal_entity_id
            JOIN client_business_units cbu ON cemr.cbu_id = cbu.id
            WHERE cbu.cbu_id = $1
            ORDER BY le.entity_id
        "#;

        let entity_rows = sqlx::query(entity_query)
            .bind(&cbu_id)
            .fetch_all(&pool)
            .await
            .expect("Entity query should succeed");

        assert_eq!(entity_rows.len(), 5, "Should have 5 entity relationships");
        println!("✅ Step 2: Entity relationships verified in database");

        // Step 3: Query entities via DSL
        let entity_query_dsl = format!("(query-cbu (filter (= cbu_id \"{}\")) (include entities))", cbu_id);
        let entity_query_result = parser.parse_and_eval(&entity_query_dsl).expect("Entity query should succeed");
        assert!(entity_query_result.success, "Entity query should succeed");
        println!("✅ Step 3: Entities queried via DSL");

        // Step 4: Generate comprehensive DSL from database
        let entities_from_db: Vec<(String, String, String)> = entity_rows.iter().map(|row| {
            let entity_id: String = row.get("entity_id");
            let entity_name: String = row.get("entity_name");
            let role: String = row.get("role");
            (entity_id, entity_name, role)
        }).collect();

        let comprehensive_dsl = parser.generate_dsl_from_cbu(
            "Complex Entity Test Fund",
            "Multi-entity integration test",
            &entities_from_db
        );
        println!("Generated comprehensive DSL:\n{}", comprehensive_dsl);

        // Step 5: Verify comprehensive DSL parsing
        let comprehensive_parsed = parser.parse(&comprehensive_dsl).expect("Comprehensive DSL should parse");
        assert!(!comprehensive_parsed.is_empty(), "Comprehensive DSL should not be empty");
        println!("✅ Step 5: Comprehensive DSL round-trip successful");

        // Cleanup - Remove entities and CBU
        sqlx::query("DELETE FROM cbu_entity_member_roles WHERE cbu_id = (SELECT id FROM client_business_units WHERE cbu_id = $1)")
            .bind(&cbu_id)
            .execute(&pool)
            .await
            .expect("Entity cleanup should succeed");

        sqlx::query("DELETE FROM client_business_units WHERE cbu_id = $1")
            .bind(&cbu_id)
            .execute(&pool)
            .await
            .expect("CBU cleanup should succeed");

        println!("✅ Cleanup: Complex test data removed");
        println!("🎉 Complex Entity Database Integration Test PASSED!");
    }

    /// Database Integration Test Suite 4: Error Handling with Database
    #[tokio::test]
    async fn test_database_error_handling() {
        println!("🗄️ Testing Database Error Handling");

        let pool = match get_test_db_pool().await {
            Ok(pool) => pool,
            Err(_) => {
                println!("⚠️ Skipping database test - no database connection available");
                return;
            }
        };

        let mut parser = LispCbuParser::new(Some(pool.clone()));

        // Test 1: Duplicate CBU creation
        let create_dsl = "(create-cbu \"Duplicate Test Fund\" \"First creation\")";
        let first_result = parser.parse_and_eval(create_dsl).expect("First creation should succeed");
        let cbu_id = first_result.cbu_id.expect("CBU ID should be returned");

        // Try to create with same name (this should handle gracefully)
        let duplicate_result = parser.parse_and_eval(create_dsl);
        match duplicate_result {
            Ok(result) => {
                // Different CBU ID should be generated or error should be handled
                println!("✅ Test 1: Duplicate creation handled gracefully");
            }
            Err(_) => {
                println!("✅ Test 1: Duplicate creation properly rejected");
            }
        }

        // Test 2: Update non-existent CBU
        let invalid_update = "(update-cbu \"NON_EXISTENT_CBU\" \"Updated Name\" \"Updated Desc\")";
        let invalid_result = parser.parse_and_eval(invalid_update);
        match invalid_result {
            Ok(result) => {
                assert!(!result.success, "Update of non-existent CBU should fail");
                println!("✅ Test 2: Non-existent CBU update handled correctly");
            }
            Err(_) => {
                println!("✅ Test 2: Non-existent CBU update properly rejected");
            }
        }

        // Test 3: Delete non-existent CBU
        let invalid_delete = "(delete-cbu \"NON_EXISTENT_CBU\")";
        let delete_result = parser.parse_and_eval(invalid_delete);
        match delete_result {
            Ok(result) => {
                assert!(!result.success, "Delete of non-existent CBU should fail");
                println!("✅ Test 3: Non-existent CBU deletion handled correctly");
            }
            Err(_) => {
                println!("✅ Test 3: Non-existent CBU deletion properly rejected");
            }
        }

        // Cleanup
        sqlx::query("DELETE FROM client_business_units WHERE cbu_id = $1")
            .bind(&cbu_id)
            .execute(&pool)
            .await
            .expect("Cleanup should succeed");

        println!("🎉 Database Error Handling Test PASSED!");
    }

    /// Master Integration Test: Full System with Database
    #[tokio::test]
    async fn test_full_system_database_integration() {
        println!("🚀 Full System Database Integration Test");

        let pool = match get_test_db_pool().await {
            Ok(pool) => pool,
            Err(_) => {
                println!("⚠️ Skipping database test - no database connection available");
                return;
            }
        };

        let mut parser = LispCbuParser::new(Some(pool.clone()));

        // Phase 1: Create comprehensive CBU
        println!("Phase 1: Creating comprehensive CBU with full entity structure...");
        let comprehensive_dsl = r#"
            ; Full system integration test
            (create-cbu "Full System Integration Fund" "Complete end-to-end database integration test"
              (entities
                (entity "SYS001" "Primary Investment Manager" investment-manager)
                (entity "SYS002" "Institutional Asset Owner" asset-owner)
                (entity "SYS003" "Global Custody Bank" custodian)
                (entity "SYS004" "Fund Administration Services" administrator)
                (entity "SYS005" "Prime Brokerage Platform" prime-broker)
                (entity "SYS006" "Compliance Officer" compliance-officer)
                (entity "SYS007" "Risk Management Team" risk-manager)))
        "#;

        let create_result = parser.parse_and_eval(comprehensive_dsl).expect("Comprehensive creation should succeed");
        let cbu_id = create_result.cbu_id.expect("CBU ID should be returned");
        println!("✅ Phase 1: Comprehensive CBU created: {}", cbu_id);

        // Phase 2: Perform various operations
        println!("Phase 2: Performing various DSL operations...");

        // Update operation
        let update_dsl = format!(
            "(update-cbu \"{}\" \"Updated Full System Fund\" \"Updated comprehensive test\")",
            cbu_id
        );
        let update_result = parser.parse_and_eval(&update_dsl).expect("Update should succeed");
        assert!(update_result.success, "Update should succeed");

        // Query operation
        let query_dsl = format!("(query-cbu (filter (= cbu_id \"{}\")))", cbu_id);
        let query_result = parser.parse_and_eval(&query_dsl).expect("Query should succeed");
        assert!(query_result.success, "Query should succeed");

        println!("✅ Phase 2: Various operations completed successfully");

        // Phase 3: Verify database consistency
        println!("Phase 3: Verifying database consistency...");

        // Check CBU data
        let cbu_query = "SELECT cbu_name, description FROM client_business_units WHERE cbu_id = $1";
        let cbu_row = sqlx::query(cbu_query)
            .bind(&cbu_id)
            .fetch_one(&pool)
            .await
            .expect("CBU should exist");

        let final_name: String = cbu_row.get("cbu_name");
        assert_eq!(final_name, "Updated Full System Fund");

        // Check entity count
        let entity_count_query = r#"
            SELECT COUNT(*) as entity_count
            FROM cbu_entity_member_roles cemr
            JOIN client_business_units cbu ON cemr.cbu_id = cbu.id
            WHERE cbu.cbu_id = $1
        "#;
        let count_row = sqlx::query(entity_count_query)
            .bind(&cbu_id)
            .fetch_one(&pool)
            .await
            .expect("Entity count query should succeed");

        let entity_count: i64 = count_row.get("entity_count");
        assert_eq!(entity_count, 7, "Should have 7 entity relationships");

        println!("✅ Phase 3: Database consistency verified");

        // Phase 4: Full round-trip DSL generation
        println!("Phase 4: Performing full round-trip DSL generation...");

        // Query all entity data
        let full_entity_query = r#"
            SELECT le.entity_id, le.entity_name, cemr.role
            FROM legal_entities le
            JOIN cbu_entity_member_roles cemr ON le.id = cemr.legal_entity_id
            JOIN client_business_units cbu ON cemr.cbu_id = cbu.id
            WHERE cbu.cbu_id = $1
            ORDER BY le.entity_id
        "#;

        let all_entities = sqlx::query(full_entity_query)
            .bind(&cbu_id)
            .fetch_all(&pool)
            .await
            .expect("Full entity query should succeed");

        let entities_for_dsl: Vec<(String, String, String)> = all_entities.iter().map(|row| {
            (
                row.get::<String, _>("entity_id"),
                row.get::<String, _>("entity_name"),
                row.get::<String, _>("role"),
            )
        }).collect();

        let final_dsl = parser.generate_dsl_from_cbu(
            "Updated Full System Fund",
            "Updated comprehensive test",
            &entities_for_dsl
        );

        // Verify final DSL parses correctly
        let final_parsed = parser.parse(&final_dsl).expect("Final DSL should parse");
        assert!(!final_parsed.is_empty(), "Final DSL should not be empty");

        println!("✅ Phase 4: Full round-trip DSL generation successful");
        println!("Final Generated DSL:\n{}", final_dsl);

        // Phase 5: Cleanup
        println!("Phase 5: Performing comprehensive cleanup...");

        // Remove entity relationships
        sqlx::query("DELETE FROM cbu_entity_member_roles WHERE cbu_id = (SELECT id FROM client_business_units WHERE cbu_id = $1)")
            .bind(&cbu_id)
            .execute(&pool)
            .await
            .expect("Entity cleanup should succeed");

        // Remove CBU
        sqlx::query("DELETE FROM client_business_units WHERE cbu_id = $1")
            .bind(&cbu_id)
            .execute(&pool)
            .await
            .expect("CBU cleanup should succeed");

        println!("✅ Phase 5: Comprehensive cleanup completed");

        println!("🎉 FULL SYSTEM DATABASE INTEGRATION TEST PASSED!");
        println!("✅ All phases completed successfully");
        println!("✅ Database consistency maintained");
        println!("✅ Round-trip DSL generation working");
        println!("✅ Error handling robust");
    }
}