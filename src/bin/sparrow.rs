use ansi_term::{Color, Style};
use clap::{App, Arg, SubCommand};
use sparrow::CalendarEvent;
use sparrow::{prompts::*, Formatting, Schedule, SparrowError, Task, UserData};
use std::convert::TryFrom;
use std::path::PathBuf;

enum AddType {
    Task,
    Break,
    Event,
}

impl TryFrom<&str> for AddType {
    type Error = SparrowError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_lowercase();
        if "task".starts_with(&value) {
            Ok(Self::Task)
        } else if "break".starts_with(&value) {
            Ok(Self::Break)
        } else if "event".starts_with(&value) {
            Ok(Self::Event)
        } else {
            Err(SparrowError::BasicMessage(format!(
                "'{}' isn't something you can add",
                value
            )))
        }
    }
}

fn main() {
    let mut app = App::new("sparrow")
        .version("0.0.0")
        .author("municorn <municorn@musicaloft.com>")
        .arg(
            Arg::with_name("file")
                .long("file")
                .short("f")
                .takes_value(true)
                .value_name("PATH")
                .help("Specifies a different data file"),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add a new task, event, or break")
                .arg(Arg::with_name("type").help("Specify which type of time span to add")),
        )
        .subcommand(SubCommand::with_name("delete").about("Remove a task, event, or break"))
        .subcommand(SubCommand::with_name("check").about("Check off tasks past their due date"))
        .subcommand(SubCommand::with_name("set-sleep").about("Set your sleep schedule"))
        .subcommand(SubCommand::with_name("make").about("Create your schedule"))
        .subcommand(SubCommand::with_name("show").about("View your schedule"));

    if std::env::args().count() <= 1 {
        app.print_help().unwrap();
        return;
    }

    let clap_matches = app.get_matches();

    let formatting = Formatting {
        prompt: Style::new().bold(),
        prompt_format: Style::new().bold().italic(),
        error: Color::Red.bold(),
    };

    let data_file_path = clap_matches
        .value_of("file")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".sparrow"));

    // get data
    let mut data = UserData::from_file(&data_file_path).unwrap();

    if let Some(add_matches) = clap_matches.subcommand_matches("add") {
        let add_type = if let Some(ty_str) = add_matches.value_of("type") {
            AddType::try_from(ty_str).unwrap()
        } else {
            prompt_add_type(&formatting)
        };
        add(&formatting, &mut data, add_type)
    } else if let Some(_delete_matches) = clap_matches.subcommand_matches("delete") {
        todo!()
    } else if let Some(_check_matches) = clap_matches.subcommand_matches("check") {
        todo!()
    } else if let Some(_set_sleep_matches) = clap_matches.subcommand_matches("set-sleep") {
        todo!()
    } else if let Some(_make_matches) = clap_matches.subcommand_matches("make") {
        make_schedule(&mut data)
    } else if let Some(_show_matches) = clap_matches.subcommand_matches("show") {
        if let Some(pomodoro) = data.get_pomodoro_schedule() {
            println!("{}", pomodoro.display(data.get_config()))
        } else {
            eprintln!("no schedule here! try adding tasks with `sparrow add task` and then making a schedule with `sparrow make`")
        }
    }

    data.write_to_file(data_file_path).unwrap();
}

fn add(formatting: &Formatting, data: &mut UserData, add_type: AddType) {
    match add_type {
        AddType::Task => {
            let new_task = Task::prompt_new(&formatting, &data.get_config()).unwrap();
            data.add_task(new_task);
        }
        AddType::Break => {
            let new_break = CalendarEvent::prompt_break(formatting, data.get_config()).unwrap();
            data.add_event(new_break);
        }
        AddType::Event => {
            let new_event = CalendarEvent::prompt_event(formatting, data.get_config()).unwrap();
            data.add_event(new_event);
        }
    }
}

fn prompt_add_type(formatting: &Formatting) -> AddType {
    prompt_strict(
        &formatting,
        "What do you want to add?",
        Some("[T]ask, [b]reak, [e]vent"),
        |i| {
            let i = i.trim();
            if i.is_empty() {
                Ok(AddType::Task)
            } else {
                AddType::try_from(i).map_err(|_| {
                    SparrowError::BasicMessage("Enter 'type', 'break', or 'event'".to_string())
                })
            }
        },
    )
    .unwrap()
}

fn make_schedule(data: &mut UserData) {
    data.set_pomodoro_schedule(
        Schedule::make(data.get_config(), data.get_tasks(), data.get_events(), data.get_bedtime()).unwrap(),
    );
    println!("Done!");
}
