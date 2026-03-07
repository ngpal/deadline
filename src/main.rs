use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use colored::Colorize;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Hash)]
struct Task {
    title: String,
    end: NaiveDate,

    #[serde(default)]
    autoclear: bool,

    #[serde(default)]
    hash: Option<u32>,

    #[serde(default)]
    completed: bool,
}

impl Task {
    fn new(title: String, end: NaiveDate, autoclear: bool) -> Self {
        let mut s = Self {
            title,
            end,
            autoclear,
            completed: false,
            hash: None,
        };

        s.hash = Some(s.get_id());
        return s;
    }

    fn display(&self) {
        let days = (self.end - Local::now().date_naive()).num_days();

        let days_colored = if days < 0 {
            days.to_string().red()
        } else if days < 3 {
            days.to_string().yellow()
        } else {
            days.to_string().green()
        };

        println!(
            "{} {:>3} days - {}",
            format!("[{:0>6X}]", self.get_id()).cyan(),
            days_colored,
            self.title
        );
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
    Add {
        title: String,
        end: String,

        #[arg(long, short = 'c')]
        autoclear: bool,
    },
    View,
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
            tasks.push(task);

            save_tasks(&data_path, &mut tasks);

            println!("Task added.");
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
