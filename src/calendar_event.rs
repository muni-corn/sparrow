use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize)]
pub struct CalendarEvent {
    pub name: String,
    pub time_span: TimeSpan,
    pub event_type: CalendarEventType,
    pub repeat: Repeat,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CalendarEventType {
    Break,
    Event,
}

/// A single block of time. Multiple, repeated blocks of time are covered by `Occupancy`.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct TimeSpan {
    start: DateTime<Local>,
    minutes: u32,
}

impl TimeSpan {
    fn beginning(&self) -> &DateTime<Local> {
        &self.start
    }

    fn end(&self) -> DateTime<Local> {
        self.start + self.minutes_as_duration()
    }

    fn minutes_as_duration(&self) -> chrono::Duration {
        chrono::Duration::minutes(self.minutes as i64)
    }

    fn overlaps(&self, other: &TimeSpan) -> bool {
        let self_end = self.end();
        let other_end = other.end();
        let total_duration = self.minutes_as_duration() + other.minutes_as_duration();
        if other_end > self.start {
            other_end - self.start < total_duration
        } else if self_end > other.start {
            self_end - other.start < total_duration
        } else {
            true
        }
    }
}

/// How to repeat a span of time.
#[derive(Debug, Deserialize, Serialize)]
pub enum Repeat {
    /// The span of time only occurs once.
    No,

    /// The span of time repeats daily at the same time every day.
    Daily,

    /// The span of time repeats weekly on the specificed weekdays.
    Weekly(HashSet<chrono::Weekday>),
}
