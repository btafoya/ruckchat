# ruckchat-common

Shared primitives for RuckChat: errors, validation rules, and time helpers.

## Usage

```rust
use ruckchat_common::{validate_email, validate_slug, time::now_utc, Error};

assert!(validate_email("user@example.com"));
assert!(validate_slug("acme-corp"));
```

This crate intentionally contains only reusable, dependency-light utilities.
Service-specific wiring belongs in the server or client crates.
