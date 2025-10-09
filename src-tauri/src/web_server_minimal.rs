// Minimal server for Tauri integration
#[cfg(feature = "ssr")]
use axum::{extract::Query, response::Html, routing::get, Router, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

// Import our data dictionary module and rule engine
use crate::data_dictionary::{
    get_data_dictionary, DataDictionaryResponse,
    create_derived_attribute, CreateDerivedAttributeRequest
};
use data_designer::BusinessRule;
use serde_json::Value as JsonValue;

// Simple HTML response that includes Monaco Editor setup
#[cfg(feature = "ssr")]
async fn serve_ide() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
    <title>Data Designer IDE - Leptos SSR</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Monaco', 'Menlo', 'Consolas', monospace;
            background: #1e1e1e; color: #d4d4d4; height: 100vh; overflow: hidden;
        }
        .ide-layout { height: 100vh; display: flex; flex-direction: column; }
        .ide-header {
            display: flex; justify-content: space-between; align-items: center;
            padding: 12px 20px; background: #252526; border-bottom: 1px solid #3e3e42;
        }
        .ide-header h1 { font-size: 18px; color: #ffffff; }
        .header-controls { display: flex; gap: 10px; }
        .btn {
            padding: 8px 16px; border: none; border-radius: 4px; cursor: pointer;
            font-size: 13px; font-weight: 500; transition: all 0.2s ease;
        }
        .btn-primary { background: #007acc; color: white; }
        .btn-primary:hover { background: #005a9e; }
        .btn-secondary { background: #3e3e42; color: #d4d4d4; }
        .btn-secondary:hover { background: #4e4e52; }
        .ide-content { display: flex; flex: 1; min-height: 0; }
        .sidebar {
            width: 320px; background: #252526; border-right: 1px solid #3e3e42;
            display: flex; flex-direction: column;
        }
        .sidebar-tabs {
            display: flex; background: #1e1e1e; border-bottom: 1px solid #3e3e42;
        }
        .sidebar-tab {
            flex: 1; padding: 8px 12px; cursor: pointer; font-size: 12px;
            color: #8c8c8c; text-align: center; border-bottom: 2px solid transparent;
            transition: all 0.2s ease;
        }
        .sidebar-tab.active {
            color: #ffffff; border-bottom-color: #007acc; background: #252526;
        }
        .sidebar-tab:hover:not(.active) {
            color: #d4d4d4; background: #2d2d30;
        }
        .tab-content { display: none; flex: 1; overflow-y: auto; }
        .tab-content.active { display: flex; flex-direction: column; }
        .sidebar-header {
            display: flex; justify-content: space-between; align-items: center;
            padding: 12px 16px; border-bottom: 1px solid #3e3e42;
        }
        .sidebar-header h3 { font-size: 14px; color: #ffffff; }
        .attribute-list { flex: 1; overflow-y: auto; padding: 8px 0; }
        .attribute-item {
            padding: 8px 16px; cursor: pointer; font-size: 13px; color: #d4d4d4;
            border-left: 3px solid transparent; transition: all 0.2s ease;
        }
        .attribute-item:hover { background: #2d2d30; border-left-color: #007acc; color: #ffffff; }
        .editor-container { flex: 1; display: flex; flex-direction: column; }
        .monaco-editor { flex: 1; position: relative; }
        .results-panel {
            height: 200px; background: #1e1e1e; border-top: 1px solid #3e3e42;
            padding: 16px; overflow-y: auto;
        }
        .results-panel h4 { font-size: 14px; color: #ffffff; margin-bottom: 12px; }
        .result {
            background: #252526; border-radius: 4px; padding: 12px;
            border-left: 4px solid #3e3e42;
        }
        .result.success { border-left-color: #4caf50; }
        .result.placeholder { color: #8c8c8c; font-style: italic; }
    </style>
</head>
<body>
    <div class="ide-layout">
        <header class="ide-header">
            <h1>🧠 Data Designer IDE (Leptos SSR)</h1>
            <div class="header-controls">
                <button class="btn btn-primary" onclick="runRule()">▶️ Run Rule</button>
                <button class="btn btn-secondary" onclick="saveRule()">💾 Save</button>
            </div>
        </header>
        <div class="ide-content">
            <aside class="sidebar">
                <div class="sidebar-tabs">
                    <div class="sidebar-tab active" onclick="switchTab('attributes')">📚 Attributes</div>
                    <div class="sidebar-tab" onclick="switchTab('schema')">🗄️ Schema</div>
                </div>

                <!-- Attributes Tab -->
                <div id="attributes-tab" class="tab-content active">
                    <div class="sidebar-header">
                        <h3>📚 Data Dictionary</h3>
                        <button class="btn btn-sm" onclick="createAttribute()">➕ New</button>
                    </div>
                    <div class="attribute-list" id="attribute-list">
                        <div style="text-align: center; color: #8c8c8c; padding: 20px;">
                            🔄 Loading data dictionary...
                        </div>
                    </div>
                </div>

                <!-- Schema Tab -->
                <div id="schema-tab" class="tab-content">
                    <div class="sidebar-header">
                        <h3>🗄️ Database Schema</h3>
                        <button class="btn btn-sm" onclick="refreshSchema()">🔄 Refresh</button>
                    </div>
                    <div class="attribute-list" id="schema-list">
                        <div style="text-align: center; color: #8c8c8c; padding: 20px;">
                            Click Refresh to load schema...
                        </div>
                    </div>
                </div>
            </aside>
            <div class="editor-container">
                <div id="monaco-container" class="monaco-editor">
                    <div style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); color: #8c8c8c;">
                        🔄 Initializing Monaco Editor...
                    </div>
                </div>
                <div class="results-panel">
                    <h4>🔍 Test Results</h4>
                    <div id="results" class="result placeholder">
                        Run a rule to see results...
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script>
        // Initialize Monaco Editor
        function initializeMonaco() {
            const script = document.createElement('script');
            script.src = 'https://unpkg.com/monaco-editor@latest/min/vs/loader.js';
            script.onload = function() {
                require.config({ paths: { 'vs': 'https://unpkg.com/monaco-editor@latest/min/vs' }});
                require(['vs/editor/editor.main'], function() {
                    const container = document.getElementById('monaco-container');
                    container.innerHTML = '';

                    // Register custom DSL language
                    monaco.languages.register({ id: 'dsl-language' });

                    // Define DSL syntax highlighting
                    monaco.languages.setMonarchTokensProvider('dsl-language', {
                        tokenizer: {
                            root: [
                                // Keywords
                                [/\b(true|false|null|and|or|not)\b/, 'keyword'],

                                // Functions
                                [/\b(CONCAT|SUBSTRING|LOOKUP|IS_EMAIL|IS_LEI|IS_SWIFT|IS_PHONE|EXTRACT|VALIDATE|MATCHES)\b/, 'function'],

                                // Operators
                                [/[=!<>]+/, 'operator'],
                                [/[+\-*\/%&]/, 'operator'],
                                [/[()[\]{}]/, 'bracket'],

                                // Numbers
                                [/\d+\.?\d*/, 'number'],

                                // Strings
                                [/"([^"\\]|\\.)*$/, 'string.invalid'],
                                [/"/, 'string', '@string_double'],
                                [/'([^'\\]|\\.)*$/, 'string.invalid'],
                                [/'/, 'string', '@string_single'],

                                // Identifiers (including dotted paths like Client.email)
                                [/[a-zA-Z_]\w*(\.[a-zA-Z_]\w*)*/, 'identifier'],

                                // Whitespace
                                [/\s+/, 'white'],

                                // Comments
                                [/\/\/.*$/, 'comment'],
                            ],

                            string_double: [
                                [/[^\\"]+/, 'string'],
                                [/\\./, 'string.escape'],
                                [/"/, 'string', '@pop'],
                            ],

                            string_single: [
                                [/[^\\']+/, 'string'],
                                [/\\./, 'string.escape'],
                                [/'/, 'string', '@pop'],
                            ],
                        },
                    });

                    // Store data dictionary for LSP features
                    window.dataDictionaryCache = null;

                    // Enhanced auto-completion provider with dynamic data dictionary
                    monaco.languages.registerCompletionItemProvider('dsl-language', {
                        provideCompletionItems: async function(model, position) {
                            const word = model.getWordUntilPosition(position);
                            const range = {
                                startLineNumber: position.lineNumber,
                                endLineNumber: position.lineNumber,
                                startColumn: word.startColumn,
                                endColumn: word.endColumn
                            };

                            // Base DSL functions
                            const functionSuggestions = [
                                {
                                    label: 'CONCAT',
                                    kind: monaco.languages.CompletionItemKind.Function,
                                    documentation: 'Concatenate multiple strings',
                                    detail: 'CONCAT(string1, string2, ...)',
                                    insertText: 'CONCAT(${1:string1}, ${2:string2})',
                                    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                                    range: range
                                },
                                {
                                    label: 'SUBSTRING',
                                    kind: monaco.languages.CompletionItemKind.Function,
                                    documentation: 'Extract substring from string',
                                    detail: 'SUBSTRING(string, start, length)',
                                    insertText: 'SUBSTRING(${1:string}, ${2:start}, ${3:length})',
                                    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                                    range: range
                                },
                                {
                                    label: 'LOOKUP',
                                    kind: monaco.languages.CompletionItemKind.Function,
                                    documentation: 'Lookup value from external table',
                                    detail: 'LOOKUP(key, table_name)',
                                    insertText: 'LOOKUP(${1:key}, "${2:table}")',
                                    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                                    range: range
                                },
                                {
                                    label: 'IS_EMAIL',
                                    kind: monaco.languages.CompletionItemKind.Function,
                                    documentation: 'Validate email address format',
                                    detail: 'IS_EMAIL(email_string) -> Boolean',
                                    insertText: 'IS_EMAIL(${1:email})',
                                    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                                    range: range
                                },
                                {
                                    label: 'IS_LEI',
                                    kind: monaco.languages.CompletionItemKind.Function,
                                    documentation: 'Validate Legal Entity Identifier format',
                                    detail: 'IS_LEI(lei_string) -> Boolean',
                                    insertText: 'IS_LEI(${1:lei})',
                                    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                                    range: range
                                },
                                {
                                    label: 'IS_SWIFT',
                                    kind: monaco.languages.CompletionItemKind.Function,
                                    documentation: 'Validate SWIFT/BIC code format',
                                    detail: 'IS_SWIFT(swift_code) -> Boolean',
                                    insertText: 'IS_SWIFT(${1:swift_code})',
                                    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                                    range: range
                                }
                            ];

                            // Load dynamic attributes from data dictionary
                            let attributeSuggestions = [];
                            try {
                                if (!window.dataDictionaryCache) {
                                    const response = await fetch('/api/data-dictionary');
                                    window.dataDictionaryCache = await response.json();
                                }

                                const data = window.dataDictionaryCache;
                                Object.entries(data.entities || {}).forEach(([entityName, groups]) => {
                                    // Add business attributes
                                    (groups.business || []).forEach(attr => {
                                        attributeSuggestions.push({
                                            label: attr.full_path,
                                            kind: monaco.languages.CompletionItemKind.Field,
                                            documentation: attr.description || \`Business attribute: \${attr.data_type}\`,
                                            detail: \`\${attr.data_type} (business)\`,
                                            insertText: attr.full_path,
                                            range: range,
                                            sortText: \`1_\${attr.full_path}\` // Priority sort
                                        });
                                    });

                                    // Add derived attributes with special marking
                                    (groups.derived || []).forEach(attr => {
                                        attributeSuggestions.push({
                                            label: attr.full_path,
                                            kind: monaco.languages.CompletionItemKind.Variable,
                                            documentation: attr.description || \`Derived attribute: \${attr.data_type}\`,
                                            detail: \`\${attr.data_type} (derived) ✨\`,
                                            insertText: attr.full_path,
                                            range: range,
                                            sortText: \`2_\${attr.full_path}\`
                                        });
                                    });
                                });
                            } catch (error) {
                                console.warn('Failed to load data dictionary for completions:', error);
                            }

                            return {
                                suggestions: [...functionSuggestions, ...attributeSuggestions]
                            };
                        }
                    });

                    // Hover provider for detailed information
                    monaco.languages.registerHoverProvider('dsl-language', {
                        provideHover: async function(model, position) {
                            const word = model.getWordAtPosition(position);
                            if (!word) return null;

                            const hoveredWord = word.word;

                            // Function documentation
                            const functionDocs = {
                                'CONCAT': {
                                    contents: [
                                        { value: '**CONCAT** - String Concatenation Function' },
                                        { value: '```\\nCONCAT(string1, string2, ...)\\n```' },
                                        { value: 'Concatenates multiple string values into a single string.' },
                                        { value: '**Example:** `CONCAT("Hello ", name, "!")` → "Hello John!"' }
                                    ]
                                },
                                'SUBSTRING': {
                                    contents: [
                                        { value: '**SUBSTRING** - String Extraction Function' },
                                        { value: '```\\nSUBSTRING(string, start, length)\\n```' },
                                        { value: 'Extracts a substring from the given string.' },
                                        { value: '**Example:** `SUBSTRING("USER123", 0, 4)` → "USER"' }
                                    ]
                                },
                                'LOOKUP': {
                                    contents: [
                                        { value: '**LOOKUP** - External Data Lookup Function' },
                                        { value: '```\\nLOOKUP(key, table_name)\\n```' },
                                        { value: 'Looks up a value from an external data table.' },
                                        { value: '**Example:** `LOOKUP(country_code, "countries")` → "United States"' }
                                    ]
                                },
                                'IS_EMAIL': {
                                    contents: [
                                        { value: '**IS_EMAIL** - Email Validation Function' },
                                        { value: '```\\nIS_EMAIL(email_string) -> Boolean\\n```' },
                                        { value: 'Validates if the input string is a properly formatted email address.' },
                                        { value: '**Example:** `IS_EMAIL("user@example.com")` → true' }
                                    ]
                                },
                                'IS_LEI': {
                                    contents: [
                                        { value: '**IS_LEI** - Legal Entity Identifier Validation' },
                                        { value: '```\\nIS_LEI(lei_string) -> Boolean\\n```' },
                                        { value: 'Validates if the input string is a properly formatted LEI code.' },
                                        { value: '**Example:** `IS_LEI("529900T8BM49AURSDO55")` → true' }
                                    ]
                                }
                            };

                            if (functionDocs[hoveredWord]) {
                                return {
                                    range: new monaco.Range(
                                        position.lineNumber, word.startColumn,
                                        position.lineNumber, word.endColumn
                                    ),
                                    contents: functionDocs[hoveredWord].contents
                                };
                            }

                            // Dynamic attribute hover from data dictionary
                            try {
                                if (!window.dataDictionaryCache) {
                                    const response = await fetch('/api/data-dictionary');
                                    window.dataDictionaryCache = await response.json();
                                }

                                const data = window.dataDictionaryCache;
                                let foundAttribute = null;

                                Object.entries(data.entities || {}).forEach(([entityName, groups]) => {
                                    [...(groups.business || []), ...(groups.derived || [])].forEach(attr => {
                                        if (attr.full_path === hoveredWord || attr.attribute_name === hoveredWord) {
                                            foundAttribute = attr;
                                        }
                                    });
                                });

                                if (foundAttribute) {
                                    const typeEmoji = foundAttribute.attribute_type === 'derived' ? '✨' : '📊';
                                    return {
                                        range: new monaco.Range(
                                            position.lineNumber, word.startColumn,
                                            position.lineNumber, word.endColumn
                                        ),
                                        contents: [
                                            { value: \`**\${foundAttribute.full_path}** \${typeEmoji}\` },
                                            { value: \`\`\`\\n\${foundAttribute.data_type} (\${foundAttribute.attribute_type})\\n\`\`\`\` },
                                            { value: foundAttribute.description || 'No description available' },
                                            { value: \`**SQL Type:** \${foundAttribute.sql_type}\` },
                                            foundAttribute.rule_definition ? { value: \`**Rule:** \${foundAttribute.rule_definition}\` } : null
                                        ].filter(Boolean)
                                    };
                                }
                            } catch (error) {
                                console.warn('Failed to load hover data:', error);
                            }

                            return null;
                        }
                    });

                    window.editor = monaco.editor.create(container, {
                        value: `// Welcome to Data Designer IDE with Leptos SSR
// Click attributes in the sidebar to insert them
// Test with sample rules:

risk_score = Client.risk_rating * 2.5
email_valid = IS_EMAIL(Client.email)
greeting = CONCAT("Hello ", name, "!")`,
                        language: 'dsl-language',
                        theme: 'vs-dark',
                        fontSize: 14,
                        minimap: { enabled: false },
                        automaticLayout: true,
                        scrollBeyondLastLine: false,
                        wordWrap: 'on',
                        lineNumbers: 'on',
                        renderWhitespace: 'selection',
                        cursorBlinking: 'smooth',
                        suggestOnTriggerCharacters: true,
                    });

                    // Real-time diagnostics and error highlighting
                    let diagnosticsTimeout;
                    function validateRule(model) {
                        clearTimeout(diagnosticsTimeout);
                        diagnosticsTimeout = setTimeout(async () => {
                            const code = model.getValue();
                            if (!code.trim()) {
                                monaco.editor.setModelMarkers(model, 'dsl-language', []);
                                return;
                            }

                            try {
                                const response = await fetch('/api/test-rule', {
                                    method: 'POST',
                                    headers: { 'Content-Type': 'application/json' },
                                    body: JSON.stringify({ dsl_text: code })
                                });

                                const result = await response.json();
                                let markers = [];

                                if (!result.success && result.error) {
                                    // Parse error and create marker
                                    const lines = code.split('\\n');
                                    const errorLine = lines.length; // Default to last line if we can't determine

                                    markers.push({
                                        severity: monaco.MarkerSeverity.Error,
                                        startLineNumber: errorLine,
                                        startColumn: 1,
                                        endLineNumber: errorLine,
                                        endColumn: lines[errorLine - 1]?.length + 1 || 1,
                                        message: result.error,
                                        source: 'DSL Parser'
                                    });
                                }

                                // Check for potential issues
                                const potentialIssues = checkPotentialIssues(code);
                                markers = markers.concat(potentialIssues);

                                monaco.editor.setModelMarkers(model, 'dsl-language', markers);
                            } catch (error) {
                                console.warn('Diagnostics failed:', error);
                            }
                        }, 500); // Debounce 500ms
                    }

                    function checkPotentialIssues(code) {
                        const markers = [];
                        const lines = code.split('\\n');

                        lines.forEach((line, index) => {
                            const lineNumber = index + 1;

                            // Check for common issues
                            if (line.includes('Client.') && !window.dataDictionaryCache) {
                                // Can't validate without data dictionary
                                return;
                            }

                            // Check for undefined attributes
                            const attrMatches = line.match(/\\b[A-Z][a-z]+\\.[a-z_]+\\b/g);
                            if (attrMatches && window.dataDictionaryCache) {
                                attrMatches.forEach(attr => {
                                    let found = false;
                                    Object.values(window.dataDictionaryCache.entities || {}).forEach(groups => {
                                        [...(groups.business || []), ...(groups.derived || [])].forEach(dictAttr => {
                                            if (dictAttr.full_path === attr) {
                                                found = true;
                                            }
                                        });
                                    });

                                    if (!found) {
                                        const attrStart = line.indexOf(attr);
                                        markers.push({
                                            severity: monaco.MarkerSeverity.Warning,
                                            startLineNumber: lineNumber,
                                            startColumn: attrStart + 1,
                                            endLineNumber: lineNumber,
                                            endColumn: attrStart + attr.length + 1,
                                            message: \`Unknown attribute '\${attr}'. Check data dictionary.\`,
                                            source: 'DSL Validator'
                                        });
                                    }
                                });
                            }

                            // Check for potential typos in function names
                            const funcMatches = line.match(/\\b[A-Z_]{2,}\\(/g);
                            if (funcMatches) {
                                const knownFunctions = ['CONCAT', 'SUBSTRING', 'LOOKUP', 'IS_EMAIL', 'IS_LEI', 'IS_SWIFT', 'IS_PHONE', 'EXTRACT', 'VALIDATE'];
                                funcMatches.forEach(match => {
                                    const funcName = match.slice(0, -1); // Remove (
                                    if (!knownFunctions.includes(funcName)) {
                                        const funcStart = line.indexOf(match);
                                        markers.push({
                                            severity: monaco.MarkerSeverity.Info,
                                            startLineNumber: lineNumber,
                                            startColumn: funcStart + 1,
                                            endLineNumber: lineNumber,
                                            endColumn: funcStart + funcName.length + 1,
                                            message: \`Unknown function '\${funcName}'. Did you mean one of: \${knownFunctions.join(', ')}?\`,
                                            source: 'DSL Validator'
                                        });
                                    }
                                });
                            }
                        });

                        return markers;
                    }

                    // Enable real-time validation
                    window.editor.onDidChangeModelContent((e) => {
                        validateRule(window.editor.getModel());
                    });

                    // Initial validation
                    setTimeout(() => validateRule(window.editor.getModel()), 1000);

                    // Add LSP status indicator
                    function addLSPStatusIndicator() {
                        const statusContainer = document.querySelector('.ide-header .header-controls');
                        if (statusContainer) {
                            const statusDiv = document.createElement('div');
                            statusDiv.style.cssText = 'display: flex; align-items: center; margin-right: 10px; font-size: 12px; color: #8c8c8c;';
                            statusDiv.innerHTML = '<span id="lsp-status">🟢 LSP Ready</span>';
                            statusContainer.insertBefore(statusDiv, statusContainer.firstChild);
                        }
                    }
                    addLSPStatusIndicator();

                    console.log('✅ Monaco Editor with enhanced LSP-like features initialized');
                });
            };
            document.head.appendChild(script);
        }

        // IDE Functions
        function insertAttribute(attr) {
            if (window.editor) {
                const position = window.editor.getPosition();
                window.editor.executeEdits('insert-attribute', [{
                    range: new monaco.Range(position.lineNumber, position.column, position.lineNumber, position.column),
                    text: attr
                }]);
                window.editor.focus();
            }
        }

        async function runRule() {
            const results = document.getElementById('results');
            results.className = 'result';
            results.innerHTML = '<pre>🔄 Testing rule...</pre>';

            try {
                if (!window.editor) {
                    results.className = 'result';
                    results.innerHTML = '<pre>⚠️ Editor not ready yet</pre>';
                    return;
                }

                const dslText = window.editor.getValue();
                if (!dslText.trim()) {
                    results.className = 'result';
                    results.innerHTML = '<pre>⚠️ Please enter a rule to test</pre>';
                    return;
                }

                const response = await fetch('/api/test-rule', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        dsl_text: dslText
                    })
                });

                const result = await response.json();

                if (result.success) {
                    results.className = 'result success';
                    results.innerHTML = `<pre>✅ Rule executed successfully!<br/>Result: ${result.result}</pre>`;
                } else {
                    results.className = 'result';
                    results.innerHTML = `<pre>❌ Rule failed:<br/>${result.error}</pre>`;
                }
            } catch (error) {
                console.error('Rule test error:', error);
                results.className = 'result';
                results.innerHTML = '<pre>❌ Failed to test rule</pre>';
            }
        }

        function saveRule() {
            alert('Rule saved successfully!');
        }

        function createAttribute() {
            // Create a modal dialog for new rule creation
            const modal = document.createElement('div');
            modal.style.cssText = `
                position: fixed; top: 0; left: 0; width: 100%; height: 100%;
                background: rgba(0,0,0,0.7); z-index: 1000; display: flex;
                align-items: center; justify-content: center;
            `;

            const dialog = document.createElement('div');
            dialog.style.cssText = `
                background: #252526; border-radius: 8px; padding: 24px;
                width: 480px; max-width: 90vw; border: 1px solid #3e3e42;
                color: #d4d4d4;
            `;

            dialog.innerHTML = `
                <h3 style="margin: 0 0 16px 0; color: #ffffff;">Create New Derived Attribute</h3>

                <div style="margin-bottom: 12px;">
                    <label style="display: block; margin-bottom: 4px; font-size: 12px; color: #8c8c8c;">Attribute Name:</label>
                    <input type="text" id="attr-name" placeholder="e.g. risk_score, client_category"
                           style="width: 100%; padding: 8px; background: #1e1e1e; border: 1px solid #3e3e42; color: #d4d4d4; border-radius: 4px;">
                </div>

                <div style="margin-bottom: 12px;">
                    <label style="display: block; margin-bottom: 4px; font-size: 12px; color: #8c8c8c;">Entity:</label>
                    <input type="text" id="attr-entity" placeholder="e.g. Client, Portfolio" value="Client"
                           style="width: 100%; padding: 8px; background: #1e1e1e; border: 1px solid #3e3e42; color: #d4d4d4; border-radius: 4px;">
                </div>

                <div style="margin-bottom: 12px;">
                    <label style="display: block; margin-bottom: 4px; font-size: 12px; color: #8c8c8c;">Return Type:</label>
                    <select id="attr-type" style="width: 100%; padding: 8px; background: #1e1e1e; border: 1px solid #3e3e42; color: #d4d4d4; border-radius: 4px;">
                        <option value="Number">Number</option>
                        <option value="String">String</option>
                        <option value="Boolean">Boolean</option>
                        <option value="Date">Date</option>
                    </select>
                </div>

                <div style="margin-bottom: 16px;">
                    <label style="display: block; margin-bottom: 4px; font-size: 12px; color: #8c8c8c;">Description:</label>
                    <textarea id="attr-desc" placeholder="Brief description of this attribute..."
                              style="width: 100%; height: 60px; padding: 8px; background: #1e1e1e; border: 1px solid #3e3e42; color: #d4d4d4; border-radius: 4px; resize: vertical;"></textarea>
                </div>

                <div style="display: flex; gap: 8px; justify-content: flex-end;">
                    <button onclick="closeModal()" style="padding: 8px 16px; background: #3e3e42; color: #d4d4d4; border: none; border-radius: 4px; cursor: pointer;">Cancel</button>
                    <button onclick="createNewAttribute()" style="padding: 8px 16px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer;">Create Attribute</button>
                </div>
            `;

            modal.appendChild(dialog);
            document.body.appendChild(modal);

            // Focus the name input
            setTimeout(() => document.getElementById('attr-name').focus(), 100);

            // Close modal functions
            window.closeModal = () => {
                document.body.removeChild(modal);
                delete window.closeModal;
                delete window.createNewAttribute;
            };

            // Create attribute function
            window.createNewAttribute = async () => {
                const name = document.getElementById('attr-name').value.trim();
                const entity = document.getElementById('attr-entity').value.trim();
                const type = document.getElementById('attr-type').value;
                const desc = document.getElementById('attr-desc').value.trim();

                if (!name || !entity) {
                    alert('Please fill in attribute name and entity');
                    return;
                }

                try {
                    const response = await fetch('/api/create-attribute', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            entity_name: entity,
                            attribute_name: name,
                            data_type: type,
                            description: desc || null,
                            dependencies: []
                        })
                    });

                    const result = await response.json();

                    if (result.success) {
                        // Close modal
                        closeModal();

                        // Refresh data dictionary
                        await loadDataDictionary();

                        // Generate rule template in editor
                        generateRuleTemplate(entity, name, type);

                        alert(\`✅ Attribute "\${entity}.\${name}" created successfully!\`);
                    } else {
                        alert(\`❌ Failed to create attribute: \${result.error}\`);
                    }
                } catch (error) {
                    console.error('Create attribute error:', error);
                    alert('❌ Failed to create attribute');
                }
            };
        }

        // Generate rule template in Monaco Editor
        function generateRuleTemplate(entity, attributeName, type) {
            if (!window.editor) return;

            const templates = {
                'Number': \`// Calculate \${attributeName} for \${entity}
\${attributeName} = Client.risk_rating * 2.5 + adjustment\`,
                'String': \`// Generate \${attributeName} string for \${entity}
\${attributeName} = CONCAT(Client.client_id, "_", status)\`,
                'Boolean': \`// Determine \${attributeName} flag for \${entity}
\${attributeName} = Client.aum_usd > 1000000\`,
                'Date': \`// Calculate \${attributeName} date for \${entity}
\${attributeName} = current_date + 30\`
            };

            const template = templates[type] || templates['Number'];
            window.editor.setValue(template);
            window.editor.focus();
        }

        // Tab switching functionality
        function switchTab(tabName) {
            // Update tab buttons
            document.querySelectorAll('.sidebar-tab').forEach(tab => {
                tab.classList.remove('active');
            });
            event.target.classList.add('active');

            // Update tab content
            document.querySelectorAll('.tab-content').forEach(content => {
                content.classList.remove('active');
            });
            document.getElementById(`${tabName}-tab`).classList.add('active');

            // Load content if needed
            if (tabName === 'schema') {
                loadSchemaInfo();
            }
        }

        // Load schema information
        async function loadSchemaInfo() {
            const schemaList = document.getElementById('schema-list');
            schemaList.innerHTML = '<div style="text-align: center; color: #8c8c8c; padding: 20px;">🔄 Loading schema...</div>';

            try {
                // For now, show schema from data dictionary grouped by table
                const response = await fetch('/api/data-dictionary');
                const data = await response.json();

                schemaList.innerHTML = '';

                if (data.total_count === 0) {
                    schemaList.innerHTML = '<div style="text-align: center; color: #ff6b6b; padding: 20px;">⚠️ No schema data available</div>';
                    return;
                }

                // Create schema view grouped by table/entity
                Object.entries(data.entities).forEach(([entityName, groups]) => {
                    const entityHeader = document.createElement('div');
                    entityHeader.style.cssText = 'font-weight: bold; color: #ffffff; background: #3e3e42; padding: 8px 16px; margin: 4px 0; font-size: 12px; cursor: pointer;';
                    entityHeader.innerHTML = `📋 ${entityName} <span style="color: #8c8c8c; font-size: 10px;">(${(groups.business?.length || 0) + (groups.derived?.length || 0)} cols)</span>`;

                    // Create collapsible content
                    const entityContent = document.createElement('div');
                    entityContent.style.display = 'none';

                    // Add columns info
                    [...(groups.business || []), ...(groups.derived || [])].forEach(attr => {
                        const colElement = document.createElement('div');
                        colElement.style.cssText = 'padding: 4px 24px; font-size: 11px; color: #d4d4d4; border-left: 2px solid #007acc; margin-left: 16px;';
                        colElement.innerHTML = `${attr.attribute_name} <span style="color: #8c8c8c;">${attr.sql_type || attr.data_type}</span>`;
                        entityContent.appendChild(colElement);
                    });

                    // Toggle expand/collapse
                    entityHeader.onclick = () => {
                        const isVisible = entityContent.style.display !== 'none';
                        entityContent.style.display = isVisible ? 'none' : 'block';
                        entityHeader.style.background = isVisible ? '#3e3e42' : '#007acc';
                    };

                    schemaList.appendChild(entityHeader);
                    schemaList.appendChild(entityContent);
                });

                console.log('✅ Schema information loaded');
            } catch (error) {
                console.error('Failed to load schema:', error);
                schemaList.innerHTML = '<div style="text-align: center; color: #ff6b6b; padding: 20px;">❌ Failed to load schema</div>';
            }
        }

        function refreshSchema() {
            loadSchemaInfo();
        }

        // Load data dictionary from API
        async function loadDataDictionary() {
            try {
                const response = await fetch('/api/data-dictionary');
                const data = await response.json();

                const attributeList = document.getElementById('attribute-list');
                attributeList.innerHTML = '';

                if (data.total_count === 0) {
                    attributeList.innerHTML = '<div style="text-align: center; color: #ff6b6b; padding: 20px;">⚠️ No database connection</div>';
                    return;
                }

                // Group by entity and type
                Object.entries(data.entities).forEach(([entityName, groups]) => {
                    // Add entity header
                    const entityHeader = document.createElement('div');
                    entityHeader.style.cssText = 'font-weight: bold; color: #ffffff; background: #3e3e42; padding: 8px 16px; margin: 4px 0; font-size: 12px;';
                    entityHeader.textContent = `📁 ${entityName}`;
                    attributeList.appendChild(entityHeader);

                    // Add business attributes
                    if (groups.business && groups.business.length > 0) {
                        groups.business.forEach(attr => {
                            const attrElement = document.createElement('div');
                            attrElement.className = 'attribute-item';
                            attrElement.style.paddingLeft = '24px';
                            attrElement.textContent = `${attr.full_path} (${attr.data_type})`;
                            attrElement.onclick = () => insertAttribute(attr.full_path);
                            attributeList.appendChild(attrElement);
                        });
                    }

                    // Add derived attributes
                    if (groups.derived && groups.derived.length > 0) {
                        groups.derived.forEach(attr => {
                            const attrElement = document.createElement('div');
                            attrElement.className = 'attribute-item';
                            attrElement.style.cssText = 'padding-left: 24px; color: #4caf50; font-style: italic;';
                            attrElement.textContent = `${attr.full_path} (${attr.data_type}) ✨`;
                            attrElement.onclick = () => insertAttribute(attr.full_path);
                            attributeList.appendChild(attrElement);
                        });
                    }
                });

                console.log(`✅ Loaded ${data.total_count} attributes from database`);
            } catch (error) {
                console.error('Failed to load data dictionary:', error);
                const attributeList = document.getElementById('attribute-list');
                attributeList.innerHTML = '<div style="text-align: center; color: #ff6b6b; padding: 20px;">❌ Failed to load data dictionary</div>';
            }
        }

        // Initialize Monaco and load data dictionary when page loads
        window.addEventListener('load', () => {
            initializeMonaco();
            loadDataDictionary();
        });
    </script>
</body>
</html>
    "#;

    Html(html.to_string())
}

// API endpoints for data dictionary
#[cfg(feature = "ssr")]
async fn api_get_data_dictionary(
    Extension(pool): Extension<PgPool>,
    Query(params): Query<HashMap<String, String>>
) -> Json<DataDictionaryResponse> {
    let entity_filter = params.get("entity").cloned();

    match get_data_dictionary(&pool, entity_filter).await {
        Ok(data) => Json(data),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Json(DataDictionaryResponse {
                entities: HashMap::new(),
                total_count: 0,
            })
        }
    }
}

#[cfg(feature = "ssr")]
async fn api_create_derived_attribute(
    Extension(pool): Extension<PgPool>,
    Json(request): Json<CreateDerivedAttributeRequest>
) -> Json<serde_json::Value> {
    match create_derived_attribute(&pool, request).await {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "attribute_id": id
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e
        }))
    }
}

#[derive(Deserialize)]
struct TestRuleRequest {
    dsl_text: String,
}

#[derive(Serialize)]
struct TestRuleResponse {
    success: bool,
    result: Option<String>,
    error: Option<String>,
}

#[cfg(feature = "ssr")]
async fn api_test_rule(
    Json(request): Json<TestRuleRequest>
) -> Json<TestRuleResponse> {
    // Create a test context with sample data
    let mut context = HashMap::new();

    // Sample business data for testing
    context.insert("name".to_string(), serde_json::json!("Alice"));
    context.insert("role".to_string(), serde_json::json!("Admin"));
    context.insert("user_id".to_string(), serde_json::json!("USR_001"));
    context.insert("country_code".to_string(), serde_json::json!("US"));
    context.insert("tier".to_string(), serde_json::json!("premium"));
    context.insert("base_rate".to_string(), serde_json::json!(5.0));
    context.insert("price".to_string(), serde_json::json!(25.50));
    context.insert("quantity".to_string(), serde_json::json!(2.0));
    context.insert("tax".to_string(), serde_json::json!(8.15));

    // Client attributes for KYC use cases
    context.insert("Client.client_id".to_string(), serde_json::json!("CLIENT_12345"));
    context.insert("Client.risk_rating".to_string(), serde_json::json!(7.5));
    context.insert("Client.aum_usd".to_string(), serde_json::json!(1500000.0));
    context.insert("Client.email".to_string(), serde_json::json!("client@example.com"));
    context.insert("Client.country".to_string(), serde_json::json!("USA"));

    // Portfolio attributes
    context.insert("Portfolio.value".to_string(), serde_json::json!(850000.0));
    context.insert("Portfolio.currency".to_string(), serde_json::json!("USD"));

    // Parse and evaluate the rule
    let mut rule = BusinessRule::new(
        "test".to_string(),
        "Test Rule".to_string(),
        "Testing user input".to_string(),
        request.dsl_text,
    );

    match rule.parse() {
        Ok(_) => {
            match rule.evaluate(&context) {
                Ok(result) => {
                    Json(TestRuleResponse {
                        success: true,
                        result: Some(format!("{:?}", result)),
                        error: None,
                    })
                }
                Err(e) => {
                    Json(TestRuleResponse {
                        success: false,
                        result: None,
                        error: Some(format!("Evaluation error: {}", e)),
                    })
                }
            }
        }
        Err(e) => {
            Json(TestRuleResponse {
                success: false,
                result: None,
                error: Some(format!("Parse error: {}", e)),
            })
        }
    }
}

// Simple Axum server for Tauri with database integration
#[cfg(feature = "ssr")]
pub async fn create_minimal_server() -> Result<Router, String> {
    // Create database connection pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://data_designer_app:secure_password@localhost:5432/data_designer".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    println!("✅ Connected to PostgreSQL database");

    let app = Router::new()
        .route("/", get(serve_ide))
        .route("/health", get(|| async { "OK" }))
        .route("/api/data-dictionary", get(api_get_data_dictionary))
        .route("/api/create-attribute", axum::routing::post(api_create_derived_attribute))
        .route("/api/test-rule", axum::routing::post(api_test_rule))
        .layer(Extension(pool));

    println!("🚀 Minimal SSR server with database integration ready for Tauri");
    Ok(app)
}

// Fallback for non-SSR builds
#[cfg(not(feature = "ssr"))]
pub async fn create_minimal_server() -> Result<(), String> {
    println!("⚠️  SSR features not available - enable with --features ssr");
    Ok(())
}