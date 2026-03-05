use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use colored::Colorize;

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

        let days_colored = if days < 2 {
            days.to_string().red()
        } else {
            days.to_string().green()
        };

        println!("{:>3} days - {}", days_colored, self.title);
    }
}

#[derive(Parser)]
#[command(name = "deadline")]
#[command(about = "A tiny CLI deadline tracker", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { title: String, end: String },
    View,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { title, end } => {
            let date = NaiveDate::parse_from_str(&end, "%Y-%m-%d")
                .expect("Invalid date format. Use YYYY-MM-DD");

            let task = Task::new(title, date);

            println!("Added task:");
            task.display();
        }

        Commands::View => {
            // For now: hardcoded tasks
            let task = Task::new(
                "Christmas".to_string(),
                NaiveDate::from_ymd_opt(2026, 12, 25).unwrap(),
            );

            let ny = Task::new(
                "New Years".to_string(),
                NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            );

            task.display();
            ny.display();
        }
    }
}
