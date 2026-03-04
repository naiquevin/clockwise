use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::Parser;

use crate::{
    breakdown::{Breakdown, bucket_label},
    datetime_util::secs_to_rounded_hours_mins,
    error::Error,
    heatmap::print_heatmap,
    org_parser::parse_org_clock_entries,
    time_duration::parse_time_duration,
};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    path: PathBuf,
    #[arg(short, long, default_value = "d")]
    time_duration: String,
    #[arg(short, long, value_enum)]
    breakdown: Option<Breakdown>,
    #[arg(long)]
    heatmap: bool,
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
        // println!("Date time range: {time_duration:?}; Breakdown: {breakdown:?}");

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let lines = reader
            .lines()
            .collect::<Result<Vec<String>, std::io::Error>>()?;
        let entries = parse_org_clock_entries(lines);

        if self.heatmap {
            println!("{}", "-".repeat(30));
            print_heatmap(&entries, &time_duration);
        }

        println!("{}", "-".repeat(30));
        println!(
            "From {} to {}",
            time_duration.start.format("%Y-%m-%d"),
            time_duration.end.format("%Y-%m-%d")
        );
        println!("{}", "-".repeat(30));

        if let Some(b) = breakdown {
            let buckets = b.buckets(&time_duration);
            let mut agg: Vec<i64> = vec![0; buckets.len()];
            for entry in entries {
                if entry.start < time_duration.start || entry.end >= time_duration.end {
                    continue;
                }
                for (i, bucket) in buckets.iter().enumerate() {
                    if entry.is_between(bucket) {
                        let duration = entry.duration();
                        agg[i] += duration.seconds();
                        continue;
                    }
                }
            }

            let mut total_seconds = 0;

            for (i, bucket) in buckets.iter().enumerate() {
                // println!("Bucket = {bucket:?}");
                let label = bucket_label(b, &bucket.start, &time_duration);
                let (hours, minutes) = secs_to_rounded_hours_mins(agg[i]);
                println!("{label} {hours:02}:{minutes:02}");
                total_seconds += agg[i];
            }

            let (hours, minutes) = secs_to_rounded_hours_mins(total_seconds);
            println!("{}", "-".repeat(30));
            println!("Total: {hours:02} hours {minutes:02} minutes");
        } else {
            let mut total_seconds = 0;
            for entry in entries {
                if entry.is_between(&time_duration) {
                    let entry_duration = entry.duration();
                    total_seconds += entry_duration.seconds();
                }
            }
            let (hours, minutes) = secs_to_rounded_hours_mins(total_seconds);
            println!("Total: {hours:02} hours {minutes:02} minutes");
        }
        Ok(0)
    }
}
