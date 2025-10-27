-- Generate 100 diverse legal entities for smart entity picker testing
-- Mix of Asset Owners, Investment Managers, Custodians, and Fund Administrators

-- Clear existing data first (optional - comment out if you want to keep existing 16)
-- TRUNCATE TABLE legal_entities CASCADE;

-- Generate 100 entities with realistic data
INSERT INTO legal_entities (entity_id, entity_name, entity_type, incorporation_jurisdiction, incorporation_country, lei_code, status, metadata, created_at, updated_at)
SELECT
    CASE
        WHEN i <= 25 THEN 'AO-' || LPAD(i::text, 4, '0')
        WHEN i <= 50 THEN 'IM-' || LPAD((i-25)::text, 4, '0')
        WHEN i <= 75 THEN 'CUST-' || LPAD((i-50)::text, 4, '0')
        ELSE 'ADMIN-' || LPAD((i-75)::text, 4, '0')
    END as entity_id,

    CASE
        WHEN i <= 25 THEN
            CASE i % 5
                WHEN 0 THEN 'Public Pension Fund ' || i
                WHEN 1 THEN 'University Endowment ' || i
                WHEN 2 THEN 'Sovereign Wealth Fund ' || i
                WHEN 3 THEN 'Insurance Company ' || i
                ELSE 'Corporate Pension Fund ' || i
            END
        WHEN i <= 50 THEN
            CASE (i-25) % 5
                WHEN 0 THEN 'Global Asset Management ' || (i-25)
                WHEN 1 THEN 'Boutique Investment Advisors ' || (i-25)
                WHEN 2 THEN 'Private Equity Partners ' || (i-25)
                WHEN 3 THEN 'Hedge Fund Strategies ' || (i-25)
                ELSE 'Quantitative Investments ' || (i-25)
            END
        WHEN i <= 75 THEN
            CASE (i-50) % 5
                WHEN 0 THEN 'Global Custody Services ' || (i-50)
                WHEN 1 THEN 'Prime Brokerage Solutions ' || (i-50)
                WHEN 2 THEN 'Securities Custodian ' || (i-50)
                WHEN 3 THEN 'Trust & Custody Bank ' || (i-50)
                ELSE 'Asset Servicing Group ' || (i-50)
            END
        ELSE
            CASE (i-75) % 5
                WHEN 0 THEN 'Fund Administration Services ' || (i-75)
                WHEN 1 THEN 'Transfer Agent Solutions ' || (i-75)
                WHEN 2 THEN 'Registry & Compliance Services ' || (i-75)
                WHEN 3 THEN 'Investment Operations Hub ' || (i-75)
                ELSE 'Alternative Investment Admin ' || (i-75)
            END
    END as entity_name,

    CASE
        WHEN i <= 25 THEN 'Asset Owner'
        WHEN i <= 50 THEN 'Investment Manager'
        WHEN i <= 75 THEN 'Custodian'
        ELSE 'Fund Administrator'
    END as entity_type,

    CASE i % 20
        WHEN 0 THEN 'Delaware'
        WHEN 1 THEN 'London'
        WHEN 2 THEN 'Luxembourg'
        WHEN 3 THEN 'Singapore'
        WHEN 4 THEN 'Hong Kong'
        WHEN 5 THEN 'Dublin'
        WHEN 6 THEN 'Zurich'
        WHEN 7 THEN 'Tokyo'
        WHEN 8 THEN 'Cayman Islands'
        WHEN 9 THEN 'British Virgin Islands'
        WHEN 10 THEN 'New York'
        WHEN 11 THEN 'Paris'
        WHEN 12 THEN 'Frankfurt'
        WHEN 13 THEN 'Amsterdam'
        WHEN 14 THEN 'Toronto'
        WHEN 15 THEN 'Sydney'
        WHEN 16 THEN 'Stockholm'
        WHEN 17 THEN 'Copenhagen'
        WHEN 18 THEN 'Oslo'
        ELSE 'Jersey'
    END as incorporation_jurisdiction,

    CASE i % 20
        WHEN 0 THEN 'US'
        WHEN 1 THEN 'GB'
        WHEN 2 THEN 'LU'
        WHEN 3 THEN 'SG'
        WHEN 4 THEN 'HK'
        WHEN 5 THEN 'IE'
        WHEN 6 THEN 'CH'
        WHEN 7 THEN 'JP'
        WHEN 8 THEN 'KY'
        WHEN 9 THEN 'VG'
        WHEN 10 THEN 'US'
        WHEN 11 THEN 'FR'
        WHEN 12 THEN 'DE'
        WHEN 13 THEN 'NL'
        WHEN 14 THEN 'CA'
        WHEN 15 THEN 'AU'
        WHEN 16 THEN 'SE'
        WHEN 17 THEN 'DK'
        WHEN 18 THEN 'NO'
        ELSE 'JE'
    END as incorporation_country,

    -- Generate valid LEI format (20 characters)
    SUBSTRING(MD5(RANDOM()::text) FROM 1 FOR 20) as lei_code,

    CASE WHEN i % 20 = 0 THEN 'pending' ELSE 'active' END as status,

    jsonb_build_object(
        'regulatory_status', CASE i % 3 WHEN 0 THEN 'regulated' WHEN 1 THEN 'registered' ELSE 'exempt' END,
        'aum_usd_millions', (RANDOM() * 10000)::int,
        'year_established', 1980 + (i % 44)
    ) as metadata,

    NOW() - (RANDOM() * INTERVAL '365 days') as created_at,
    NOW() - (RANDOM() * INTERVAL '90 days') as updated_at
FROM generate_series(1, 100) as i
ON CONFLICT (entity_id) DO UPDATE
SET
    entity_name = EXCLUDED.entity_name,
    entity_type = EXCLUDED.entity_type,
    incorporation_jurisdiction = EXCLUDED.incorporation_jurisdiction,
    incorporation_country = EXCLUDED.incorporation_country,
    lei_code = EXCLUDED.lei_code,
    status = EXCLUDED.status,
    metadata = EXCLUDED.metadata,
    updated_at = EXCLUDED.updated_at;

-- Verify the insert
SELECT entity_type, COUNT(*) as count
FROM legal_entities
GROUP BY entity_type
ORDER BY entity_type;

SELECT COUNT(*) as total_entities FROM legal_entities;
