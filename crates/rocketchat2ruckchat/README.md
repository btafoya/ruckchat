# rocketchat2ruckchat

Standalone migration tool that copies a RocketChat workspace into a RuckChat
organization. It inventories users, rooms, messages, reactions, files, custom
emoji, teams, roles, and permissions; maps RocketChat identifiers to
deterministic RuckChat identifiers; and imports the result through the RuckChat
admin migration API.

## Usage

```bash
cargo run -p rocketchat2ruckchat -- --config migration.yaml --dry-run
cargo run -p rocketchat2ruckchat -- --config migration.yaml --apply
```

Run with `--interactive` to be prompted for source/target credentials and the
SQLite mapping store path.
