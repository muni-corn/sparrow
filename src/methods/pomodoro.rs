use crate::{
    errors::SparrowResult,
    spans::{CalendarEventType, Repeat},
    task::{Task, TaskDuration},
    Bedtime, CalendarEvent, Config, Schedule, SparrowError, TimeSpan,
};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Deserialize, Serialize)]
pub struct PomodoroSchedule {
    entries: Vec<PomodoroScheduleEntry>,
}

impl<'a> Schedule<'a> for PomodoroSchedule {
    type Display = PomodoroDisplay<'a>;

    fn make(
        config: &Config,
        tasks: &[Task],
        events: &[CalendarEvent],
        bedtime: &Bedtime,
    ) -> Result<Self, SparrowError> {
        // intentionally shadow `tasks`. we want `tasks` to be mutable (for sorting) but we don't
        // want to modify the original reference
        let mut tasks = tasks.to_vec();

        // make sure tasks are sorted by due date
        tasks.sort_by_cached_key(|t| t.due_date);

        if let Some(last_due_date) = tasks.last().map(|t| t.due_date) {
            let mut entries = Self::breaks_to_schedule_entries(events, last_due_date, bedtime);

            // entries should stay sorted
            sort_entries(&mut entries);

            #[cfg(debug_assertions)]
            dbg!(&entries);

            let mut result = Self { entries };

            result.fill_free_time(config, &tasks, last_due_date);

            // make sure entries are sorted correctly
            result.entries.sort_by_cached_key(|e| *e.span().start());

            Ok(result)
        } else {
            Err(SparrowError::BasicMessage(
                "can't make a schedule without tasks".to_string(),
            ))
        }
    }

    fn display(&'a self, config: &'a Config) -> Self::Display {
        Self::Display {
            schedule: self,
            config,
        }
    }
}

impl PomodoroSchedule {
    fn fill_free_time(&mut self, config: &Config, tasks: &[Task], until: DateTime<Local>) {
        if tasks.is_empty() {
            return;
        }

        let mut periods_left = Self::unscheduled_periods_from_tasks(config, tasks);
        let mut open_sessions = self.get_open_work_sessions(&config, until);

        let now = Local::now();

        let should_retain =
            |u: &UnscheduledPeriod| u.periods_left > 0 && u.task.due_date > now && !u.task.done;

        'sessions: for open_session in open_sessions.iter_mut() {
            periods_left.retain(should_retain);

            for unscheduled in periods_left.iter_mut() {
                if !should_retain(unscheduled) {
                    continue;
                } else if open_session.full() {
                    continue 'sessions;
                } else {
                    while unscheduled.periods_left > 0 && !open_session.full() {
                        open_session.add_job(&unscheduled.name).unwrap();
                        unscheduled.periods_left -= 1;
                    }
                }
            }
        }

        periods_left.retain(|p| p.periods_left > 0);

        if !periods_left.is_empty() {
            println!(
                "WARNING: There wasn't enough free time to finish scheduling the following tasks:"
            );
            for p in periods_left {
                println!(
                    "\t{}, {} minutes unscheduled",
                    p.name,
                    p.periods_left * config.work_minutes
                )
            }
        }

        for work_session in open_sessions {
            let long_break = PomodoroScheduleEntry::Break(TimeSpan::new(
                work_session.ending(),
                config.long_break_minutes,
            ));
            self.entries.append(&mut work_session.into());
            self.entries.push(long_break);
        }
    }

    fn unscheduled_periods_from_tasks<'a>(
        config: &Config,
        tasks: &'a [Task],
    ) -> Vec<UnscheduledPeriod<'a>> {
        // why aren't we using iter().map()? see match pattern for TaskDuration::Subtasks. not
        // every pattern can be mapped to a *single* UnscheduledPeriod, and the implementation with
        // flat_map is kinda awkward. this just makes better sense to me, at least.
        let mut v = Vec::new();
        for t in tasks {
            match &t.duration {
                TaskDuration::Minutes(m) => v.push(UnscheduledPeriod {
                    task: t,
                    name: t.name.clone(),
                    periods_left: (*m as f64 / config.work_minutes as f64).ceil() as u32,
                }),
                TaskDuration::Subtasks(subs) => {
                    for s in subs {
                        v.push(UnscheduledPeriod {
                            task: t,
                            name: format!("{}: {}", t.name, s.name),
                            periods_left: (s.duration as f64 / config.work_minutes as f64).ceil()
                                as u32,
                        });
                    }
                }
            }
        }

        v
    }

    fn get_open_work_sessions(&self, config: &Config, until: DateTime<Local>) -> Vec<WorkSession> {
        use std::iter::once;

        let now = Local::now();
        let filtered_entries = self
            .entries
            .iter()
            .skip_while(|e| e.span().end() <= now)
            .take_while(|e| *e.span().start() < until);
        let span_beginnings = filtered_entries
            .clone()
            .map(|e| *e.span().start())
            .chain(once(until));
        let span_endings = once(now).chain(filtered_entries.clone().map(|e| e.span().end()));

        let work_session_len = WorkSession::len_minutes(config) as i64;
        span_endings
            .zip(span_beginnings)
            .flat_map(|pair| {
                #[cfg(debug_assertions)]
                dbg!(&pair);

                let mut v = Vec::new();
                let end = pair.0;
                let beginning_next = pair.1;

                if beginning_next > end {
                    let free_minutes = (beginning_next - end).num_minutes();
                    let num_possible_work_sessions = free_minutes / work_session_len;

                    for i in 0..num_possible_work_sessions {
                        v.push(WorkSession::new(
                            end + chrono::Duration::minutes(i * work_session_len),
                            config,
                        ));
                    }
                }

                v
            })
            .collect()
    }

    fn breaks_to_schedule_entries(
        events: &[CalendarEvent],
        until: DateTime<Local>,
        bedtime: &Bedtime,
    ) -> Vec<PomodoroScheduleEntry> {
        let mut v: Vec<PomodoroScheduleEntry> = events
            .iter()
            .cloned()
            .flat_map(|e| {
                PomodoroScheduleEntryIter::from(&e)
                    .take_while(|s: &PomodoroScheduleEntry| *s.span().start() < until)
                    .collect::<Vec<PomodoroScheduleEntry>>()
            })
            .chain(
                BedtimeScheduleEntryIter::from(bedtime).take_while(|s| *s.span().start() < until),
            )
            .collect();

        v.sort_by_cached_key(|e| *e.span().start());

        v
    }

    pub fn get_entries(&self) -> &[PomodoroScheduleEntry] {
        &self.entries
    }
}

#[derive(Debug)]
struct WorkSession {
    start: DateTime<Local>,
    job_names: Vec<String>,

    max_jobs: usize,
    job_len_minutes: u32,
    break_len_minutes: u32,
}

impl WorkSession {
    fn len_minutes(config: &Config) -> u32 {
        config.work_periods_per_job_session * config.work_minutes
            + (config.work_periods_per_job_session - 1) * config.short_break_minutes
            + config.long_break_minutes
    }

    fn new(start: DateTime<Local>, config: &Config) -> Self {
        Self {
            start,
            job_names: Vec::new(),
            max_jobs: config.work_periods_per_job_session as usize,
            job_len_minutes: config.work_minutes,
            break_len_minutes: config.short_break_minutes,
        }
    }

    fn full(&self) -> bool {
        self.job_names.len() >= self.max_jobs
    }

    fn add_job(&mut self, name: &str) -> SparrowResult<()> {
        if !self.full() {
            self.job_names.push(name.to_string());
            Ok(())
        } else {
            Err(SparrowError::BasicMessage(
                "work session is full; no more jobs can be scheduled".to_string(),
            ))
        }
    }

    fn ending(&self) -> DateTime<Local> {
        if self.job_names.is_empty() {
            self.start
        } else {
            self.start
                + chrono::Duration::minutes(
                    self.job_names.len() as i64 * self.job_len_minutes as i64,
                )
                + chrono::Duration::minutes(
                    (self.job_names.len() - 1) as i64 * self.break_len_minutes as i64,
                )
        }
    }
}

impl Into<Vec<PomodoroScheduleEntry>> for WorkSession {
    fn into(self) -> Vec<PomodoroScheduleEntry> {
        if self.job_names.is_empty() {
            vec![]
        } else {
            let job_break_len = self.job_len_minutes + self.break_len_minutes;

            self.job_names
                .iter()
                .enumerate()
                .flat_map(|pair| {
                    let i = pair.0;
                    let name = pair.1;

                    let job = PomodoroScheduleEntry::Job {
                        title: name.to_string(),
                        span: TimeSpan::new(
                            self.start + chrono::Duration::minutes(i as i64 * job_break_len as i64),
                            self.job_len_minutes,
                        ),
                    };

                    let break_time = PomodoroScheduleEntry::Break(TimeSpan::new(
                        self.start
                            + chrono::Duration::minutes(self.job_len_minutes as i64)
                            + chrono::Duration::minutes(i as i64 * job_break_len as i64),
                        self.job_len_minutes,
                    ));

                    vec![job, break_time]
                })
                .take(self.job_names.len() * 2 - 1) // this trims off that last short break we won't need
                .collect()
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PomodoroScheduleEntry {
    /// Work time, part of a Task.
    Job { title: String, span: TimeSpan },

    /// Event time.
    Calendar { name: String, span: TimeSpan },

    /// Break time.
    Break(TimeSpan),

    /// Sleep time.
    Sleep(TimeSpan),
}

impl PomodoroScheduleEntry {
    pub fn span(&self) -> &TimeSpan {
        match self {
            Self::Job { span, .. } => span,
            Self::Calendar { span, .. } => span,
            Self::Break(span) => span,
            Self::Sleep(span) => span,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::Job { title, .. } => &title,
            Self::Calendar { name, .. } => &name,
            Self::Break(_) => "Break",
            Self::Sleep(_) => "Sleep",
        }
    }
}

#[derive(Clone, Debug)]
struct UnscheduledPeriod<'a> {
    task: &'a Task,
    name: String,
    periods_left: u32,
}

fn sort_entries(entries: &mut [PomodoroScheduleEntry]) {
    entries.sort_by_cached_key(|e| *e.span().start());
}

/// Produces `PomodoroScheduleEntry`s from repeated `CalendarEvent`s.
struct PomodoroScheduleEntryIter<'a> {
    calendar_event: &'a CalendarEvent,
    next: Option<PomodoroScheduleEntry>,
}

impl<'a> From<&'a CalendarEvent> for PomodoroScheduleEntryIter<'a> {
    fn from(event: &'a CalendarEvent) -> Self {
        let initial_item = match event.event_type {
            CalendarEventType::Event => PomodoroScheduleEntry::Calendar {
                name: event.name.clone(),
                span: event.time_span,
            },
            CalendarEventType::Break => PomodoroScheduleEntry::Break(event.time_span),
        };

        PomodoroScheduleEntryIter {
            calendar_event: event,
            next: Some(initial_item),
        }
    }
}

impl Iterator for PomodoroScheduleEntryIter<'_> {
    type Item = PomodoroScheduleEntry;

    fn next(&mut self) -> Option<Self::Item> {
        // do not mutate. this is what is returned at the end of the function. self.next is
        // modified here to determine what entry there will be (if any) after this one.
        let result = self.next.clone();

        self.next = if let Some(current_entry) = &self.next {
            let current_date = current_entry.span().start().date();
            let current_time = current_entry.span().start().time();
            let duration = current_entry.span().minutes();
            let new_span = match &self.calendar_event.repeat {
                Repeat::No => return None,
                Repeat::Daily => TimeSpan::new(
                    current_date.succ().and_time(current_time).unwrap(),
                    duration,
                ),
                Repeat::Weekly => TimeSpan::new(
                    (current_date + chrono::Duration::days(7))
                        .and_time(current_time)
                        .unwrap(),
                    duration,
                ),
            };

            Some(match current_entry {
                PomodoroScheduleEntry::Job { title, .. } => PomodoroScheduleEntry::Job {
                    title: title.clone(),
                    span: new_span,
                },
                PomodoroScheduleEntry::Calendar { name, .. } => PomodoroScheduleEntry::Calendar {
                    name: name.clone(),
                    span: new_span,
                },
                PomodoroScheduleEntry::Break(_) => PomodoroScheduleEntry::Break(new_span),
                PomodoroScheduleEntry::Sleep(_) => PomodoroScheduleEntry::Sleep(new_span),
            })
        } else {
            return None;
        };

        result
    }
}

pub struct BedtimeScheduleEntryIter {
    current: TimeSpan,
}

impl From<&Bedtime> for BedtimeScheduleEntryIter {
    fn from(b: &Bedtime) -> Self {
        Self::new(b)
    }
}

impl BedtimeScheduleEntryIter {
    pub fn new(bedtime: &Bedtime) -> Self {
        // sleep starts on the day before, in case you're like me and decide to make a
        // schedule in the middle of bedtime (this is a reference that future me won't understand)
        let yesterday = Local::today() - chrono::Duration::days(1);
        Self {
            current: TimeSpan::new(
                yesterday.and_time(*bedtime.start()).unwrap(),
                (bedtime.duration_hours() * 60.0) as u32,
            ),
        }
    }
}

impl Iterator for BedtimeScheduleEntryIter {
    type Item = PomodoroScheduleEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = PomodoroScheduleEntry::Sleep(self.current);

        self.current
            .set_start(*self.current.start() + chrono::Duration::days(1));

        Some(ret)
    }
}

pub struct PomodoroDisplay<'a> {
    schedule: &'a PomodoroSchedule,
    config: &'a Config,
}

impl Display for PomodoroDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for e in self
            .schedule
            .entries
            .iter()
            .filter(|e| e.span().end() >= Local::now())
        {
            let format = format!("{} {}", self.config.date_format, self.config.time_format);
            writeln!(f, "{} :: {}", e.span().start().format(&format), e.title())?;
        }

        Ok(())
    }
}
