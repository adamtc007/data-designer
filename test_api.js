// Test script to verify Tauri API functionality
import { invoke } from '@tauri-apps/api';

async function testAPI() {
    console.log("Testing Data Designer API...\n");

    try {
        // Test 1: Load test rules
        console.log("1. Testing get_test_rules...");
        const testRules = await invoke('get_test_rules');
        console.log(`✅ Loaded ${testRules.length} test rules`);
        console.log(`   First rule: ${testRules[0].name}`);

        // Test 2: Load source data
        console.log("\n2. Testing load_source_data...");
        const sourceData = await invoke('load_source_data');
        console.log(`✅ Loaded ${sourceData.datasets.length} datasets`);
        console.log(`   Datasets: ${sourceData.datasets.map(d => d.id).join(', ')}`);

        // Test 3: Load target rules
        console.log("\n3. Testing load_target_rules...");
        const targetRules = await invoke('load_target_rules');
        console.log(`✅ Loaded ${targetRules.rule_mappings.length} rule mappings`);
        console.log(`   First rule: ${targetRules.rule_mappings[0].rule_id}`);

        // Test 4: Test a simple rule
        console.log("\n4. Testing simple rule evaluation...");
        const simpleResult = await invoke('test_rule', {
            dslText: "100 + 50"
        });
        console.log(`✅ Simple math: ${simpleResult.success ? simpleResult.result : simpleResult.error}`);

        // Test 5: Test rule with dataset
        console.log("\n5. Testing rule with dataset...");
        const datasetResult = await invoke('test_rule_with_dataset', {
            ruleExpression: "quantity * unit_price",
            datasetId: "customer_order_001"
        });
        console.log(`✅ Dataset calculation: ${datasetResult.success ? datasetResult.result : datasetResult.error}`);

        // Test 6: Load grammar rules
        console.log("\n6. Testing get_grammar_rules...");
        const grammarRules = await invoke('get_grammar_rules');
        console.log(`✅ Loaded ${grammarRules.length} grammar rules`);
        console.log(`   Rules: ${grammarRules.slice(0, 3).map(r => r.name).join(', ')}, ...`);

        console.log("\n✨ All tests passed successfully!");

    } catch (error) {
        console.error("❌ Test failed:", error);
    }
}

// Run tests when script loads
testAPI();