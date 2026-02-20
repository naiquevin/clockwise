mod org_parser;
mod time_duration;

use time_duration::DateRange;

#[derive(Debug, Clone, Copy)]
pub enum Breakdown {
    Day,
    Week,
    Month,
    Quarter,
}

impl Breakdown {
    /// Checks if the breakdown duration is smaller than the time duration
    /// represented by the DateRange.
    pub fn is_smaller_than_duration(&self, date_range: &DateRange) -> bool {
        let days = (date_range.end - date_range.start).num_days();

        match self {
            Breakdown::Day => days > 1,
            Breakdown::Week => days > 7,
            Breakdown::Month => days > 30,
            Breakdown::Quarter => days > 90,
        }
    }
}

fn main() {
    println!("Hello, world!");
}
