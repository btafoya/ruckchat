# ruckchat-config

Configuration primitives shared by RuckChat applications.

## Usage

```rust
use ruckchat_config::AppConfig;

let cfg = AppConfig::load()?;
cfg.validate()?;
```

Configuration is loaded from `ruckchat.toml`, environment variables prefixed
with `RUCKCHAT_`, and sensible defaults. Secrets are wrapped with
[`secrecy::SecretString`] so they are not logged accidentally.
