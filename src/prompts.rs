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
    Ok(s)
}

/// a fancy bool. deal with it
pub enum Decision {
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
