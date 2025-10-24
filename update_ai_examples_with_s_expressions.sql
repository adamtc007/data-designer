-- Update AI Generation Examples with S-Expression Format
-- This script adds LISP-style S-expression examples to the AI generation system

-- Add S-expression examples for CBU creation
UPDATE attribute_objects SET
    generation_examples = '[
        {
            "prompt": "Create a new investment fund called Growth Alpha with Asset Owner Alpha Capital and Investment Manager Beta Management",
            "response": "(create-cbu \"Growth Alpha\" \"Diversified growth investment fund\"\n  (entities\n    (entity \"AC001\" \"Alpha Capital\" asset-owner)\n    (entity \"BM002\" \"Beta Management\" investment-manager)))",
            "format": "s-expression"
        },
        {
            "prompt": "Set up a CBU for Deutsche Family Office with managing company Gamma Services",
            "response": "(create-cbu \"Deutsche Family Office\" \"High net worth family wealth management\"\n  (entities\n    (entity \"GS003\" \"Gamma Services\" managing-company)))",
            "format": "s-expression"
        },
        {
            "prompt": "Screen the client name against the sanctions list",
            "response": "(screen-entity \"client_legal_name\" sanctions-list)",
            "format": "s-expression"
        },
        {
            "prompt": "Validate customer identity documents",
            "response": "(validate-identity\n  (document \"passport\" \"A1234567\")\n  (name \"John Michael Smith\")\n  (date-of-birth \"1985-03-15\"))",
            "format": "s-expression"
        }
    ]'::jsonb
WHERE attribute_name = 'cbu_name' OR attribute_name = 'fund_name';

-- Add S-expression examples for entity management
UPDATE attribute_objects SET
    generation_examples = '[
        {
            "prompt": "Add an entity with role Asset Owner",
            "response": "(entity \"AC001\" \"Alpha Capital Management\" asset-owner)",
            "format": "s-expression"
        },
        {
            "prompt": "Create an investment manager entity",
            "response": "(entity \"IM002\" \"Beta Investment Advisors\" investment-manager)",
            "format": "s-expression"
        },
        {
            "prompt": "Define a custodian for the fund",
            "response": "(entity \"CS003\" \"Gamma Custody Services\" custodian)",
            "format": "s-expression"
        }
    ]'::jsonb
WHERE attribute_name IN ('entity_name', 'entity_role', 'entity_id');

-- Add S-expression examples for KYC operations
UPDATE attribute_objects SET
    generation_examples = '[
        {
            "prompt": "Verify customer identity with full name and date of birth",
            "response": "(verify-kyc\n  (customer\n    (name \"María Elena García-López\")\n    (dob \"1990-07-22\")\n    (nationality \"ES\")))",
            "format": "s-expression"
        },
        {
            "prompt": "Check if customer meets minimum age requirement",
            "response": "(validate-age\n  (dob \"1990-07-22\")\n  (min-age 18)\n  (jurisdiction \"EU\"))",
            "format": "s-expression"
        },
        {
            "prompt": "Screen customer against sanctions lists",
            "response": "(screen-sanctions\n  (name \"John Michael Smith\")\n  (dob \"1985-03-15\")\n  (lists \"OFAC\" \"EU\" \"UN\"))",
            "format": "s-expression"
        }
    ]'::jsonb
WHERE attribute_name IN ('full_name', 'date_of_birth', 'kyc_status');

-- Add S-expression examples for workflow operations
UPDATE attribute_objects SET
    generation_examples = '[
        {
            "prompt": "Create an onboarding workflow with identity verification and document collection",
            "response": "(workflow \"customer-onboarding\"\n  (step \"identity-verification\"\n    (verify-kyc customer-data)\n    (validate-documents identity-docs))\n  (step \"document-collection\"\n    (collect \"passport\" \"address-proof\")\n    (validate-authenticity documents)))",
            "format": "s-expression"
        },
        {
            "prompt": "Define a workflow for fund setup",
            "response": "(workflow \"fund-setup\"\n  (step \"entity-creation\"\n    (create-cbu fund-details)\n    (assign-roles entities))\n  (step \"compliance-check\"\n    (screen-sanctions all-entities)\n    (validate-licenses required-permissions)))",
            "format": "s-expression"
        }
    ]'::jsonb
WHERE attribute_name IN ('workflow_name', 'workflow_step', 'onboarding_stage');

-- Add S-expression examples for financial operations
UPDATE attribute_objects SET
    generation_examples = '[
        {
            "prompt": "Calculate portfolio value with multiple currencies",
            "response": "(calculate-portfolio-value\n  (positions\n    (position \"AAPL\" 100 \"USD\")\n    (position \"ASML\" 50 \"EUR\")\n    (position \"TSM\" 200 \"USD\"))\n  (base-currency \"USD\")\n  (fx-rates current-market))",
            "format": "s-expression"
        },
        {
            "prompt": "Execute a trade order with risk checks",
            "response": "(execute-trade\n  (order\n    (symbol \"MSFT\")\n    (quantity 100)\n    (side \"buy\")\n    (type \"market\"))\n  (risk-checks\n    (position-limit max-position)\n    (liquidity-check required)\n    (compliance-screen enabled)))",
            "format": "s-expression"
        }
    ]'::jsonb
WHERE attribute_name IN ('portfolio_value', 'trade_amount', 'position_size');

-- Add S-expression examples for reporting operations
UPDATE attribute_objects SET
    generation_examples = '[
        {
            "prompt": "Generate a compliance report for regulatory submission",
            "response": "(generate-report \"regulatory-compliance\"\n  (period \"2024-Q1\")\n  (jurisdiction \"EU\")\n  (format \"MiFID-II\")\n  (include\n    (portfolio-holdings)\n    (transaction-history)\n    (risk-metrics)))",
            "format": "s-expression"
        },
        {
            "prompt": "Create a performance report for investors",
            "response": "(generate-report \"performance-summary\"\n  (period \"2024-YTD\")\n  (benchmark \"S&P500\")\n  (metrics\n    (total-return)\n    (sharpe-ratio)\n    (max-drawdown)\n    (volatility)))",
            "format": "s-expression"
        }
    ]'::jsonb
WHERE attribute_name IN ('report_type', 'report_period', 'compliance_report');

-- Update the AI context to include S-expression format support
UPDATE attribute_objects SET
    ai_context = jsonb_set(
        COALESCE(ai_context, '{}'::jsonb),
        '{supported_formats}',
        '["traditional", "s-expression", "lisp"]'::jsonb
    ),
    ai_context = jsonb_set(
        COALESCE(ai_context, '{}'::jsonb),
        '{preferred_format}',
        '"s-expression"'::jsonb
    )
WHERE generation_examples IS NOT NULL
  AND generation_examples != '[]'::jsonb;

-- Add S-expression syntax documentation
INSERT INTO attribute_objects (
    resource_id,
    attribute_name,
    data_type,
    description,
    extended_description,
    business_context,
    ai_summary,
    generation_examples,
    ai_context,
    semantic_tags,
    search_keywords
) SELECT
    1, -- Assuming resource_id 1 exists
    's_expression_syntax',
    'documentation',
    'S-expression syntax guide for LISP-style DSL',
    'Comprehensive guide to S-expression syntax used in the LISP-style DSL for financial entity management and workflow definition.',
    'S-expressions provide a clean, functional approach to defining complex financial workflows and entity relationships using list processing syntax.',
    'LISP-style syntax documentation for functional financial DSL',
    '[
        {
            "concept": "Basic S-expression structure",
            "example": "(function-name arg1 arg2 arg3)",
            "description": "Functions are lists where the first element is the function name followed by arguments"
        },
        {
            "concept": "Nested expressions",
            "example": "(create-cbu \"Fund Name\" (entities (entity \"ID\" \"Name\" role)))",
            "description": "S-expressions can be nested to create complex hierarchical structures"
        },
        {
            "concept": "Atoms and literals",
            "example": "\"string\" 123 true asset-owner",
            "description": "Strings in quotes, numbers, booleans, and symbols (hyphenated identifiers)"
        },
        {
            "concept": "Comments",
            "example": "; This is a comment\n(create-cbu \"Fund\" ; inline comment\n  entities)",
            "description": "Semicolon starts a comment that continues to end of line"
        }
    ]'::jsonb,
    '{"domain": "dsl_documentation", "format": "s-expression", "syntax": "lisp"}'::jsonb,
    '["lisp", "s-expression", "syntax", "functional", "dsl"]'::jsonb,
    ARRAY['lisp', 'syntax', 's-expression', 'functional', 'dsl']
WHERE NOT EXISTS (
    SELECT 1 FROM attribute_objects WHERE attribute_name = 's_expression_syntax'
);

-- Create a comprehensive CBU S-expression example
INSERT INTO attribute_objects (
    resource_id,
    attribute_name,
    data_type,
    description,
    generation_examples,
    ai_context
) SELECT
    1,
    'cbu_s_expression_examples',
    'examples',
    'Comprehensive S-expression examples for CBU operations',
    '[
        {
            "name": "Complete CBU Creation",
            "prompt": "Create a comprehensive CBU with multiple entities and roles",
            "response": "(create-cbu \"Goldman Sachs Investment Fund\" \"Multi-strategy hedge fund operations\"\n  (entities\n    (entity \"GS001\" \"Goldman Sachs Asset Management\" asset-owner)\n    (entity \"GS002\" \"Goldman Sachs Investment Advisors\" investment-manager)\n    (entity \"GS003\" \"Goldman Sachs Services\" managing-company)\n    (entity \"BNY001\" \"BNY Mellon\" custodian)\n    (entity \"PWC001\" \"PricewaterhouseCoopers\" administrator))\n  (metadata\n    (domicile \"Delaware\")\n    (currency \"USD\")\n    (strategy \"multi-strategy\")\n    (risk-profile \"high\")))",
            "format": "s-expression"
        },
        {
            "name": "CBU Update Operation",
            "prompt": "Update an existing CBU with new entity relationships",
            "response": "(update-cbu \"CBU001\"\n  (add-entities\n    (entity \"NEW001\" \"New Prime Broker\" prime-broker))\n  (update-metadata\n    (aum 1500000000)\n    (status \"active\")))",
            "format": "s-expression"
        },
        {
            "name": "CBU Query with Filters",
            "prompt": "Query CBUs with specific criteria",
            "response": "(query-cbu\n  (where\n    (status \"active\")\n    (aum-range 100000000 5000000000)\n    (domicile \"Delaware\" \"Luxembourg\"))\n  (include\n    (entities)\n    (metadata)\n    (performance-metrics)))",
            "format": "s-expression"
        }
    ]'::jsonb,
    '{"domain": "cbu_management", "format": "s-expression", "complexity": "comprehensive"}'::jsonb
WHERE NOT EXISTS (
    SELECT 1 FROM attribute_objects WHERE attribute_name = 'cbu_s_expression_examples'
);

-- Update the main CBU DSL examples to include more S-expression patterns
UPDATE attribute_objects SET
    generation_examples = generation_examples || '[
        {
            "pattern": "Conditional entity assignment",
            "example": "(if (> aum 1000000000)\n    (assign-entity prime-broker \"PB001\")\n    (assign-entity standard-broker \"SB001\"))",
            "description": "Conditional logic in S-expressions for dynamic entity assignment"
        },
        {
            "pattern": "Entity validation",
            "example": "(validate-entities\n  (check-licenses required-permissions)\n  (verify-credentials security-clearance)\n  (screen-sanctions global-lists))",
            "description": "Multi-step validation workflow using S-expressions"
        },
        {
            "pattern": "Workflow composition",
            "example": "(compose-workflow\n  (parallel\n    (kyc-verification customer-data)\n    (document-collection required-docs))\n  (sequential\n    (compliance-approval)\n    (account-activation)))",
            "description": "Complex workflow orchestration with parallel and sequential steps"
        }
    ]'::jsonb
WHERE attribute_name = 'cbu_name';

COMMIT;

-- Display summary of updates
SELECT
    attribute_name,
    jsonb_array_length(generation_examples) as example_count,
    ai_context->>'preferred_format' as preferred_format
FROM attribute_objects
WHERE generation_examples IS NOT NULL
  AND generation_examples != '[]'::jsonb
ORDER BY example_count DESC;