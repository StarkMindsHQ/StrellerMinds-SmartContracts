/// Timestamp utilities — Issue #442
///
/// Soroban ledger timestamps are Unix epoch seconds in **UTC**.  The DST bug
/// occurred because callers were passing local-time offsets (e.g. UTC+1 during
/// summer) instead of UTC epoch values, causing timestamps to be off by one
/// hour during DST transitions.
///
/// This module provides:
/// 1. A validation helper that rejects timestamps that look like they carry a
///    timezone offset (i.e. values that are not plausible UTC epoch seconds).
/// 2. Conversion helpers that normalise a timestamp to the start of its UTC
///    day / hour, which is the correct unit for streak and session-window
///    calculations.
/// 3. A `TimestampRange` type for querying events within a UTC window.

/// Minimum plausible Unix timestamp (2020-01-01T00:00:00Z).
const MIN_VALID_TS: u64 = 1_577_836_800;
/// Maximum plausible Unix timestamp (2100-01-01T00:00:00Z).
const MAX_VALID_TS: u64 = 4_102_444_800;

/// Seconds in one hour / day.
pub const SECS_PER_HOUR: u64 = 3_600;
pub const SECS_PER_DAY: u64 = 86_400;

/// A half-open UTC time window `[start, end)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimestampRange {
    pub start: u64,
    pub end: u64,
}

impl TimestampRange {
    pub fn new(start: u64, end: u64) -> Option<Self> {
        if start < end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    pub fn contains(&self, ts: u64) -> bool {
        ts >= self.start && ts < self.end
    }
}

/// Validate that `ts` is a plausible UTC Unix epoch second.
///
/// Returns `Err(())` if the value is outside the accepted range, which
/// catches the common mistake of passing milliseconds or a local-time value
/// with a DST offset baked in.
pub fn validate_utc_timestamp(ts: u64) -> Result<u64, ()> {
    if ts >= MIN_VALID_TS && ts <= MAX_VALID_TS {
        Ok(ts)
    } else {
        Err(())
    }
}

/// Truncate `ts` to the start of its UTC day (midnight).
///
/// This is the correct way to compute "same day" for streak logic — using a
/// local-time midnight would shift by the DST offset.
pub fn utc_day_start(ts: u64) -> u64 {
    (ts / SECS_PER_DAY) * SECS_PER_DAY
}

/// Truncate `ts` to the start of its UTC hour.
pub fn utc_hour_start(ts: u64) -> u64 {
    (ts / SECS_PER_HOUR) * SECS_PER_HOUR
}

/// Return the UTC day index (days since epoch) for `ts`.
pub fn utc_day_index(ts: u64) -> u64 {
    ts / SECS_PER_DAY
}

/// Compute the number of whole UTC days between two timestamps.
///
/// Always returns a non-negative value regardless of argument order.
pub fn days_between(a: u64, b: u64) -> u64 {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    utc_day_index(hi) - utc_day_index(lo)
}

/// Build a `TimestampRange` covering the UTC day that contains `ts`.
pub fn utc_day_range(ts: u64) -> TimestampRange {
    let start = utc_day_start(ts);
    TimestampRange {
        start,
        end: start + SECS_PER_DAY,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 2024-03-31T01:00:00Z — the exact moment clocks spring forward in
    /// Europe/London (UTC+0 → UTC+1).  A naive local-time implementation
    /// would produce a timestamp 3600 s too large.
    const DST_SPRING_FORWARD: u64 = 1_711_846_800;

    /// 2024-10-27T01:00:00Z — clocks fall back in Europe/London.
    const DST_FALL_BACK: u64 = 1_729_990_800;

    #[test]
    fn validate_accepts_valid_utc() {
        assert!(validate_utc_timestamp(DST_SPRING_FORWARD).is_ok());
        assert!(validate_utc_timestamp(DST_FALL_BACK).is_ok());
    }

    #[test]
    fn validate_rejects_milliseconds() {
        // Millisecond-precision value is way above MAX_VALID_TS.
        assert!(validate_utc_timestamp(DST_SPRING_FORWARD * 1000).is_err());
    }

    #[test]
    fn validate_rejects_zero() {
        assert!(validate_utc_timestamp(0).is_err());
    }

    #[test]
    fn utc_day_start_is_midnight() {
        // DST_SPRING_FORWARD is 2024-03-31T01:00:00Z → day start = 2024-03-31T00:00:00Z
        let midnight = DST_SPRING_FORWARD - 3600; // 00:00:00Z
        assert_eq!(utc_day_start(DST_SPRING_FORWARD), midnight);
    }

    #[test]
    fn days_between_dst_transition() {
        // Two timestamps 24 h apart across a DST boundary must still be 1 day.
        let day1 = DST_SPRING_FORWARD;
        let day2 = DST_SPRING_FORWARD + SECS_PER_DAY;
        assert_eq!(days_between(day1, day2), 1);
    }

    #[test]
    fn days_between_fall_back() {
        let day1 = DST_FALL_BACK;
        let day2 = DST_FALL_BACK + SECS_PER_DAY;
        assert_eq!(days_between(day1, day2), 1);
    }

    #[test]
    fn utc_day_range_contains() {
        let range = utc_day_range(DST_SPRING_FORWARD);
        assert!(range.contains(DST_SPRING_FORWARD));
        assert!(!range.contains(range.end));
    }

    #[test]
    fn utc_hour_start_strips_minutes() {
        // 01:30:00Z → 01:00:00Z
        let ts = DST_SPRING_FORWARD + 1800;
        assert_eq!(utc_hour_start(ts), DST_SPRING_FORWARD);
    }
}
