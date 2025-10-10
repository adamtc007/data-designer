// Dynamic DSL Language Definition for Monaco Editor
// Updates based on current grammar from backend

let currentGrammarInfo = null;

// Base language configuration (static)
export const DSL_LANGUAGE_ID = 'dsl-kyc';

export const DSL_LANGUAGE_CONFIG = {
    comments: {
        lineComment: '#',
    },
    brackets: [
        ['(', ')'],
        ['[', ']'],
        ['{', '}'],
    ],
    autoClosingPairs: [
        { open: '(', close: ')' },
        { open: '[', close: ']' },
        { open: '{', close: '}' },
        { open: '"', close: '"', notIn: ['string'] },
        { open: "'", close: "'", notIn: ['string'] },
    ],
    surroundingPairs: [
        { open: '(', close: ')' },
        { open: '[', close: ']' },
        { open: '{', close: '}' },
        { open: '"', close: '"' },
        { open: "'", close: "'" },
    ],
};

// Default/fallback values if grammar fetch fails
const FALLBACK_GRAMMAR = {
    keywords: ['IF', 'THEN', 'ELSE', 'AND', 'OR', 'NOT', 'true', 'false', 'null'],
    functions: [
        { name: 'CONCAT', signature: '(string1, string2, ...)', description: 'Concatenate multiple strings' },
        { name: 'SUBSTRING', signature: '(string, start, length)', description: 'Extract substring' },
        { name: 'LOOKUP', signature: '(key, table)', description: 'Look up value from table' },
        { name: 'UPPER', signature: '(string)', description: 'Convert to uppercase' },
        { name: 'LOWER', signature: '(string)', description: 'Convert to lowercase' },
    ],
    operators: ['=', '+', '-', '*', '/', '&', '==', '!=', '>', '<', '>=', '<=', '~', 'MATCHES'],
    kyc_attributes: [
        'client_id', 'legal_entity_name', 'legal_entity_identifier',
        'risk_rating', 'aum_usd', 'kyc_completeness'
    ]
};

// Function to fetch grammar from backend
export async function fetchGrammarInfo() {
    try {
        // Use the same invoke method as the rest of the application
        const invoke = window.__TAURI_INVOKE__ || (window.__TAURI__ && window.__TAURI__.invoke);
        console.log('🔥 DEBUG: invoke function available?', !!invoke);
        console.log('🔥 DEBUG: window.__TAURI_INVOKE__?', !!window.__TAURI_INVOKE__);
        console.log('🔥 DEBUG: window.__TAURI__?', !!window.__TAURI__);

        if (invoke) {
            console.log('📡 Fetching grammar info from PostgreSQL database...');
            console.log('🔥 DEBUG: About to call get_grammar_info command...');
            const grammarInfo = await invoke('get_grammar_info');
            currentGrammarInfo = grammarInfo;
            console.log('✅ Grammar info loaded from database:', grammarInfo);
            return grammarInfo;
        } else {
            console.warn('⚠️ Tauri not available, using fallback grammar');
            currentGrammarInfo = FALLBACK_GRAMMAR;
            return FALLBACK_GRAMMAR;
        }
    } catch (error) {
        console.error('❌ Failed to fetch grammar info from database:', error);
        console.error('🔥 DEBUG: Full error details:', error);
        currentGrammarInfo = FALLBACK_GRAMMAR;
        return FALLBACK_GRAMMAR;
    }
}

// Generate dynamic Monaco language definition
export function generateDynamicLanguageDefinition(grammarInfo = null) {
    const info = grammarInfo || currentGrammarInfo || FALLBACK_GRAMMAR;

    const functionNames = info.functions.map(f => f.name);

    // Create operator regex pattern from the operators array
    const operatorRegex = new RegExp(
        info.operators
            .map(op => op.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')) // Escape special regex chars
            .sort((a, b) => b.length - a.length) // Sort by length (longest first) to match multi-char operators first
            .join('|')
    );

    return {
        keywords: info.keywords,
        functions: functionNames,
        kycAttributes: info.kyc_attributes,

        // Tokenizer
        tokenizer: {
            root: [
                // Comments
                [/#.*$/, 'comment'],

                // Identifiers and keywords
                [/[a-z_$][\w$]*/, {
                    cases: {
                        '@keywords': 'keyword',
                        '@functions': 'keyword.function',
                        '@kycAttributes': 'variable.predefined',
                        '@default': 'identifier'
                    }
                }],

                // Functions (uppercase)
                [/[A-Z][\w$]*/, {
                    cases: {
                        '@functions': 'keyword.function',
                        '@keywords': 'keyword',
                        '@default': 'identifier'
                    }
                }],

                // Whitespace
                { include: '@whitespace' },

                // Delimiters and operators
                [/[{}()\[\]]/, '@brackets'],
                [operatorRegex, 'operator'],

                // Numbers
                [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
                [/0[xX][0-9a-fA-F]+/, 'number.hex'],
                [/\d+/, 'number'],

                // Regex patterns
                [/\/.*?\/[gimuy]*/, 'regexp'],
                [/r".*?"/, 'regexp'],

                // Strings
                [/"([^"\\]|\\.)*$/, 'string.invalid'],  // non-terminated string
                [/'([^'\\]|\\.)*$/, 'string.invalid'],  // non-terminated string
                [/"/, 'string', '@string_double'],
                [/'/, 'string', '@string_single'],
            ],

            string_double: [
                [/[^\\"]+/, 'string'],
                [/\\./, 'string.escape'],
                [/"/, 'string', '@pop']
            ],

            string_single: [
                [/[^\\']+/, 'string'],
                [/\\./, 'string.escape'],
                [/'/, 'string', '@pop']
            ],

            whitespace: [
                [/[ \t\r\n]+/, 'white'],
                [/#.*$/, 'comment'],
            ],
        },
    };
}

// Enhanced theme with more token types
export const DSL_THEME = {
    base: 'vs-dark',
    inherit: true,
    rules: [
        { token: 'keyword', foreground: 'C586C0' },
        { token: 'keyword.function', foreground: 'DCDCAA' },
        { token: 'variable.predefined', foreground: '4FC1FF' },
        { token: 'identifier', foreground: '9CDCFE' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'number.float', foreground: 'B5CEA8' },
        { token: 'number.hex', foreground: 'B5CEA8' },
        { token: 'operator', foreground: 'D4D4D4' },
        { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
        { token: 'regexp', foreground: 'FF6B9D' },
        { token: 'string.escape', foreground: 'D7BA7D' },
        { token: 'string.invalid', foreground: 'F44747' },
    ],
    colors: {
        'editor.foreground': '#D4D4D4',
        'editor.background': '#1E1E1E',
    }
};

// Function to update Monaco language registration
export async function updateMonacoLanguage(monaco, editor = null) {
    try {
        console.log('Updating Monaco language definition...');

        // Fetch latest grammar
        const grammarInfo = await fetchGrammarInfo();

        // Generate new language definition
        const newLanguageDef = generateDynamicLanguageDefinition(grammarInfo);

        // Dispose existing language if it exists
        try {
            monaco.languages.getLanguages().forEach(lang => {
                if (lang.id === DSL_LANGUAGE_ID) {
                    // Monaco doesn't have a direct dispose method,
                    // but re-registering overwrites the previous definition
                }
            });
        } catch (e) {
            // Ignore disposal errors
        }

        // Register the updated language
        monaco.languages.register({ id: DSL_LANGUAGE_ID });
        monaco.languages.setLanguageConfiguration(DSL_LANGUAGE_ID, DSL_LANGUAGE_CONFIG);
        monaco.languages.setMonarchTokensProvider(DSL_LANGUAGE_ID, newLanguageDef);

        // Re-register theme
        monaco.editor.defineTheme('dsl-theme', DSL_THEME);

        // Update editor language if editor is provided
        if (editor) {
            const model = editor.getModel();
            if (model) {
                monaco.editor.setModelLanguage(model, DSL_LANGUAGE_ID);
                editor.updateOptions({ theme: 'dsl-theme' });
            }
        }

        console.log('Monaco language updated successfully with grammar:', grammarInfo);
        return true;
    } catch (error) {
        console.error('Failed to update Monaco language:', error);
        return false;
    }
}

// Function to get current grammar info
export function getCurrentGrammar() {
    return currentGrammarInfo;
}

// Function to check if grammar is loaded
export function isGrammarLoaded() {
    return currentGrammarInfo !== null;
}