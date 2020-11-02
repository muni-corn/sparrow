use crate::Bedtime;
use crate::task::{Task, TaskDuration};
use crate::Config;
use crate::SparrowError;
use crate::TimeSpan;
use crate::CalendarEvent;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Schedule {
    entries: Vec<ScheduleEntry>,
}

impl Schedule {
    pub fn make(
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

            let mut result = Self { entries };

            result.fill_free_time(config, &tasks);

            // make sure entries are sorted correctly
            result.entries.sort_by_cached_key(|e| *e.span().beginning());

            Ok(result)
        } else {
            Err(SparrowError::BasicMessage(
                "can't make a schedule without tasks".to_string(),
            ))
        }
    }

    fn fill_free_time(&mut self, config: &Config, tasks: &[Task]) {
        if tasks.is_empty() {
            return;
        }

        let mut periods_left = Self::unscheduled_periods_from_tasks(config, tasks);
        let mut free_spans = self.get_free_times(&config);

        'free_spans: for free_span in free_spans.iter_mut() {
            if free_span.minutes() < config.work_minutes {
                // no free time left to schedule here, continue
                continue 'free_spans;
            }

            let mut periods_left_before_big_break = config.work_periods_per_job_session;
            'periods: for periods in periods_left.iter_mut() {
                while periods.periods_left > 0 {
                    if periods_left_before_big_break > 0 {
                        // add a job session, if the task should be considered
                        let t = periods.task;
                        if *free_span.beginning()
                            >= t.due_date
                                - chrono::Duration::days(t.consideration_period_days as i64)
                        {
                            self.add_job_and_break(periods, free_span, config);

                            // decrement this thing
                            periods_left_before_big_break -= 1;
                        } else {
                            // task is not considered yet at this point in time, so move on
                            continue 'periods;
                        }
                    } else {
                        // add a bigger break
                        self.add_long_break(free_span, config);

                        // reset the big-break counter
                        periods_left_before_big_break = config.work_periods_per_job_session;

                        // should we continue adding jobs from this task after this session? if
                        // not, move onto the next task
                        if !config.allow_repeats {
                            continue 'periods;
                        }
                    }
                }
            }

            periods_left.retain(|p| p.periods_left > 0);
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
    }

    fn add_long_break(&mut self, free_time_span: &mut TimeSpan, config: &Config) {
        // add a big break. `actual_length` prevents overlap
        let actual_length = config.long_break_minutes.min(free_time_span.minutes());

        self.entries.push(ScheduleEntry::Break(TimeSpan::new(
            *free_time_span.beginning(),
            actual_length,
        )));

        // be sure to shorten free_time_span
        free_time_span.advance_and_shorten(config.long_break_minutes.min(free_time_span.minutes()));
    }

    fn add_job_and_break(
        &mut self,
        current_task: &mut UnscheduledPeriod,
        free_time_span: &mut TimeSpan,
        config: &Config,
    ) {
        if free_time_span.minutes() < config.work_minutes || current_task.periods_left == 0 {
            return;
        }

        // add a work period and then a short break
        self.entries.push(ScheduleEntry::Job {
            title: current_task.name.clone(),
            span: TimeSpan::new(*free_time_span.beginning(), config.work_minutes),
        });

        current_task.periods_left -= 1;

        // advance free_time_span before adding short break
        free_time_span.advance_and_shorten(config.work_minutes);

        // add the short break, if space still allows
        if free_time_span.minutes() > 0 {
            self.entries.push(ScheduleEntry::Break(TimeSpan::new(
                *free_time_span.beginning(),
                config.short_break_minutes,
            )));

            // and advance again
            free_time_span.advance_and_shorten(config.short_break_minutes);
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
                            name: s.name.clone(),
                            periods_left: (s.duration as f64 / config.work_minutes as f64).ceil()
                                as u32,
                        });
                    }
                }
            }
        }

        v
    }

    fn get_free_times(&self, config: &Config) -> Vec<TimeSpan> {
        // iterate over pairs of (time_span, next_time_span) to determine spans of free time. if
        // the two spans overlap, there's obviously no free time. otherwise, free time is
        // calculated by next_time_span.beginning() - time_span.end()
        self.entries
            .iter()
            .zip(self.entries.iter().skip(1))
            .filter_map(|pair| {
                let earlier_span = pair.0.span();
                let later_span = pair.1.span();

                if earlier_span.touches(later_span) {
                    None
                } else if let Some(span) = TimeSpan::time_between(earlier_span, later_span) {
                    if span.minutes() < config.work_minutes {
                        None
                    } else {
                        Some(span)
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    fn breaks_to_schedule_entries(
        events: &[CalendarEvent],
        until: DateTime<Local>,
        bedtime: &Bedtime,
    ) -> Vec<ScheduleEntry> {
        let mut v: Vec<ScheduleEntry> = events
            .iter()
            .cloned()
            .flat_map(|e| {
                e.iter()
                    .take_while(|s| *s.span().beginning() < until)
                    .collect::<Vec<ScheduleEntry>>()
            })
            .chain(bedtime.iter().take_while(|s| *s.span().beginning() < until))
            .collect();

        v.sort_by_cached_key(|e| *e.span().beginning());

        v
    }

    pub fn print(&self, config: &Config) {
        for e in self.entries.iter().filter(|e| *e.span().beginning() >= Local::now()) {
            let format = format!("{} {}", config.date_format, config.time_format);
            println!("{} :: {}", e.span().beginning().format(&format), e.title());
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum ScheduleEntry {
    /// Work time, part of a Task.
    Job { title: String, span: TimeSpan },

    /// Event time.
    Calendar { name: String, span: TimeSpan },

    /// Break time.
    Break(TimeSpan),

    /// Sleep time.
    Sleep(TimeSpan),
}

impl ScheduleEntry {
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

fn sort_entries(entries: &mut [ScheduleEntry]) {
    entries.sort_by_cached_key(|e| *e.span().beginning());
}
