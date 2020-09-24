use sparrow::Formatting;

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

