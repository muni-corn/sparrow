use crate::errors::SparrowError;
use crate::{prompt_yn, Decision, Formatting};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{stdin, stdout, Write};

const DATE_FORMAT: &str = "%Y/%m/%d";
const TIME_FORMAT: &str = "%H:%M";

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    /// The name of the task.
    pub name: String,

    /// When the task is due.
    pub due_date: chrono::DateTime<chrono::Local>,

    /// The user's estimation on how long the Task will take.
    pub duration: TaskDuration,

    /// True if the task is complete.
    pub done: bool,

    /// How many days in advance of the Task's due date this Task should be considered for
    /// scheduling.
    pub consideration_period_days: u32,
}

impl Task {
    pub fn prompt_new(formatting: &Formatting) -> Result<Self, SparrowError> {
        let name = {
            let mut s = String::new();
            print!(
                "{}  ",
                formatting
                    .prompt
                    .paint("What do you want to name this task?")
            );
            stdout().flush()?;
            stdin().read_line(&mut s)?;
            s = s.trim().to_string();

            if s.is_empty() {
                return Err(SparrowError::InputCanceled);
            }

            s
        };

        let date_str = Self::prompt_date(formatting)?;
        let time_str_opt = Self::prompt_time(&date_str, formatting)?;

        // parse date and time
        let due_date = {
            let time_str = if let Some(t) = time_str_opt {
                t
            } else {
                chrono::NaiveTime::from_hms(0, 0, 0)
                    .format(TIME_FORMAT)
                    .to_string()
            };
            let (combined_str, combined_format) = (
                format!("{} {}", date_str.trim(), time_str.trim()),
                format!("{} {}", DATE_FORMAT, TIME_FORMAT),
            );

            let naive_due_date =
                chrono::NaiveDateTime::parse_from_str(&combined_str, &combined_format)?;

            if let Some(d) = Local.from_local_datetime(&naive_due_date).earliest() {
                d
            } else {
                return Err(SparrowError::BasicMessage(String::from(
                    "for some reason, the date you entered isn't valid",
                )));
            }
        };

        let duration = Self::prompt_task_duration(&name, formatting)?;

        Ok(Self {
            name,
            due_date,
            duration,
            done: false,
            consideration_period_days: 3,
        })
    }

    fn prompt_date(formatting: &Formatting) -> Result<String, SparrowError> {
        let date_str = {
            let mut s = String::new();
            print!(
                "{} ({})  ",
                formatting.prompt.paint("What day is this task due?"),
                formatting.prompt_format.paint(DATE_FORMAT)
            );
            stdout().flush()?;
            stdin().read_line(&mut s)?;
            s = s.trim().to_string();

            s
        };

        Ok(date_str)
    }

    fn prompt_time(
        date_str: &str,
        formatting: &Formatting,
    ) -> Result<Option<String>, SparrowError> {
        let response = prompt_yn(&format!(
            "{} {}",
            formatting.prompt.paint("Add a time?"),
            formatting.prompt_format.paint("[y/N]"),
        ))?
        .unwrap_or(Decision::No);

        if response.is_yes() {
            let time_str = {
                let mut s = String::new();
                print!(
                    "{} ({})  ",
                    formatting
                        .prompt
                        .paint(format!("What time on {} is this task due?", date_str)),
                    formatting.prompt_format.paint(TIME_FORMAT)
                );
                stdout().flush()?;
                stdin().read_line(&mut s)?;
                s = s.trim().to_string();

                s
            };

            Ok(Some(time_str))
        } else {
            Ok(None)
        }
    }

    fn prompt_task_duration(
        task_name: &str,
        formatting: &Formatting,
    ) -> Result<TaskDuration, SparrowError> {
        match prompt_yn(&format!(
            "{} {}",
            formatting.prompt.paint("Add subtasks?"),
            formatting.prompt_format.paint("[y/N]")
        ))?
        .unwrap_or(Decision::No)
        {
            Decision::Yes => Ok(TaskDuration::Subtasks(Self::prompt_subtasks(formatting))),
            Decision::No => Ok(TaskDuration::Minutes(prompt_time_duration(
                task_name, formatting,
            )?)),
        }
    }

    fn prompt_subtasks(formatting: &Formatting) -> Vec<Subtask> {
        let mut v = Vec::new();

        loop {
            print!(
                "{}  ",
                formatting.prompt.paint(&format!("#{}:", v.len() + 1))
            );
            match Subtask::prompt_new(formatting) {
                Ok(o) => {
                    if let Some(s) = o {
                        v.push(s)
                    } else {
                        break v;
                    }
                }
                Err(e) => print!(
                    "{}  ",
                    formatting
                        .error
                        .paint(&format!("There was an error: {}\nTry again?", e))
                ),
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TaskDuration {
    Minutes(u64),
    Subtasks(Vec<Subtask>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Subtask {
    pub name: String,
    pub duration: u64,
}

impl Subtask {
    pub fn prompt_new(formatting: &Formatting) -> Result<Option<Self>, SparrowError> {
        let name = {
            let mut s = String::new();
            print!(
                "{}  ",
                formatting
                    .prompt
                    .paint("What do you want to name this subtask?"),
            );
            stdout().flush()?;
            stdin().read_line(&mut s)?;
            s = s.trim().to_string();

            s
        };

        if name.trim().is_empty() {
            Ok(None)
        } else {
            let duration = prompt_time_duration(&name, formatting)?;

            Ok(Some(Self { name, duration }))
        }
    }
}

fn prompt_time_duration(task_name: &str, formatting: &Formatting) -> Result<u64, SparrowError> {
    print!(
        "{} ({})  ",
        formatting
            .prompt
            .paint(format!("How long will \"{}\" take to complete?", task_name)),
        formatting.prompt_format.paint("minutes")
    );

    loop {
        let mut s = String::new();
        stdout().flush()?;
        stdin().read_line(&mut s)?;
        s = s.trim().to_string();

        match s.parse::<f64>() {
            Ok(n) => break Ok(n as u64),
            Err(_) => print!(
                "{}",
                formatting
                    .error
                    .paint("That doesn't seem like a number. Try again?  ")
            ),
        }
    }
}
