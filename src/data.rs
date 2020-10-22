use crate::Task;
use crate::{CalendarEvent, Schedule, SparrowError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize)]
pub struct Config {
    /// How long a work period should last.
    pub work_minutes: u32,

    /// How long to take a break between a single work periods.
    pub short_break_minutes: u32,

    /// How long to take a break between job sessions.
    pub long_break_minutes: u32,

    /// How many work periods make up a job session.
    pub work_periods_per_job_session: u32,

    /// Allow repeating job sessions for the same Task one after another.
    pub allow_repeats: bool,

    /// How long before the next job/break starts sparrowd notifies the user.
    pub next_event_warning_minutes: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            work_periods_per_job_session: 4,
            allow_repeats: false,
            next_event_warning_minutes: 5,
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct UserData {
    config: Config,
    tasks: Vec<Task>,
    events: Vec<CalendarEvent>,
    schedule: Schedule,
}

impl UserData {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, SparrowError> {
        if !path.as_ref().exists() {
            Ok(Self::default())
        } else {
            Ok(serde_yaml::from_reader(fs::File::open(path)?)?)
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SparrowError> {
        Ok(fs::write(path, serde_yaml::to_string(self)?)?)
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn get_tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn get_events(&self) -> &[CalendarEvent] {
        &self.events
    }

    pub fn get_schedule(&mut self) -> &Schedule {
        &self.schedule
    }

    pub fn set_schedule(&mut self, schedule: Schedule) {
        self.schedule = schedule;
    }
}
