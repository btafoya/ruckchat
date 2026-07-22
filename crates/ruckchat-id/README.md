# ruckchat-id

Strongly typed UUID identifiers for RuckChat domain entities.

## Usage

```rust
use ruckchat_id::{UserId, ChannelId};

let user_id = UserId::new();
let channel_id = ChannelId::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8")?;
```

Identifiers are newtype wrappers around [`uuid::Uuid`]. They implement
`Display`, `Serialize`/`Deserialize`, and `From<Uuid>` conversions.
