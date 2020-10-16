use crate::schedule::ScheduleEntry;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CalendarEvent {
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
pub(crate) struct CalendarScheduleEntryIter {
    calendar_event: CalendarEvent,
    next: Option<ScheduleEntry>,
}

impl Iterator for CalendarScheduleEntryIter {
    type Item = ScheduleEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.next.clone();

        self.next = if let Some(current) = &self.next {
            match &self.calendar_event.repeat {
                Repeat::No => None,
                Repeat::Daily => todo!(),
                Repeat::Weekly(day_set) => {
                    if day_set.is_empty() {
                        None
                    } else {
                        let next_day = current.span();
                        todo!()
                    }
                }
            }
        } else {
            None
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
        Self {
            start,
            minutes
        }
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

    /// The span of time repeats weekly on the specificed weekdays.
    Weekly(HashSet<chrono::Weekday>),
}
