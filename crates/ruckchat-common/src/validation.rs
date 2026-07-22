//! Domain validation constraints and helpers.

/// Minimum display name length in characters.
pub const DISPLAY_NAME_MIN_LEN: usize = 1;
/// Maximum display name length in characters.
pub const DISPLAY_NAME_MAX_LEN: usize = 100;
/// Minimum organization/channel slug length in characters.
pub const SLUG_MIN_LEN: usize = 3;
/// Maximum organization/channel slug length in characters.
pub const SLUG_MAX_LEN: usize = 63;
/// Minimum channel name length in characters.
pub const CHANNEL_NAME_MIN_LEN: usize = 1;
/// Maximum channel name length in characters.
pub const CHANNEL_NAME_MAX_LEN: usize = 80;
/// Default maximum message content length in characters.
pub const MESSAGE_CONTENT_MAX_LEN: usize = 4_000;
/// Default maximum file size in bytes.
pub const DEFAULT_MAX_FILE_SIZE_BYTES: i64 = 25 * 1_024 * 1_024;
/// Default organization storage quota in bytes.
pub const DEFAULT_ORG_STORAGE_QUOTA_BYTES: i64 = 10 * 1_024 * 1_024 * 1_024;

/// Validates that a display name is within allowed length bounds.
pub fn validate_display_name(name: &str) -> bool {
    let len = name.chars().count();
    (DISPLAY_NAME_MIN_LEN..=DISPLAY_NAME_MAX_LEN).contains(&len)
}

/// Validates that a channel name length is within allowed bounds.
pub fn validate_channel_name_length(name: &str) -> bool {
    let len = name.chars().count();
    (CHANNEL_NAME_MIN_LEN..=CHANNEL_NAME_MAX_LEN).contains(&len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_bounds() {
        assert!(!validate_display_name(""));
        assert!(validate_display_name("a"));
        assert!(validate_display_name(&"x".repeat(DISPLAY_NAME_MAX_LEN)));
        assert!(!validate_display_name(
            &"x".repeat(DISPLAY_NAME_MAX_LEN + 1)
        ));
    }

    #[test]
    fn channel_name_length_bounds() {
        assert!(!validate_channel_name_length(""));
        assert!(validate_channel_name_length("general"));
        assert!(validate_channel_name_length(
            &"x".repeat(CHANNEL_NAME_MAX_LEN)
        ));
        assert!(!validate_channel_name_length(
            &"x".repeat(CHANNEL_NAME_MAX_LEN + 1)
        ));
    }
}
