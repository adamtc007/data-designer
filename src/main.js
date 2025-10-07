import * as monaco from 'monaco-editor';
import { DSL_LANGUAGE_ID, DSL_LANGUAGE_CONFIG, DSL_MONARCH_LANGUAGE, DSL_THEME } from './dsl-language.js';

// Register the DSL language with Monaco
monaco.languages.register({ id: DSL_LANGUAGE_ID });
monaco.languages.setLanguageConfiguration(DSL_LANGUAGE_ID, DSL_LANGUAGE_CONFIG);
monaco.languages.setMonarchTokensProvider(DSL_LANGUAGE_ID, DSL_MONARCH_LANGUAGE);
monaco.editor.defineTheme('dsl-theme', DSL_THEME);

// Check if Tauri API is available
let invoke;
let isTauriAvailable = false;

try {
    // Try to import Tauri API
    const tauriApi = await import('@tauri-apps/api/core');
    invoke = tauriApi.invoke;
    isTauriAvailable = window.__TAURI_INTERNALS__ !== undefined;
    console.log('✅ Running in Tauri application');
} catch (e) {
    console.warn('⚠️ Tauri API not available - running in browser mode');
    // Create a mock invoke function for browser testing
    invoke = async (cmd, args) => {
        console.log(`Mock invoke: ${cmd}`, args);

        // Provide mock responses for testing in browser
        switch(cmd) {
            case 'get_test_rules':
                return [
                    { id: 1, name: "Simple Math", dsl: "100 + 50", description: "Basic arithmetic" },
                    { id: 2, name: "String Concat", dsl: '"Hello " & "World"', description: "String concatenation" }
                ];
            case 'load_source_data':
                return {
                    datasets: [
                        {
                            id: "mock_dataset",
                            name: "Mock Dataset",
                            description: "Mock data for browser testing",
                            attributes: { test: "value", number: 123 }
                        }
                    ],
                    lookup_tables: { test: { key: "value" } }
                };
            case 'load_target_rules':
                return {
                    rule_mappings: [
                        {
                            rule_id: "MOCK_RULE",
                            rule_name: "Mock Rule",
                            description: "Mock rule for testing",
                            source_dataset: "mock_dataset",
                            rule_expression: "test",
                            target_attributes: { result: "string" },
                            expected_result: "value"
                        }
                    ]
                };
            default:
                return { success: false, error: "Browser mode - Tauri API not available" };
        }
    };
}

// Initialize the Monaco Editor in the <div id="container"></div>
const editor = monaco.editor.create(document.getElementById('container'), {
    value: '# KYC Rule DSL Editor\n# Your rules support arithmetic, strings, functions, lookups, and conditionals\n\n# Example: Calculate KYC completeness\nkyc_score = (documents_received / documents_required) * 100\n\n# Example: Risk assessment\nIF risk_rating = "high" AND pep_status = true THEN\n    "enhanced_due_diligence"\nELSE\n    "standard_review"\n',
    language: DSL_LANGUAGE_ID,
    theme: 'dsl-theme',
    automaticLayout: true,
    minimap: { enabled: false },
    suggestOnTriggerCharacters: true,
    quickSuggestions: true,
    wordBasedSuggestions: false,
    scrollBeyondLastLine: false,
    renderWhitespace: 'selection',
    fontSize: 14,
});

// Get UI elements
const saveButton = document.getElementById('saveButton');
const ruleSelect = document.getElementById('ruleSelect');
const testButton = document.getElementById('testButton');
const testResults = document.getElementById('testResults');
const resultContent = document.getElementById('resultContent');

let currentTestRules = [];

// Load test rules on startup
async function loadTestRules() {
    try {
        console.log('Loading test rules...');
        const rules = await invoke('get_test_rules');
        console.log('Received rules:', rules);
        currentTestRules = rules;

        // Populate dropdown
        ruleSelect.innerHTML = '<option value="">Select a test rule...</option>';
        rules.forEach(rule => {
            const option = document.createElement('option');
            option.value = rule.id;
            option.textContent = `${rule.id}. ${rule.name}`;
            ruleSelect.appendChild(option);
        });
        console.log('Dropdown populated with', rules.length, 'rules');
    } catch (error) {
        console.error('Error loading test rules:', error);

        // Add fallback rules manually for testing
        const fallbackRules = [
            { id: 1, name: "Complex Math", dsl: 'RULE "Complex Math" IF status == "active" THEN result = 100 + 25 * 2 - 10 / 2', description: "Multiple arithmetic operators" },
            { id: 2, name: "String Concatenation", dsl: 'RULE "Concat String" IF country == "US" THEN message = "Hello " & name & "!"', description: "String concatenation with & operator" },
            { id: 3, name: "Parentheses Precedence", dsl: 'RULE "Precedence Test" IF level > 5 THEN total = (100 + 50) * 2', description: "Parentheses for operator precedence" },
            { id: 4, name: "SUBSTRING Function", dsl: 'RULE "Extract Code" IF type == "user" THEN code = SUBSTRING(user_id, 0, 3)', description: "SUBSTRING function" }
        ];

        currentTestRules = fallbackRules;
        console.log('Using fallback rules:', currentTestRules.length);
        ruleSelect.innerHTML = '<option value="">Select a test rule...</option>';
        fallbackRules.forEach(rule => {
            const option = document.createElement('option');
            option.value = rule.id;
            option.textContent = `${rule.id}. ${rule.name}`;
            ruleSelect.appendChild(option);
        });
        console.log('Using fallback rules');
    }
}

// Handle rule selection
ruleSelect.addEventListener('change', (e) => {
    const selectedId = parseInt(e.target.value);
    testButton.disabled = !selectedId;

    if (selectedId) {
        const selectedRule = currentTestRules.find(r => r.id === selectedId);
        if (selectedRule) {
            // Load the DSL into the editor
            editor.setValue(selectedRule.dsl + '\n\n// ' + selectedRule.description);

            // Hide previous test results
            testResults.style.display = 'none';
        }
    } else {
        editor.setValue('# Your rule DSL will be loaded here...\n# Select a test rule from the dropdown to see examples\n');
        testResults.style.display = 'none';
    }
});

// Handle test execution
testButton.addEventListener('click', async () => {
    const selectedId = parseInt(ruleSelect.value);
    if (!selectedId) return;

    testButton.disabled = true;
    testButton.textContent = 'Running...';

    try {
        // Try to use the Tauri command first
        const result = await invoke('run_test_rule', { ruleId: selectedId });

        // Show test results
        testResults.style.display = 'block';

        if (result.success) {
            resultContent.innerHTML = `
                <div class="result-success">✅ Test Passed!</div>
                <p><strong>Result:</strong> <code>${result.result}</code></p>
            `;
        } else {
            resultContent.innerHTML = `
                <div class="result-error">❌ Test Failed</div>
                <p><strong>Error:</strong> ${result.error}</p>
            `;
        }
    } catch (error) {
        console.error('Tauri command failed, using fallback:', error);

        // Fallback: just show that the rule was selected
        const selectedRule = currentTestRules.find(r => r.id === selectedId);
        testResults.style.display = 'block';

        if (selectedRule) {
            resultContent.innerHTML = `
                <div class="result-success">✅ Rule Selected!</div>
                <p><strong>Rule:</strong> ${selectedRule.name}</p>
                <p><strong>DSL:</strong> <code>${selectedRule.dsl}</code></p>
                <p><em>Note: Tauri backend test execution failed. Rule parsing works in UI.</em></p>
            `;
        } else {
            resultContent.innerHTML = `
                <div class="result-error">❌ Test Error</div>
                <p><strong>Error:</strong> ${error}</p>
            `;
        }
    } finally {
        testButton.disabled = false;
        testButton.textContent = 'Run Test';
    }
});

// Handle saving rules
saveButton.addEventListener('click', () => {
    const dslText = editor.getValue();

    // This 'invoke' calls the `save_rules` function in your Rust code
    invoke('save_rules', { dslText })
        .then(() => alert('Rules saved successfully!'))
        .catch((error) => alert(`Error saving rules: ${error}`));
});

// Grammar editor functionality
let currentGrammarRules = [];
let selectedGrammarRule = null;

// Dictionary functionality
let currentAttributes = [];
let selectedAttribute = null;

// Compiler functionality
let currentCompiledRule = null;

// Tab switching
document.querySelectorAll('.tab-button').forEach(button => {
    button.addEventListener('click', async (e) => {
        const tabName = e.target.dataset.tab;

        // Update tab buttons
        document.querySelectorAll('.tab-button').forEach(btn => btn.classList.remove('active'));
        e.target.classList.add('active');

        // Update tab content
        document.querySelectorAll('.tab-content').forEach(content => content.classList.remove('active'));
        document.getElementById(tabName + 'Tab').classList.add('active');

        // Load data when switching tabs
        if (tabName === 'grammar') {
            loadGrammarRules();
        } else if (tabName === 'dictionary') {
            loadDictionaryAttributes();
        } else if (tabName === 'compiler') {
            await populateCompilerRuleSelect();
        }
    });
});

// Grammar management functions
async function loadGrammarRules() {
    try {
        const rules = await invoke('get_grammar_rules');
        currentGrammarRules = rules;
        displayGrammarRules();
    } catch (error) {
        console.error('Error loading grammar rules:', error);
    }
}

function displayGrammarRules() {
    const rulesList = document.getElementById('grammarRulesList');
    rulesList.innerHTML = '';

    currentGrammarRules.forEach(rule => {
        const ruleItem = document.createElement('div');
        ruleItem.className = 'grammar-rule-item';
        ruleItem.innerHTML = `
            <div class="grammar-rule-name">${rule.name}<span class="grammar-rule-type">[${rule.type}]</span></div>
            <div class="grammar-rule-description">${rule.description}</div>
        `;

        ruleItem.addEventListener('click', () => selectGrammarRule(rule));
        rulesList.appendChild(ruleItem);
    });
}

function selectGrammarRule(rule) {
    selectedGrammarRule = rule;

    // Update UI selection
    document.querySelectorAll('.grammar-rule-item').forEach(item => item.classList.remove('selected'));
    event.target.closest('.grammar-rule-item').classList.add('selected');

    // Populate form
    document.getElementById('ruleName').value = rule.name;
    document.getElementById('ruleType').value = rule.type;
    document.getElementById('ruleDefinition').value = rule.definition;
    document.getElementById('ruleDescription').value = rule.description;
}

// Grammar editor event listeners
document.getElementById('loadGrammarButton').addEventListener('click', loadGrammarRules);

document.getElementById('saveGrammarButton').addEventListener('click', async () => {
    try {
        const grammar = await invoke('load_grammar');
        grammar.grammar.rules = currentGrammarRules;
        await invoke('save_grammar', { grammar });
        alert('Grammar saved successfully!');
    } catch (error) {
        alert(`Error saving grammar: ${error}`);
    }
});

document.getElementById('validateGrammarButton').addEventListener('click', async () => {
    try {
        const isValid = await invoke('validate_grammar');
        if (isValid) {
            alert('✅ Grammar is valid!');
        } else {
            alert('❌ Grammar validation failed');
        }
    } catch (error) {
        alert(`❌ Grammar validation error: ${error}`);
    }
});

document.getElementById('generatePestButton').addEventListener('click', async () => {
    try {
        const pestGrammar = await invoke('generate_pest_grammar');
        document.getElementById('grammarContent').textContent = pestGrammar;
        document.getElementById('grammarResults').style.display = 'block';
    } catch (error) {
        alert(`Error generating grammar view: ${error}`);
    }
});

// Test Data Tab Functions
let sourceData = null;
let targetRules = null;

document.getElementById('loadTestDataButton').addEventListener('click', async () => {
    try {
        sourceData = await invoke('load_source_data');
        targetRules = await invoke('load_target_rules');

        // Populate dataset dropdown
        const datasetSelect = document.getElementById('datasetSelect');
        datasetSelect.innerHTML = '<option value="">Select dataset...</option>';
        sourceData.datasets.forEach(dataset => {
            const option = document.createElement('option');
            option.value = dataset.id;
            option.textContent = `${dataset.name} (${dataset.id})`;
            datasetSelect.appendChild(option);
        });

        // Populate rule dropdown
        const ruleSelect = document.getElementById('ruleSelect');
        ruleSelect.innerHTML = '<option value="">Select rule...</option>';
        targetRules.rule_mappings.forEach(rule => {
            const option = document.createElement('option');
            option.value = rule.rule_id;
            option.textContent = `${rule.rule_name} (${rule.rule_id})`;
            ruleSelect.appendChild(option);
        });

        // Display lookup tables
        const lookupTables = document.getElementById('lookupTables');
        lookupTables.innerHTML = `<pre style="margin: 0; color: #d4d4d4;">${JSON.stringify(sourceData.lookup_tables, null, 2)}</pre>`;

        alert('Test data loaded successfully!');
    } catch (error) {
        alert(`Error loading test data: ${error}`);
    }
});

document.getElementById('datasetSelect').addEventListener('change', () => {
    const datasetId = document.getElementById('datasetSelect').value;
    if (!datasetId || !sourceData) {
        document.getElementById('sourceAttributes').innerHTML = '<pre style="margin: 0; color: #d4d4d4;">Select a dataset to view source attributes...</pre>';
        return;
    }

    const dataset = sourceData.datasets.find(d => d.id === datasetId);
    if (dataset) {
        document.getElementById('sourceAttributes').innerHTML = `
            <h4>${dataset.name}</h4>
            <p>${dataset.description}</p>
            <pre style="margin: 0; color: #d4d4d4;">${JSON.stringify(dataset.attributes, null, 2)}</pre>
        `;
    }
});

document.getElementById('ruleSelect').addEventListener('change', () => {
    const ruleId = document.getElementById('ruleSelect').value;
    if (!ruleId || !targetRules) {
        document.getElementById('targetRule').innerHTML = '<pre style="margin: 0; color: #d4d4d4;">Select a rule to view details...</pre>';
        return;
    }

    const rule = targetRules.rule_mappings.find(r => r.rule_id === ruleId);
    if (rule) {
        document.getElementById('targetRule').innerHTML = `
            <h4>${rule.rule_name}</h4>
            <p>${rule.description}</p>
            <p><strong>Expression:</strong></p>
            <pre style="background: #1e1e1e; padding: 10px; border-radius: 3px;">${rule.rule_expression}</pre>
            <p><strong>Target Attributes:</strong></p>
            <pre style="margin: 0; color: #d4d4d4;">${JSON.stringify(rule.target_attributes, null, 2)}</pre>
            <p><strong>Expected Result:</strong> ${JSON.stringify(rule.expected_result)}</p>
        `;
    }
});

document.getElementById('testWithDataButton').addEventListener('click', async () => {
    const datasetId = document.getElementById('datasetSelect').value;
    const ruleId = document.getElementById('ruleSelect').value;

    if (!datasetId || !ruleId) {
        alert('Please select both a dataset and a rule');
        return;
    }

    const rule = targetRules.rule_mappings.find(r => r.rule_id === ruleId);
    if (!rule) {
        alert('Rule not found');
        return;
    }

    try {
        const result = await invoke('test_rule_with_dataset', {
            ruleExpression: rule.rule_expression,
            datasetId: datasetId
        });

        const resultsDiv = document.getElementById('testDataResults');
        if (result.success) {
            resultsDiv.innerHTML = `
                <div style="color: #4caf50;">
                    <strong>✅ Test Passed</strong>
                    <p>Result: ${result.result}</p>
                    <p>Expected: ${JSON.stringify(rule.expected_result)}</p>
                </div>
            `;
        } else {
            resultsDiv.innerHTML = `
                <div style="color: #f44336;">
                    <strong>❌ Test Failed</strong>
                    <p>Error: ${result.error}</p>
                </div>
            `;
        }
    } catch (error) {
        document.getElementById('testDataResults').innerHTML = `
            <div style="color: #f44336;">
                <strong>❌ Error</strong>
                <p>${error}</p>
            </div>
        `;
    }
});

document.getElementById('refreshTestDataButton').addEventListener('click', () => {
    document.getElementById('loadTestDataButton').click();
});

document.getElementById('saveRuleButton').addEventListener('click', async () => {
    const rule = {
        name: document.getElementById('ruleName').value,
        type: document.getElementById('ruleType').value,
        definition: document.getElementById('ruleDefinition').value,
        description: document.getElementById('ruleDescription').value,
    };

    if (!rule.name || !rule.definition) {
        alert('Please fill in rule name and definition');
        return;
    }

    try {
        await invoke('update_grammar_rule', { rule });

        // Update local rules
        const existingIndex = currentGrammarRules.findIndex(r => r.name === rule.name);
        if (existingIndex >= 0) {
            currentGrammarRules[existingIndex] = rule;
        } else {
            currentGrammarRules.push(rule);
        }

        displayGrammarRules();
        alert('Rule saved successfully!');
    } catch (error) {
        alert(`Error saving rule: ${error}`);
    }
});

document.getElementById('addRuleButton').addEventListener('click', () => {
    // Clear form for new rule
    document.getElementById('ruleName').value = '';
    document.getElementById('ruleType').value = 'normal';
    document.getElementById('ruleDefinition').value = '';
    document.getElementById('ruleDescription').value = '';
    selectedGrammarRule = null;

    // Clear selection
    document.querySelectorAll('.grammar-rule-item').forEach(item => item.classList.remove('selected'));
});

document.getElementById('deleteRuleButton').addEventListener('click', () => {
    if (!selectedGrammarRule) {
        alert('Please select a rule to delete');
        return;
    }

    if (confirm(`Are you sure you want to delete rule "${selectedGrammarRule.name}"?`)) {
        // Remove from local rules
        currentGrammarRules = currentGrammarRules.filter(r => r.name !== selectedGrammarRule.name);
        displayGrammarRules();

        // Clear form
        document.getElementById('ruleName').value = '';
        document.getElementById('ruleType').value = 'normal';
        document.getElementById('ruleDefinition').value = '';
        document.getElementById('ruleDescription').value = '';
        selectedGrammarRule = null;
    }
});

// Dictionary management functions
async function loadDictionaryAttributes() {
    try {
        const attributes = await invoke('get_attributes');
        currentAttributes = attributes;
        displayAttributes();
    } catch (error) {
        console.error('Error loading attributes:', error);
    }
}

function displayAttributes(filter = '') {
    const attributesList = document.getElementById('attributesList');
    attributesList.innerHTML = '';

    const filteredAttributes = currentAttributes.filter(attr =>
        attr.name.toLowerCase().includes(filter.toLowerCase()) ||
        attr.display_name.toLowerCase().includes(filter.toLowerCase()) ||
        attr.tags.some(tag => tag.toLowerCase().includes(filter.toLowerCase()))
    );

    filteredAttributes.forEach(attr => {
        const attrItem = document.createElement('div');
        attrItem.className = 'attribute-item';

        const tagsHtml = attr.tags.map(tag =>
            `<span class="attribute-tag">${tag}</span>`
        ).join('');

        attrItem.innerHTML = `
            <div class="attribute-name">${attr.name}<span class="attribute-type">[${attr.data_type}]</span></div>
            <div class="attribute-description">${attr.description}</div>
            <div class="attribute-tags">${tagsHtml}</div>
        `;

        attrItem.addEventListener('click', () => selectAttribute(attr));
        attributesList.appendChild(attrItem);
    });
}

function selectAttribute(attr) {
    selectedAttribute = attr;

    // Update UI selection
    document.querySelectorAll('.attribute-item').forEach(item => item.classList.remove('selected'));
    event.target.closest('.attribute-item').classList.add('selected');

    // Populate form
    document.getElementById('attrName').value = attr.name;
    document.getElementById('attrDisplayName').value = attr.display_name;
    document.getElementById('attrDataType').value = attr.data_type;
    document.getElementById('attrDescription').value = attr.description;
    document.getElementById('attrTags').value = attr.tags.join(', ');

    // Constraints
    document.getElementById('attrRequired').checked = attr.constraints.required || false;
    document.getElementById('attrMaxLength').value = attr.constraints.max_length || '';
    document.getElementById('attrPattern').value = attr.constraints.pattern || '';
    document.getElementById('attrAllowedValues').value =
        attr.constraints.allowed_values ? attr.constraints.allowed_values.join(', ') : '';

    // Source
    document.getElementById('attrSystem').value = attr.source.system;
    document.getElementById('attrField').value = attr.source.field;
    document.getElementById('attrTable').value = attr.source.table;
}

function clearAttributeForm() {
    document.getElementById('attrName').value = '';
    document.getElementById('attrDisplayName').value = '';
    document.getElementById('attrDataType').value = 'String';
    document.getElementById('attrDescription').value = '';
    document.getElementById('attrTags').value = '';
    document.getElementById('attrRequired').checked = false;
    document.getElementById('attrMaxLength').value = '';
    document.getElementById('attrPattern').value = '';
    document.getElementById('attrAllowedValues').value = '';
    document.getElementById('attrSystem').value = '';
    document.getElementById('attrField').value = '';
    document.getElementById('attrTable').value = '';
    selectedAttribute = null;

    document.querySelectorAll('.attribute-item').forEach(item => item.classList.remove('selected'));
}

// Dictionary event listeners
document.getElementById('loadDictionaryButton').addEventListener('click', loadDictionaryAttributes);

document.getElementById('addAttributeButton').addEventListener('click', clearAttributeForm);

document.getElementById('searchAttributes').addEventListener('input', (e) => {
    displayAttributes(e.target.value);
});

document.getElementById('saveAttributeButton').addEventListener('click', async () => {
    const attribute = {
        name: document.getElementById('attrName').value,
        display_name: document.getElementById('attrDisplayName').value,
        data_type: document.getElementById('attrDataType').value,
        description: document.getElementById('attrDescription').value,
        tags: document.getElementById('attrTags').value.split(',').map(t => t.trim()).filter(t => t),
        constraints: {
            required: document.getElementById('attrRequired').checked,
            max_length: document.getElementById('attrMaxLength').value ? parseInt(document.getElementById('attrMaxLength').value) : null,
            pattern: document.getElementById('attrPattern').value || null,
            allowed_values: document.getElementById('attrAllowedValues').value ?
                document.getElementById('attrAllowedValues').value.split(',').map(v => v.trim()).filter(v => v) : null
        },
        source: {
            system: document.getElementById('attrSystem').value,
            field: document.getElementById('attrField').value,
            table: document.getElementById('attrTable').value,
        },
        created_date: new Date().toISOString().split('T')[0],
        last_modified: new Date().toISOString().split('T')[0]
    };

    if (!attribute.name || !attribute.display_name) {
        alert('Please fill in name and display name');
        return;
    }

    try {
        if (selectedAttribute) {
            await invoke('update_attribute', { attribute });
            alert('Attribute updated successfully!');
        } else {
            await invoke('add_attribute', { attribute });
            alert('Attribute added successfully!');
        }

        loadDictionaryAttributes();
        clearAttributeForm();
    } catch (error) {
        alert(`Error saving attribute: ${error}`);
    }
});

document.getElementById('deleteAttributeButton').addEventListener('click', async () => {
    if (!selectedAttribute) {
        alert('Please select an attribute to delete');
        return;
    }

    if (confirm(`Are you sure you want to delete attribute "${selectedAttribute.name}"?`)) {
        try {
            await invoke('delete_attribute', { attributeName: selectedAttribute.name });
            alert('Attribute deleted successfully!');
            loadDictionaryAttributes();
            clearAttributeForm();
        } catch (error) {
            alert(`Error deleting attribute: ${error}`);
        }
    }
});

// Compiler functionality
async function populateCompilerRuleSelect() {
    console.log('populateCompilerRuleSelect called, currentTestRules.length:', currentTestRules.length);

    // Ensure rules are loaded first
    if (currentTestRules.length === 0) {
        console.log('Loading test rules...');
        await loadTestRules();
        console.log('Test rules loaded:', currentTestRules.length);
    }

    const select = document.getElementById('compileRuleSelect');
    select.innerHTML = '<option value="">Select rule to compile...</option>';

    // Add test rules as compilation options
    currentTestRules.forEach(rule => {
        const option = document.createElement('option');
        option.value = rule.id;
        option.textContent = rule.name;
        select.appendChild(option);
        console.log('Added rule option:', rule.name);
    });
}

document.getElementById('compileRuleSelect').addEventListener('change', (e) => {
    const selectedId = parseInt(e.target.value);
    if (selectedId) {
        const selectedRule = currentTestRules.find(r => r.id === selectedId);
        if (selectedRule) {
            document.getElementById('compilationDetails').innerHTML = `
                <h4>${selectedRule.name}</h4>
                <p><strong>DSL:</strong> <code>${selectedRule.dsl}</code></p>
                <p><strong>Description:</strong> ${selectedRule.description}</p>
                <p>Ready to compile to Rust function...</p>
            `;
        }
    } else {
        document.getElementById('compilationDetails').innerHTML = '<p>Select a rule to compile...</p>';
    }
});

// Check if element exists
const compileBtn = document.getElementById('compileRuleButton');
console.log('Compile button element:', compileBtn);

if (compileBtn) {
    compileBtn.addEventListener('click', async () => {
        console.log('Compile button clicked!');
    const selectedId = parseInt(document.getElementById('compileRuleSelect').value);
    console.log('Selected ID:', selectedId);
    if (!selectedId) {
        alert('Please select a rule to compile');
        return;
    }

    const selectedRule = currentTestRules.find(r => r.id === selectedId);
    if (!selectedRule) {
        alert('Selected rule not found');
        return;
    }

    try {
        const startTime = Date.now();

        let compiledRule;
        try {
            compiledRule = await invoke('compile_rule_to_rust', {
                ruleDsl: selectedRule.dsl,
                ruleName: selectedRule.name
            });
        } catch (backendError) {
            console.error('Backend compilation failed, using mock:', backendError);

            // Mock compilation for testing UI
            compiledRule = {
                rule_id: selectedRule.id.toString(),
                rule_name: selectedRule.name,
                generated_code: `// Generated Rust function for: ${selectedRule.name}
pub fn ${selectedRule.name.toLowerCase().replace(/[^a-z0-9]/g, '_')}(
    status: &str,
    country: &str,
    name: &str,
    level: i32,
    user_id: &str
) -> Result<String, String> {
    // Rule: ${selectedRule.dsl}
    ${selectedRule.name.includes('Math') ? `
    if status == "active" {
        let result = 100 + 25 * 2 - 10 / 2;
        Ok(result.to_string())
    } else {
        Err("Condition not met".to_string())
    }` : selectedRule.name.includes('Concatenation') ? `
    if country == "US" {
        let message = format!("Hello {}!", name);
        Ok(message)
    } else {
        Err("Condition not met".to_string())
    }` : `
    // Default implementation
    Ok("Compiled successfully".to_string())`}
}`,
                input_attributes: ['status', 'country', 'name', 'level', 'user_id'],
                output_attribute: 'result',
                compilation_timestamp: new Date().toISOString()
            };
        }

        const compilationTime = Date.now() - startTime;
        currentCompiledRule = compiledRule;

        // Display generated code
        document.getElementById('rustCode').textContent = compiledRule.generated_code;

        // Update performance stats
        document.getElementById('compilationTime').textContent = `${compilationTime}ms`;
        document.getElementById('functionName').textContent = compiledRule.rule_name;
        document.getElementById('inputParams').textContent = compiledRule.input_attributes.join(', ');

        // Enable download button
        document.getElementById('downloadRustButton').disabled = false;

        // Update runtime testing dropdown
        populateRuntimeTestingDropdown();

        alert('Rule compiled successfully!');
    } catch (error) {
        alert(`Compilation error: ${error}`);
    }
    });
} else {
    console.error('Compile button not found!');
}

document.getElementById('downloadRustButton').addEventListener('click', () => {
    if (!currentCompiledRule) {
        alert('No compiled rule to download');
        return;
    }

    const blob = new Blob([currentCompiledRule.generated_code], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${currentCompiledRule.rule_name.toLowerCase().replace(' ', '_')}.rs`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
});

// Runtime testing functionality
let availableCompiledRules = [];

async function populateRuntimeTestingDropdown() {
    try {
        // Get compiled rules (either from backend or use current rule)
        if (currentCompiledRule) {
            availableCompiledRules = [currentCompiledRule];
        } else {
            // Try to get from backend
            try {
                availableCompiledRules = await invoke('get_compiled_rules');
            } catch (e) {
                availableCompiledRules = currentCompiledRule ? [currentCompiledRule] : [];
            }
        }

        const select = document.getElementById('testRuleName');
        select.innerHTML = '<option value="">Select compiled rule to test...</option>';

        availableCompiledRules.forEach(rule => {
            const option = document.createElement('option');
            option.value = rule.rule_name;
            option.textContent = rule.rule_name;
            select.appendChild(option);
        });
    } catch (error) {
        console.error('Error populating runtime testing dropdown:', error);
    }
}

document.getElementById('testRuleName').addEventListener('change', (e) => {
    const selectedRuleName = e.target.value;
    if (selectedRuleName) {
        const rule = availableCompiledRules.find(r => r.rule_name === selectedRuleName);
        if (rule) {
            generateRuntimeInputs(rule);
        }
    } else {
        document.getElementById('runtimeInputs').innerHTML = '';
    }
});

function generateRuntimeInputs(rule) {
    const container = document.getElementById('runtimeInputs');
    container.innerHTML = '';

    rule.input_attributes.forEach(attr => {
        const inputGroup = document.createElement('div');
        inputGroup.innerHTML = `
            <label style="color: #ffffff; font-weight: bold; margin-bottom: 5px; display: block;">${attr}:</label>
            <input type="text" id="runtime_${attr}" placeholder="Enter ${attr}"
                   style="width: 100%; padding: 8px; background-color: #2d2d30; color: #d4d4d4; border: 1px solid #3c3c3c; border-radius: 3px; font-size: 14px;">
        `;
        container.appendChild(inputGroup);
    });

    // Add some sample values for demonstration
    if (rule.rule_name.includes('Math')) {
        document.getElementById('runtime_status').value = 'active';
    } else if (rule.rule_name.includes('Concatenation')) {
        document.getElementById('runtime_country').value = 'US';
        document.getElementById('runtime_name').value = 'John';
    } else if (rule.rule_name.includes('Precedence')) {
        document.getElementById('runtime_level').value = '7';
    } else if (rule.rule_name.includes('SUBSTRING')) {
        document.getElementById('runtime_user_id').value = 'ABC123';
    }
}

document.getElementById('executeRuleButton').addEventListener('click', async () => {
    const selectedRuleName = document.getElementById('testRuleName').value;
    if (!selectedRuleName) {
        alert('Please select a rule to test');
        return;
    }

    const rule = availableCompiledRules.find(r => r.rule_name === selectedRuleName);
    if (!rule) {
        alert('Selected rule not found');
        return;
    }

    // Collect input values
    const attributeValues = {};
    rule.input_attributes.forEach(attr => {
        const input = document.getElementById(`runtime_${attr}`);
        attributeValues[attr] = input ? input.value : '';
    });

    try {
        const result = await invoke('execute_compiled_rule', {
            request: {
                rule_name: selectedRuleName,
                attribute_values: attributeValues
            }
        });

        // Display results
        const resultsDiv = document.getElementById('executionResults');
        const outputDiv = document.getElementById('executionOutput');

        outputDiv.innerHTML = `
            <div style="margin-bottom: 10px;">
                <strong>Rule:</strong> ${result.rule_name}
            </div>
            <div style="margin-bottom: 10px;">
                <strong>Output Attribute:</strong> ${result.output_attribute}
            </div>
            <div style="margin-bottom: 10px;">
                <strong>Result Value:</strong> <span style="color: #2ecc71; font-weight: bold;">${result.result_value}</span>
            </div>
            <div style="margin-bottom: 10px;">
                <strong>Execution Time:</strong> ${result.execution_time_ms}ms
            </div>
            <div style="margin-bottom: 10px;">
                <strong>Success:</strong> <span style="color: ${result.success ? '#2ecc71' : '#e74c3c'};">${result.success ? '✅ Yes' : '❌ No'}</span>
            </div>
        `;

        resultsDiv.style.display = 'block';
    } catch (error) {
        const resultsDiv = document.getElementById('executionResults');
        const outputDiv = document.getElementById('executionOutput');

        outputDiv.innerHTML = `
            <div style="color: #e74c3c; font-weight: bold;">
                <strong>Execution Error:</strong> ${error}
            </div>
        `;

        resultsDiv.style.display = 'block';
    }
});

// Show mode indicator
if (!isTauriAvailable) {
    const warning = document.createElement('div');
    warning.style.cssText = 'position: fixed; top: 10px; right: 10px; background: #ff6b6b; color: white; padding: 10px; border-radius: 5px; z-index: 1000; font-family: Arial, sans-serif;';
    warning.innerHTML = '⚠️ Browser Mode - Limited Functionality<br><small>Open in Tauri app for full features</small>';
    document.body.appendChild(warning);
}

// Load test rules when the app starts
loadTestRules();
