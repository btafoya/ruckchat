//! Channel membership aggregate.

use ruckchat_common::{Result, time::OffsetDateTime};
use ruckchat_id::{ChannelId, UserId};
use serde::{Deserialize, Serialize};

/// Links a user to a channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelMembership {
    /// User identifier.
    pub user_id: UserId,
    /// Channel identifier.
    pub channel_id: ChannelId,
    /// Timestamp when the user joined.
    pub joined_at: OffsetDateTime,
}

impl ChannelMembership {
    /// Creates a new channel membership.
    pub fn new(user_id: UserId, channel_id: ChannelId) -> Result<Self> {
        Ok(Self {
            user_id,
            channel_id,
            joined_at: OffsetDateTime::now_utc(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_membership() {
        let user_id = UserId::new();
        let channel_id = ChannelId::new();
        let membership = ChannelMembership::new(user_id, channel_id).expect("valid membership");
        assert_eq!(membership.user_id, user_id);
        assert_eq!(membership.channel_id, channel_id);
    }
}
