-- Grammar Storage Migration
-- Store dynamic grammar definitions in PostgreSQL

-- Table for storing grammar rules
CREATE TABLE IF NOT EXISTS grammar_rules (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    definition TEXT NOT NULL,
    rule_type VARCHAR(20) NOT NULL DEFAULT 'normal', -- 'normal', 'silent', 'atomic'
    description TEXT,
    category VARCHAR(50), -- 'keyword', 'operator', 'function', 'literal', etc.
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Table for storing grammar metadata
CREATE TABLE IF NOT EXISTS grammar_metadata (
    id SERIAL PRIMARY KEY,
    version VARCHAR(20) NOT NULL DEFAULT '1.0',
    description TEXT,
    author VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT true
);

-- Table for storing grammar extensions (operators, functions, keywords)
CREATE TABLE IF NOT EXISTS grammar_extensions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    type VARCHAR(20) NOT NULL, -- 'operator', 'function', 'keyword'
    signature VARCHAR(200), -- For functions
    description TEXT,
    category VARCHAR(50), -- 'arithmetic', 'string', 'comparison', etc.
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_grammar_rules_name ON grammar_rules(name);
CREATE INDEX IF NOT EXISTS idx_grammar_rules_category ON grammar_rules(category);
CREATE INDEX IF NOT EXISTS idx_grammar_extensions_type ON grammar_extensions(type);
CREATE INDEX IF NOT EXISTS idx_grammar_extensions_category ON grammar_extensions(category);

-- Insert default grammar metadata
INSERT INTO grammar_metadata (version, description, author)
VALUES ('1.0', 'Dynamic DSL Grammar Rules', 'Data Designer')
ON CONFLICT DO NOTHING;

-- Insert core grammar rules from existing grammar_rules.json
-- Whitespace and comments
INSERT INTO grammar_rules (name, definition, rule_type, description, category) VALUES
('WHITESPACE', '_{ " " | "\\t" | "\\n" | "\\r" }', 'silent', 'Whitespace characters', 'literal'),
('COMMENT', '_{ "//" ~ (!"\\n" ~ ANY)* }', 'silent', 'Single line comments', 'literal'),
('identifier', '@{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }', 'atomic', 'Variable and attribute names', 'literal'),
('number', '@{ "-"? ~ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }', 'atomic', 'Numeric literals (integers and floats)', 'literal'),
('string', '@{ ASCII_ALPHANUMERIC+ }', 'atomic', 'String literals with single or double quotes', 'literal')
ON CONFLICT (name) DO NOTHING;

-- Operators
INSERT INTO grammar_rules (name, definition, rule_type, description, category) VALUES
('add_op', '{ "+" }', 'normal', 'Addition operator', 'operator'),
('sub_op', '{ "-" }', 'normal', 'Subtraction operator', 'operator'),
('mul_op', '{ "*" }', 'normal', 'Multiplication operator', 'operator'),
('div_op', '{ "/" }', 'normal', 'Division operator', 'operator'),
('concat_op', '{ "&" }', 'normal', 'String concatenation operator', 'operator'),
('comparison_op', '{ "==" | "!=" | ">=" | "<=" | ">" | "<" }', 'normal', 'Comparison operators', 'operator')
ON CONFLICT (name) DO NOTHING;

-- Functions
INSERT INTO grammar_rules (name, definition, rule_type, description, category) VALUES
('substring_fn', '{ "SUBSTRING" ~ "(" ~ expression ~ "," ~ expression ~ "," ~ expression ~ ")" }', 'normal', 'SUBSTRING function call', 'function'),
('concat_fn', '{ "CONCAT" ~ "(" ~ expression ~ ("," ~ expression)* ~ ")" }', 'normal', 'CONCAT function call', 'function'),
('lookup_fn', '{ "LOOKUP" ~ "(" ~ expression ~ "," ~ string ~ ")" }', 'normal', 'LOOKUP function call', 'function'),
('function_call', '_{ substring_fn | concat_fn | lookup_fn }', 'silent', 'Any function call', 'function')
ON CONFLICT (name) DO NOTHING;

-- Core expression rules
INSERT INTO grammar_rules (name, definition, rule_type, description, category) VALUES
('primary', '_{ function_call | "(" ~ expression ~ ")" | identifier | number | string }', 'silent', 'Primary expressions (atoms with parentheses)', 'expression'),
('term', '{ primary ~ (mul_op ~ primary | div_op ~ primary)* }', 'normal', 'Terms with multiplication and division', 'expression'),
('expression', '{ term ~ (add_op ~ term | sub_op ~ term | concat_op ~ term)* }', 'normal', 'Full expressions with operator precedence', 'expression'),
('assignment', '{ identifier ~ "=" ~ expression }', 'normal', 'Variable assignment statements', 'statement'),
('condition', '{ identifier ~ comparison_op ~ (identifier | number | string) }', 'normal', 'Boolean conditions', 'expression'),
('if_clause', '{ "IF" ~ condition ~ ("AND" ~ condition | "OR" ~ condition)* }', 'normal', 'IF conditional clause', 'statement'),
('then_clause', '{ "THEN" ~ assignment }', 'normal', 'THEN action clause', 'statement'),
('rule', '{ "RULE" ~ string ~ if_clause ~ then_clause }', 'normal', 'Complete business rule', 'statement'),
('file', '{ SOI ~ rule* ~ EOI }', 'normal', 'Root rule for parsing files', 'root')
ON CONFLICT (name) DO NOTHING;

-- Insert grammar extensions
-- Operators
INSERT INTO grammar_extensions (name, type, category, description) VALUES
('+', 'operator', 'arithmetic', 'Addition operator'),
('-', 'operator', 'arithmetic', 'Subtraction operator'),
('*', 'operator', 'arithmetic', 'Multiplication operator'),
('/', 'operator', 'arithmetic', 'Division operator'),
('&', 'operator', 'string', 'String concatenation operator'),
('==', 'operator', 'comparison', 'Equality comparison'),
('!=', 'operator', 'comparison', 'Inequality comparison'),
('>', 'operator', 'comparison', 'Greater than'),
('<', 'operator', 'comparison', 'Less than'),
('>=', 'operator', 'comparison', 'Greater than or equal'),
('<=', 'operator', 'comparison', 'Less than or equal'),
('AND', 'operator', 'logical', 'Logical AND'),
('OR', 'operator', 'logical', 'Logical OR'),
('~', 'operator', 'regex', 'Regex match operator'),
('MATCHES', 'operator', 'regex', 'Pattern matching operator')
ON CONFLICT DO NOTHING;

-- Functions
INSERT INTO grammar_extensions (name, type, signature, description, category) VALUES
('SUBSTRING', 'function', '(string, start, length)', 'Extract substring from position', 'string'),
('CONCAT', 'function', '(string1, string2, ...)', 'Concatenate multiple strings', 'string'),
('LOOKUP', 'function', '(key, table)', 'Look up value from external table', 'data'),
('UPPER', 'function', '(string)', 'Convert to uppercase', 'string'),
('LOWER', 'function', '(string)', 'Convert to lowercase', 'string'),
('LENGTH', 'function', '(string)', 'Get string length', 'string'),
('ROUND', 'function', '(number, decimals)', 'Round number to decimals', 'math'),
('ABS', 'function', '(number)', 'Absolute value', 'math'),
('MAX', 'function', '(value1, value2, ...)', 'Maximum value', 'math'),
('MIN', 'function', '(value1, value2, ...)', 'Minimum value', 'math'),
('IS_EMAIL', 'function', '(email)', 'Validate email format', 'validation'),
('IS_LEI', 'function', '(lei)', 'Validate Legal Entity Identifier', 'validation'),
('IS_SWIFT', 'function', '(code)', 'Validate SWIFT/BIC code', 'validation'),
('IS_PHONE', 'function', '(number)', 'Validate phone number', 'validation'),
('VALIDATE', 'function', '(value, pattern)', 'Generic pattern validation', 'validation'),
('EXTRACT', 'function', '(value, pattern)', 'Extract pattern matches', 'validation'),
('MATCHES', 'function', '(text, pattern)', 'Pattern matching function', 'validation')
ON CONFLICT DO NOTHING;

-- Keywords
INSERT INTO grammar_extensions (name, type, description, category) VALUES
('RULE', 'keyword', 'Rule declaration keyword', 'declaration'),
('IF', 'keyword', 'Conditional keyword', 'control'),
('THEN', 'keyword', 'Action keyword', 'control'),
('ELSE', 'keyword', 'Alternative keyword', 'control'),
('AND', 'keyword', 'Logical AND keyword', 'logical'),
('OR', 'keyword', 'Logical OR keyword', 'logical'),
('NOT', 'keyword', 'Logical NOT keyword', 'logical'),
('true', 'keyword', 'Boolean true literal', 'literal'),
('false', 'keyword', 'Boolean false literal', 'literal'),
('null', 'keyword', 'Null literal', 'literal')
ON CONFLICT DO NOTHING;

-- Update timestamp function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add triggers for automatic timestamp updates
DROP TRIGGER IF EXISTS update_grammar_rules_updated_at ON grammar_rules;
CREATE TRIGGER update_grammar_rules_updated_at
    BEFORE UPDATE ON grammar_rules
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_grammar_metadata_updated_at ON grammar_metadata;
CREATE TRIGGER update_grammar_metadata_updated_at
    BEFORE UPDATE ON grammar_metadata
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_grammar_extensions_updated_at ON grammar_extensions;
CREATE TRIGGER update_grammar_extensions_updated_at
    BEFORE UPDATE ON grammar_extensions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();