mod cli;
mod error;
mod org_parser;
mod time_duration;

use std::process;

use clap::{Parser, ValueEnum};
use time_duration::DateTimeRange;

use crate::cli::Cli;

#[derive(Debug, Clone, ValueEnum)]
pub enum Breakdown {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl Breakdown {
    /// Checks if the breakdown duration is smaller than the time duration
    /// represented by the DateRange.
    pub fn is_within_duration(&self, date_range: &DateTimeRange) -> bool {
        let days = (date_range.end - date_range.start).num_days();

        match self {
            Breakdown::Day => days > 1,
            Breakdown::Week => days > 7,
            Breakdown::Month => days > 30,
            Breakdown::Quarter => days > 90,
            Breakdown::Year => days > 365,
        }
    }
}

fn main() {
    let cli = Cli::parse();
    println!("{cli:?}");
    match cli.execute() {
        Ok(exit_code) => process::exit(exit_code as i32),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1)
        }
    }
}
