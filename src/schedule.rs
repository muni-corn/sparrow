use crate::task::{Task, TaskDuration};
use crate::Config;
use crate::SparrowError;
use crate::TimeSpan;
use crate::{CalendarEvent, CalendarEventType};
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
    ) -> Result<Self, SparrowError> {
        // intentionally shadow `tasks`. we want `tasks` to be mutable (for sorting) but we don't
        // want to modify the original reference
        let mut tasks = tasks.to_vec();

        // make sure tasks are sorted by due date
        tasks.sort_by_cached_key(|t| t.due_date);

        let mut entries = Self::events_to_schedule_entries(events);

        // entries should stay sorted
        sort_entries(&mut entries);

        let mut result = Self { entries };

        result.fill_free_time(config, &tasks);

        Ok(result)
    }

    fn fill_free_time(&mut self, config: &Config, tasks: &[Task]) {
        let mut periods_left = Self::unscheduled_periods_from_tasks(config, tasks);

        let mut free_spans = self.get_free_times(config);

        let mut periods_iter = periods_left.iter_mut();
        let mut current_periods_opt = periods_iter.next();

        'outer: for free_span in free_spans.iter_mut() {
            let mut periods_left_before_big_break = config.work_periods_per_job_session;

            // while this free span still has time left to fill, fill 'er up
            'inner: while free_span.minutes() >= config.work_minutes {
                if let Some(current_task) = &current_periods_opt {
                    if current_task.periods_left == 0 {
                        // no more work to schedule, so move to next task (skip decrementing work
                        // period counter with `continue`)
                        current_periods_opt = periods_iter.next();
                        continue 'inner;
                    } else if periods_left_before_big_break == 0 {
                        self.add_long_break(free_span, config);

                        // TODO advance to next task if config.allow_repeats is false
                        if !config.allow_repeats {}

                        // reset the period counter
                        periods_left_before_big_break = config.work_periods_per_job_session;
                    } else {
                        // add a job (if it should be considered, TODO)
                        self.add_job_and_break(current_task, free_span, config);
                    }

                    periods_left_before_big_break -= 1;
                } else {
                    break 'outer;
                }
            }
        }

        periods_left.retain(|p| p.periods_left > 0);

        if !periods_left.is_empty() {
            println!("WARNING: There wasn't enough free time to finish scheduling the following tasks:");
            for p in periods_left {
                println!("\t{}, {} minutes unscheduled", p.name, p.periods_left * config.work_minutes)
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
        current_task: &UnscheduledPeriod,
        free_time_span: &mut TimeSpan,
        config: &Config,
    ) {
        // add a work period and then a short break
        self.entries.push(ScheduleEntry::Job {
            title: current_task.name.clone(),
            span: TimeSpan::new(*free_time_span.beginning(), config.work_minutes),
        });

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

    fn events_to_schedule_entries(events: &[CalendarEvent]) -> Vec<ScheduleEntry> {
        events
            .iter()
            .map(|e| match e.event_type {
                CalendarEventType::Event => ScheduleEntry::Calendar {
                    name: e.name.clone(),
                    span: e.time_span,
                },
                CalendarEventType::Break => ScheduleEntry::Break(e.time_span),
            })
            .collect()
    }

    pub fn print(&self) {
        todo!()
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
}

struct UnscheduledPeriod<'a> {
    task: &'a Task,
    name: String,
    periods_left: u32,
}

fn sort_entries(entries: &mut [ScheduleEntry]) {
    entries.sort_by_cached_key(|e| *e.span().beginning());
}
