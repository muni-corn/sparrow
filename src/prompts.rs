use chrono::prelude::*;

use crate::errors::SparrowResult;
use crate::{Formatting, SparrowError};
use std::io::{stdin, stdout, Write};

pub fn prompt(
    formatting: &Formatting,
    prompt: &str,
    prompt_format: Option<&str>,
) -> Result<String, SparrowError> {
    if let Some(f) = prompt_format {
        print!(
            "{} ({})  ",
            formatting.prompt.paint(prompt),
            formatting.prompt_format.paint(f)
        );
    } else {
        print!("{}  ", formatting.prompt.paint(prompt));
    }
    get_input()
}

/// Prompts the user for an input, but will prompt the user again if a condition isn't met,
/// specified by `checker`. `checker` takes a string, the user's input, as input. If `checker`
/// returns Ok, `prompt_strict` returns the value inside the Ok. If `checker` returns Err, the
/// prompt will display the error, and ask for input again, over and over until `checker` returns
/// an Ok.
pub fn prompt_strict<F, T, E>(
    formatting: &Formatting,
    prompt: &str,
    prompt_format: Option<&str>,
    checker: F,
) -> Result<T, SparrowError>
where
    F: Fn(&str) -> Result<T, E>,
    E: std::error::Error,
{
    let mut input = self::prompt(formatting, prompt, prompt_format)?;
    loop {
        match checker(&input) {
            Ok(v) => return Ok(v),
            Err(e) => print!("{}. Try again?  ", e),
        }
        input = get_input()?
    }
}

fn get_input() -> Result<String, SparrowError> {
    let mut s = String::new();
    stdout().flush()?;
    stdin().read_line(&mut s)?;
    s = s.trim_end_matches('\n').to_string();
    Ok(s)
}

/// a fancy bool. deal with it
pub enum Decision {
    Yes,
    No,
}

impl Decision {
    pub fn is_yes(&self) -> bool {
        matches!(self, Decision::Yes)
    }

    #[allow(dead_code)]
    pub fn is_no(&self) -> bool {
        matches!(self, Decision::No)
    }
}

pub fn prompt_yn(prompt_string: &str) -> Result<Option<Decision>, SparrowError> {
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

pub fn prompt_datetime(
    formatting: &Formatting,
    date_format: &str,
    time_format: &str,
    allow_midnight_on_empty: bool,
) -> SparrowResult<DateTime<Local>> {
    let time_prompt_format = if allow_midnight_on_empty {
        format!("{}, or empty for midnight", time_format)
    } else {
        time_format.to_string()
    };

    let date = prompt_strict(formatting, "Date?", Some(&date_format), |i| {
        NaiveDate::parse_from_str(i.trim(), date_format)
    })?;
    let time_opt = prompt_strict(formatting, "Time?", Some(&time_prompt_format), |i| {
        if i.is_empty() && allow_midnight_on_empty {
            Ok(None)
        } else {
            NaiveTime::parse_from_str(i, &time_format).map(Some)
        }
    })?;

    // determine date/time from user input
    if let Some(time) = time_opt {
        if let Some(d) = Local.from_local_datetime(&date.and_time(time)).earliest() {
            Ok(d)
        } else {
            Err(SparrowError::BasicMessage(
                "Sorry, the time you entered can't be converted to your local timezone."
                    .to_string(),
            ))
        }
    } else if let Some(d) = Local.from_local_datetime(&date.and_hms(0, 0, 0)).earliest() {
        Ok(d)
    } else {
        Err(SparrowError::BasicMessage(
            "Sorry, the date you entered can't be converted to your local timezone.".to_string(),
        ))
    }
}
