// Test file to demonstrate DSL syntax highlighting capabilities
// This is a comprehensive test showing that our tokenizer correctly identifies
// all elements in actual DSL content from the resource templates

use crate::code_editor::{CodeEditor, TokenType};

const REAL_KYC_DSL: &str = r#"WORKFLOW "StandardClientKYC"

STEP "InitialAssessment"
    LOG "Starting KYC for client: " + client_legal_name
    DERIVE_REGULATORY_CONTEXT FOR_JURISDICTION client_jurisdiction WITH_PRODUCTS ["Trading"]
    ASSESS_RISK USING_FACTORS ["jurisdiction", "product", "client"] OUTPUT "risk_rating"
PROCEED_TO STEP "Screening"

STEP "Screening"
    SCREEN_ENTITY client_legal_name AGAINST "SanctionsList" THRESHOLD 0.85
    SCREEN_ENTITY client_legal_name AGAINST "PEPList" THRESHOLD 0.90
    STORE_RESULTS AS "screening_results"
PROCEED_TO STEP "DocumentCollection"

STEP "DocumentCollection"
    COLLECT_DOCUMENT "PassportCopy" FROM Client REQUIRED true
    COLLECT_DOCUMENT "ProofOfAddress" FROM Client REQUIRED true
    COLLECT_DOCUMENT "FinancialStatements" FROM Client REQUIRED false
PROCEED_TO STEP "Decision"

STEP "Decision"
    IF risk_rating == "High" THEN
        SET status TO "Review"
        FLAG_FOR_REVIEW "High risk client requires manual review" PRIORITY High
    ELSE IF screening_results.sanctions_match > 0.85 THEN
        SET status TO "Rejected"
        REJECT_CASE "Client found on sanctions list"
    ELSE
        SET status TO "Approved"
        APPROVE_CASE WITH_CONDITIONS ["Annual review required"]
    END_IF
    LOG "KYC completed for " + client_legal_name + " with status: " + status"#;

const BASELINE_DSL: &str = r#"WORKFLOW "DefaultWorkflow"

STEP "Start"
    # Add your logic here
    LOG "Starting workflow for case: " + case_id
PROCEED_TO STEP "End"

STEP "End"
    LOG "Workflow complete for case: " + case_id
    # Workflow complete"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyc_dsl_tokenization() {
        let mut editor = CodeEditor::default();
        editor.set_content(REAL_KYC_DSL.to_string());

        // Verify that keywords are properly identified
        let keyword_tokens: Vec<_> = editor.tokens.iter()
            .filter(|t| matches!(t.token_type, TokenType::Keyword))
            .collect();

        // Should find WORKFLOW, STEP, LOG, IF, THEN, ELSE, END_IF, etc.
        assert!(keyword_tokens.len() > 10, "Expected many keyword tokens, found {}", keyword_tokens.len());

        // Verify that commands are properly identified
        let command_tokens: Vec<_> = editor.tokens.iter()
            .filter(|t| matches!(t.token_type, TokenType::Command))
            .collect();

        // Should find DERIVE_REGULATORY_CONTEXT, ASSESS_RISK, SCREEN_ENTITY, etc.
        assert!(command_tokens.len() > 5, "Expected many command tokens, found {}", command_tokens.len());

        // Verify that strings are properly identified
        let string_tokens: Vec<_> = editor.tokens.iter()
            .filter(|t| matches!(t.token_type, TokenType::String))
            .collect();

        // Should find quoted strings like "StandardClientKYC", "InitialAssessment", etc.
        assert!(string_tokens.len() > 10, "Expected many string tokens, found {}", string_tokens.len());
    }

    #[test]
    fn test_baseline_dsl_tokenization() {
        let mut editor = CodeEditor::default();
        editor.set_content(BASELINE_DSL.to_string());

        // Verify comments are identified
        let comment_tokens: Vec<_> = editor.tokens.iter()
            .filter(|t| matches!(t.token_type, TokenType::Comment))
            .collect();

        assert!(comment_tokens.len() >= 2, "Expected at least 2 comment tokens, found {}", comment_tokens.len());
    }

    #[test]
    fn test_syntax_validation() {
        let mut editor = CodeEditor::default();
        editor.set_content(REAL_KYC_DSL.to_string());

        // The KYC DSL should validate successfully
        assert!(editor.last_parse_result.is_ok(), "KYC DSL should validate successfully");

        // Test invalid DSL
        editor.set_content("INVALID_SYNTAX without proper structure".to_string());
        assert!(editor.last_parse_result.is_err(), "Invalid DSL should fail validation");
    }

    #[test]
    fn test_specific_token_identification() {
        let mut editor = CodeEditor::default();
        editor.set_content("WORKFLOW \"Test\" STEP \"One\" DERIVE_REGULATORY_CONTEXT 0.85".to_string());

        // Check for specific tokens
        let has_workflow = editor.tokens.iter().any(|t| t.text == "WORKFLOW" && matches!(t.token_type, TokenType::Keyword));
        let has_step = editor.tokens.iter().any(|t| t.text == "STEP" && matches!(t.token_type, TokenType::Keyword));
        let has_command = editor.tokens.iter().any(|t| t.text == "DERIVE_REGULATORY_CONTEXT" && matches!(t.token_type, TokenType::Command));
        let has_string = editor.tokens.iter().any(|t| t.text == "\"Test\"" && matches!(t.token_type, TokenType::String));
        let has_number = editor.tokens.iter().any(|t| t.text == "0.85" && matches!(t.token_type, TokenType::Number));

        assert!(has_workflow, "Should identify WORKFLOW as keyword");
        assert!(has_step, "Should identify STEP as keyword");
        assert!(has_command, "Should identify DERIVE_REGULATORY_CONTEXT as command");
        assert!(has_string, "Should identify quoted strings");
        assert!(has_number, "Should identify numeric values");
    }

    #[test]
    fn test_complete_dsl_analysis() {
        let mut editor = CodeEditor::default();
        editor.set_content(REAL_KYC_DSL.to_string());

        println!("=== DSL SYNTAX ANALYSIS ===");
        println!("Total tokens: {}", editor.tokens.len());

        // Count tokens by type
        let mut type_counts = std::collections::HashMap::new();
        for token in &editor.tokens {
            *type_counts.entry(&token.token_type).or_insert(0) += 1;
        }

        for (token_type, count) in type_counts {
            println!("{:?}: {}", token_type, count);
        }

        println!("=== VALIDATION RESULT ===");
        match &editor.last_parse_result {
            Ok(msg) => println!("✅ Validation passed: {}", msg),
            Err(err) => println!("❌ Validation failed: {}", err),
        }

        // Ensure we have a reasonable distribution of tokens
        assert!(editor.tokens.len() > 50, "Should have many tokens for complex DSL");
    }
}

/// Public function to demonstrate DSL tokenization in the UI
pub fn demonstrate_dsl_analysis() -> String {
    let mut editor = CodeEditor::default();
    editor.set_content(REAL_KYC_DSL.to_string());

    let mut result = String::new();
    result.push_str("🚨 DSL SYNTAX HIGHLIGHTING DEMONSTRATION\n");
    result.push_str("========================================\n\n");

    result.push_str(&format!("📊 Total tokens identified: {}\n", editor.tokens.len()));

    // Group tokens by type and show counts
    let mut type_counts = std::collections::HashMap::new();
    for token in &editor.tokens {
        *type_counts.entry(&token.token_type).or_insert(0) += 1;
    }

    result.push_str("\n🏷️ Token Type Distribution:\n");
    for (token_type, count) in type_counts {
        result.push_str(&format!("   {:?}: {} tokens\n", token_type, count));
    }

    result.push_str("\n✅ Validation Status:\n");
    match &editor.last_parse_result {
        Ok(msg) => result.push_str(&format!("   ✅ PASSED: {}\n", msg)),
        Err(err) => result.push_str(&format!("   ❌ FAILED: {}\n", err)),
    }

    result.push_str("\n🎨 Sample tokens with syntax highlighting:\n");
    for (i, token) in editor.tokens.iter().take(10).enumerate() {
        result.push_str(&format!("   {}: {:?} = '{}'\n", i + 1, token.token_type, token.text));
    }

    if editor.tokens.len() > 10 {
        result.push_str(&format!("   ... and {} more tokens\n", editor.tokens.len() - 10));
    }

    result
}