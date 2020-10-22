use ansi_term::{Color, Style};
use clap::{App, Arg, SubCommand};
use sparrow::{Formatting, Schedule, Task, UserData};
use std::path::PathBuf;

fn main() {
    let mut app = App::new("sparrow")
        .version("0.0.0")
        .author("Harrison Thorne <harrisonthorne@protonmail.com>")
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
        return
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
        if let Some(ty) = add_matches.value_of("type") {
            match ty {
                
            }
        }
        add_task(&formatting, &mut data);
    } else if let Some(delete_matches) = clap_matches.subcommand_matches("delete") {
        todo!()
    } else if let Some(check_matches) = clap_matches.subcommand_matches("check") {
        todo!()
    } else if let Some(set_sleep_matches) = clap_matches.subcommand_matches("set-sleep") {
        todo!()
    } else if let Some(make_matches) = clap_matches.subcommand_matches("make") {
        make_schedule(&formatting, &mut data)
    } else if let Some(show_matches) = clap_matches.subcommand_matches("show") {
        data.get_schedule().print();
    }

    data.write_to_file(data_file_path).unwrap();
}

fn add_task(formatting: &Formatting, data: &mut UserData) {
    let new_task = Task::prompt_new(&formatting).unwrap();
    data.add_task(new_task);
}

fn prompt_add_type() {}

fn make_schedule(formatting: &Formatting, data: &mut UserData) {
    data.set_schedule(
        Schedule::make(data.get_config(), data.get_tasks(), data.get_events()).unwrap(),
    );
}
