#[cfg(feature = "chrono")]
use chrono::{DateTime as ChronoDateTime, FixedOffset, Local, TimeZone, Utc};

/// A datetime type.
///
/// If the `chrono` feature is enabled, then `From<chrono::DateTime<TZ>>` and `Into<chrono::DateTime<TZ>>` are implemented for it.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct DateTime {
    /// Number of non-leap-milliseconds since January 1, 1970 UTC.
    pub seconds: i64,

    /// Number of nanoseconds since the last second boundary.
    pub nano_seconds: u32,
}

#[cfg(feature = "chrono")]
impl From<DateTime> for ChronoDateTime<Utc> {
    fn from(t: DateTime) -> Self {
        Utc.timestamp(t.seconds, t.nano_seconds)
    }
}

#[cfg(feature = "chrono")]
impl From<ChronoDateTime<Utc>> for DateTime {
    fn from(t: ChronoDateTime<Utc>) -> Self {
        Self {
            seconds: t.timestamp_millis(),
            nano_seconds: t.timestamp_subsec_nanos(),
        }
    }
}

#[cfg(feature = "chrono")]
impl From<DateTime> for ChronoDateTime<Local> {
    fn from(t: DateTime) -> Self {
        Local.timestamp(t.seconds, t.nano_seconds)
    }
}

#[cfg(feature = "chrono")]
impl From<ChronoDateTime<Local>> for DateTime {
    fn from(t: ChronoDateTime<Local>) -> Self {
        Self {
            seconds: t.timestamp_millis(),
            nano_seconds: t.timestamp_subsec_nanos(),
        }
    }
}
