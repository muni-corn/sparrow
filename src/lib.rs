use ansi_term::Style;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{stdin, stdout, Write};

pub mod calendar_event;
pub mod data;
pub mod errors;
pub mod schedule;
pub mod task;

pub use calendar_event::*;
pub use data::UserData;
pub use errors::SparrowError;
pub use schedule::Schedule;
pub use task::Task;

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

        print!("{}  ", prompt_string);
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
            print!("(What?)  ");
        }
    }
}

/// A single block of time. Multiple, repeated blocks of time are covered by `Occupancy`.
#[derive(Debug, Deserialize, Serialize)]
pub struct TimeSpan {
    start: DateTime<Local>,
    minutes: u32,
}

impl TimeSpan {
    fn beginning(&self) -> &DateTime<Local> {
        &self.start
    }

    fn end(&self) -> DateTime<Local> {
        self.start + chrono::Duration::minutes(self.minutes as i64)
    }
}

/// How to repeat a span of time.
#[derive(Debug, Deserialize, Serialize)]
pub enum Repeat {
    /// The span of time only occurs once.
    No,

    /// The span of time repeats daily at the same time every day.
    Daily,

    /// The span of time repeats weekly on the specificed weekdays.
    Weekly(HashSet<chrono::Weekday>),
}
