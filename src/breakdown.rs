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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn dt(year: i32, month: u32, day: u32, hour: u32, min: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, min, 0)
            .unwrap()
    }

    fn range(start: NaiveDateTime, end: NaiveDateTime) -> DateTimeRange {
        DateTimeRange::new(start, end).unwrap()
    }

    fn assert_contiguous(buckets: &[DateTimeRange]) {
        for i in 0..buckets.len() - 1 {
            assert_eq!(
                buckets[i].end, buckets[i + 1].start,
                "gap between bucket {i} and {}",
                i + 1
            );
        }
    }

    #[test]
    fn test_day_buckets() {
        // Mid-day range covering parts of 4 days: first bucket aligns to midnight,
        // last bucket extends past dtr.end
        let dtr = range(dt(2026, 3, 2, 14, 0), dt(2026, 3, 5, 9, 0));
        let buckets = Breakdown::Day.buckets(&dtr);
        assert_eq!(buckets.len(), 4);
        assert_eq!(buckets[0].start, dt(2026, 3, 2, 0, 0));
        assert_eq!(buckets[0].end, dt(2026, 3, 3, 0, 0));
        assert_eq!(buckets[3].start, dt(2026, 3, 5, 0, 0));
        assert_eq!(buckets[3].end, dt(2026, 3, 6, 0, 0));
        assert!(buckets[0].start <= dtr.start);
        assert!(buckets.last().unwrap().end >= dtr.end);
        assert_contiguous(&buckets);

        // Exact 2-day range: no overhang at either end
        let dtr = range(dt(2026, 3, 2, 0, 0), dt(2026, 3, 4, 0, 0));
        let buckets = Breakdown::Day.buckets(&dtr);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].start, dt(2026, 3, 2, 0, 0));
        assert_eq!(buckets[1].end, dt(2026, 3, 4, 0, 0));
        assert_contiguous(&buckets);
    }

    #[test]
    fn test_week_buckets() {
        // Mid-week start (Wednesday): first bucket aligns to Monday, spans 3 weeks
        let dtr = range(dt(2026, 3, 4, 9, 0), dt(2026, 3, 18, 9, 0));
        let buckets = Breakdown::Week.buckets(&dtr);
        assert_eq!(buckets.len(), 3);
        assert_eq!(buckets[0].start, dt(2026, 3, 2, 0, 0)); // Monday
        assert_eq!(buckets[0].end, dt(2026, 3, 9, 0, 0));
        assert_eq!(buckets[1].start, dt(2026, 3, 9, 0, 0));
        assert_eq!(buckets[1].end, dt(2026, 3, 16, 0, 0));
        assert_eq!(buckets[2].start, dt(2026, 3, 16, 0, 0));
        assert_eq!(buckets[2].end, dt(2026, 3, 23, 0, 0));
        assert!(buckets[0].start <= dtr.start);
        assert!(buckets.last().unwrap().end >= dtr.end);
        assert_contiguous(&buckets);

        // Year-boundary crossing: week starting Dec 29 spans into January
        let dtr = range(dt(2025, 12, 29, 0, 0), dt(2026, 1, 6, 12, 0));
        let buckets = Breakdown::Week.buckets(&dtr);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].start, dt(2025, 12, 29, 0, 0));
        assert_eq!(buckets[0].end, dt(2026, 1, 5, 0, 0));
        assert_eq!(buckets[1].start, dt(2026, 1, 5, 0, 0));
        assert_eq!(buckets[1].end, dt(2026, 1, 12, 0, 0));
        assert_contiguous(&buckets);
    }

    #[test]
    fn test_month_buckets() {
        // Mid-month start spanning 3 months: first bucket aligns to Jan 1
        let dtr = range(dt(2026, 1, 15, 9, 0), dt(2026, 3, 10, 17, 0));
        let buckets = Breakdown::Month.buckets(&dtr);
        assert_eq!(buckets.len(), 3);
        assert_eq!(buckets[0].start, dt(2026, 1, 1, 0, 0));
        assert_eq!(buckets[0].end, dt(2026, 2, 1, 0, 0));
        assert_eq!(buckets[1].start, dt(2026, 2, 1, 0, 0));
        assert_eq!(buckets[1].end, dt(2026, 3, 1, 0, 0));
        assert_eq!(buckets[2].start, dt(2026, 3, 1, 0, 0));
        assert_eq!(buckets[2].end, dt(2026, 4, 1, 0, 0));
        assert!(buckets[0].start <= dtr.start);
        assert!(buckets.last().unwrap().end >= dtr.end);
        assert_contiguous(&buckets);

        // December → January wrap: dtr.end lands exactly on last bucket's end
        let dtr = range(dt(2025, 11, 1, 0, 0), dt(2026, 2, 1, 0, 0));
        let buckets = Breakdown::Month.buckets(&dtr);
        assert_eq!(buckets.len(), 3);
        assert_eq!(buckets[0].start, dt(2025, 11, 1, 0, 0));
        assert_eq!(buckets[0].end, dt(2025, 12, 1, 0, 0));
        assert_eq!(buckets[1].start, dt(2025, 12, 1, 0, 0));
        assert_eq!(buckets[1].end, dt(2026, 1, 1, 0, 0));
        assert_eq!(buckets[2].start, dt(2026, 1, 1, 0, 0));
        assert_eq!(buckets[2].end, dt(2026, 2, 1, 0, 0));
        assert_contiguous(&buckets);
    }

    #[test]
    fn test_quarter_buckets() {
        // Mid-quarter start: first bucket aligns to Q1 start, dtr.end on exact boundary
        let dtr = range(dt(2026, 1, 15, 0, 0), dt(2026, 7, 1, 0, 0));
        let buckets = Breakdown::Quarter.buckets(&dtr);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].start, dt(2026, 1, 1, 0, 0));
        assert_eq!(buckets[0].end, dt(2026, 4, 1, 0, 0));
        assert_eq!(buckets[1].start, dt(2026, 4, 1, 0, 0));
        assert_eq!(buckets[1].end, dt(2026, 7, 1, 0, 0));
        assert!(buckets[0].start <= dtr.start);
        assert_contiguous(&buckets);

        // Q4 → Q1 year wrap spanning 3 quarters
        let dtr = range(dt(2025, 10, 1, 0, 0), dt(2026, 4, 15, 0, 0));
        let buckets = Breakdown::Quarter.buckets(&dtr);
        assert_eq!(buckets.len(), 3);
        assert_eq!(buckets[0].start, dt(2025, 10, 1, 0, 0));
        assert_eq!(buckets[0].end, dt(2026, 1, 1, 0, 0));
        assert_eq!(buckets[1].start, dt(2026, 1, 1, 0, 0));
        assert_eq!(buckets[1].end, dt(2026, 4, 1, 0, 0));
        assert_eq!(buckets[2].start, dt(2026, 4, 1, 0, 0));
        assert_eq!(buckets[2].end, dt(2026, 7, 1, 0, 0));
        assert!(buckets.last().unwrap().end >= dtr.end);
        assert_contiguous(&buckets);
    }

    #[test]
    fn test_year_buckets() {
        // Mid-year start spanning 3 years: first bucket aligns to Jan 1 of start year
        let dtr = range(dt(2024, 6, 1, 0, 0), dt(2026, 9, 1, 0, 0));
        let buckets = Breakdown::Year.buckets(&dtr);
        assert_eq!(buckets.len(), 3);
        assert_eq!(buckets[0].start, dt(2024, 1, 1, 0, 0));
        assert_eq!(buckets[0].end, dt(2025, 1, 1, 0, 0));
        assert_eq!(buckets[1].start, dt(2025, 1, 1, 0, 0));
        assert_eq!(buckets[1].end, dt(2026, 1, 1, 0, 0));
        assert_eq!(buckets[2].start, dt(2026, 1, 1, 0, 0));
        assert_eq!(buckets[2].end, dt(2027, 1, 1, 0, 0));
        assert!(buckets[0].start <= dtr.start);
        assert!(buckets.last().unwrap().end >= dtr.end);
        assert_contiguous(&buckets);

        // Exact 2-year range: no overhang
        let dtr = range(dt(2025, 1, 1, 0, 0), dt(2027, 1, 1, 0, 0));
        let buckets = Breakdown::Year.buckets(&dtr);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].start, dt(2025, 1, 1, 0, 0));
        assert_eq!(buckets[1].end, dt(2027, 1, 1, 0, 0));
        assert_contiguous(&buckets);
    }
}
