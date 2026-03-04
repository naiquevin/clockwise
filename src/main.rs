mod breakdown;
mod cli;
mod datetime_util;
mod error;
mod heatmap;
mod org_parser;
mod time_duration;

use std::process;

use clap::Parser;

use crate::cli::Cli;

fn main() {
    let cli = Cli::parse();
    match cli.execute() {
        Ok(exit_code) => process::exit(exit_code as i32),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1)
        }
    }
}
