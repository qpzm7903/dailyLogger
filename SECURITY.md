# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 1.13.x  | :white_check_mark: |
| 1.12.x  | :white_check_mark: |
| < 1.12  | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in DailyLogger, please report it by creating a private security advisory on GitHub:

1. Go to the [Security tab](https://github.com/qpzm7903/dailyLogger/security)
2. Click "Report a vulnerability"
3. Provide a detailed description of the vulnerability

Alternatively, you can email the maintainers directly (check the repository for contact information).

### What to include in your report

- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Suggested fix (if you have one)

### What to expect

- We will acknowledge your report within 48 hours
- We will provide a more detailed response within 7 days
- We will work on a fix and release a security patch as soon as possible
- We will credit you in the release notes (unless you prefer to remain anonymous)

## Security Best Practices

When using DailyLogger:

- Keep your installation up to date
- Use strong API keys for AI services
- Be cautious about what information you capture in screenshots
- Review the Obsidian output directory permissions
- Regularly backup your data

## Known Security Considerations

- **API Keys**: Stored encrypted (AES-256-GCM) in local SQLite database
- **Screenshots**: Stored locally in app data directory
- **Network**: All AI API calls use HTTPS
- **Local Data**: SQLite database stored in user's app data directory

Thank you for helping keep DailyLogger secure!
