# KYC Data Validation Rules with Regex Support
# =============================================

# 1. Email validation
email_valid = IS_EMAIL(client_email)

# 2. LEI (Legal Entity Identifier) validation
lei_valid = IS_LEI(legal_entity_identifier)

# 3. SWIFT code validation
swift_valid = IS_SWIFT(swift_code)

# 4. Phone number validation
phone_valid = IS_PHONE(phone_number)

# 5. Custom pattern matching using MATCHES operator
# Check if client ID follows pattern: INST_YYYY_NNNNN
client_id_valid = client_id MATCHES /^INST_\d{4}_\d{5}$/

# 6. Extract year from client ID
client_year = EXTRACT(client_id, r"\d{4}")

# 7. Validate tax ID format (US EIN: XX-XXXXXXX)
tax_id_valid = tax_id MATCHES /^\d{2}-\d{7}$/

# 8. Check if website URL is valid
website_valid = website MATCHES /^(https?:\/\/)?([\da-z\.-]+)\.([a-z\.]{2,6})([\/\w \.-]*)*\/?$/

# 9. Validate GIIN format (Global Intermediary Identification Number)
giin_valid = giin MATCHES /^[A-Z0-9]{6}\.\d{5}\.[A-Z]{2}\.\d{3}$/

# 10. Complex validation rule combining multiple checks
kyc_validation_status = IF email_valid = true AND lei_valid = true AND swift_valid = true THEN
    "PASSED"
ELSE IF email_valid = false THEN
    "FAILED: Invalid email"
ELSE IF lei_valid = false THEN
    "FAILED: Invalid LEI"
ELSE IF swift_valid = false THEN
    "FAILED: Invalid SWIFT code"
ELSE
    "FAILED: Other validation error"

# 11. Extract domain from email
email_domain = EXTRACT(client_email, r"@([a-zA-Z0-9.-]+\.[a-zA-Z]{2,})")

# 12. Validate passport number (alphanumeric, 6-9 characters)
passport_valid = passport_number MATCHES /^[A-Z0-9]{6,9}$/

# 13. Check if country code is valid ISO 3166-1 alpha-2
country_valid = registration_country MATCHES /^[A-Z]{2}$/

# 14. Validate regulatory registration number format
reg_number_valid = registration_number MATCHES /^\d{3}-\d{6}$/

# 15. Generic custom validation using VALIDATE function
custom_pattern = "^[A-Z]{3}_[0-9]{4}$"
custom_valid = VALIDATE(department_code, custom_pattern)

# 16. Extract numbers from mixed content
amount_extracted = EXTRACT(transaction_description, r"\d+\.?\d*")

# 17. Validate date format (YYYY-MM-DD)
date_valid = onboarding_date MATCHES /^\d{4}-\d{2}-\d{2}$/

# 18. Check for prohibited characters in names (no special chars except space, hyphen, apostrophe)
name_valid = legal_entity_name MATCHES /^[A-Za-z\s\-']+$/

# 19. Validate risk rating values
risk_valid = risk_rating MATCHES /^(low|medium|high|very_high)$/

# 20. Create comprehensive validation report
validation_report = CONCAT(
    "Email: ", IF email_valid THEN "✓" ELSE "✗",
    " | LEI: ", IF lei_valid THEN "✓" ELSE "✗",
    " | SWIFT: ", IF swift_valid THEN "✓" ELSE "✗",
    " | Phone: ", IF phone_valid THEN "✓" ELSE "✗",
    " | Overall: ", kyc_validation_status
)