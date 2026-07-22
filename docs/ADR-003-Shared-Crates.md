# ADR 003: Shared Crate Layout

## Status

Accepted

## Context

The RuckChat workspace must share code between the server, desktop client,
mobile client, plugin SDK, and MCP server. A single `shared` crate would
force every consumer to compile utilities it does not need and would blur
public API boundaries.

## Decision

Split shared code into three focused crates under `crates/`:

- `ruckchat-id` — strongly typed UUID identifiers for domain entities.
- `ruckchat-common` — errors, validation rules, and time helpers.
- `ruckchat-config` — environment/file configuration primitives and secrets handling.

A thin `server` crate re-exports these for its own use. No umbrella `shared`
crate is kept; consumers depend only on the crates they need.

## Consequences

- Each crate has a small, well-defined public API and can be tested in
  isolation.
- Downstream crates avoid unnecessary dependencies.
- New shared concepts must land in the correct crate rather than a catch-all
  module.
