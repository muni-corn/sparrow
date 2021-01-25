use crate::Config;
use crate::Formatting;
use crate::SparrowError;
use crate::errors::SparrowResult;
use crate::prompts::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

/// A CalendarEvent that can optionally be repeated. TODO: Make this an enum instead of containing
/// an enum type like CalendarEventType.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CalendarEvent {
    pub name: String,
    pub time_span: TimeSpan,
    pub event_type: CalendarEventType,
    pub repeat: Repeat,
}

impl CalendarEvent {
    pub fn prompt_event(formatting: &Formatting, config: &Config) -> SparrowResult<Self> {
        let name = prompt(&formatting, "What should this event be called?", None)?;
        let span = TimeSpan::prompt(&formatting, "When?", &config.date_format, &config.time_format)?;
        let repeat = Repeat::prompt(&formatting)?;
        Ok(Self {
            name,
            time_span: span,
            event_type: CalendarEventType::Event,
            repeat,
        })
    }

    pub fn prompt_break(formatting: &Formatting, config: &Config) -> SparrowResult<Self> {
        let span = TimeSpan::prompt(&formatting, "When?", &config.date_format, &config.time_format)?;
        let repeat = Repeat::prompt(&formatting)?;
        Ok(Self {
            name: String::new(),
            time_span: span,
            event_type: CalendarEventType::Break,
            repeat,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CalendarEventType {
    Break,
    Event,
}

/// A single block of time.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct TimeSpan {
    start: DateTime<Local>,
    minutes: u32,
}

impl TimeSpan {
    pub fn new(start: DateTime<Local>, minutes: u32) -> Self {
        Self { start, minutes }
    }

    pub fn prompt(formatting: &Formatting, question: &str, date_format: &str, time_format: &str) -> SparrowResult<Self> {
        let initial_question = format!("{}\nDay?", question);
        let date = prompt_strict(&formatting, &initial_question, Some(date_format), |i| {
            NaiveDate::parse_from_str(i.trim(), date_format)
        })?;
        let time = prompt_strict(&formatting, "Time?", Some(time_format), |i| {
            NaiveTime::parse_from_str(i.trim(), time_format)
        })?;

        let start = Local.from_local_datetime(&date.and_time(time)).earliest().unwrap();

        let minutes = prompt_strict(&formatting, "How long?", Some("minutes"), |i| {
            i.trim().parse::<u32>()
        })?;

        Ok(Self {
            start, minutes
        })
    }

    pub fn start(&self) -> &DateTime<Local> {
        &self.start
    }

    pub fn end(&self) -> DateTime<Local> {
        self.start + self.minutes_as_duration()
    }

    pub fn set_start(&mut self, start: DateTime<Local>) {
        self.start = start;
    }

    fn minutes_as_duration(&self) -> chrono::Duration {
        chrono::Duration::minutes(self.minutes as i64)
    }

    pub fn minutes(&self) -> u32 {
        self.minutes
    }

    /// Returns true if the spans share some time, but *not* if the end of one and the start of the
    /// other are the same.
    pub fn overlaps(&self, other: &TimeSpan) -> bool {
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

    /// Returns true if the time spans either overlap *or* if the end of one *is* the same as the
    /// start of the other.
    pub fn touches(&self, other: &TimeSpan) -> bool {
        self.overlaps(other) || self.start == other.end() || self.end() == other.start
    }

    /// Returns the free time between two spans, if any.
    pub fn time_between(first: &TimeSpan, second: &TimeSpan) -> Option<TimeSpan> {
        if first.touches(second) {
            None
        } else if first.start > second.end() {
            Some(Self {
                start: second.end(),
                minutes: (second.end() - first.start).num_minutes() as u32,
            })
        } else {
            Some(Self {
                start: first.end(),
                minutes: (first.end() - second.start).num_minutes() as u32,
            })
        }
    }

    /// Moves the start of this span forward in time, keeping the end of the original span in
    /// place. TODO: make a unit test to prove this.
    pub fn advance_and_shorten(&mut self, minutes: u32) {
        // to prevent negative overflow (underflow?)
        let actual_minutes = minutes.min(self.minutes);

        self.start = self.start + chrono::Duration::minutes(actual_minutes as i64);
        self.minutes -= actual_minutes;
    }
}

/// How to repeat a span of time.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Repeat {
    /// The span of time only occurs once.
    No,

    /// The span of time repeats daily at the same time every day.
    Daily,

    /// The span of time repeats weekly.
    Weekly,
}

impl Repeat {
    pub fn prompt(formatting: &Formatting) -> SparrowResult<Self> {
        prompt_strict(formatting, "Repeat?", Some("[N]o, [d]aily, [w]eekly"), |i| {
            let i = i.trim().to_lowercase();
            if i.is_empty() || "no".starts_with(&i) {
                Ok(Self::No)
            } else if "daily".starts_with(&i) {
                Ok(Self::Daily)
            } else if "weekly".starts_with(&i) {
                Ok(Self::Weekly)
            } else {
                Err(SparrowError::BasicMessage(String::from("What?")))
            }
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bedtime {
    start: NaiveTime,
    hours: f32,
}

impl Bedtime {
    pub fn new(start: NaiveTime, hours: f32) -> Self { Self { start, hours } }

    pub fn start(&self) -> &NaiveTime {
        &self.start
    }

    pub fn end(&self) -> NaiveTime {
        self.start + chrono::Duration::minutes((self.hours * 60.0) as i64)
    }

    pub fn duration_hours(&self) -> f32 {
        self.hours
    }
}

impl Default for Bedtime {
    fn default() -> Self {
        Self {
            start: NaiveTime::from_hms(20, 0, 0),
            hours: 10.0,
        }
    }
}
