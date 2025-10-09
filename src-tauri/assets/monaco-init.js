// Monaco Editor initialization for Leptos SSR
(function() {
    // Wait for DOM to be ready
    function initializeMonaco() {
        const container = document.getElementById('monaco-container');
        if (!container) {
            setTimeout(initializeMonaco, 100);
            return;
        }

        // Clear loading placeholder
        container.innerHTML = '';

        // Load Monaco Editor from CDN
        const script = document.createElement('script');
        script.src = 'https://unpkg.com/monaco-editor@latest/min/vs/loader.js';
        script.onload = function() {
            require.config({ paths: { 'vs': 'https://unpkg.com/monaco-editor@latest/min/vs' }});
            require(['vs/editor/editor.main'], function() {
                // Initialize Monaco
                window.editor = monaco.editor.create(container, {
                    value: `// Welcome to Data Designer IDE
// Click attributes in the sidebar to insert them into your rules

risk_score = base_rate * (1 + risk_factor) + adjustment`,
                    language: 'dsl-language',
                    theme: 'vs-dark',
                    fontSize: 14,
                    minimap: { enabled: false },
                    automaticLayout: true,
                    scrollBeyondLastLine: false,
                });

                // Register custom DSL language
                monaco.languages.register({ id: 'dsl-language' });
                monaco.languages.setMonarchTokensProvider('dsl-language', {
                    tokenizer: {
                        root: [
                            [/[a-zA-Z_][a-zA-Z0-9_]*/, 'identifier'],
                            [/\d+\.?\d*/, 'number'],
                            [/"[^"]*"/, 'string'],
                            [/[+\-*/=<>!&|]/, 'operator'],
                            [/[(){}[\]]/, 'bracket'],
                            [/\/\/.*/, 'comment'],
                        ]
                    }
                });

                // Set up completion provider for attributes
                monaco.languages.registerCompletionItemProvider('dsl-language', {
                    provideCompletionItems: function(model, position) {
                        // Get attributes from Leptos state
                        const attributes = window.currentAttributes || [];
                        const suggestions = attributes.map(attr => ({
                            label: attr,
                            kind: monaco.languages.CompletionItemKind.Field,
                            insertText: attr,
                            documentation: `Business attribute: ${attr}`
                        }));

                        return { suggestions };
                    }
                });

                console.log('✅ Monaco Editor initialized with Leptos SSR');
            });
        };
        document.head.appendChild(script);
    }

    // Global functions for Leptos integration
    window.insertAttributeIntoEditor = function(attr) {
        if (window.editor) {
            const position = window.editor.getPosition();
            window.editor.executeEdits('insert-attribute', [{
                range: new monaco.Range(position.lineNumber, position.column, position.lineNumber, position.column),
                text: attr
            }]);
            window.editor.focus();
        }
    };

    window.createNewAttribute = function() {
        // Trigger Leptos component to show attribute creation dialog
        console.log('Creating new attribute...');
    };

    window.getCurrentRule = function() {
        return window.editor ? window.editor.getValue() : '';
    };

    // Update attributes list (called from Leptos)
    window.updateAttributesList = function(attributes) {
        window.currentAttributes = attributes;
    };

    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initializeMonaco);
    } else {
        initializeMonaco();
    }
})();