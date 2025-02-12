use crate::api::clock;
use crate::sys;

use time::{Duration, OffsetDateTime, UtcOffset};

pub fn now() -> OffsetDateTime {
    now_utc().to_offset(offset())
}

pub fn now_utc() -> OffsetDateTime {
    let s = clock::realtime(); // Since Unix Epoch
    let ns = Duration::nanoseconds(libm::floor(1e9 * (s - libm::floor(s))) as i64);
    OffsetDateTime::from_unix_timestamp(s as i64) + ns
}

pub fn from_timestamp(ts: i64) -> OffsetDateTime {
    from_timestamp_utc(ts).to_offset(offset())
}

pub fn from_timestamp_utc(ts: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(ts)
}

fn offset() -> UtcOffset {
    if let Some(tz) = sys::process::env("TZ") {
        if let Ok(offset) = tz.parse::<i32>() {
            return UtcOffset::seconds(offset);
        }
    }
    UtcOffset::UTC
}
