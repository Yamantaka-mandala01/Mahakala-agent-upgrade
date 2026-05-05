# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please follow these steps:

1. **DO NOT** create a public GitHub issue
2. Email us at: mahakala.hum.pate@gmail.com
3. Include the following information:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## Response Timeline

- We will acknowledge receipt of your report within 48 hours
- We will provide a status update within 7 days
- We aim to release a fix within 30 days for critical vulnerabilities

## Security Best Practices

When using Mahakala Agent:

- **API Keys**: Never commit API keys to the repository. Use environment variables or the WebUI settings panel.
- **Shell Execution**: The `shell_exec` tool can run arbitrary commands. Be cautious when enabling this tool.
- **File Access**: File tools can read/write to the filesystem. Configure appropriate access restrictions.
- **Network**: The `web_fetch` tool can make outbound HTTP requests. Consider network restrictions in your environment.

## Known Security Considerations

- The application runs locally and does not transmit data to external servers except when explicitly configured with cloud AI providers
- SQLite database (`mahakala.db`) contains conversation history and should be protected
- JWT tokens are used for authentication; keep your secret key secure
