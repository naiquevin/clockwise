use chrono::{
    Datelike, Days, Duration, NaiveDateTime, NaiveTime,
    format::{DelayedFormat, StrftimeItems},
};
use clap::ValueEnum;

use crate::{
    datetime_util::{
        quarter_of, start_of_next_month, start_of_next_quarter, start_of_next_year,
        start_of_this_month, start_of_this_quarter, start_of_this_week, start_of_this_year,
    },
    time_duration::DateTimeRange,
};

#[derive(Debug, Clone, ValueEnum)]
pub enum Breakdown {
    #[value(alias = "d")]
    Day,
    #[value(alias = "w")]
    Week,
    #[value(alias = "m")]
    Month,
    #[value(alias = "q")]
    Quarter,
    #[value(alias = "y")]
    Year,
}

impl Breakdown {
    /// Checks if at least two buckets of the breakdown duration are
    /// possible within the time range.
    pub fn is_within_duration(&self, range: &DateTimeRange) -> bool {
        if range.start >= range.end {
            return false;
        }

        // end is exclusive — move one nanosecond back
        let last_inclusive = range.end - Duration::nanoseconds(1);

        match self {
            Breakdown::Day => range.start.date() != last_inclusive.date(),

            Breakdown::Week => range.start.iso_week() != last_inclusive.iso_week(),

            Breakdown::Month => {
                range.start.year() != last_inclusive.year()
                    || range.start.month() != last_inclusive.month()
            }

            Breakdown::Quarter => quarter_of(range.start) != quarter_of(last_inclusive),

            Breakdown::Year => range.start.year() != last_inclusive.year(),
        }
    }

    pub fn buckets(&self, dtr: &DateTimeRange) -> Vec<DateTimeRange> {
        assert!(self.is_within_duration(dtr));

        let mut result = Vec::new();

        let mut bucket_start = match self {
            Self::Day => dtr.start.date().and_time(NaiveTime::MIN),
            Self::Week => start_of_this_week(dtr.start),
            Self::Month => start_of_this_month(dtr.start),
            Self::Quarter => start_of_this_quarter(dtr.start),
            Self::Year => start_of_this_year(dtr.start),
        };

        while bucket_start < dtr.end {
            let bucket_end = match self {
                Self::Day => bucket_start + Days::new(1),
                Self::Week => bucket_start + Days::new(7),
                Self::Month => start_of_next_month(bucket_start),
                Self::Quarter => start_of_next_quarter(bucket_start),
                Self::Year => start_of_next_year(bucket_start),
            };

            result.push(DateTimeRange::new(bucket_start, bucket_end).unwrap());

            bucket_start = bucket_end;
        }

        result
    }
}

pub fn bucket_label<'a>(
    breakdown: &Breakdown,
    bucket_start: &'a NaiveDateTime,
    time_range: &DateTimeRange,
) -> DelayedFormat<StrftimeItems<'a>> {
    let duration = time_range.duration();
    match breakdown {
        Breakdown::Day => {
            // If time range is <= 1 week, use Mon, Tue, ...
            if duration.inner().num_days() <= 7 {
                bucket_start.format("%a")
            } else {
                bucket_start.format("%a %d/%m")
            }
        }
        Breakdown::Week => bucket_start.format("W%V"),
        Breakdown::Month => bucket_start.format("%B"),
        Breakdown::Quarter => bucket_start.format("Q%q"),
        Breakdown::Year => bucket_start.format("%Y"),
    }
}
