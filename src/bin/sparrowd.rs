//! Runs a daemon that periodically checks for the next event in the user's PomodoroSchedule, notifying
//! them with libnotify before and once an event starts.

use clap::{App, Arg};
use notify::Watcher;
use sparrow::{
    methods::pomodoro::{PomodoroSchedule, PomodoroScheduleEntry},
    SparrowError, UserData,
};
use std::{
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

fn main() {
    // init libnotify
    libnotify::init("sparrowd").unwrap();

    // clap dat app
    let app = App::new("sparrowd")
        .version("0.0.0")
        .author("Harrison Thorne <harrisonthorne@protonmail.com>")
        .arg(
            Arg::with_name("file")
                .long("file")
                .short("f")
                .takes_value(true)
                .value_name("FILE")
                .help("Specifies a different data file"),
        );

    let clap_matches = app.get_matches();
    let data_file_path = clap_matches
        .value_of("file")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".sparrow"));

    // get data
    let data = UserData::from_file(&data_file_path).unwrap();
    let schedule_mutex = if let Some(pomodoro) = data.get_pomodoro_schedule() {
        Arc::new(Mutex::new(pomodoro.clone()))
    } else {
        eprintln!("no pomodoro schedule found! try adding tasks with `sparrow add task` and then making a schedule with `sparrow make`");
        return;
    };

    // start watching!
    watch_file(data_file_path, schedule_mutex.clone());

    let mut current_event: Option<PomodoroScheduleEntry> = None;
    let mut next_event: Option<PomodoroScheduleEntry> = None;
    let mut notified_of_current_event = false;
    let mut warned_of_next_event = false;
    loop {
        let now = chrono::Local::now();
        if let (Some(current), Some(next)) = (&current_event, &next_event) {
            if (current.span().end() < now || *next.span().start() <= now)
                || (current_event.is_none() && next_event.is_none())
            {
                reassign_current_next_events(&schedule_mutex, &mut current_event, &mut next_event);
                notified_of_current_event = false;
                warned_of_next_event = false;
                continue;
            }
        } else if current_event.is_none() && next_event.is_none() {
            reassign_current_next_events(&schedule_mutex, &mut current_event, &mut next_event);
            notified_of_current_event = false;
            warned_of_next_event = false;
            continue;
        }

        if !notified_of_current_event {
            if let Some(current) = &current_event {
                let summary = "Sparrow notification";
                let now_text = format!("Now: {}", current.title());
                if let Some(next) = &next_event {
                    let _ = libnotify::Notification::new(
                        &summary,
                        format!("{}\nNext: {}", now_text, next.title()).as_str(),
                        None,
                    )
                    .show();
                } else {
                    let _ = libnotify::Notification::new(&summary, now_text.as_str(), None).show();
                }
            }
            notified_of_current_event = true;
        } else if !warned_of_next_event {
            // using else-if because I don't want a spam of two notifications at the same time, if applicable
            if let Some(next) = &next_event {
                if now
                    >= *next.span().start()
                        - chrono::Duration::minutes(
                            data.get_config().next_event_warning_minutes as i64,
                        )
                {
                    let minutes_until = (*next.span().start() - now).num_minutes();
                    let _ = libnotify::Notification::new(
                        "Sparrow notification",
                        format!("In {} minutes: {}", minutes_until, next.title()).as_str(),
                        None,
                    )
                    .show();
                    warned_of_next_event = true;
                }
            } else {
                warned_of_next_event = true;
            }
        }

        thread::sleep(std::time::Duration::from_secs(30));
    }

    #[allow(unreachable_code)]
    {
        libnotify::uninit();
    }
}

fn reassign_current_next_events(
    schedule_mutex: &Arc<Mutex<PomodoroSchedule>>,
    current_event: &mut Option<PomodoroScheduleEntry>,
    next_event: &mut Option<PomodoroScheduleEntry>,
) {
    let now = chrono::Local::now();

    let schedule = match schedule_mutex.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };

    let mut skipped = schedule
        .get_entries()
        .iter()
        .cloned()
        .skip_while(|e| e.span().end() <= now);

    *current_event = skipped.next();
    *next_event = skipped.next();
}

/// Starts a new thread which reloads the user data if it is changed
fn watch_file(path: PathBuf, schedule_mutex: Arc<Mutex<PomodoroSchedule>>) -> JoinHandle<()> {
    use notify::DebouncedEvent::*;

    thread::Builder::new()
        .name("data file watcher".to_string())
        .spawn(move || {
            let (tx, rx) = mpsc::channel();

            let mut watcher = notify::watcher(tx, std::time::Duration::from_secs(0)).unwrap();
            watcher
                .watch(path, notify::RecursiveMode::NonRecursive)
                .unwrap();

            let err: Box<dyn std::error::Error> = loop {
                match rx.recv() {
                    Ok(result) => match result {
                        Write(p) | Rename(_, p) => {
                            let mut schedule = match schedule_mutex.lock() {
                                Ok(g) => g,
                                Err(poisoned) => poisoned.into_inner(),
                            };
                            *schedule = match UserData::from_file(p) {
                                Ok(u) => if let Some(pomodoro) = u.get_pomodoro_schedule() {
                                    pomodoro.clone()
                                } else {
                                    eprintln!("no schedule anymore. finna quit");
                                    break Box::new(SparrowError::BasicMessage("the schedule in sparrow's data file went missing".to_string()));
                                },
                                Err(e) => break Box::new(e),
                            }
                        },
                        Remove(_) => break Box::new(SparrowError::BasicMessage("the sparrow data file was deleted".to_string())),
                        Error(e, _) => break Box::new(e),
                        _ => {}
                    },
                    Err(e) => break Box::new(e),
                }
            };

            let noti = libnotify::Notification::new(&format!("Sparrow watcher crashed ({})", err), "If you change your sparrow data file, you won't be notified of your new changes until you restart sparrowd.", None);
            noti.set_timeout(10000);
            noti.show().unwrap();
        })
    .unwrap()
}
