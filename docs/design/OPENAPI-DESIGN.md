# RuckChat v1 OpenAPI Design

## 1. Purpose

This document outlines the REST API contract for RuckChat v1. The authoritative spec lives in `docs/openapi.yaml`. This design doc captures endpoints, request/response schemas, and conventions before the OpenAPI file is maintained alongside code.

## 2. OpenAPI Version

- OpenAPI 3.1.0
- Base URL: `https://{host}/api/v1`

## 3. Common Components

### 3.1 Error Schema

```yaml
Error:
  type: object
  required: [code, message]
  properties:
    code:
      type: string
    message:
      type: string
    details:
      type: array
      items:
        type: object
        properties:
          field:
            type: string
          message:
            type: string
```

### 3.2 Pagination

Offset pagination:

```yaml
OffsetPagination:
  type: object
  properties:
    offset:
      type: integer
      default: 0
    limit:
      type: integer
      default: 20
      maximum: 100
```

Cursor pagination:

```yaml
CursorPagination:
  type: object
  properties:
    cursor:
      type: string
    limit:
      type: integer
      default: 20
      maximum: 100
```

## 4. Endpoints

### 4.1 Authentication

#### POST /auth/register

Request:

```json
{
  "email": "alice@example.com",
  "password": "correct-horse-battery-staple",
  "display_name": "Alice"
}
```

Response: `201 Created`

```json
{
  "id": "uuid",
  "email": "alice@example.com",
  "display_name": "Alice",
  "created_at": "2026-07-21T14:30:00Z"
}
```

#### POST /auth/login

Request:

```json
{
  "email": "alice@example.com",
  "password": "correct-horse-battery-staple"
}
```

Response: `200 OK` with session cookie set.

#### POST /auth/logout

Response: `200 OK` with cleared session cookie.

#### POST /auth/password-reset/request

Request:

```json
{
  "email": "alice@example.com"
}
```

Response: `202 Accepted`

#### POST /auth/password-reset/confirm

Request:

```json
{
  "token": "reset-token",
  "new_password": "new-password"
}
```

Response: `200 OK`

#### GET /auth/me

Response:

```json
{
  "id": "uuid",
  "email": "alice@example.com",
  "display_name": "Alice",
  "avatar_url": "https://.../avatar.png"
}
```

### 4.2 Users

#### GET /users/{id}

Response:

```json
{
  "id": "uuid",
  "email": "alice@example.com",
  "display_name": "Alice",
  "avatar_url": "https://.../avatar.png",
  "created_at": "2026-07-21T14:30:00Z"
}
```

#### PATCH /users/{id}

Request:

```json
{
  "display_name": "Alice Smith"
}
```

#### POST /users/{id}/avatar

Multipart file upload. Response returns updated user profile.

### 4.3 Organizations

#### POST /organizations

Request:

```json
{
  "name": "Acme Corp",
  "slug": "acme-corp"
}
```

Response: `201 Created`

#### GET /organizations

Response:

```json
{
  "items": [
    {
      "id": "uuid",
      "name": "Acme Corp",
      "slug": "acme-corp",
      "role": "owner"
    }
  ],
  "offset": 0,
  "limit": 20,
  "total": 1
}
```

#### GET /organizations/{id}

#### PATCH /organizations/{id}

#### DELETE /organizations/{id}

#### GET /organizations/{id}/members

#### POST /organizations/{id}/invitations

Request:

```json
{
  "email": "bob@example.com",
  "role": "member"
}
```

#### POST /organizations/{id}/invitations/{token}/accept

#### DELETE /organizations/{id}/members/{user_id}

#### PATCH /organizations/{id}/members/{user_id}/role

Request:

```json
{
  "role": "admin"
}
```

### 4.4 Channels

#### GET /organizations/{id}/channels

Response:

```json
{
  "items": [
    {
      "id": "uuid",
      "name": "general",
      "topic": "General discussion",
      "is_private": false,
      "member_count": 12,
      "unread_count": 3
    }
  ],
  "offset": 0,
  "limit": 20,
  "total": 1
}
```

#### POST /organizations/{id}/channels

Request:

```json
{
  "name": "engineering",
  "topic": "Engineering team",
  "is_private": false
}
```

#### GET /channels/{id}

#### PATCH /channels/{id}

#### DELETE /channels/{id}

#### POST /channels/{id}/join

#### POST /channels/{id}/leave

#### GET /channels/{id}/members

### 4.5 Direct Messages

#### GET /organizations/{id}/dms

#### POST /organizations/{id}/dms

Request:

```json
{
  "member_ids": ["uuid-1", "uuid-2"]
}
```

#### GET /dms/{id}

### 4.6 Messages

#### GET /conversations/{id}/messages

Query parameters: `limit`, `cursor`, `parent_id` (for threads).

Response:

```json
{
  "items": [
    {
      "id": "uuid",
      "author_id": "uuid",
      "content": "Hello team",
      "created_at": "2026-07-21T14:30:00Z",
      "updated_at": "2026-07-21T14:30:00Z",
      "deleted_at": null,
      "reactions": {
        "👍": 3
      },
      "thread_reply_count": 0
    }
  ],
  "next_cursor": "opaque-cursor",
  "has_more": true
}
```

#### POST /conversations/{id}/messages

Request:

```json
{
  "content": "Hello team",
  "parent_id": null,
  "file_ids": []
}
```

#### GET /messages/{id}

#### PATCH /messages/{id}

Request:

```json
{
  "content": "Updated message"
}
```

#### DELETE /messages/{id}

#### GET /messages/{id}/replies

#### POST /messages/{id}/reactions

Request:

```json
{
  "emoji": "👍"
}
```

#### DELETE /messages/{id}/reactions/{emoji}

### 4.7 Search

#### GET /organizations/{id}/search

Query parameters: `q`, `type` (default `messages`), `conversation_id`, `limit`, `offset`.

Response:

```json
{
  "items": [
    {
      "message_id": "uuid",
      "conversation_id": "uuid",
      "conversation_type": "channel",
      "author_id": "uuid",
      "content": "...",
      "rank": 0.95,
      "created_at": "2026-07-21T14:30:00Z"
    }
  ],
  "offset": 0,
  "limit": 20,
  "total": 42
}
```

### 4.8 Files

#### POST /files

Multipart upload. Response:

```json
{
  "id": "uuid",
  "file_name": "diagram.png",
  "mime_type": "image/png",
  "size_bytes": 123456,
  "storage_path": "...",
  "thumbnail_path": "...",
  "created_at": "2026-07-21T14:30:00Z"
}
```

#### GET /files/{id}

#### GET /files/{id}/download

#### DELETE /files/{id}

## 5. WebSocket Endpoint

- `wss://{host}/api/v1/ws`
- Authenticated by session cookie.
- Messages are JSON envelopes.

## 6. Status Codes

| Code | Use |
|------|-----|
| 200  | Success |
| 201  | Created |
| 202  | Accepted (async work) |
| 204  | No content (not used in v1 by default) |
| 400  | Bad request / validation error |
| 401  | Unauthenticated |
| 403  | Forbidden |
| 404  | Not found |
| 409  | Conflict (duplicate channel, etc.) |
| 429  | Rate limited |
| 500  | Internal server error |

## 7. Naming Conventions

- Resource IDs in paths use the singular resource name: `/channels/{id}`.
- Nested actions use verbs: `/channels/{id}/join`.
- Collection filters use query parameters.
- Request/response fields use `snake_case`.

## 8. Security Headers

All responses include:

- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Referrer-Policy: strict-origin-when-cross-origin`

## 9. Files Produced

- `docs/design/OPENAPI-DESIGN.md` (this file)
- `docs/design/ARCHITECTURE-DESIGN.md`
- `docs/design/DATABASE-SCHEMA-DESIGN.md`
