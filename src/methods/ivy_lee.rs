use crate::{Bedtime, CalendarEvent, Config, Schedule, SparrowError, SparrowResult, Task};
use chrono::Datelike;
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

#[derive(Clone, Deserialize, Serialize)]
pub struct IvyLeeSchedule {
    task_days: HashMap<NaiveDate, Vec<String>>,
}

impl<'d> Schedule<'d> for IvyLeeSchedule {
    type Display = IvyLeeScheduleDisplay<'d>;

    fn make(
        config: &Config,
        tasks: &[Task],
        _: &[CalendarEvent],
        bedtime: &Bedtime,
    ) -> SparrowResult<Self> {
        let mut task_days = HashMap::<NaiveDate, Vec<String>>::new();

        // tasks will need to be sorted by due date
        let mut sorted_tasks = {
            let mut v = tasks.to_vec();
            v.sort_by_key(|t| t.due_date);
            v
        };

        // get latest due due of the tasks
        let latest_due_date = if let Some(d) =
            sorted_tasks.last()
        {
            d.due_date
        } else if tasks.is_empty() {
            return Err(SparrowError::BasicMessage(String::from(
                "can't make a schedule without tasks. try `sparrow add task` to add something",
            )));
        } else {
            return Err(SparrowError::BasicMessage(String::from("for some reason, sparrow can't find the last due date out of all your tasks and can't make this schedule for you. sorry :(")));
        };

        let mut day = Local::today();

        dbg!(&config.skip_days);
        dbg!(&day.weekday());
        dbg!(&config.skip_days.contains(&day.weekday()));

        while day <= latest_due_date.date() {
            // if we're not to skip the day in question, we can schedule for it
            if !config.skip_days.contains(&day.weekday()) {
                // get the time for when the day begins (when the user wakes up)
                let start_of_day = if let Some(s) = day.and_time(bedtime.end()) {
                    s
                } else {
                    break;
                };

                // add tasks to the day
                let mut day_tasks = Vec::new();
                sorted_tasks.retain(|t|  
                    // if the task is considered at `start_of_day`, we can add it to the day if
                    // there is room
                    if day_tasks.len() < config.ivy_lee_tasks_per_day as usize && !t.is_past_due(&start_of_day) && t.is_considered(&start_of_day) {
                        let days_until_due = (t.due_date - start_of_day).num_days() + 1;
                        if days_until_due == 1 {
                            day_tasks.push(format!("Finish {}", t.name));

                            // return false, as this task is finished and won't be done again
                            false
                        } else {
                            day_tasks.push(format!("1/{} of remaining {}", days_until_due, t.name));

                            // since the task was only partially complete, keep it
                            true
                        }
                    } else {
                        true
                    }
                );

                task_days.insert(day.naive_local(), day_tasks);
            }

            // move to the next day, if any
            if let Some(d) = day.succ_opt() {
                day = d
            } else {
                break;
            }
        }

        // warn of any unscheduled tasks
        eprintln!("WARNING: the following tasks couldn't be scheduled completely:");
        for t in sorted_tasks {
            eprintln!("\t{}", t.name)
        }

        Ok(Self { task_days })
    }

    fn display(&'d self, _config: &'d crate::Config) -> Self::Display {
        let today = Local::today();
        let tomorrow = today.succ_opt();

        IvyLeeScheduleDisplay {
            today: self.task_days.get(&today.naive_local()),
            tomorrow: if let Some(d) = tomorrow {
                self.task_days.get(&d.naive_local())
            } else {
                None
            }
        }
    }
}

pub struct IvyLeeScheduleDisplay<'a> {
    today: Option<&'a Vec<String>>,
    tomorrow: Option<&'a Vec<String>>,
}

impl Display for IvyLeeScheduleDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(tasks_today) = self.today {
            writeln!(f, "Here are tasks for you to do today:")?;
            for t in tasks_today {
                writeln!(f, "-\t{}", t)?;
            }
        } else {
            writeln!(f, "Nothing to do today :) Enjoy your day off!")?;
        }

        writeln!(f)?;

        if let Some(tasks_tomorrow) = self.tomorrow {
            writeln!(f, "There are tasks for you to do tomorrow:")?;
            for t in tasks_tomorrow {
                writeln!(f, "-\t{}", t)?;
            }
        } else {
            writeln!(f, "Nothing to do tomorrow :) Have a good day!")?;
        }

        Ok(())
    }
}
