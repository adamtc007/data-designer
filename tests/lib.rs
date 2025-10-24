/// Integration test suite for Data Designer
///
/// This test suite demonstrates the comprehensive testing strategy:
/// - Elasticsearch integration for debugging
/// - End-to-end data flow testing
/// - Performance benchmarking
/// - Error handling verification
/// - Service integration testing

pub mod common;
pub mod unit;
pub mod integration;
pub mod e2e;

// Re-export test harness for convenience
pub use test_harness::*;

/// Test configuration
pub struct TestConfig {
    pub elasticsearch_url: String,
    pub database_url: String,
    pub grpc_port: u16,
    pub cleanup_on_failure: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            elasticsearch_url: std::env::var("TEST_ELASTICSEARCH_URL")
                .unwrap_or_else(|_| "http://localhost:9200".to_string()),
            database_url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://adamtc007@localhost/data_designer_test".to_string()),
            grpc_port: 50051,
            cleanup_on_failure: true,
        }
    }
}

/// Test utilities module
pub mod test_utils {
    use super::*;

    /// Initialize test environment
    pub async fn init_test_env() -> anyhow::Result<TestHarness> {
        // Set up logging for tests
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .try_init();

        TestHarness::setup().await
    }

    /// Cleanup test environment
    pub async fn cleanup_test_env(harness: TestHarness) -> anyhow::Result<()> {
        harness.cleanup().await
    }

    /// Assert test completion with metrics
    pub async fn assert_test_success(harness: &TestHarness, test_name: &str) -> anyhow::Result<()> {
        let metrics = harness.get_test_metrics().await?;

        assert!(
            metrics.failed_tests == 0,
            "Test '{}' had {} failures",
            test_name,
            metrics.failed_tests
        );

        println!("✅ Test '{}' completed successfully", test_name);
        println!("📊 Metrics: {} passed, {} failed",
                metrics.passed_tests, metrics.failed_tests);

        Ok(())
    }
}