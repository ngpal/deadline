use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use colored::Colorize;
use directories::ProjectDirs;
use serde::{Deserialize, Deserializer, Serialize};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Write, stdin, stdout};
use std::path::PathBuf;

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
    },

    /// Print the path to data file
    Path,
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
            let date = if let Some(days) = end.strip_suffix('d') {
                let days: i64 = days.parse().expect("Invalid day format. Use Xd (e.g. 3d)");

                Local::now().date_naive() + chrono::Duration::days(days)
            } else {
                NaiveDate::parse_from_str(&end, "%Y-%m-%d")
                    .expect("Invalid date format. Use YYYY-MM-DD or Xd")
            };

            let mut tasks = load_tasks(&data_path);

            let task = Task::new(title, date, autoclear);
            task.display(DisplayOpts::default());
            tasks.push(task);

            save_tasks(&data_path, &mut tasks);
        }

        Commands::Strike { hash } => {
            // fetch tasks
            let mut tasks = load_tasks(&data_path);

            let target_task = match find_task(hash, &tasks) {
                Some(value) => value,
                None => return,
            };

            tasks[target_task].strike();
            tasks[target_task].display(DisplayOpts::default());

            save_tasks(&data_path, &mut tasks);
        }
        Commands::Unstrike { hash } => {
            // fetch tasks
            let mut tasks = load_tasks(&data_path);
            let target_task = match find_task(hash, &tasks) {
                Some(value) => value,
                None => return,
            };
            tasks[target_task].unstrike();
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
            tasks.remove(target_task);
            println!(
                "Task {} successfully deleted",
                format!("[{:0<6X}]", tasks[target_task].get_id()).cyan()
            );
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

            #[allow(unused)] // default
            no_title,
        } => {
            if title.is_some() {
                println!("{}", title.unwrap().bold().underline());
            }
            let tasks = load_tasks(&data_path);

            if tasks.is_empty() {
                println!("No tasks yet.");
                return;
            }

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

            if visible_tasks.is_empty() {
                println!("No visible tasks.");
                return;
            }

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

            if reverse {
                visible_tasks.reverse();
            }

            for task in visible_tasks {
                task.display(DisplayOpts::new(!no_hash, !no_flags));
            }
        }

        Commands::Path => {
            println!("{}", data_path.display());
        }
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
