//! Time helpers built on top of [`time`].

pub use time::OffsetDateTime;

/// Returns the current UTC time.
#[must_use]
pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_is_recent() {
        let t = now_utc();
        let lower = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
        assert!(t > lower);
    }
}
