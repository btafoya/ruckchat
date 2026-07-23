# 018 - Roadmap

## Phases

### Phase 0: Foundation

- Complete the production handbook.
- Bootstrap Cargo workspace.
- Set up CI, linting, and test infrastructure.
- Deliver a working local development environment.

### Phase 1: MVP

The MVP delivers the ten features listed in the product specification:

1. Authentication
2. Organizations
3. Channels
4. Direct Messages
5. Real-time messaging
6. File uploads
7. Search
8. Notifications
9. Plugin SDK
10. Packaging

### Phase 2: Hardening

- Performance optimization for large organizations.
- Improved search (potentially adding trigram or custom ranking).
- Mobile push notifications (FCM/APNs).
- Admin dashboard and audit logging.
- Federation research and design.

### Phase 3: Scale and Ecosystem

- Horizontal scaling options (requires architecture change).
- Official plugin marketplace.
- Managed hosting offering.
- Advanced compliance and data retention features.

## Sprint Mapping

The implementation plan in `docs/IMPLEMENTATION_PLAN.md` defines the following sprints:

| Sprint | Focus | Handbook Chapters |
|--------|-------|-------------------|
| 1 | Workspace bootstrap | 003, 017 |
| 2 | Configuration | 003, 006, 014 |
| 3 | Database and migrations | 005, 015 |
| 4 | Authentication | 004, 006, 009, 013 |
| 5 | Organizations | 004, 006, 009 |
| 6 | Channels | 004, 006, 009, 010 |
| 7 | Messaging | 004, 006, 009, 010 |
| 8 | WebSockets | 006, 010 |
| 9 | Desktop client | 007, 002 |
| 10 | Mobile client | 008, 002 |
| 11 | Search | 005, 006, 009 |
| 12 | File uploads | 006, 009, 013 |
| 13 | Notifications | 006, 010 |
| 14 | Plugin SDK | 012 |
| 15 | Packaging and release | 014, 017 |

## Definition of Done

A feature is complete when:

- Code is merged to the main branch.
- Unit and integration tests pass.
- `cargo fmt` and `cargo clippy` report no warnings.
- Documentation is updated, including this handbook if affected.
- OpenAPI specification is updated for API changes.
- CI is green.
- No `TODO` markers remain in the touched code.

## Release Cadence

- MVP target: complete all sprints before the v1.0.0 release.
- Patch releases ship bug fixes and security updates as needed.
- Minor releases ship new features after MVP.
- Major releases signal breaking changes or architectural shifts.

## Current Status

- Phases 1–12 are complete.
- The server runs as a single Rust binary or Docker container and supports
  schema migrations, domain-data export/import, WebSocket real-time events,
  MCP, plugins, runtime YAML configuration, and the browser Web UI.
- The desktop client builds cross-platform installers via GitHub Actions on
  version tags.
- Phase 13 (mobile Flutter client) is planned next.
