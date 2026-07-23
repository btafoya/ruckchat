# 003 - ADRs

This directory contains Architecture Decision Records (ADRs) for RuckChat.

## Active ADRs

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-003](ADR-003-Shared-Crates.md) | Shared Crates | Accepted |
| [ADR-004](ADR-004-Migrations.md) | Database Migrations | Accepted |
| [ADR-005](ADR-005-Domain-Crate.md) | Domain Crate | Accepted |
| [ADR-006](ADR-006-WebSocket-Real-Time-Events.md) | WebSocket Real-Time Events | Accepted |
| [ADR-007](ADR-007-MCP-Server.md) | MCP Server | Accepted |
| [ADR-008](ADR-008-Desktop-Client.md) | Desktop Client | Accepted |
| [ADR-009](ADR-009-Plugin-SDK.md) | Plugin SDK | Accepted |
| [ADR-010](ADR-010-Runtime-YAML-Configuration.md) | Runtime YAML Configuration | Accepted |

## Purpose

ADRs record significant architectural decisions, the context in which they were
made, and the consequences we accept. Each ADR is a short, durable document that
helps future contributors understand why the codebase is shaped the way it is.

## When to write an ADR

Create a new ADR when a decision:

- Changes a public API or cross-crate boundary.
- Introduces a new dependency or removes a widely used one.
- Alters deployment, operations, or security posture.
- Is likely to be revisited or questioned later.

## Format

ADRs follow the template used in this directory:

- Status
- Context
- Decision
- Consequences
- Implementation notes
- Related documents
