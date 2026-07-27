# ADR-018: Transactional Email via Postmark

## Status

Accepted — implemented alongside the RocketChat migration tool redesign
(`docs/DESIGN-RocketChat-Migration.md`).

## Context

RuckChat had no email-sending capability of any kind. The server-admin
`reset_password` action (`server/src/services/server_admin.rs`) generated a
temporary password and returned it in the JSON response with no delivery
mechanism — an admin had to read it off `EditUserModal.tsx` and communicate
it manually. This became a hard blocker while redesigning the RocketChat
migration tool: migrating N users each needing a real, usable password meant
either building a one-off notification path inside the migration tool, or
finally giving RuckChat a real email primitive. We chose the latter, since
the same gap existed independently of migration.

We needed to decide:

- Whether to build a general-purpose email capability or scope it narrowly to
  the migration tool's own needs.
- Which Postmark Rust client to use and how to structure email bodies.
- Whether to wire it into the existing `reset_password` action.
- How to handle servers that haven't configured Postmark yet.
- How to keep bulk migration-triggered email sending an explicit, reversible
  step rather than an automatic side effect of writing data.

## Decision

### General-purpose `ruckchat-email` crate

A new crate, `crates/ruckchat-email`, wraps the `postmark` crate (v2.0.1,
reqwest-backed) behind `EmailClient` with two typed methods:
`send_migration_credentials` and `send_password_reset`. Message bodies are
composed inline as HTML/text in Rust code, not via Postmark's server-side
template feature — wording changes ship through a normal code deploy, with no
Postmark-dashboard dependency. Both `ruckchat-server` and the standalone
`rocketchat2ruckchat` migration tool depend on this crate directly, avoiding
a duplicate Postmark integration in each.

### Optional, graceful-degradation configuration

`ruckchat.yaml` gained an optional `email:` section
(`ruckchat_config::EmailConfig { server_token, from_address }`). When absent,
`AppState` holds no `EmailClient` and `reset_password` keeps returning the
plaintext password in its response exactly as before — no regression for
servers that haven't configured Postmark.

### `reset_password` wiring

`ServerAdminService::reset_password` now attempts an email send when
configured, after hashing and persisting the new password. A failed send
does not fail the request — the password is still returned in the response
as a fallback — and the outcome (`email_sent: bool`) is recorded in the
existing `user.password_reset` audit log entry rather than as a new action
type.

### Migration credential delivery is a separate, explicit step

The migration tool generates a real, usable temporary password per migrated
user at creation time (superseding an earlier draft that generated an
unusable placeholder hash relying on a post-migration admin action per
user — that didn't scale past a handful of accounts). Emailing those
passwords requires an explicit `--send-emails` flag, independent of
`--apply`: writing data to Postgres is inspectable and low-risk, emailing
real people is neither, so an operator can migrate data first and decide to
notify users as a deliberate second step. Individual send failures are
collected into the migration report rather than aborting the run.

## Consequences

### Positive

- RuckChat now has a general transactional-email primitive, reusable for
  future flows (self-service forgot-password, signup verification, org
  invites) without another vendor-integration decision.
- Migrated users get real, working passwords via email instead of requiring
  N individual admin-triggered resets.
- Existing `reset_password` behavior is unchanged for servers without
  Postmark configured.

### Negative

- Only Postmark is supported; swapping providers means changing
  `ruckchat-email`'s internals, not just configuration.
- Email bodies are hardcoded in Rust rather than editable via a
  Postmark-side template, trading dashboard-editability for code-review
  control.
- No self-service forgot-password, signup verification, or org-invite email
  flows were added — `ruckchat-email` is general-purpose but only wired into
  the two use cases above.

## Implementation

- `crates/ruckchat-email/src/lib.rs` — `EmailConfig`, `EmailClient`,
  `EmailError`, `send_migration_credentials`, `send_password_reset`.
- `crates/ruckchat-config/src/lib.rs` — `AppConfig.email: Option<EmailConfig>`.
- `server/src/state.rs` — builds an optional `ruckchat_email::EmailClient`
  from config and threads it into `ServerAdminServiceDeps`.
- `server/src/services/server_admin.rs` — `reset_password` sends the email
  when configured; audit log records `email_sent`.
- `crates/rocketchat2ruckchat/src/transform.rs` — generates real temporary
  passwords per newly migrated user.
- `crates/rocketchat2ruckchat/src/pipeline.rs` — `send_credential_emails`,
  gated by the `--send-emails` CLI flag.
- `crates/rocketchat2ruckchat/src/config.rs` — `Cli.send_emails`,
  `ResolvedConfig.email: Option<EmailConfig>`.
- `crates/rocketchat2ruckchat/src/report.rs` — `CredentialEmailSummary` in
  the migration report.

## Related

- `docs/DESIGN-RocketChat-Migration.md`
- `docs/ADR-012-Migration-and-Packaging.md`
- `docs/ADR-013-Web-UI-Admin-Panel.md` (server-admin reset-password action)
