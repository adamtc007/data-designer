pub mod models;
pub mod engine;
pub mod parser;
pub mod evaluator;
pub mod transpiler;

// Resource sheet orchestration system
pub mod resource_sheets;
pub mod orchestration_dsl;
pub mod kyc_dsl;

// Runtime execution system
pub mod runtime_orchestrator;

// Configuration
pub mod config;

// Database layer
pub mod db;
pub mod embeddings;
pub mod schema_visualizer;

// Capability execution engine
pub mod capability_engine;
pub mod capability_execution_engine;

// Onboarding orchestration engine
pub mod onboarding_orchestrator;

// CBU DSL for CRUD operations
pub mod cbu_dsl;

// LISP-based CBU DSL for list processing
pub mod lisp_cbu_dsl;

// Onboarding Request DSL for CRUD operations with Deal Record integration
pub mod onboarding_request_dsl;

// Deal Record DSL - Master orchestrator for comprehensive business relationship management
pub mod deal_record_dsl;

// Opportunity DSL for investment opportunity management
pub mod opportunity_dsl;

// Shared DSL utilities
pub mod dsl_utils;

// CBU DSL integration tests for API validation
#[cfg(test)]
pub mod cbu_dsl_integration_tests;

// S-expression DSL round trip tests
#[cfg(test)]
pub mod s_expression_round_trip_tests;

#[cfg(test)]
mod test_integration;
