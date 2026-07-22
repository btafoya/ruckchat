# Build Guide — Alpha Release

This document describes how to build the RuckChat server and desktop client for
alpha testing.

## Prerequisites

- Rust toolchain (see `rust-version` in `Cargo.toml`)
- Node.js and `pnpm`
- PostgreSQL server running locally (for integration tests and the running server)
- Linux system dependencies for Tauri desktop bundling:
  - `libayatana-appindicator3-dev` (or `libappindicator3-dev` on older distros)
  - Standard Tauri prerequisites (`libwebkit2gtk-4.1-dev`, `build-essential`, etc.)

## Environment

A `.env.testing` file is provided at the repository root. Source it before any
Cargo command that uses `sqlx` query macros or the integration tests:

```bash
set -a
source .env.testing
set +a
```

Required variables:

- `DATABASE_URL` — PostgreSQL connection string used by `cargo build`/`check`
  and the running server.
- `RUCKCHAT_TEST_ADMIN_DATABASE_URL` — only required for schema/migration
  integration tests.

## Server build

Clean and release-build the entire workspace:

```bash
cargo clean
cargo build --workspace --release
```

Quality gates to run before shipping:

```bash
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Server artifact:

- Binary: `target/release/ruckchat-server`

Run the server:

```bash
./target/release/ruckchat-server
```

The server serves HTTP (REST + WebSocket + MCP) on the configured address
(`RUCKCHAT_BASE_URL`, default `http://localhost:3000`).

## Desktop build

Install dependencies:

```bash
cd desktop
pnpm install
```

Type-check and run unit tests:

```bash
pnpm typecheck
pnpm test
```

Build the production frontend and the release Tauri binary:

```bash
pnpm tauri build --no-bundle
```

Desktop artifact:

- Binary: `target/release/ruckchat-desktop`

### Linux bundling note

On this Linux environment, full installer bundling (`pnpm tauri build`) fails at
the packaging stage with:

```
Can't detect any appindicator library
```

Install the development headers and rerun the full build to produce installers:

```bash
sudo apt-get install -y libayatana-appindicator3-dev
cd desktop
pnpm tauri build
```

Full bundling produces platform-specific installers under:

- `target/release/bundle/deb/`
- `target/release/bundle/appimage/`
- `target/release/bundle/dmg/` (macOS)
- `target/release/bundle/msi/` / `target/release/bundle/nsis/` (Windows)

## Alpha packaging checklist

- [ ] `cargo build --workspace --release` succeeds
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [ ] `pnpm typecheck` and `pnpm test` pass
- [ ] Desktop release binary builds (`pnpm tauri build --no-bundle`)
- [ ] Linux installer bundles built after installing `libayatana-appindicator3-dev`
- [ ] `.env` files and secrets are not committed (verify `.gitignore`)
