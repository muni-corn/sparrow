use ansi_term::{Color, Style};
use std::io::{stdin, stdout, Write};
use serde::{Deserialize, Serialize};

pub mod data;
pub mod task;
pub mod errors;

use data::UserData;
pub use errors::SparrowError;
pub use task::Task;

fn main() {
    let formatting = Formatting {
        prompt: Style::new().bold(),
        prompt_format: Style::new().bold().italic(),
        error: Color::Red.bold(),
    };

    let task_list_path = dirs::home_dir().unwrap().join(".sparrow");

    let mut data = UserData::from_file(&task_list_path).unwrap();
    let task = Task::prompt_new(&formatting).unwrap();
    data.add_task(task);

    data.write_to_file(&task_list_path).unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CalendarEvent {
    pub name: String,
    pub start_time: chrono::DateTime<chrono::Local>,
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

