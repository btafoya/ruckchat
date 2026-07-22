# 004-Domain-Model

## Purpose

Define the RuckChat domain model and the invariants that protect data integrity
across organizations, channels, messages, and users.

## Requirements

- Entities must be strongly typed and serializable.
- Domain invariants must be enforceable without a database.
- Repository interfaces must provide a stable boundary for the service layer.
- The model must align with the PostgreSQL schema in `migrations/`.

## Design

The domain model is implemented in the shared `ruckchat-domain` crate. Each
aggregate has its own module with a struct and a validated constructor. Value
objects such as `Role` and `ConversationType` are represented as enums.

Repository traits are defined with `async-trait` and kept minimal. The server
crate will provide SQLx implementations in a later phase.

Key design points:

- Constructors return `ruckchat_common::Result<Self>` and validate inputs.
- Soft deletes are represented by `Option<OffsetDateTime>` fields.
- Timestamps use `OffsetDateTime` from `ruckchat-common`.
- Identifiers use the strongly typed UUID wrappers from `ruckchat-id`.

## Entities

| Entity | Key Invariants |
|--------|----------------|
| `User` | Email format valid; display name 1-100 characters; password hash non-empty. |
| `Organization` | Name non-empty; slug 3-63 URL-safe characters. |
| `OrganizationMembership` | One membership per user per organization; role is `owner`, `admin`, or `member`. |
| `OrganizationSettings` | File size and storage quotas must be positive. |
| `Channel` | Name 1-80 URL-safe characters; belongs to an organization and creator. |
| `ChannelMembership` | One membership per user per channel. |
| `DirectMessageConversation` | At least two unique members; no duplicate members. |
| `Message` | Content 1-4000 characters; soft delete clears content. |
| `Reaction` | Emoji non-empty. |
| `File` | File name, MIME type, and storage path non-empty; size positive. |
| `Session` | Token hash non-empty; expiration in the future. |

## Acceptance Criteria

- `cargo test -p ruckchat-domain` passes with unit tests covering every entity constructor and invariant.
- `cargo check --workspace`, `cargo fmt --check`, and `cargo clippy --workspace -- -D warnings` pass.
- Repository traits compile against `async-trait` and reference only domain types.
- `book/004-Domain.md` and this document describe the implemented model.
