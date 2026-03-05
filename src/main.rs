use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use colored::Colorize;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct Task {
    title: String,
    end: NaiveDate,
}

impl Task {
    fn new(title: String, end: NaiveDate) -> Self {
        Self { title, end }
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

        println!("{:>3} days - {}", days_colored, self.title);
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
    Add { title: String, end: String },
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

fn save_tasks(path: &PathBuf, tasks: &[Task]) {
    let json = serde_json::to_string_pretty(tasks).expect("Could not serialize tasks");

    fs::write(path, json).expect("Could not write tasks file");
}

fn main() {
    let cli = Cli::parse();
    let data_path = data_file_path();

    match cli.command {
        Commands::Add { title, end } => {
            let date = NaiveDate::parse_from_str(&end, "%Y-%m-%d")
                .expect("Invalid date format. Use YYYY-MM-DD");

            let mut tasks = load_tasks(&data_path);

            let task = Task::new(title, date);
            tasks.push(task);

            save_tasks(&data_path, &tasks);

            println!("Task added.");
        }

        Commands::View => {
            let tasks = load_tasks(&data_path);

            if tasks.is_empty() {
                println!("No tasks yet.");
                return;
            }

            for task in tasks {
                task.display();
            }
        }

        Commands::Path => {
            println!("{}", data_path.display());
        }
    }
}
