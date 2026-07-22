use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use colored::Colorize;
use directories::ProjectDirs;
use serde::{Deserialize, Deserializer, Serialize};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Write, stdin, stdout};
use std::path::PathBuf;
use std::process::Command;

// patchwork because of my poor schema planning :P
fn deserialize_completed<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum CompletedField {
        Bool(bool),
        Date(NaiveDate),
    }

    let value = Option::<CompletedField>::deserialize(deserializer)?;

    match value {
        Some(CompletedField::Bool(true)) => Ok(Some(Local::now().date_naive())),
        Some(CompletedField::Bool(false)) => Ok(None),
        Some(CompletedField::Date(date)) => Ok(Some(date)),
        None => Ok(None),
    }
}

#[derive(Serialize, Deserialize, Hash)]
struct Task {
    title: String,
    end: NaiveDate,

    #[serde(default, alias = "autoclear")]
    autostrike: bool,

    #[serde(default)]
    hash: Option<u32>,

    #[serde(default, deserialize_with = "deserialize_completed")]
    completed: Option<NaiveDate>,
}

impl Task {
    fn new(title: String, end: NaiveDate, autostrike: bool) -> Self {
        let mut s = Self {
            title,
            end,
            autostrike,
            completed: None,
            hash: None,
        };

        s.hash = Some(s.get_id());
        return s;
    }

    fn display(&self, opts: DisplayOpts) {
        let today = Local::now().date_naive();
        let id = if opts.show_hash {
            format!("[{:0>6X}] ", self.get_id()).cyan()
        } else {
            "".normal()
        };

        let raw_status = match self.completed {
            None => {
                let delta = (self.end - today).num_days();
                format!("{:>3}d", delta)
            }
            Some(done) => {
                let delta = (self.end - done).num_days();
                format!("✓{:>3}d", delta)
            }
        };

        let status = match self.completed {
            None => {
                let delta = (self.end - today).num_days();

                if delta < 2 {
                    raw_status.red()
                } else if delta < 5 {
                    raw_status.yellow()
                } else {
                    raw_status.green()
                }
            }
            Some(_) => raw_status.blue(),
        };

        // enforce fixed column width
        let status = format!("{status:>5}");

        let title = match self.completed {
            Some(_) => self.title.dimmed().strikethrough(),
            None => self.title.normal(),
        };

        let autoclear = if self.autostrike && opts.show_flags {
            " [-s]".yellow()
        } else {
            "".normal()
        };

        println!("{id}{status}  {title}{autoclear}");
    }

    fn get_id(&self) -> u32 {
        if let Some(id) = self.hash {
            id
        } else {
            let mut hasher = DefaultHasher::new();
            self.hash(&mut hasher);
            (hasher.finish() & 0x00FFFFFF) as u32
        }
    }

    fn ensure_hash(&mut self) {
        if self.hash.is_none() {
            let mut hasher = DefaultHasher::new();
            self.hash(&mut hasher);
            self.hash = Some((hasher.finish() & 0x00FFFFFF) as u32);
        }
    }

    fn strike(&mut self) {
        if self.completed.is_none() {
            self.completed = Some(Local::now().date_naive());
        }
    }

    fn unstrike(&mut self) {
        if self.completed.is_some() {
            self.completed = None
        }
    }

    fn apply_autostrike(&mut self) {
        if self.autostrike && self.completed.is_none() {
            let today = Local::now().date_naive();

            if self.end < today {
                self.completed = Some(today);
            }
        }
    }
}

#[derive(Deserialize)]
struct CalendarEvent {
    title: String,
    start: String,
    end: String,
    #[serde(rename = "allDay")]
    all_day: bool,
    calendar: String,

    #[serde(skip)]
    hash: u32,

    #[serde(skip)]
    struck: bool,
}

impl CalendarEvent {
    fn compute_hash(&mut self) {
        let mut hasher = DefaultHasher::new();
        self.title.hash(&mut hasher);
        self.start.hash(&mut hasher);
        self.hash = (hasher.finish() & 0x00FFFFFF) as u32;
    }

    fn get_id(&self) -> u32 {
        self.hash
    }

    fn start_date(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.start[..10], "%Y-%m-%d").ok()
    }

    fn end_date(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.end[..10], "%Y-%m-%d").ok()
    }

    fn display(&self, opts: DisplayOpts) {
        let today = Local::now().date_naive();
        let id = if opts.show_hash {
            format!("[{:0>6X}] ", self.get_id()).cyan()
        } else {
            "".normal()
        };

        let start = self.start_date().unwrap_or(today);
        let delta = (start - today).num_days();

        let raw_status = if self.struck {
            format!("✓{:>3}d", delta)
        } else {
            format!("{:>3}d", delta)
        };

        let status = if self.struck {
            raw_status.blue()
        } else if delta < 2 {
            raw_status.red()
        } else if delta < 5 {
            raw_status.yellow()
        } else {
            raw_status.green()
        };

        let status = format!("{status:>5}");

        let title_text = if self.struck {
            format!("{} (calendar)", self.title).dimmed().strikethrough()
        } else {
            format!("{} (calendar)", self.title).dimmed()
        };

        let cal_name = if opts.show_flags {
            format!(" [{}]", self.calendar).dimmed().italic()
        } else {
            "".normal()
        };

        println!("{id}{status}  {title_text}{cal_name}");
    }
}

fn load_calendar_events(days: i64) -> Vec<CalendarEvent> {
    if cfg!(not(target_os = "macos")) {
        eprintln!(
            "{}: calendar integration is only available on macOS",
            "WARN".yellow().bold()
        );
        return Vec::new();
    }

    let binary_names = ["calendar-reader", "calendar_reader"];
    let mut binary_path = None;

    for name in &binary_names {
        let path = PathBuf::from(format!("./{}", name));
        if path.exists() {
            binary_path = Some(path);
            break;
        }
    }

    let binary = match binary_path {
        Some(p) => p,
        None => {
            eprintln!(
                "{}: calendar-reader binary not found in current directory",
                "WARN".yellow().bold()
            );
            return Vec::new();
        }
    };

    let output = Command::new(&binary)
        .arg(days.to_string())
        .output();

    let mut events: Vec<CalendarEvent> = match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            serde_json::from_str(&stdout).unwrap_or_else(|e| {
                eprintln!(
                    "{}: failed to parse calendar output: {}",
                    "WARN".yellow().bold(),
                    e
                );
                Vec::new()
            })
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            eprintln!(
                "{}: calendar-reader failed: {}",
                "WARN".yellow().bold(),
                stderr.trim()
            );
            Vec::new()
        }
        Err(e) => {
            eprintln!(
                "{}: could not run calendar-reader: {}",
                "WARN".yellow().bold(),
                e
            );
            Vec::new()
        }
    };

    let struck_hashes = load_struck_events();
    for event in events.iter_mut() {
        event.compute_hash();
        event.struck = struck_hashes.contains(&event.hash);
    }

    events
}

fn struck_events_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "nandu", "deadline")
        .expect("Could not determine project directories");
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir).expect("Could not create data directory");
    data_dir.join("struck_events.json")
}

fn load_struck_events() -> Vec<u32> {
    let path = struck_events_path();
    if !path.exists() {
        return Vec::new();
    }
    let content = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_struck_events(hashes: &[u32]) {
    let path = struck_events_path();
    let json = serde_json::to_string_pretty(hashes).expect("Could not serialize struck events");
    fs::write(path, json).expect("Could not write struck events file");
}

fn find_calendar_event(hash: &str, events: &[CalendarEvent]) -> Option<usize> {
    let mut matches = Vec::new();
    for (i, event) in events.iter().enumerate() {
        let id = format!("{:0>6X}", event.get_id());
        if id.starts_with(&hash.to_uppercase()) {
            matches.push(i);
        }
    }
    match matches.len() {
        0 => {
            eprintln!(
                "{}: could not find calendar event with hash '{}'",
                "ERROR".red().bold(),
                hash
            );
            None
        }
        1 => Some(matches[0]),
        _ => {
            eprintln!(
                "{}: hash '{}' is ambiguous ({} matches)",
                "ERROR".red().bold(),
                hash,
                matches.len()
            );
            None
        }
    }
}

enum DisplayItem<'a> {
    Task(&'a Task),
    Calendar(&'a CalendarEvent),
}

impl<'a> DisplayItem<'a> {
    fn sort_key(&self, today: NaiveDate) -> (i32, NaiveDate) {
        match self {
            DisplayItem::Task(task) => {
                if task.completed.is_some() {
                    (2, task.end)
                } else if (task.end - today).num_days() < 0 {
                    (0, task.end)
                } else {
                    (1, task.end)
                }
            }
            DisplayItem::Calendar(event) => {
                let date = event.start_date().unwrap_or(today);
                if event.struck {
                    (2, date)
                } else if (date - today).num_days() < 0 {
                    (0, date)
                } else {
                    (1, date)
                }
            }
        }
    }

    fn display(&self, opts: DisplayOpts) {
        match self {
            DisplayItem::Task(task) => task.display(opts),
            DisplayItem::Calendar(event) => event.display(opts),
        }
    }

    fn is_struck(&self) -> bool {
        match self {
            DisplayItem::Task(task) => task.completed.is_some(),
            DisplayItem::Calendar(event) => event.struck,
        }
    }
}

#[derive(Clone, Copy)]
struct DisplayOpts {
    show_hash: bool,
    show_flags: bool,
}

impl DisplayOpts {
    fn new(show_hash: bool, show_flags: bool) -> Self {
        Self {
            show_hash,
            show_flags,
        }
    }

    fn default() -> Self {
        Self {
            show_hash: true,
            show_flags: true,
        }
    }
}

#[derive(Parser)]
#[command(name = "deadline")]
#[command(about = "A tiny CLI deadline tracker")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add {
        /// Title/short description of task
        title: String,

        /// Deadline of the task in YYYY-MM-DD
        end: String,

        /// Task will be striked after deadline
        #[arg(long, short = 's')]
        autostrike: bool,
    },

    /// Delete an existing task
    Del {
        /// Hash of the task
        hash: String,

        /// Task will be deleted without confirmation
        #[arg(long, short)]
        force: bool,
    },

    /// Strike/mark a task as completed
    Strike {
        /// Hash of the task
        hash: String,
    },

    /// Unstrike a task
    Unstrike {
        /// Hash of the task
        hash: String,
    },

    /// Push a task
    Push { hash: String, date: String },

    /// Toggle autostrike for a task
    Astrike { hash: String },

    /// View all the tasks
    View {
        #[arg(long, short)]
        reverse: bool,

        #[arg(long, short)]
        completed: bool,

        #[arg(long, short)]
        overdue: bool,

        #[arg(long = "no-hash")]
        no_hash: bool,

        /// Default behaviour; left for backwards compatibility
        #[arg(long = "no-title")]
        no_title: bool,

        #[arg(long = "no-flags")]
        no_flags: bool,

        #[arg(long, short)]
        title: Option<String>,

        #[arg(long, short)]
        all: bool,

        /// Number of lines to be printed (shows n-1 tasks, n at most)
        #[arg(long, short = 'l')]
        lines: Option<usize>,

        /// Include upcoming calendar events
        #[arg(long, short = 'C')]
        calendar: bool,

        /// Number of days to look ahead for calendar events (default 14)
        #[arg(long = "cal-days", default_value = "14")]
        cal_days: i64,

        /// Filter calendar events by calendar name
        #[arg(long = "name")]
        cal_name: Option<String>,
    },

    /// Print the path to data file
    Path,

    /// View upcoming calendar events
    #[command(alias = "cal")]
    Calendar {
        /// Number of days to look ahead
        #[arg(long, short = 'd', default_value = "14")]
        days: i64,

        #[arg(long = "no-hash")]
        no_hash: bool,

        /// Number of lines to be printed
        #[arg(long, short = 'l')]
        lines: Option<usize>,

        /// Show struck (completed) calendar events
        #[arg(long, short)]
        completed: bool,

        /// Filter by calendar name
        #[arg(long = "name")]
        cal_name: Option<String>,
    },
}

fn data_file_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "nandu", "deadline")
        .expect("Could not determine project directories");

    let data_dir = proj_dirs.data_dir();

    fs::create_dir_all(data_dir).expect("Could not create data directory");

    data_dir.join("tasks.json")
}

fn load_tasks(path: &PathBuf) -> Vec<Task> {
    if !path.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(path).expect("Could not read tasks file");

    let mut tasks: Vec<Task> = serde_json::from_str(&content).unwrap_or_default();

    for task in tasks.iter_mut() {
        task.apply_autostrike();
    }

    tasks
}

fn save_tasks(path: &PathBuf, tasks: &mut [Task]) {
    for task in tasks.iter_mut() {
        task.ensure_hash();
    }

    let json = serde_json::to_string_pretty(tasks).expect("Could not serialize tasks");
    fs::write(path, json).expect("Could not write tasks file");
}

fn main() {
    let cli = Cli::parse();
    let data_path = data_file_path();

    match cli.command {
        Commands::Add {
            title,
            end,
            autostrike: autoclear,
        } => {
            let date = parse_date(end);

            let mut tasks = load_tasks(&data_path);

            let task = Task::new(title, date, autoclear);
            task.display(DisplayOpts::default());
            tasks.push(task);

            save_tasks(&data_path, &mut tasks);
        }

        Commands::Strike { hash } => {
            let mut tasks = load_tasks(&data_path);

            if let Some(target_task) = find_task(hash.clone(), &tasks) {
                tasks[target_task].strike();
                tasks[target_task].display(DisplayOpts::default());
                save_tasks(&data_path, &mut tasks);
                return;
            }

            let mut events = load_calendar_events(90);
            if let Some(target_event) = find_calendar_event(&hash, &events) {
                events[target_event].struck = true;
                events[target_event].display(DisplayOpts::default());
                let hashes: Vec<u32> = events.iter().filter(|e| e.struck).map(|e| e.hash).collect();
                save_struck_events(&hashes);
                return;
            }

            eprintln!(
                "{}: could not find task or calendar event with hash '{}'",
                "ERROR".red().bold(),
                hash
            );
        }

        Commands::Unstrike { hash } => {
            let mut tasks = load_tasks(&data_path);

            if let Some(target_task) = find_task(hash.clone(), &tasks) {
                tasks[target_task].unstrike();
                tasks[target_task].display(DisplayOpts::default());
                save_tasks(&data_path, &mut tasks);
                return;
            }

            let mut events = load_calendar_events(90);
            if let Some(target_event) = find_calendar_event(&hash, &events) {
                events[target_event].struck = false;
                events[target_event].display(DisplayOpts::default());
                let hashes: Vec<u32> = events.iter().filter(|e| e.struck).map(|e| e.hash).collect();
                save_struck_events(&hashes);
                return;
            }

            eprintln!(
                "{}: could not find task or calendar event with hash '{}'",
                "ERROR".red().bold(),
                hash
            );
        }

        Commands::Astrike { hash } => {
            let mut tasks = load_tasks(&data_path);
            let target_task = match find_task(hash, &tasks) {
                Some(value) => value,
                None => return,
            };

            let task = &mut tasks[target_task];
            (*task).autostrike = !task.autostrike;
            task.display(DisplayOpts::default());

            save_tasks(&data_path, &mut tasks);
        }

        Commands::Push { hash, date } => {
            let mut tasks = load_tasks(&data_path);

            let target_task = match find_task(hash, &tasks) {
                Some(value) => value,
                None => return,
            };
            let date = parse_date(date);

            tasks[target_task].end = date;
            println!(
                "Task pushed to {} successfully",
                date.format("%Y-%m-%d").to_string().green()
            );
            tasks[target_task].display(DisplayOpts::default());

            save_tasks(&data_path, &mut tasks);
        }

        Commands::Del { hash, force } => {
            // fetch tasks
            let mut tasks = load_tasks(&data_path);
            let target_task = match find_task(hash.clone(), &tasks) {
                Some(value) => value,
                None => return,
            };

            tasks[target_task].display(DisplayOpts::default());

            // confirmation message if not forced
            if !force {
                println!(
                    "{}",
                    "Hint: Use -f or --force to delete without a confirmation".yellow()
                );
                print!(
                    "Are you sure you want to delete the above task? This action cannot be undone [N/y]: "
                );
                stdout().flush().unwrap();

                let mut input = String::new();
                stdin().read_line(&mut input).unwrap();

                if input.as_str().to_lowercase().trim() != "y" {
                    println!("Deletion cancelled");
                    return;
                }
            }

            // delete the task
            println!(
                "Task {} successfully deleted",
                format!("[{:0<6X}]", tasks[target_task].get_id()).cyan()
            );
            tasks.remove(target_task);
            save_tasks(&data_path, &mut tasks);
        }

        Commands::View {
            reverse,
            completed,
            overdue,
            no_hash,
            title,
            all,
            no_flags,
            lines: count,
            calendar,
            cal_days,
            cal_name,

            #[allow(unused)] // default
            no_title,
        } => {
            if title.is_some() {
                println!("{}", title.unwrap().bold().underline());
            }
            let tasks = load_tasks(&data_path);

            let today = Local::now().date_naive();

            let mut visible_tasks: Vec<_> = tasks
                .into_iter()
                .filter(|task| {
                    let days = (task.end - today).num_days();

                    if all {
                        return true;
                    }

                    if completed {
                        return task.completed.is_some();
                    }

                    if overdue {
                        return days < 0 && task.completed.is_none();
                    }

                    // default
                    task.completed.is_none()
                })
                .collect();

            visible_tasks.sort_by_key(|task| task.end);
            visible_tasks.sort_by_key(|task| {
                if task.completed.is_some() {
                    2
                } else if (task.end - today).num_days() < 0 {
                    0
                } else {
                    1
                }
            });

            let calendar_events = if calendar {
                let mut events = load_calendar_events(cal_days);

                if let Some(ref name) = cal_name {
                    events.retain(|e| e.calendar.to_lowercase() == name.to_lowercase());
                }

                if !all && !completed {
                    events.retain(|e| !e.struck);
                } else if completed && !all {
                    events.retain(|e| e.struck);
                }

                events
            } else {
                Vec::new()
            };

            let mut all_items: Vec<DisplayItem> = visible_tasks
                .iter()
                .map(DisplayItem::Task)
                .collect();

            for event in &calendar_events {
                all_items.push(DisplayItem::Calendar(event));
            }

            all_items.sort_by_key(|item| item.sort_key(today));

            if all_items.is_empty() {
                println!("No visible tasks.");
                return;
            }

            if reverse {
                all_items.reverse();
            }

            let total = all_items.len();
            let limit = count.unwrap_or(total);

            if total <= limit {
                for item in &all_items {
                    item.display(DisplayOpts::new(!no_hash, !no_flags));
                }
            } else {
                let shown = limit.saturating_sub(1);
                for item in all_items.iter().take(shown) {
                    item.display(DisplayOpts::new(!no_hash, !no_flags));
                }
                let remaining = total - shown;
                println!(
                    "{}",
                    format!(
                        "+{} more item{}",
                        remaining,
                        if remaining > 1 { "s" } else { "" }
                    )
                    .dimmed()
                    .italic()
                );
            }
        }

        Commands::Path => {
            println!("{}", data_path.display());
        }

        Commands::Calendar {
            days,
            no_hash,
            lines: count,
            completed,
            cal_name,
        } => {
            let mut events = load_calendar_events(days);

            if let Some(ref name) = cal_name {
                events.retain(|e| e.calendar.to_lowercase() == name.to_lowercase());
            }

            if !completed {
                events.retain(|e| !e.struck);
            }

            if events.is_empty() {
                println!("No upcoming calendar events.");
                return;
            }

            let today = Local::now().date_naive();
            let opts = DisplayOpts::new(!no_hash, true);

            events.sort_by_key(|e| e.start_date().unwrap_or(today));

            let total = events.len();
            let limit = count.unwrap_or(total);

            if total <= limit {
                for event in &events {
                    event.display(opts);
                }
            } else {
                let shown = limit.saturating_sub(1);
                for event in events.iter().take(shown) {
                    event.display(opts);
                }
                let remaining = total - shown;
                println!(
                    "{}",
                    format!(
                        "+{} more event{}",
                        remaining,
                        if remaining > 1 { "s" } else { "" }
                    )
                    .dimmed()
                    .italic()
                );
            }
        }
    }
}

fn parse_date(end: String) -> NaiveDate {
    if let Some(days) = end.strip_suffix('d') {
        let days: i64 = days.parse().expect("Invalid day format. Use Xd (e.g. 3d)");

        Local::now().date_naive() + chrono::Duration::days(days)
    } else {
        NaiveDate::parse_from_str(&end, "%Y-%m-%d")
            .expect("Invalid date format. Use YYYY-MM-DD or Xd")
    }
}

fn find_task(hash: String, tasks: &Vec<Task>) -> Option<usize> {
    let mut matches = Vec::new();

    for (i, task) in tasks.iter().enumerate() {
        let id = format!("{:0>6X}", task.get_id());

        if id.starts_with(&hash.to_uppercase()) {
            matches.push(i);
        }
    }

    match matches.len() {
        0 => {
            eprintln!(
                "{}: could not find task with hash '{}'",
                "ERROR".red().bold(),
                hash
            );
            None
        }

        1 => Some(matches[0]),

        _ => {
            eprintln!(
                "{}: hash '{}' is ambiguous ({} matches)",
                "ERROR".red().bold(),
                hash,
                matches.len()
            );
            None
        }
    }
}
