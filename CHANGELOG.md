# Changelog

## [v0.2.1-alpha-r2] - 2026-07-23

- Fix release.sh local-ahead check when origin/main is behind
- Fix release workflow Tauri build target flag
- Stop tracking .serena/ directory
- Fix release tag signing flag (-S is for commits, -s for tags)

## [v0.2.1-alpha-r1] - 2026-07-23

- Source .env.testing in release.sh for SQLx compile-time checks
- Update README and CLAUDE.md for release automation
- Add release automation script
- Install desktop dependencies in build-server.sh before Web UI build
- Install desktop dependencies before the Web UI build in CI
- Install desktop dependencies and exclude test helpers in Web UI build
- Exclude desktop test files from the Web UI TypeScript build
- Use pnpm 11 in GitHub Actions workflows
- Add missing packages field to pnpm workspace configs
- Build Web UI assets in CI before workspace cargo commands
- Fix GitHub Actions failures and migrate workflows to Node 24
- Split Docker Compose into runtime and source-build variants and publish server image in releases
- Refresh SQLx offline metadata for latest migrations and repositories
- Implement standalone rocketchat2ruckchat migration tool
- Set sqlx to b0.8.6 for docker build
- Add admin import API and MigrationData v2 for RocketChat migration (Phase A)
- Implement Phase 12 migration and packaging tools
- For RocketChat to RuclChat transfer tool
- Implement Phase 10 browser-based Web UI with Web Push, PWA, and shared React platform
- Add Phase 10 Web UI design and update planning docs
- Update README and CLAUDE.md for completed Phases 7-9
- Implement Phase 9 runtime YAML configuration
- Implement Phase 7 plugin SDK
- Fix server startup: support localhost in base_url and RUCKCHAT_BASE_URL env var
- Fix Tauri bundle category and update BUILD.md for Linux deb bundling
- Fix release-build unused variable and add alpha build guide
- Update root README and CLAUDE for Phase 8 completion
- Complete Phase 8 desktop native integrations and production packaging
- Implement desktop messaging features: composer, reactions, attachments, threads, DMs
- Update CLAUDE.md: add quick start, align clippy command, note DATABASE_URL for builds
- Rules for Claude
- Rules for Claude
- Update README: correct desktop dev URL, test commands, phase 8 status, and layout
- Add ponytail direction to CLAUDE.md rules
- Apply ponytail: tighten CLAUDE.md, remove redundant sections
- Update CLAUDE.md: fix codegraph command, desktop layout, phase 8 status, env sourcing, and gotchas
- Implement desktop state stores and real-time sync
- Implement desktop API client, auth flow, and three-pane UI shell
- Update README.md and CLAUDE.md for Phase 8 desktop client
- Scaffold Phase 8 desktop client with Tauri v2 and React 19
- Update README, server README, and CLAUDE.md with test env details and MCP service bridge
- Update README and server README for Phase 6 MCP completion
- Complete Phase 6 MCP server with Streamable HTTP transport, tools, resources, tests, and docs
- Status update
- Implement Phase 5 WebSocket server, reaction service, and real-time events
- Update README and set project license to MIT
- Implement Phase 4 REST API layer
- Update CLAUDE.md with project status, commands, architecture, and implementation loop
- Implement Phase 3 service and repository layer in ruckchat-server
- Implement Phase 2 domain layer with shared ruckchat-domain crate
- Initialize workspace and implement Phase 1 foundation

