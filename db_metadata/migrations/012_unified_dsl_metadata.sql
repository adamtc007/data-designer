-- Migration: Unified DSL Metadata Storage
-- Purpose: Create unified table to store DSL content for both CBUs and Resources
--          with composite primary key (entity_id, dsl_domain)

-- Create unified DSL metadata table
CREATE TABLE IF NOT EXISTS dsl_metadata (
    entity_id TEXT NOT NULL,           -- CBU ID (from cbu_id) or Resource ID (from resource_id)
    dsl_domain TEXT NOT NULL,          -- 'CBU' or 'Resource'
    dsl_content TEXT NOT NULL,         -- The actual DSL content (S-expression or other format)
    dsl_version INTEGER DEFAULT 1,     -- Version number for tracking changes
    syntax_valid BOOLEAN DEFAULT true, -- Whether the DSL syntax is valid
    last_validated_at TIMESTAMPTZ,     -- When the DSL was last validated
    metadata JSONB,                    -- Additional metadata (parsing results, compilation cache, etc.)
    created_by VARCHAR(100),           -- Who created this DSL
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by VARCHAR(100),           -- Who last updated this DSL
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (entity_id, dsl_domain),
    CONSTRAINT dsl_metadata_domain_check CHECK (dsl_domain IN ('CBU', 'Resource'))
);

-- Add comments for documentation
COMMENT ON TABLE dsl_metadata IS 'Unified storage for DSL content across different entity types (CBU, Resource)';
COMMENT ON COLUMN dsl_metadata.entity_id IS 'Foreign key to cbu_id or resource_id depending on dsl_domain';
COMMENT ON COLUMN dsl_metadata.dsl_domain IS 'Domain of the DSL: CBU or Resource';
COMMENT ON COLUMN dsl_metadata.dsl_content IS 'The actual DSL source code (S-expression format for Resources)';
COMMENT ON COLUMN dsl_metadata.dsl_version IS 'Version number incremented on each update';
COMMENT ON COLUMN dsl_metadata.syntax_valid IS 'Whether the DSL passes syntax validation';
COMMENT ON COLUMN dsl_metadata.metadata IS 'Additional metadata: parse trees, compilation results, execution logs';

-- Create indexes for efficient querying
CREATE INDEX idx_dsl_metadata_domain ON dsl_metadata(dsl_domain);
CREATE INDEX idx_dsl_metadata_entity_id ON dsl_metadata(entity_id);
CREATE INDEX idx_dsl_metadata_updated_at ON dsl_metadata(updated_at DESC);
CREATE INDEX idx_dsl_metadata_syntax_valid ON dsl_metadata(syntax_valid) WHERE syntax_valid = false;

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_dsl_metadata_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    NEW.dsl_version = OLD.dsl_version + 1;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_dsl_metadata_updated_at
    BEFORE UPDATE ON dsl_metadata
    FOR EACH ROW
    EXECUTE FUNCTION update_dsl_metadata_updated_at();

-- Insert sample CBU DSL data (migrating from existing if any)
INSERT INTO dsl_metadata (entity_id, dsl_domain, dsl_content, metadata, created_by) VALUES
('CBU-001', 'CBU', '# CBU DSL Example
CREATE CBU "Alpha Investment" WITH
  legal_entity = "Alpha Investments LLC"
  entity_lei = "549300ABCDEF123456001"
  jurisdiction = "Delaware"
END', '{"example": true, "template": "basic_cbu"}', 'system')
ON CONFLICT (entity_id, dsl_domain) DO NOTHING;

-- Insert sample Resource DSL data using S-expression format
INSERT INTO dsl_metadata (entity_id, dsl_domain, dsl_content, metadata, created_by) VALUES
('RES-CUSTODY-001', 'Resource', '(resource
  (kind CustodySafekeeping)
  (version 1)
  (attributes
    (attr bank_code (type string) (visibility public) (required true) (value "ABCD"))
    (attr branch_code (type string) (visibility public) (required true) (value "0123"))
    (attr account_number (type string) (visibility public) (required true) (value "000123456789"))
    (attr account_iban (type string) (visibility private) (required true)
          (rule (concat bank_code branch_code account_number)))
    (attr safekeeping_location (type enum) (visibility private) (required true)
          (rule (format "LDN"))))
  (provisioning
    (endpoint "grpc://custody.setup/CreateSafekeeping")))',
 '{"example": true, "resource_type": "CustodySafekeeping", "version": 1}', 'system'),

('RES-TRADE-001', 'Resource', '(resource
  (kind TradeCapture)
  (version 2)
  (attributes
    (attr mic_code (type string) (visibility public) (required true) (value "XLON"))
    (attr order_flow_class (type enum(Agency Principal)) (visibility public) (required true) (value "Agency"))
    (attr stp_endpoint (type url) (visibility private) (required true)
          (rule (format "https://stp.%s.trd/%s" mic_code (lower order_flow_class))))
    (attr auth_token (type string) (visibility private) (required true)
          (rule (format "vault://secrets/trade/%s" cbu_id))))
  (provisioning
    (endpoint "grpc://trade.capture/Setup")))',
 '{"example": true, "resource_type": "TradeCapture", "version": 2}', 'system')
ON CONFLICT (entity_id, dsl_domain) DO NOTHING;

-- Verification query
DO $$
BEGIN
    RAISE NOTICE 'DSL Metadata table created successfully!';
    RAISE NOTICE 'Sample DSL records: % total', (SELECT COUNT(*) FROM dsl_metadata);
    RAISE NOTICE '  - CBU DSLs: %', (SELECT COUNT(*) FROM dsl_metadata WHERE dsl_domain = 'CBU');
    RAISE NOTICE '  - Resource DSLs: %', (SELECT COUNT(*) FROM dsl_metadata WHERE dsl_domain = 'Resource');
END $$;
