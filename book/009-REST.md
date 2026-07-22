# 009 - REST API

## API Conventions

- Base path: `/api/v1`.
- JSON request and response bodies.
- HTTP verbs follow standard semantics:
  - `GET` for reads.
  - `POST` for creation.
  - `PATCH` for partial updates.
  - `DELETE` for removal.
- Resource IDs are UUIDs in path segments.
- Dates and times are ISO 8601 strings with timezone offset (e.g., `2026-07-21T14:30:00Z`).

## Authentication

- Endpoints requiring authentication expect a session cookie named `ruckchat_session`.
- The cookie is `HttpOnly`, `Secure` in production, and `SameSite=Lax`.
- Unauthenticated requests receive `401 Unauthorized`.
- Requests to resources the user cannot access receive `403 Forbidden`.

## Pagination

- List endpoints use cursor-based pagination where order matters (messages).
- Offset-based pagination is acceptable for stable lists (channels, members).
- Standard query parameters:
  - `limit`: number of items to return (default 20, max 100).
  - `cursor`: opaque cursor for cursor pagination.
  - `offset`: integer offset for offset pagination.

## Error Format

```json
{
  "error": {
    "code": "validation_failed",
    "message": "One or more fields are invalid.",
    "details": [
      { "field": "email", "message": "Email is required." }
    ]
  }
}
```

## Endpoints Overview

### Authentication

- `POST /api/v1/auth/register`
- `POST /api/v1/auth/login`
- `POST /api/v1/auth/logout`
- `POST /api/v1/auth/password-reset/request`
- `POST /api/v1/auth/password-reset/confirm`
- `GET /api/v1/auth/me`

### Users

- `GET /api/v1/users/:id`
- `PATCH /api/v1/users/:id`
- `POST /api/v1/users/:id/avatar`

### Organizations

- `POST /api/v1/organizations`
- `GET /api/v1/organizations`
- `GET /api/v1/organizations/:id`
- `PATCH /api/v1/organizations/:id`
- `DELETE /api/v1/organizations/:id`
- `GET /api/v1/organizations/:id/members`
- `POST /api/v1/organizations/:id/invitations`
- `POST /api/v1/organizations/:id/invitations/:token/accept`
- `DELETE /api/v1/organizations/:id/members/:user_id`
- `PATCH /api/v1/organizations/:id/members/:user_id/role`

### Channels

- `GET /api/v1/organizations/:id/channels`
- `POST /api/v1/organizations/:id/channels`
- `GET /api/v1/channels/:id`
- `PATCH /api/v1/channels/:id`
- `DELETE /api/v1/channels/:id`
- `POST /api/v1/channels/:id/join`
- `POST /api/v1/channels/:id/leave`
- `GET /api/v1/channels/:id/members`

### Direct Messages

- `GET /api/v1/organizations/:id/dms`
- `POST /api/v1/organizations/:id/dms`
- `GET /api/v1/dms/:id`

### Messages

- `GET /api/v1/conversations/:id/messages`
- `POST /api/v1/conversations/:id/messages`
- `GET /api/v1/messages/:id`
- `PATCH /api/v1/messages/:id`
- `DELETE /api/v1/messages/:id`
- `GET /api/v1/messages/:id/replies`
- `POST /api/v1/messages/:id/reactions`
- `DELETE /api/v1/messages/:id/reactions/:emoji`

### Search

- `GET /api/v1/organizations/:id/search?q=...&type=messages`

### Files

- `POST /api/v1/files`
- `GET /api/v1/files/:id`
- `GET /api/v1/files/:id/download`
- `DELETE /api/v1/files/:id`

## OpenAPI

- The API is documented in `docs/openapi.yaml`.
- Every new endpoint or schema change must update the OpenAPI file.
- The OpenAPI spec is validated in CI and can be used to generate client stubs.

## Rate Limiting

- Authentication endpoints are rate-limited per IP.
- Message creation is rate-limited per user to prevent spam.
- Rate limits are enforced in Axum middleware.
- v1 uses in-memory counters; distributed rate limiting is not required because deployments are single-server.
