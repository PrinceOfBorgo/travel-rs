use chrono::{Datelike, Timelike};
use fluent_datetime::FluentDateTime;

/// An extension trait for `surrealdb::Datetime` to convert it to `FluentDateTime`.
pub trait ToFluentDateTime {
    /// Converts a `surrealdb::Datetime` into a `fluent_datetime::FluentDateTime`.
    ///
    /// This method extracts the internal `chrono::DateTime<Utc>` from the SurrealDB
    /// datetime, then converts it to an `icu_calendar::DateTime` (using ISO calendar),
    /// and finally wraps it in `FluentDateTime`.
    ///
    /// Returns `None` if the conversion to `icu_calendar::DateTime` fails, though for valid
    /// `chrono::DateTime` instances, this should typically succeed.
    fn to_fluent_datetime(&self) -> Option<FluentDateTime>;
}

// Implement the trait for `surrealdb::Datetime`
impl ToFluentDateTime for surrealdb::Datetime {
    fn to_fluent_datetime(&self) -> Option<FluentDateTime> {
        // 1. Extract the inner chrono::DateTime<Utc>
        let chrono_dt = self.clone().into_inner().0;

        // 2. Extract components for icu_calendar::DateTime
        let year = chrono_dt.year();
        let month = chrono_dt.month() as u8;
        let day = chrono_dt.day() as u8;
        let hour = chrono_dt.hour() as u8;
        let minute = chrono_dt.minute() as u8;
        let second = chrono_dt.second() as u8;
        let _nanosecond = chrono_dt.nanosecond();

        // 3. Create icu_calendar::DateTime using the ISO calendar
        let icu_dt =
            icu_calendar::DateTime::try_new_iso_datetime(year, month, day, hour, minute, second)
                .ok()?;

        Some(FluentDateTime::from(icu_dt))
    }
}
