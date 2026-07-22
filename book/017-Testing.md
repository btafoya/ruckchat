# 017 - Testing

## Testing Strategy

RuckChat uses three levels of automated testing:

1. **Unit tests** — isolated logic, validators, and pure functions.
2. **Integration tests** — repository and service layer tests against a real PostgreSQL database.
3. **End-to-end tests** — API contract tests using the REST API and WebSocket endpoints.

## Unit Tests

- Live in the same file as the code under test or in a `tests/` module within the crate.
- Do not require a database or network.
- Cover domain invariants, validation rules, error mapping, and utility functions.

## Integration Tests

- Run against a temporary PostgreSQL database created for the test run.
- Use `sqlx::test` or a custom test harness that applies migrations.
- Cover repository queries, service orchestration, and authentication flows.
- Each test starts from a clean database state.

## End-to-End Tests

- Use the running server binary with a test configuration.
- Exercise the REST API and WebSocket protocol with a test client.
- Cover critical user flows:
  - Registration and login.
  - Organization creation and invitation.
  - Channel creation and messaging.
  - File upload and download.
  - Search.

## Test Data

- Test fixtures are defined in `server/tests/fixtures.sql` and loaded per test.
- Fixtures use deterministic UUIDs and timestamps where possible.
- No production data or secrets are used in tests.

## Client Tests

- Desktop React components use Vitest and React Testing Library.
- Mobile Flutter widgets use Flutter's built-in widget testing.
- Client tests do not require a running server; API calls are mocked.

## Continuous Integration

CI runs on every pull request and push to main:

1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test --workspace`
4. `cargo sqlx prepare --check`
5. `pnpm lint` and `pnpm test` for the desktop client.
6. `flutter test` for the mobile client.
7. OpenAPI spec validation.

## Test Database

- CI uses a PostgreSQL service container.
- Local development uses a Docker Compose PostgreSQL container started with `docker compose up db`.
- Database URL for tests: `postgres://ruckchat_test:ruckchat_test@localhost/ruckchat_test`.

## Coverage

- Line coverage is reported but not enforced as a hard gate in v1.
- Critical paths (auth, messaging, authorization) should have near-complete coverage.
- Coverage reports are generated with `cargo tarpaulin` in CI artifacts.

## Manual Testing

- Smoke tests are documented in `docs/smoke-tests.md`.
- Release candidates pass a manual smoke checklist before tagging.

## Regression Tests

- Bugs receive an automated regression test before the fix is merged.
- Regression tests reference the issue or PR number in a comment.
