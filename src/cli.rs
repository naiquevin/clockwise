use std::path::PathBuf;

use clap::Parser;

use crate::{Breakdown, error::Error, time_duration::parse_time_duration};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    path: PathBuf,
    #[arg(short, long, default_value = "d")]
    time_duration: String,
    #[arg(short, long, value_enum)]
    breakdown: Option<Breakdown>,
}

impl Cli {
    pub fn execute(&self) -> Result<i8, Error> {
        let time_duration = parse_time_duration(&self.time_duration)?;
        let breakdown = self.breakdown.as_ref().and_then(|b| {
            if b.is_within_duration(&time_duration) {
                Some(b)
            } else {
                None
            }
        });
        println!("Date time range: {time_duration:?}; Breakdown: {breakdown:?}");
        Ok(0)
    }
}
