# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

The Data Designer team takes security vulnerabilities seriously. We appreciate your efforts to responsibly disclose your findings.

### How to Report a Security Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to the project maintainer. You can find the maintainer's contact information in the repository profile.

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

Please include the following information in your report:

- Type of issue (e.g., buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

This information will help us triage your report more quickly.

## Disclosure Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine the affected versions
2. Audit code to find any potential similar problems
3. Prepare fixes for all supported releases
4. Release new security fix versions as soon as possible

## Security Update Process

Security updates will be released as patch versions and announced through:

- GitHub Security Advisories
- Release notes
- README updates

## Known Security Considerations

### Database Security
- The application uses PostgreSQL with user authentication
- Database credentials should never be committed to the repository
- Use environment variables for `DATABASE_URL` configuration

### API Key Management
- AI API keys are stored in the system keychain
- Keys are never logged or exposed in error messages
- Use the gRPC key management API for secure storage

### WASM Security
- The web application runs in a sandboxed browser environment
- All network requests use HTTPS in production
- No sensitive data is stored in browser localStorage

### Development Security
- The `.gitignore` file excludes sensitive files (`.env`, `*.key`, `*.pem`, etc.)
- Configuration files should be reviewed before committing
- Test data should not contain real credentials or sensitive information

## Best Practices for Contributors

1. **Never commit secrets**: Use environment variables or secure key storage
2. **Validate input**: Always validate and sanitize user input
3. **Use parameterized queries**: Prevent SQL injection with prepared statements
4. **Keep dependencies updated**: Regularly update dependencies to patch known vulnerabilities
5. **Review security tools**: Run `cargo audit` before submitting pull requests
6. **Follow secure coding guidelines**: Avoid unsafe Rust code unless absolutely necessary

## Security Tools

We recommend using the following tools for security analysis:

```bash
# Install cargo-audit for dependency vulnerability scanning
cargo install cargo-audit

# Run security audit
cargo audit

# Run clippy for code quality and potential issues
cargo clippy -- -D warnings
```

## Contact

For any security-related questions or concerns, please contact the project maintainers through the GitHub repository.
