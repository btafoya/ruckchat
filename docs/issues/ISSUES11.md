# ISSUES11 — Shell Commands Wrapped by Unrequested `tokenjuice wrap`

## Source

> agent identified error: "Stopping Bash use — even a bare echo hello is now being blocked with the same message: something is wrapping every shell command through an unrequested binary (tokenjuice wrap --source claude-code) that neither I nor your config invoked. This started appearing partway through this session (earlier commands like cargo test, pnpm test, git status --porcelain ran fine and returned real output; the wrapping/blocking only kicked in on the later calls). I won't try to route around it further — this looks like either a compromised PATH/shell rc file or an external process injecting itself into the session, and it's worth checking on your end before running any more shell commands in this session." — open

## Research Summary

### Current state

- This is an environment/session issue rather than a RuckChat product bug.
- The report states that earlier shell commands executed normally, and later commands began being intercepted by an unrequested `tokenjuice wrap --source claude-code` wrapper.
- No RuckChat code references `tokenjuice`.
- The issue was observed during an agent session while running build/test commands.

### Gaps

1. **No documented cause** — it is unclear whether a shell configuration, IDE plugin, security tool, or another process injected the wrapper.
2. **No recovery procedure** — if the wrapper returns, there is no known safe workaround documented for this project.
3. **No prevention step** — the build/development workflow does not currently include a check for unexpected command interceptors.

### Affected files

- None in the RuckChat codebase.
- Potentially: local shell profile (`~/.bashrc`, `~/.zshrc`), IDE settings, or system security tooling.

## Open Questions

1. **Is this reproducible?**
   - Does it happen in a fresh terminal/session?
   - Does it correlate with a specific tool or extension being active?

2. **What is the source of `tokenjuice`?**
   - Is it installed intentionally as a security/audit tool?
   - Is it present in `PATH` or shell aliases/functions?

3. **Should the project document a workaround?**
   - Yes, add a troubleshooting note to `CLAUDE.md` or a developer README.
   - No, treat it as a one-off local environment issue.

## Decisions

- Pending investigation by the user; not a RuckChat feature or code change.

## Status

Open (environment/infrastructure).
