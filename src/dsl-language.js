// DSL Language Definition for Monaco Editor
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

export const DSL_MONARCH_LANGUAGE = {
    keywords: [
        'IF', 'THEN', 'ELSE', 'AND', 'OR', 'NOT', 'true', 'false', 'null'
    ],

    functions: [
        'CONCAT', 'SUBSTRING', 'LOOKUP', 'UPPER', 'LOWER',
        'LENGTH', 'ROUND', 'ABS', 'MAX', 'MIN'
    ],

    operators: [
        '=', '>', '<', '!', '~', '?', ':', '==', '<=', '>=', '!=',
        '&&', '||', '++', '--', '+', '-', '*', '/', '&', '|', '^', '%',
        '<<', '>>', '>>>', '+=', '-=', '*=', '/=', '&=', '|=', '^=',
        '%=', '<<=', '>>=', '>>>='
    ],

    // KYC domain specific attributes
    kycAttributes: [
        'client_id', 'legal_entity_name', 'legal_entity_identifier',
        'risk_rating', 'aum_usd', 'kyc_completeness', 'documents_received',
        'documents_required', 'aml_risk_score', 'pep_status', 'sanctions_check',
        'fatca_status', 'crs_reporting', 'entity_type', 'jurisdiction',
        'regulatory_status', 'onboarding_date', 'compliance_officer',
        'trading_authority', 'authority_limit_usd', 'background_check_status'
    ],

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
            [/@operators/, 'operator'],

            // Numbers
            [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
            [/0[xX][0-9a-fA-F]+/, 'number.hex'],
            [/\d+/, 'number'],

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

// Syntax highlighting theme specifically for KYC DSL
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
        { token: 'operator', foreground: 'D4D4D4' },
        { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
    ],
    colors: {
        'editor.foreground': '#D4D4D4',
        'editor.background': '#1E1E1E',
    }
};