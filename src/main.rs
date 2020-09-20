use ansi_term::{Color, Style};
use chrono::{prelude::*, Duration};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{self, stdin, stdout, Write};

const DATE_FORMAT: &str = "%Y/%m/%d";
const TIME_FORMAT: &str = "%H:%M";

fn main() {
    let formatting = Formatting {
        prompt: Style::new().bold(),
        prompt_format: Style::new().bold().italic(),
        error: Color::Red.bold(),
    };

    let task = Task::prompt_new(&formatting).unwrap();

    dbg!(&task);
}

pub struct CalendarEvent {
    pub name: String,
    pub start_time: chrono::DateTime<chrono::Local>,
}

#[derive(Debug)]
pub struct Task {
    pub name: String,
    pub due_date: chrono::DateTime<chrono::Local>,
    pub duration: TaskDuration,
}

impl Task {
    pub fn prompt_new(formatting: &Formatting) -> Result<Self, SparrowError> {
        let name = {
            let mut s = String::new();
            print!(
                "{} ",
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
                chrono::NaiveTime::from_hms(0, 0, 0).format(TIME_FORMAT).to_string()
            };
            let (combined_str, combined_format) = (format!("{} {}", date_str.trim(), time_str.trim()), format!("{} {}", DATE_FORMAT, TIME_FORMAT));

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
        })
    }

    fn prompt_date(formatting: &Formatting) -> Result<String, SparrowError> {
        let date_str = {
            let mut s = String::new();
            print!(
                "{} ({}) ",
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
                    "{} ({}) ",
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
            Decision::No => Ok(TaskDuration::TimeDuration(prompt_time_duration(
                task_name, formatting,
            )?)),
        }
    }

    fn prompt_subtasks(formatting: &Formatting) -> Vec<Subtask> {
        let mut v = Vec::new();

        loop {
            print!(
                "{} ",
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
                    "{} ",
                    formatting
                        .error
                        .paint(&format!("There was an error: {}\nTry again?", e))
                ),
            }
        }
    }
}

#[derive(Debug)]
pub enum TaskDuration {
    TimeDuration(Duration),
    Subtasks(Vec<Subtask>),
}

#[derive(Debug)]
pub struct Subtask {
    pub name: String,
    pub duration: Duration,
}

impl Subtask {
    pub fn prompt_new(formatting: &Formatting) -> Result<Option<Self>, SparrowError> {
        let name = {
            let mut s = String::new();
            print!(
                "{} ",
                formatting
                    .prompt
                    .paint("What do you want to name this subtask?")
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

#[derive(Debug)]
pub enum SparrowError {
    InputCanceled,
    BasicMessage(String),
    ChronoParse(chrono::ParseError),
    Io(io::Error),
}

impl Display for SparrowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputCanceled => write!(f, "input canceled"),
            Self::BasicMessage(b) => write!(f, "sparrow hit an error: {}", b),
            Self::ChronoParse(e) => write!(f, "{}", e),
            Self::Io(i) => write!(f, "there was an i/o error: {}", i),
        }
    }
}

impl Error for SparrowError {}

impl From<chrono::ParseError> for SparrowError {
    fn from(e: chrono::ParseError) -> Self {
        Self::ChronoParse(e)
    }
}

impl From<io::Error> for SparrowError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

pub struct Formatting {
    prompt: Style,
    prompt_format: Style,
    error: Style,
}

/// a fancy bool. deal with it
enum Decision {
    Yes,
    No,
}

impl Decision {
    fn is_yes(&self) -> bool {
        match self {
            Decision::Yes => true,
            _ => false,
        }
    }

    #[allow(dead_code)]
    fn is_no(&self) -> bool {
        match self {
            Decision::No => true,
            _ => false,
        }
    }
}

fn prompt_yn(prompt_string: &str) -> Result<Option<Decision>, SparrowError> {
    loop {
        let mut s = String::new();

        print!("{} ", prompt_string);
        stdout().flush()?;
        stdin().read_line(&mut s)?;
        s = s.trim().to_string();

        s = s.trim().to_lowercase();

        if s.is_empty() {
            break Ok(None);
        } else if s.starts_with('y') {
            break Ok(Some(Decision::Yes));
        } else if s.starts_with('n') {
            break Ok(Some(Decision::No));
        } else {
            print!("(What?) ");
        }
    }
}

fn prompt_time_duration(
    task_name: &str,
    formatting: &Formatting,
) -> Result<Duration, SparrowError> {
    print!(
        "{} ({}) ",
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

        match s.parse::<f32>() {
            Ok(n) => break Ok(Duration::minutes(n as i64)),
            Err(_) => print!(
                "{}",
                formatting
                    .error
                    .paint("That doesn't seem like a number. Try again? ")
            ),
        }
    }
}
