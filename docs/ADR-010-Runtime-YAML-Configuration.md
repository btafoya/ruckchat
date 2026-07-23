# ADR-010: Runtime YAML Configuration

## Status

Accepted — implemented in Phase 9.

## Context

The server previously loaded runtime settings through a combination of an optional
`ruckchat.toml` file in the working directory and `RUCKCHAT_*` environment
variables. The database URL was read separately from `DATABASE_URL`. This
scatter created several operational problems:

- No standard, out-of-tree location for production configuration.
- Docker Compose, systemd, and bare-metal installs each used different conventions.
- Environment-variable overrides made it hard to know the effective configuration
  without inspecting the process environment.
- There was no first-run or template-generation experience.

We needed a single, explicit, administrator-friendly configuration layer that
lives outside the project directory and works uniformly across deployment targets.

## Decision

We will use a single YAML file, `ruckchat.yaml`, as the sole source of truth for
server runtime configuration.

- **Format:** YAML, for readability and ops familiarity.
- **Default path:** platform-specific.
  - Linux: `/etc/ruckchat/ruckchat.yaml`
  - macOS: `/Library/Application Support/RuckChat/ruckchat.yaml`
  - Windows: `%ProgramData%\RuckChat\ruckchat.yaml`
- **Override:** `--config <path>` CLI argument.
- **Precedence:** the YAML file is the only runtime source. No `.env` files and no
  environment-variable overrides are read for server runtime behavior.
- **Reload:** none. The file is read once at startup; administrators restart the
  service after edits.
- **Generation:** `ruckchat-server --init-config [path]` writes a commented
  default file and exits.

The configuration schema is:

```yaml
app_name: "RuckChat"
environment: "development"
base_url: "http://localhost:3000"
log_level: "info"
database:
  url: "postgres://ruckchat:ruckchat@localhost/ruckchat"
  max_connections: 10
mcp:
  enabled: true
  require_confirmation: true
plugins:
  directory: "/var/lib/ruckchat/plugins"
```

The schema also includes commented placeholders for future phases (retention,
federation, limits) so administrators can see the roadmap without the server
failing on unknown keys.

## Consequences

### Positive

- One file, one location, one mental model for operators.
- systemd units and Docker mounts both point to the same path convention.
- Configuration is version-controllable and diffable.
- Secrets are kept in one place and never mixed with `.env` files.
- `--init-config` gives administrators a documented starting point.

### Negative

- `DATABASE_URL` must still exist at **compile time** for SQLx query verification,
  which can be surprising. We document it clearly as a build/development requirement,
  not a runtime setting.
- Admins must restart the service to apply configuration changes.
- We retired the `config` crate merge behavior; any future need for env overrides
  must be re-evaluated against the sole-source-of-truth rule.

## Implementation

- `crates/ruckchat-config/src/lib.rs` uses `yaml_serde` for parsing, exposes
  `AppConfig::load()`, `AppConfig::load_from_path()`, and
  `AppConfig::write_default_to()`, and defines the nested config structs.
- `server/src/main.rs` parses `--config` and `--init-config`, loads the file,
  and wires the database URL from `config.database.url`.
- `server/src/state.rs` adds `AppState::from_config` so production wiring is
  centralized.
- Integration tests remain hermetic: `server/tests/common/mod.rs` continues to
  build `AppState` directly from a test pool, bypassing file-based config.

## Related

- `book/014-Deployment.md`
- `book/016-Operations.md`
- `server/README.md`
