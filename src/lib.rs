use ansi_term::Style;
use std::io::{stdin, stdout, Write};

pub mod spans;
pub mod data;
pub mod errors;
pub mod schedule;
pub mod task;

pub use spans::*;
pub use data::*;
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
