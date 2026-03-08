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

    #[serde(default)]
    autoclear: bool,

    #[serde(default)]
    hash: Option<u32>,

    #[serde(default, deserialize_with = "deserialize_completed")]
    completed: Option<NaiveDate>,
}

impl Task {
    fn new(title: String, end: NaiveDate, autoclear: bool) -> Self {
        let mut s = Self {
            title,
            end,
            autoclear,
            completed: None,
            hash: None,
        };

        s.hash = Some(s.get_id());
        return s;
    }

    fn display(&self) {
        let today = Local::now().date_naive();
        let id = format!("[{:0>6X}]", self.get_id()).cyan();

        let status = match self.completed {
            None => {
                let delta = (self.end - today).num_days();

                let s = format!("{:>3}d", delta);

                if delta < 0 {
                    s.red()
                } else if delta < 3 {
                    s.yellow()
                } else {
                    s.green()
                }
            }

            Some(done) => {
                let delta = (self.end - done).num_days();

                let s = format!("✓{:>3}d", delta);

                s.blue().dimmed()
            }
        };

        let title = match self.completed {
            Some(_) => self.title.dimmed().strikethrough(),
            None => self.title.normal(),
        };

        println!("{id} {status}  {title}");
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

        /// Task will be hidden/cleared after deadline
        #[arg(long, short = 'c')]
        autoclear: bool,
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
    View,

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

    serde_json::from_str(&content).unwrap_or_default()
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
            autoclear,
        } => {
            let date = NaiveDate::parse_from_str(&end, "%Y-%m-%d")
                .expect("Invalid date format. Use YYYY-MM-DD");

            let mut tasks = load_tasks(&data_path);

            let task = Task::new(title, date, autoclear);
            task.display();
            tasks.push(task);

            save_tasks(&data_path, &mut tasks);
        }

        Commands::Strike { hash } => {
            // fetch tasks
            let mut tasks = load_tasks(&data_path);

            // find task
            let mut target_task = None;
            for (i, task) in tasks.iter().enumerate() {
                if format!("{:0<6X}", task.get_id()) == hash {
                    target_task = Some(i);
                    break;
                }
            }

            // exit if invalid hash
            if target_task.is_none() {
                eprintln!(
                    "{}: could not find task with hash '{}'",
                    "ERROR".red().bold(),
                    hash
                );
                return;
            }

            let target_task = target_task.unwrap();
            tasks[target_task].strike();
            tasks[target_task].display();

            save_tasks(&data_path, &mut tasks);
        }
        Commands::Unstrike { hash } => {
            // fetch tasks
            let mut tasks = load_tasks(&data_path);

            // find task
            let mut target_task = None;
            for (i, task) in tasks.iter().enumerate() {
                if format!("{:0<6X}", task.get_id()) == hash {
                    target_task = Some(i);
                    break;
                }
            }

            // exit if invalid hash
            if target_task.is_none() {
                eprintln!(
                    "{}: could not find task with hash '{}'",
                    "ERROR".red().bold(),
                    hash
                );
                return;
            }

            let target_task = target_task.unwrap();
            tasks[target_task].unstrike();
            tasks[target_task].display();

            save_tasks(&data_path, &mut tasks);
        }

        Commands::Del { hash, force } => {
            // fetch tasks
            let mut tasks = load_tasks(&data_path);

            // find task
            let mut target_task = None;
            for (i, task) in tasks.iter().enumerate() {
                if format!("{:0<6X}", task.get_id()) == hash {
                    target_task = Some(i);
                    break;
                }
            }

            // exit if invalid hash
            if target_task.is_none() {
                eprintln!(
                    "{}: could not find task with hash '{}'",
                    "ERROR".red().bold(),
                    hash
                );
                return;
            }

            let target_task = target_task.unwrap();
            tasks[target_task].display();

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
            println!("Task {} successfully deleted", format!("[{}]", hash).cyan());
            save_tasks(&data_path, &mut tasks);
        }

        Commands::View => {
            println!("{}", "Deadline".bold().underline());
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

                    if task.autoclear && days < 0 {
                        return false;
                    }

                    true
                })
                .collect();

            if visible_tasks.is_empty() {
                println!("No visible tasks.");
                return;
            }

            visible_tasks.sort_by_key(|task| (task.end - Local::now().date_naive()).num_days());
            for task in visible_tasks {
                task.display();
            }
        }

        Commands::Path => {
            println!("{}", data_path.display());
        }
    }
}
