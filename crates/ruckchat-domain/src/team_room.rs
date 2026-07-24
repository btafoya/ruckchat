//! Team-room link aggregate.

use ruckchat_common::time::OffsetDateTime;
use ruckchat_id::{ChannelId, TeamId};
use serde::{Deserialize, Serialize};

/// Links a channel or room to a team.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamRoom {
    /// Team identifier.
    pub team_id: TeamId,
    /// Channel identifier.
    pub channel_id: ChannelId,
    /// Timestamp when the room was added to the team.
    #[serde(with = "time::serde::rfc3339")]
    pub added_at: OffsetDateTime,
}

impl TeamRoom {
    /// Creates a team-room link.
    #[must_use]
    pub fn new(team_id: TeamId, channel_id: ChannelId) -> Self {
        Self {
            team_id,
            channel_id,
            added_at: OffsetDateTime::now_utc(),
        }
    }
}
