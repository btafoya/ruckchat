# 013 - Security

## Threat Model

RuckChat v1 assumes a trusted operator and trusted users. The primary threats are:

- Unauthorized access to accounts and organizations.
- Injection attacks against the API and database.
- Abuse of file uploads and message spam.
- Information disclosure through logs or error messages.

Advanced threats such as insider abuse, client-side tampering, and server compromise are mitigated where practical but are not the v1 focus.

## Authentication

- Passwords are hashed with Argon2id.
- Sessions are represented by opaque tokens stored in `HttpOnly`, `Secure`, `SameSite=Lax` cookies.
- Session tokens are hashed before storage; the raw token exists only in the cookie.
- Sessions are invalidated on password change and explicit logout.

## Authorization

- Every API request carries an implicit user identity from the session.
- Services enforce organization, channel, and conversation membership before allowing access.
- Role checks are explicit in service functions; handlers do not make authorization decisions.
- Row-level security in PostgreSQL is not used in v1; access control is enforced in the application layer.

## Input Validation

- All request bodies are deserialized into validated structs.
- String lengths, UUID formats, and enum values are checked before database access.
- Markdown content is sanitized on render to prevent XSS.
- File uploads are checked against allowed MIME types and size limits.

## Database Security

- SQLx compile-time checks prevent accidental SQL injection.
- Dynamic queries are parameterized.
- Database credentials are passed through environment variables, never committed.

## File Upload Security

- Files are stored outside the web root.
- The server serves files through an authenticated endpoint, not direct static paths.
- File metadata (MIME type, size) is validated and the extension is not trusted alone.
- Execution permissions are stripped from uploaded files on the local filesystem.
- SVG files are sanitized if rendered inline.

## Transport Security

- HTTPS is required in production.
- WebSockets use `wss://` in production.
- TLS termination is handled by the reverse proxy (Caddy) or a load balancer.
- HSTS and secure cookie flags are enforced in production builds.

## Secrets Management

- Secrets are loaded from environment variables or a secrets file mounted at runtime.
- `.env` files are not committed to the repository.
- Session secrets must be at least 32 bytes of cryptographically random data.

## Rate Limiting

- Authentication endpoints are rate-limited per IP.
- Message creation and file uploads are rate-limited per user.
- WebSocket connection attempts are rate-limited per IP.

## Logging and Monitoring

- Failed login attempts are logged with IP and timestamp.
- Sensitive values (passwords, raw session tokens) are never logged.
- Error responses do not expose stack traces or SQL details.

## Vulnerability Response

- Security fixes are released as patch versions.
- Critical vulnerabilities are documented in the security advisory section of the repository.
- Users are notified through release notes and, when appropriate, through in-app notifications.
