use crate::errors::SparrowError;
use crate::prompts::*;
use crate::Config;
use crate::Formatting;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    pub fn prompt_new(formatting: &Formatting, config: &Config) -> Result<Self, SparrowError> {
        let name = prompt_strict(
            formatting,
            "What do you want to name this task?",
            None,
            |i| {
                let i = i.trim();
                if i.is_empty() {
                    Err(SparrowError::BasicMessage(
                        "Trust me, you don't want a task with a blank name".to_string(),
                    ))
                } else {
                    Ok(i.to_string())
                }
            },
        )?;

        // determine due date from user input
        let due_date = prompt_datetime(formatting, &config.date_format, &config.time_format, true)?;

        let duration = Self::prompt_task_duration(&name, formatting)?;

        Ok(Self {
            name,
            due_date,
            duration,
            done: false,
            consideration_period_days: 3,
        })
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TaskDuration {
    Minutes(u64),
    Subtasks(Vec<Subtask>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Subtask {
    pub name: String,
    pub duration: u64,
}

impl Subtask {
    pub fn prompt_new(formatting: &Formatting) -> Result<Option<Self>, SparrowError> {
        let name = prompt(
            formatting,
            "What do you want to name this subtask?",
            Some("leave blank to finish"),
        )?;

        if name.trim().is_empty() {
            Ok(None)
        } else {
            let duration = prompt_time_duration(&name, formatting)?;

            Ok(Some(Self { name, duration }))
        }
    }
}

fn prompt_time_duration(task_name: &str, formatting: &Formatting) -> Result<u64, SparrowError> {
    prompt_strict(
        &formatting,
        &format!("How long will \"{}\" take to complete?", task_name),
        Some("minutes"),
        |i| match i.trim().parse::<f64>() {
            Ok(n) => Ok(n as u64),
            Err(_) => Err(SparrowError::BasicMessage(String::from(
                "That doesn't seem like a number",
            ))),
        },
    )
}
