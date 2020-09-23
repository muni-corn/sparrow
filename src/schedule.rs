use crate::{CalendarEvent, Task};
use chrono::prelude::*;

pub struct Schedule {}

impl Schedule {}

enum ScheduleEntry {
    Task(Task),
    Calendar(CalendarEvent),
}

impl ScheduleEntry {
    pub fn name(&self) -> &str {}

    pub fn start_time(&self) -> &DateTime<Local> {}

    pub fn duration(&self) -> &chrono::Duration {}
}
