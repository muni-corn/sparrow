use crate::{
    methods::ivy_lee::IvyLeeSchedule, methods::pomodoro::PomodoroSchedule, Bedtime, CalendarEvent,
    SparrowError, Task,
};
use chrono::Weekday;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize)]
pub struct Config {
    /// Date format used when parsing/formatting dates.
    pub date_format: String,

    /// Time format used when parsing/formatting time.
    pub time_format: String,

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

    /// Weekdays to skip, if any
    pub skip_days: HashSet<Weekday>,

    /// Maximum number of tasks allowed to be scheduled per day with Ivy-Lee method
    pub ivy_lee_tasks_per_day: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            date_format: "%Y/%m/%d".to_string(),
            time_format: "%H:%M".to_string(),
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            work_periods_per_job_session: 4,
            allow_repeats: false,
            next_event_warning_minutes: 5,
            skip_days: HashSet::new(),
            ivy_lee_tasks_per_day: 6,
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct UserData {
    config: Config,
    bedtime: Bedtime,
    tasks: Vec<Task>,
    events: Vec<CalendarEvent>,
    pomodoro_schedule: Option<PomodoroSchedule>,
    ivy_lee_schedule: Option<IvyLeeSchedule>,
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

    pub fn add_event(&mut self, event: CalendarEvent) {
        self.events.push(event);
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

    pub fn get_pomodoro_schedule(&self) -> &Option<PomodoroSchedule> {
        &self.pomodoro_schedule
    }

    pub fn set_pomodoro_schedule(&mut self, schedule: PomodoroSchedule) {
        self.pomodoro_schedule = Some(schedule);
    }

    pub fn get_ivy_lee_schedule(&self) -> &Option<IvyLeeSchedule> {
        &self.ivy_lee_schedule
    }

    pub fn set_ivy_lee_schedule(&mut self, schedule: IvyLeeSchedule) {
        self.ivy_lee_schedule = Some(schedule);
    }

    pub fn delete_pomodoro_schedule(&mut self) {
        self.pomodoro_schedule = None;
    }

    pub fn get_bedtime(&self) -> &Bedtime {
        &self.bedtime
    }
}
