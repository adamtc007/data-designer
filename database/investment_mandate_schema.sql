-- Investment Mandate Schema for PostgreSQL
-- Generated from investment_mandate.schema.json

-- Drop existing tables if they exist
DROP TABLE IF EXISTS mandate_lifecycle_events CASCADE;
DROP TABLE IF EXISTS mandate_instruction_channels CASCADE;
DROP TABLE IF EXISTS mandate_instrument_venues CASCADE;
DROP TABLE IF EXISTS mandate_instrument_identifiers CASCADE;
DROP TABLE IF EXISTS mandate_instruments CASCADE;
DROP TABLE IF EXISTS mandate_benchmarks CASCADE;
DROP TABLE IF EXISTS investment_mandates CASCADE;

-- Create main investment mandate table
CREATE TABLE investment_mandates (
    mandate_id VARCHAR(100) PRIMARY KEY,
    cbu_id VARCHAR(100),

    -- Asset Owner (embedded object)
    asset_owner_name VARCHAR(255) NOT NULL,
    asset_owner_lei VARCHAR(20) NOT NULL,

    -- Investment Manager (embedded object)
    investment_manager_name VARCHAR(255) NOT NULL,
    investment_manager_lei VARCHAR(20) NOT NULL,

    base_currency CHAR(3) NOT NULL CHECK (base_currency ~ '^[A-Z]{3}$'),
    effective_date DATE NOT NULL,
    expiry_date DATE,

    -- Global Limits (embedded object)
    gross_exposure_pct NUMERIC(6,3),
    net_exposure_pct NUMERIC(6,3),
    leverage_max NUMERIC(10,3),
    issuer_concentration_pct NUMERIC(6,3),
    country_concentration_pct NUMERIC(6,3),
    sector_concentration_pct NUMERIC(6,3),
    duration_min NUMERIC(10,3),
    duration_max NUMERIC(10,3),
    var_limit NUMERIC(15,3),
    dv01_limit NUMERIC(15,3),

    -- Controls (embedded object)
    pre_trade_checks_required BOOLEAN DEFAULT FALSE,
    maker_checker BOOLEAN DEFAULT FALSE,
    stp_required BOOLEAN DEFAULT FALSE,
    breach_handling VARCHAR(100),

    -- Reporting (embedded object)
    intraday_status VARCHAR(20) CHECK (intraday_status IN ('none', 'hourly', 'real_time')),
    end_of_day_blotter BOOLEAN DEFAULT FALSE,
    confirmations_required BOOLEAN DEFAULT FALSE,
    matching_model VARCHAR(30) CHECK (matching_model IN ('affirmation', 'confirmation', 'central_matching')),

    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Constraints
    CHECK (expiry_date IS NULL OR expiry_date > effective_date)
);

-- Create benchmarks table (one-to-many relationship)
CREATE TABLE mandate_benchmarks (
    id SERIAL PRIMARY KEY,
    mandate_id VARCHAR(100) NOT NULL REFERENCES investment_mandates(mandate_id) ON DELETE CASCADE,
    benchmark_name VARCHAR(255) NOT NULL,
    UNIQUE(mandate_id, benchmark_name)
);

-- Create instruments table (one-to-many relationship)
CREATE TABLE mandate_instruments (
    id SERIAL PRIMARY KEY,
    mandate_id VARCHAR(100) NOT NULL REFERENCES investment_mandates(mandate_id) ON DELETE CASCADE,

    -- Basic instrument information
    instrument_family VARCHAR(30) NOT NULL CHECK (instrument_family IN (
        'equity', 'fixed_income', 'money_market', 'fund',
        'fx', 'commodity', 'derivative_otc', 'derivative_etd',
        'securities_financing', 'cash'
    )),
    subtype VARCHAR(100),

    -- Classification
    cfi_code VARCHAR(10),
    isda_taxonomy VARCHAR(100),

    -- Order Capabilities (embedded object)
    order_types TEXT[], -- Array of allowed order types
    time_in_force TEXT[], -- Array of allowed TIF values
    min_clip NUMERIC(15,3),
    algo_flags_allowed BOOLEAN DEFAULT FALSE,

    -- Settlement (embedded object)
    settlement_type VARCHAR(10) CHECK (settlement_type IN ('DVP','RVP','FOP','PvP','Cash')),
    settlement_cycle VARCHAR(20),
    place_of_settlement VARCHAR(100),
    allow_partials BOOLEAN DEFAULT FALSE,
    ssi_reference VARCHAR(100),

    -- OTC Terms (embedded object)
    clearing_required BOOLEAN DEFAULT FALSE,
    clearing_house VARCHAR(100),
    margin_model VARCHAR(20) CHECK (margin_model IN ('Bilateral','CCP_IM','SIMM')),
    eligible_collateral_schedule VARCHAR(100),
    min_tenor VARCHAR(20),
    max_tenor VARCHAR(20),

    -- Limits (embedded object)
    exposure_pct NUMERIC(6,3),
    short_allowed BOOLEAN DEFAULT FALSE,
    issuer_max_pct NUMERIC(6,3),
    rating_floor VARCHAR(10),
    limit_duration_min NUMERIC(10,3),
    limit_duration_max NUMERIC(10,3),
    dv01_cap NUMERIC(15,3),
    counterparties_whitelist TEXT[], -- Array of counterparty LEIs

    -- Other
    allocation_model VARCHAR(20) CHECK (allocation_model IN ('pre_trade','post_trade','either')),
    notes TEXT
);

-- Create instrument identifiers table (many-to-many)
CREATE TABLE mandate_instrument_identifiers (
    id SERIAL PRIMARY KEY,
    instrument_id INTEGER NOT NULL REFERENCES mandate_instruments(id) ON DELETE CASCADE,
    identifier_type VARCHAR(20) NOT NULL CHECK (identifier_type IN (
        'ISIN', 'FISN', 'UPI', 'UTI', 'RIC', 'BloombergFIGI', 'CUSIP', 'SEDOL'
    )),
    UNIQUE(instrument_id, identifier_type)
);

-- Create venues table for instruments
CREATE TABLE mandate_instrument_venues (
    id SERIAL PRIMARY KEY,
    instrument_id INTEGER NOT NULL REFERENCES mandate_instruments(id) ON DELETE CASCADE,
    mic VARCHAR(10) NOT NULL,
    preferred BOOLEAN DEFAULT FALSE
);

-- Create instruction channels table
CREATE TABLE mandate_instruction_channels (
    id SERIAL PRIMARY KEY,
    instrument_id INTEGER NOT NULL REFERENCES mandate_instruments(id) ON DELETE CASCADE,
    channel VARCHAR(30) NOT NULL CHECK (channel IN (
        'FIX','SWIFT_MT','ISO20022_XML','FpML','Portal','CSV_SFTP','PhoneRecorded'
    )),
    formats TEXT[], -- Array of format strings
    allowed_flows TEXT[], -- Array of flow types
    stp_required BOOLEAN DEFAULT FALSE
);

-- Create lifecycle events table
CREATE TABLE mandate_lifecycle_events (
    id SERIAL PRIMARY KEY,
    instrument_id INTEGER NOT NULL REFERENCES mandate_instruments(id) ON DELETE CASCADE,
    event_type VARCHAR(30) NOT NULL CHECK (event_type IN (
        'corporate_actions','option_exercise','roll','expiry','recall'
    )),
    UNIQUE(instrument_id, event_type)
);

-- Create indexes for performance
CREATE INDEX idx_mandates_cbu_id ON investment_mandates(cbu_id);
CREATE INDEX idx_mandates_asset_owner_lei ON investment_mandates(asset_owner_lei);
CREATE INDEX idx_mandates_investment_manager_lei ON investment_mandates(investment_manager_lei);
CREATE INDEX idx_mandates_effective_date ON investment_mandates(effective_date);
CREATE INDEX idx_mandates_expiry_date ON investment_mandates(expiry_date);

CREATE INDEX idx_benchmarks_mandate_id ON mandate_benchmarks(mandate_id);

CREATE INDEX idx_instruments_mandate_id ON mandate_instruments(mandate_id);
CREATE INDEX idx_instruments_family ON mandate_instruments(instrument_family);

CREATE INDEX idx_identifiers_instrument_id ON mandate_instrument_identifiers(instrument_id);
CREATE INDEX idx_venues_instrument_id ON mandate_instrument_venues(instrument_id);
CREATE INDEX idx_channels_instrument_id ON mandate_instruction_channels(instrument_id);
CREATE INDEX idx_events_instrument_id ON mandate_lifecycle_events(instrument_id);

-- Create update trigger for updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_investment_mandates_updated_at
    BEFORE UPDATE ON investment_mandates
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create a view for easier querying of complete mandate information
CREATE VIEW v_mandate_summary AS
SELECT
    m.mandate_id,
    m.cbu_id,
    m.asset_owner_name,
    m.asset_owner_lei,
    m.investment_manager_name,
    m.investment_manager_lei,
    m.base_currency,
    m.effective_date,
    m.expiry_date,
    COUNT(DISTINCT mi.id) as instrument_count,
    COUNT(DISTINCT mb.id) as benchmark_count,
    m.created_at,
    m.updated_at
FROM investment_mandates m
LEFT JOIN mandate_instruments mi ON m.mandate_id = mi.mandate_id
LEFT JOIN mandate_benchmarks mb ON m.mandate_id = mb.mandate_id
GROUP BY m.mandate_id;

-- Sample insert helper function
CREATE OR REPLACE FUNCTION insert_mandate_with_instruments(
    p_mandate JSONB
) RETURNS VARCHAR AS $$
DECLARE
    v_mandate_id VARCHAR(100);
    v_instrument_id INTEGER;
    v_instrument JSONB;
    v_identifier TEXT;
    v_venue JSONB;
    v_channel JSONB;
    v_event TEXT;
BEGIN
    -- Insert main mandate
    INSERT INTO investment_mandates (
        mandate_id, cbu_id, asset_owner_name, asset_owner_lei,
        investment_manager_name, investment_manager_lei, base_currency,
        effective_date, expiry_date,
        gross_exposure_pct, net_exposure_pct, leverage_max,
        issuer_concentration_pct, country_concentration_pct, sector_concentration_pct,
        duration_min, duration_max, var_limit, dv01_limit,
        pre_trade_checks_required, maker_checker, stp_required, breach_handling,
        intraday_status, end_of_day_blotter, confirmations_required, matching_model
    ) VALUES (
        p_mandate->>'mandate_id',
        p_mandate->>'cbu_id',
        p_mandate->'asset_owner'->>'name',
        p_mandate->'asset_owner'->>'lei',
        p_mandate->'investment_manager'->>'name',
        p_mandate->'investment_manager'->>'lei',
        p_mandate->>'base_currency',
        (p_mandate->>'effective_date')::DATE,
        (p_mandate->>'expiry_date')::DATE,
        (p_mandate->'global_limits'->>'gross_exposure_pct')::NUMERIC,
        (p_mandate->'global_limits'->>'net_exposure_pct')::NUMERIC,
        (p_mandate->'global_limits'->>'leverage_max')::NUMERIC,
        (p_mandate->'global_limits'->>'issuer_concentration_pct')::NUMERIC,
        (p_mandate->'global_limits'->>'country_concentration_pct')::NUMERIC,
        (p_mandate->'global_limits'->>'sector_concentration_pct')::NUMERIC,
        (p_mandate->'global_limits'->'duration_bounds'->>'min')::NUMERIC,
        (p_mandate->'global_limits'->'duration_bounds'->>'max')::NUMERIC,
        (p_mandate->'global_limits'->>'var_limit')::NUMERIC,
        (p_mandate->'global_limits'->>'dv01_limit')::NUMERIC,
        (p_mandate->'controls'->>'pre_trade_checks_required')::BOOLEAN,
        (p_mandate->'controls'->>'maker_checker')::BOOLEAN,
        (p_mandate->'controls'->>'stp_required')::BOOLEAN,
        p_mandate->'controls'->>'breach_handling',
        p_mandate->'reporting'->>'intraday_status',
        (p_mandate->'reporting'->>'end_of_day_blotter')::BOOLEAN,
        (p_mandate->'reporting'->>'confirmations_required')::BOOLEAN,
        p_mandate->'reporting'->>'matching_model'
    ) RETURNING mandate_id INTO v_mandate_id;

    -- Insert benchmarks
    FOR v_identifier IN SELECT jsonb_array_elements_text(p_mandate->'benchmarks')
    LOOP
        INSERT INTO mandate_benchmarks (mandate_id, benchmark_name)
        VALUES (v_mandate_id, v_identifier);
    END LOOP;

    -- Insert instruments
    FOR v_instrument IN SELECT jsonb_array_elements(p_mandate->'instruments')
    LOOP
        INSERT INTO mandate_instruments (
            mandate_id, instrument_family, subtype,
            cfi_code, isda_taxonomy,
            order_types, time_in_force, min_clip, algo_flags_allowed,
            settlement_type, settlement_cycle, place_of_settlement, allow_partials, ssi_reference,
            clearing_required, clearing_house, margin_model, eligible_collateral_schedule,
            min_tenor, max_tenor,
            exposure_pct, short_allowed, issuer_max_pct, rating_floor,
            limit_duration_min, limit_duration_max, dv01_cap,
            allocation_model, notes
        ) VALUES (
            v_mandate_id,
            v_instrument->>'instrument_family',
            v_instrument->>'subtype',
            v_instrument->'classification'->>'cfi_code',
            v_instrument->'classification'->>'isda_taxonomy',
            ARRAY(SELECT jsonb_array_elements_text(v_instrument->'order_capabilities'->'order_types')),
            ARRAY(SELECT jsonb_array_elements_text(v_instrument->'order_capabilities'->'time_in_force')),
            (v_instrument->'order_capabilities'->>'min_clip')::NUMERIC,
            (v_instrument->'order_capabilities'->>'algo_flags_allowed')::BOOLEAN,
            v_instrument->'settlement'->>'type',
            v_instrument->'settlement'->>'cycle',
            v_instrument->'settlement'->>'place_of_settlement',
            (v_instrument->'settlement'->>'allow_partials')::BOOLEAN,
            v_instrument->'settlement'->>'ssi_reference',
            (v_instrument->'otc_terms'->>'clearing_required')::BOOLEAN,
            v_instrument->'otc_terms'->>'clearing_house',
            v_instrument->'otc_terms'->>'margin_model',
            v_instrument->'otc_terms'->>'eligible_collateral_schedule',
            v_instrument->'otc_terms'->'term_bounds'->>'min_tenor',
            v_instrument->'otc_terms'->'term_bounds'->>'max_tenor',
            (v_instrument->'limits'->>'exposure_pct')::NUMERIC,
            (v_instrument->'limits'->>'short_allowed')::BOOLEAN,
            (v_instrument->'limits'->>'issuer_max_pct')::NUMERIC,
            v_instrument->'limits'->>'rating_floor',
            (v_instrument->'limits'->'duration_bounds'->>'min')::NUMERIC,
            (v_instrument->'limits'->'duration_bounds'->>'max')::NUMERIC,
            (v_instrument->'limits'->>'dv01_cap')::NUMERIC,
            v_instrument->>'allocation_model',
            v_instrument->>'notes'
        ) RETURNING id INTO v_instrument_id;

        -- Insert identifiers required
        FOR v_identifier IN SELECT jsonb_array_elements_text(v_instrument->'identifiers_required')
        LOOP
            INSERT INTO mandate_instrument_identifiers (instrument_id, identifier_type)
            VALUES (v_instrument_id, v_identifier);
        END LOOP;

        -- Insert venues
        FOR v_venue IN SELECT jsonb_array_elements(v_instrument->'venues')
        LOOP
            INSERT INTO mandate_instrument_venues (instrument_id, mic, preferred)
            VALUES (v_instrument_id, v_venue->>'mic', (v_venue->>'preferred')::BOOLEAN);
        END LOOP;

        -- Insert instruction channels
        FOR v_channel IN SELECT jsonb_array_elements(v_instrument->'instruction_channels')
        LOOP
            INSERT INTO mandate_instruction_channels (instrument_id, channel, formats, allowed_flows, stp_required)
            VALUES (
                v_instrument_id,
                v_channel->>'channel',
                ARRAY(SELECT jsonb_array_elements_text(v_channel->'formats')),
                ARRAY(SELECT jsonb_array_elements_text(v_channel->'allowed_flows')),
                (v_channel->>'stp_required')::BOOLEAN
            );
        END LOOP;

        -- Insert lifecycle events
        FOR v_event IN SELECT jsonb_array_elements_text(v_instrument->'lifecycle_events')
        LOOP
            INSERT INTO mandate_lifecycle_events (instrument_id, event_type)
            VALUES (v_instrument_id, v_event);
        END LOOP;
    END LOOP;

    RETURN v_mandate_id;
END;
$$ LANGUAGE plpgsql;

-- Grant appropriate permissions (adjust as needed)
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO data_designer_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO data_designer_app;

COMMENT ON TABLE investment_mandates IS 'Main table storing investment mandate details including parties, limits, controls and reporting requirements';
COMMENT ON TABLE mandate_instruments IS 'Instrument-specific policies and limits for each mandate';
COMMENT ON TABLE mandate_benchmarks IS 'Performance benchmarks associated with each mandate';
COMMENT ON TABLE mandate_instrument_identifiers IS 'Required identifier types for each instrument in a mandate';
COMMENT ON TABLE mandate_instrument_venues IS 'Approved trading venues for each instrument';
COMMENT ON TABLE mandate_instruction_channels IS 'Communication channels and formats for each instrument';
COMMENT ON TABLE mandate_lifecycle_events IS 'Lifecycle events applicable to each instrument';