use crate::schedule::ScheduleEntry;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CalendarEvent {
    pub name: String,
    pub time_span: TimeSpan,
    pub event_type: CalendarEventType,
    pub repeat: Repeat,
}

impl IntoIterator for CalendarEvent {
    type Item = ScheduleEntry;

    type IntoIter = CalendarScheduleEntryIter;

    fn into_iter(self) -> Self::IntoIter {
        let initial_item = match self.event_type {
            CalendarEventType::Event => ScheduleEntry::Calendar {
                name: self.name.clone(),
                span: self.time_span,
            },
            CalendarEventType::Break => ScheduleEntry::Break(self.time_span),
        };

        Self::IntoIter {
            calendar_event: self,
            next: Some(initial_item),
        }
    }
}

/// Produces `ScheduleEntry`s from repeated `CalendarEvent`s
pub struct CalendarScheduleEntryIter {
    calendar_event: CalendarEvent,
    next: Option<ScheduleEntry>,
}

impl Iterator for CalendarScheduleEntryIter {
    type Item = ScheduleEntry;

    fn next(&mut self) -> Option<Self::Item> {
        // do not mutate. this is what is returned at the end of the function. self.next is
        // modified here to determine what entry there will be (if any) after this one.j
        let result = self.next.clone();

        self.next = if let Some(current_entry) = &self.next {
            let current_date = current_entry.span().beginning().date();
            let current_time = current_entry.span().beginning().time();
            let duration = current_entry.span().minutes();
            let new_span = match &self.calendar_event.repeat {
                Repeat::No => return None,
                Repeat::Daily => TimeSpan::new(current_date.succ().and_time(current_time).unwrap(), duration),
                Repeat::Weekly => TimeSpan::new((current_date + chrono::Duration::days(7)).and_time(current_time).unwrap(), duration)
            };

            Some(match current_entry {
                ScheduleEntry::Job { title, .. } => ScheduleEntry::Job { title: title.clone(), span: new_span },
                ScheduleEntry::Calendar { name, .. } => ScheduleEntry::Calendar { name: name.clone(), span: new_span },
                ScheduleEntry::Break(_) => ScheduleEntry::Break(new_span),
                ScheduleEntry::Sleep(_) => ScheduleEntry::Sleep(new_span),
            })
        } else {
            return None
        };

        result
    }
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
    pub fn new(start: DateTime<Local>, minutes: u32) -> Self {
        Self { start, minutes }
    }

    pub fn beginning(&self) -> &DateTime<Local> {
        &self.start
    }

    pub fn end(&self) -> DateTime<Local> {
        self.start + self.minutes_as_duration()
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
#[derive(Debug, Deserialize, Serialize)]
pub enum Repeat {
    /// The span of time only occurs once.
    No,

    /// The span of time repeats daily at the same time every day.
    Daily,

    /// The span of time repeats weekly.
    Weekly,
}
