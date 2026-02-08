# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting Vulnerabilities

**Do NOT report security vulnerabilities through public GitHub issues.**

Report vulnerabilities via:

1. **GitHub Security Advisories**: Report privately at [cyberfabric/cyberfabric-core/security/advisories/new](https://github.com/cyberfabric/cyberfabric-core/security/advisories/new)
2. **Direct Contact**: Email maintainers directly (see [MAINTAINERS](https://github.com/cyberfabric/cyberfabric-core/graphs/contributors) list)

### Required Information

- Vulnerability type
- Affected source file paths
- Source location (tag/branch/commit or URL)
- Reproduction steps
- Configuration requirements (if any)
- Proof-of-concept code (if available)
- Impact assessment

### Response Timeline

- Acknowledgment: 48 hours
- Fix target: 90 days from disclosure
- Credit: Provided in security advisory (unless anonymity requested)

## Developer Security Practices

- Review [Security Guidelines](guidelines/SECURITY.md) for coding standards
- Follow [Secure ORM documentation](docs/SECURE-ORM.md) for database operations
- Dependencies scanned via `cargo audit` and Dependabot
- All changes require code review via pull requests

## Disclosure Process

1. Security advisory published
2. Patch release created
3. Notification via GitHub releases
4. Public disclosure after update window

