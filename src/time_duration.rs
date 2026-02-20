use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, TimeDelta, Weekday};

pub struct TimeDuration(TimeDelta);

impl std::fmt::Display for TimeDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let total_minutes = self.0.num_minutes();
        let hours = total_minutes / 60;
        let minutes = total_minutes % 60;
        write!(f, "{hours:02}:{minutes:02}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRange {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime, // Exclusive end
}

impl DateRange {
    pub fn new(start: NaiveDateTime, end: NaiveDateTime) -> Result<Self, ParseError> {
        if start > end {
            return Err(ParseError::InvalidRange {
                start: start.to_string(),
                end: end.to_string(),
            });
        }
        Ok(DateRange { start, end })
    }

    /// Creates a range for a single day starting at midnight
    pub fn single_day(date: NaiveDate) -> Self {
        let start = date.and_hms_opt(0, 0, 0).unwrap();
        let end = (date + chrono::Days::new(1)).and_hms_opt(0, 0, 0).unwrap();
        DateRange { start, end }
    }

    /// Returns the duration of the date range in hours and minutes
    pub fn duration(&self) -> TimeDuration {
        let delta = self.end.signed_duration_since(self.start);
        TimeDuration(delta)
    }

    /// Returns a sequence of DateRanges from the given DateRange
    /// where each range in the result falls within a single day
    /// i.e. doesn't cross over to the next day
    pub fn partition_by_day(&self) -> Vec<Self> {
        if self.start.day() == self.end.day() {
            vec![self.clone()]
        } else {
            let mut partitions = vec![];
            let mut curr = self.start;
            while curr < self.end {
                let start = curr.clone();
                let next_day_midnight = curr
                    .date()
                    .succ_opt()
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap();
                let end = std::cmp::min(next_day_midnight, self.end);
                partitions.push(Self { start, end });
                curr = (curr.date() + chrono::Days::new(1))
                    .and_hms_opt(0, 0, 0)
                    .unwrap();
            }
            partitions
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Invalid date: {0}")]
    InvalidDate(String),
    #[error("Invalid range: start {start} is after end {end}")]
    InvalidRange { start: String, end: String },
    #[error("Invalid week number: {0}")]
    InvalidWeekNumber(u32),
    #[error("Invalid quarter: {0}")]
    InvalidQuarter(u32),
}

pub fn parse_time_duration(input: &str) -> Result<DateRange, ParseError> {
    let input = input.trim();

    // Check for ranges first
    if let Some((start_str, end_str, inclusive)) = parse_range_syntax(input) {
        return parse_range(start_str, end_str, inclusive);
    }

    // Single duration parsing
    parse_single_duration(input)
}

fn parse_range_syntax(input: &str) -> Option<(&str, &str, bool)> {
    if let Some(pos) = input.find("..=") {
        let start = &input[..pos];
        let end = &input[pos + 3..];
        return Some((start, end, true));
    }
    if let Some(pos) = input.find("..") {
        let start = &input[..pos];
        let end = &input[pos + 2..];
        return Some((start, end, false));
    }
    None
}

fn parse_range(start_str: &str, end_str: &str, inclusive: bool) -> Result<DateRange, ParseError> {
    let start_range = parse_single_duration(start_str)?;
    let end_range = parse_single_duration(end_str)?;

    let start = start_range.start;
    let end = if inclusive {
        end_range.end // end_range.end is already exclusive, so for a single day it's day+1
    } else {
        end_range.start
    };

    DateRange::new(start, end)
}

fn parse_single_duration(input: &str) -> Result<DateRange, ParseError> {
    let today = Local::now().date_naive();

    // First check for absolute dates (YYYY-MM-DD or YYYY-MM)
    if input.contains('-') && input.chars().next().unwrap().is_ascii_digit() {
        return parse_absolute_date(input);
    }

    // Check for "shortcut" values before other patterns as they are
    // more specific.

    // Check for day of week (mon, tue, etc., mon-1, tue-1)
    if let Some(range) = parse_weekday_pattern(input, today) {
        return Ok(range);
    }

    // Check for month names (jan, feb, etc.)
    if let Some(range) = parse_month_name_pattern(input, today) {
        return Ok(range);
    }

    // Check for day patterns (d, d-n)
    if input.starts_with('d') {
        return parse_day_pattern(input, today);
    }

    // Check for week patterns (w, w-n, w6)
    if input.starts_with('w') {
        return parse_week_pattern(input, today);
    }

    // Check for month patterns (m, m-n)
    if input.starts_with('m') && input.len() <= 3 {
        return parse_month_pattern(input, today);
    }

    // Check for quarter patterns (q1, q2, q3, q4)
    if input.starts_with('q') {
        return parse_quarter_pattern(input, today);
    }

    Err(ParseError::InvalidFormat(input.to_string()))
}

fn parse_absolute_date(input: &str) -> Result<DateRange, ParseError> {
    // Try YYYY-MM-DD
    if let Ok(date) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        return Ok(DateRange::single_day(date));
    }

    // Try YYYY-MM (entire month)
    if let Ok(date) = NaiveDate::parse_from_str(&format!("{}-01", input), "%Y-%m-%d") {
        let start = date.and_hms_opt(0, 0, 0).unwrap();
        let end_date = if date.month() == 12 {
            NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap()
        };
        let end = end_date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(DateRange { start, end });
    }

    Err(ParseError::InvalidDate(input.to_string()))
}

fn parse_day_pattern(input: &str, today: NaiveDate) -> Result<DateRange, ParseError> {
    if input == "d" {
        return Ok(DateRange::single_day(today));
    }

    // d-n pattern
    if input.starts_with("d-") {
        let n_str = &input[2..];
        let n: u32 = n_str
            .parse()
            .map_err(|_| ParseError::InvalidFormat(input.to_string()))?;
        let date = today - chrono::Days::new(n as u64);
        return Ok(DateRange::single_day(date));
    }

    Err(ParseError::InvalidFormat(input.to_string()))
}

fn parse_week_pattern(input: &str, today: NaiveDate) -> Result<DateRange, ParseError> {
    if input == "w" {
        return Ok(get_current_week(today));
    }

    // w-n pattern (n weeks ago)
    if input.starts_with("w-") {
        let n_str = &input[2..];
        let n: u32 = n_str
            .parse()
            .map_err(|_| ParseError::InvalidFormat(input.to_string()))?;
        let target_date = today - chrono::Days::new((n * 7) as u64);
        return Ok(get_current_week(target_date));
    }

    // wN pattern (Nth week of year)
    if input.len() > 1 && input[1..].chars().all(|c| c.is_ascii_digit()) {
        let week_num: u32 = input[1..]
            .parse()
            .map_err(|_| ParseError::InvalidFormat(input.to_string()))?;
        if week_num < 1 || week_num > 53 {
            return Err(ParseError::InvalidWeekNumber(week_num));
        }
        return get_week_of_year(today.year(), week_num);
    }

    Err(ParseError::InvalidFormat(input.to_string()))
}

fn parse_month_pattern(input: &str, today: NaiveDate) -> Result<DateRange, ParseError> {
    if input == "m" {
        return Ok(get_current_month(today));
    }

    // m-n pattern
    if input.starts_with("m-") {
        let n_str = &input[2..];
        let n: i32 = n_str
            .parse()
            .map_err(|_| ParseError::InvalidFormat(input.to_string()))?;

        let target_month = today.month() as i32 - n;
        let mut year = today.year();
        let mut month = target_month;

        while month < 1 {
            month += 12;
            year -= 1;
        }

        let start_date = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1).unwrap()
        };

        let start = start_date.and_hms_opt(0, 0, 0).unwrap();
        let end = end_date.and_hms_opt(0, 0, 0).unwrap();

        return Ok(DateRange { start, end });
    }

    Err(ParseError::InvalidFormat(input.to_string()))
}

fn parse_quarter_pattern(input: &str, today: NaiveDate) -> Result<DateRange, ParseError> {
    if input.len() != 2 {
        return Err(ParseError::InvalidFormat(input.to_string()));
    }

    let quarter: u32 = input[1..]
        .parse()
        .map_err(|_| ParseError::InvalidFormat(input.to_string()))?;

    if quarter < 1 || quarter > 4 {
        return Err(ParseError::InvalidQuarter(quarter));
    }

    let year = today.year();
    let start_month = (quarter - 1) * 3 + 1;
    let end_month = start_month + 3;

    let start_date = NaiveDate::from_ymd_opt(year, start_month, 1).unwrap();
    let end_date = if end_month > 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, end_month, 1).unwrap()
    };

    let start = start_date.and_hms_opt(0, 0, 0).unwrap();
    let end = end_date.and_hms_opt(0, 0, 0).unwrap();

    Ok(DateRange { start, end })
}

fn parse_weekday_pattern(input: &str, today: NaiveDate) -> Option<DateRange> {
    let (weekday_str, offset) = if input.contains('-') {
        let parts: Vec<&str> = input.split('-').collect();
        if parts.len() != 2 {
            return None;
        }
        let offset: i32 = parts[1].parse().ok()?;
        (parts[0], -offset)
    } else {
        (input, 0)
    };

    let weekday = match weekday_str {
        "mon" => Weekday::Mon,
        "tue" => Weekday::Tue,
        "wed" => Weekday::Wed,
        "thu" => Weekday::Thu,
        "fri" => Weekday::Fri,
        "sat" => Weekday::Sat,
        "sun" => Weekday::Sun,
        _ => return None,
    };

    let target_week_start = get_week_start(today) - chrono::Days::new((offset.abs() * 7) as u64);
    let days_from_monday = weekday.num_days_from_monday();
    let target_date = target_week_start + chrono::Days::new(days_from_monday as u64);

    Some(DateRange::single_day(target_date))
}

fn parse_month_name_pattern(input: &str, today: NaiveDate) -> Option<DateRange> {
    let month_num = match input {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    };

    // Year inference logic
    let year = if month_num > today.month() {
        // Month is in the future, so it must be last year
        today.year() - 1
    } else {
        // Month is in the past or current, so it's this year
        today.year()
    };

    let start_date = NaiveDate::from_ymd_opt(year, month_num, 1)?;
    let end_date = if month_num == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(year, month_num + 1, 1)?
    };

    let start = start_date.and_hms_opt(0, 0, 0)?;
    let end = end_date.and_hms_opt(0, 0, 0)?;

    Some(DateRange { start, end })
}

fn get_current_week(date: NaiveDate) -> DateRange {
    let start_date = get_week_start(date);
    let end_date = start_date + chrono::Days::new(7);
    let start = start_date.and_hms_opt(0, 0, 0).unwrap();
    let end = end_date.and_hms_opt(0, 0, 0).unwrap();
    DateRange { start, end }
}

fn get_week_start(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday();
    let days_from_monday = weekday.num_days_from_monday();
    date - chrono::Days::new(days_from_monday as u64)
}

fn get_current_month(date: NaiveDate) -> DateRange {
    let start_date = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap();
    let end_date = if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap()
    };
    let start = start_date.and_hms_opt(0, 0, 0).unwrap();
    let end = end_date.and_hms_opt(0, 0, 0).unwrap();
    DateRange { start, end }
}

fn get_week_of_year(year: i32, week: u32) -> Result<DateRange, ParseError> {
    // ISO week date: week 1 is the week with the first Thursday of the year
    let jan_4 = NaiveDate::from_ymd_opt(year, 1, 4).unwrap();
    let week1_start = get_week_start(jan_4);

    let target_week_start = week1_start + chrono::Days::new(((week - 1) * 7) as u64);
    let target_week_end = target_week_start + chrono::Days::new(7);

    let start = target_week_start.and_hms_opt(0, 0, 0).unwrap();
    let end = target_week_end.and_hms_opt(0, 0, 0).unwrap();

    Ok(DateRange { start, end })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn get_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn get_datetime_midnight(year: i32, month: u32, day: u32) -> NaiveDateTime {
        get_date(year, month, day).and_hms_opt(0, 0, 0).unwrap()
    }

    #[test]
    fn test_today() {
        let result = parse_time_duration("d").unwrap();
        let today = Local::now().date_naive();
        let start = today.and_hms_opt(0, 0, 0).unwrap();
        let end = (today + chrono::Days::new(1)).and_hms_opt(0, 0, 0).unwrap();
        assert_eq!(result.start, start);
        assert_eq!(result.end, end);
    }

    #[test]
    fn test_yesterday() {
        let result = parse_time_duration("d-1").unwrap();
        let yesterday = Local::now().date_naive() - chrono::Days::new(1);
        let start = yesterday.and_hms_opt(0, 0, 0).unwrap();
        let end = (yesterday + chrono::Days::new(1))
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert_eq!(result.start, start);
        assert_eq!(result.end, end);
    }

    #[test]
    fn test_days_ago() {
        let result = parse_time_duration("d-5").unwrap();
        let five_days_ago = Local::now().date_naive() - chrono::Days::new(5);
        let start = five_days_ago.and_hms_opt(0, 0, 0).unwrap();
        let end = (five_days_ago + chrono::Days::new(1))
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert_eq!(result.start, start);
        assert_eq!(result.end, end);
    }

    #[test]
    fn test_current_week() {
        let result = parse_time_duration("w").unwrap();
        let today = Local::now().date_naive();
        let week_start = get_week_start(today);
        let start = week_start.and_hms_opt(0, 0, 0).unwrap();
        let end = (week_start + chrono::Days::new(7))
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert_eq!(result.start, start);
        assert_eq!(result.end, end);
    }

    #[test]
    fn test_weeks_ago() {
        let result = parse_time_duration("w-2").unwrap();
        let today = Local::now().date_naive();
        let two_weeks_ago = today - chrono::Days::new(14);
        let week_start = get_week_start(two_weeks_ago);
        let start = week_start.and_hms_opt(0, 0, 0).unwrap();
        let end = (week_start + chrono::Days::new(7))
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert_eq!(result.start, start);
        assert_eq!(result.end, end);
    }

    #[test]
    fn test_week_number() {
        let result = parse_time_duration("w1").unwrap();
        // Week 1 should be in early January
        assert!(result.start.month() == 1 || result.start.month() == 12);
    }

    #[test]
    fn test_invalid_week_number() {
        assert!(parse_time_duration("w0").is_err());
        assert!(parse_time_duration("w54").is_err());
    }

    #[test]
    fn test_current_month() {
        let result = parse_time_duration("m").unwrap();
        let today = Local::now().date_naive();
        assert_eq!(result.start.year(), today.year());
        assert_eq!(result.start.month(), today.month());
        assert_eq!(result.start.day(), 1);
    }

    #[test]
    fn test_last_month() {
        let result = parse_time_duration("m-1").unwrap();
        let today = Local::now().date_naive();

        let expected_month = if today.month() == 1 {
            12
        } else {
            today.month() - 1
        };
        let expected_year = if today.month() == 1 {
            today.year() - 1
        } else {
            today.year()
        };

        assert_eq!(result.start.year(), expected_year);
        assert_eq!(result.start.month(), expected_month);
        assert_eq!(result.start.day(), 1);
    }

    #[test]
    fn test_months_ago() {
        let result = parse_time_duration("m-3").unwrap();
        // Just check that it's a valid month
        assert!(result.start.month() >= 1 && result.start.month() <= 12);
        assert_eq!(result.start.day(), 1);
    }

    #[test]
    fn test_quarter() {
        let result = parse_time_duration("q1").unwrap();
        let today = Local::now().date_naive();
        assert_eq!(result.start, get_datetime_midnight(today.year(), 1, 1));
        assert_eq!(result.end, get_datetime_midnight(today.year(), 4, 1));

        let result = parse_time_duration("q2").unwrap();
        assert_eq!(result.start, get_datetime_midnight(today.year(), 4, 1));
        assert_eq!(result.end, get_datetime_midnight(today.year(), 7, 1));

        let result = parse_time_duration("q3").unwrap();
        assert_eq!(result.start, get_datetime_midnight(today.year(), 7, 1));
        assert_eq!(result.end, get_datetime_midnight(today.year(), 10, 1));

        let result = parse_time_duration("q4").unwrap();
        assert_eq!(result.start, get_datetime_midnight(today.year(), 10, 1));
        assert_eq!(result.end, get_datetime_midnight(today.year() + 1, 1, 1));
    }

    #[test]
    fn test_invalid_quarter() {
        assert!(parse_time_duration("q0").is_err());
        assert!(parse_time_duration("q5").is_err());
    }

    #[test]
    fn test_weekday_current_week() {
        let result = parse_time_duration("mon").unwrap();
        assert_eq!(result.start.weekday(), Weekday::Mon);

        let result = parse_time_duration("fri").unwrap();
        assert_eq!(result.start.weekday(), Weekday::Fri);
    }

    #[test]
    fn test_weekday_last_week() {
        let result = parse_time_duration("mon-1").unwrap();
        assert_eq!(result.start.weekday(), Weekday::Mon);
        let today = Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        assert!(result.start < today);

        let result = parse_time_duration("fri-1").unwrap();
        assert_eq!(result.start.weekday(), Weekday::Fri);
    }

    #[test]
    fn test_month_names() {
        // Note: These tests depend on the current date
        // The year inference logic makes it tricky to test without mocking
        let result = parse_time_duration("jan");
        assert!(result.is_ok());
        let range = result.unwrap();
        assert_eq!(range.start.month(), 1);
        assert_eq!(range.start.day(), 1);

        let result = parse_time_duration("dec");
        assert!(result.is_ok());
        let range = result.unwrap();
        assert_eq!(range.start.month(), 12);
        assert_eq!(range.start.day(), 1);
    }

    #[test]
    fn test_absolute_date() {
        let result = parse_time_duration("2026-02-05").unwrap();
        assert_eq!(result.start, get_datetime_midnight(2026, 2, 5));
        assert_eq!(result.end, get_datetime_midnight(2026, 2, 6));
    }

    #[test]
    fn test_absolute_month() {
        let result = parse_time_duration("2025-07").unwrap();
        assert_eq!(result.start, get_datetime_midnight(2025, 7, 1));
        assert_eq!(result.end, get_datetime_midnight(2025, 8, 1));

        let result = parse_time_duration("2025-12").unwrap();
        assert_eq!(result.start, get_datetime_midnight(2025, 12, 1));
        assert_eq!(result.end, get_datetime_midnight(2026, 1, 1));
    }

    #[test]
    fn test_absolute_range_exclusive() {
        let result = parse_time_duration("2025-07-01..2025-07-20").unwrap();
        assert_eq!(result.start, get_datetime_midnight(2025, 7, 1));
        assert_eq!(result.end, get_datetime_midnight(2025, 7, 20));
    }

    #[test]
    fn test_absolute_range_inclusive() {
        let result = parse_time_duration("2025-07-01..=2025-07-20").unwrap();
        assert_eq!(result.start, get_datetime_midnight(2025, 7, 1));
        assert_eq!(result.end, get_datetime_midnight(2025, 7, 21));
    }

    #[test]
    fn test_range_short_forms() {
        let result = parse_time_duration("mon..wed");
        assert!(result.is_ok());
        let range = result.unwrap();
        assert_eq!(range.start.weekday(), Weekday::Mon);
        assert_eq!(range.end.weekday(), Weekday::Wed);

        let result = parse_time_duration("mon..=wed");
        assert!(result.is_ok());
        let range = result.unwrap();
        assert_eq!(range.start.weekday(), Weekday::Mon);
        // Inclusive, so end should be day after Wednesday
        assert_eq!(range.end.weekday(), Weekday::Thu);
    }

    #[test]
    fn test_range_spanning_years() {
        let result = parse_time_duration("2025-12-20..2026-01-10").unwrap();
        assert_eq!(result.start, get_datetime_midnight(2025, 12, 20));
        assert_eq!(result.end, get_datetime_midnight(2026, 1, 10));
    }

    #[test]
    fn test_invalid_range() {
        // Start after end
        let result = parse_time_duration("2025-07-20..2025-07-01");
        assert!(result.is_err());
        if let Err(ParseError::InvalidRange { .. }) = result {
            // Expected error type
        } else {
            panic!("Expected InvalidRange error");
        }
    }

    #[test]
    fn test_invalid_formats() {
        assert!(parse_time_duration("xyz").is_err());
        assert!(parse_time_duration("d-").is_err());
        assert!(parse_time_duration("w-").is_err());
        assert!(parse_time_duration("").is_err());
    }

    #[test]
    fn test_date_range_equality() {
        let range1 = DateRange {
            start: get_datetime_midnight(2025, 1, 1),
            end: get_datetime_midnight(2025, 1, 2),
        };
        let range2 = DateRange {
            start: get_datetime_midnight(2025, 1, 1),
            end: get_datetime_midnight(2025, 1, 2),
        };
        assert_eq!(range1, range2);
    }

    #[test]
    fn test_single_day_range() {
        let date = get_date(2025, 6, 15);
        let range = DateRange::single_day(date);
        assert_eq!(range.start, get_datetime_midnight(2025, 6, 15));
        assert_eq!(range.end, get_datetime_midnight(2025, 6, 16));
    }

    #[test]
    fn test_duration() {
        let range = DateRange::new(
            get_date(2025, 1, 15).and_hms_opt(12, 10, 0).unwrap(),
            get_date(2025, 1, 15).and_hms_opt(13, 45, 0).unwrap(),
        )
        .unwrap();
        let duration = range.duration();
        assert_eq!("01:35", duration.to_string());

        // Single day range
        let range = DateRange::single_day(get_date(2025, 1, 15));
        let duration = range.duration();
        assert_eq!("24:00", duration.to_string());

        // Multi-day range
        let range = DateRange {
            start: get_date(2025, 1, 15).and_hms_opt(12, 10, 0).unwrap(),
            end: get_date(2025, 1, 18).and_hms_opt(13, 20, 0).unwrap(), // 3 days
        };
        let duration = range.duration();
        assert_eq!("73:10", duration.to_string());
    }

    #[test]
    fn test_partition_by_day_across_day() {
        let range = DateRange::new(
            get_date(2025, 1, 15).and_hms_opt(22, 10, 0).unwrap(),
            get_date(2025, 1, 16).and_hms_opt(02, 45, 0).unwrap(),
        )
        .unwrap();

        let ranges = range.partition_by_day();
        assert_eq!(2, ranges.len());

        let r1 = &ranges[0];
        assert_eq!(
            get_date(2025, 1, 15).and_hms_opt(22, 10, 0).unwrap(),
            r1.start
        );
        assert_eq!(get_date(2025, 1, 16).and_hms_opt(0, 0, 0).unwrap(), r1.end);

        let r2 = &ranges[1];
        assert_eq!(
            get_date(2025, 1, 16).and_hms_opt(0, 0, 0).unwrap(),
            r2.start
        );
        assert_eq!(get_date(2025, 1, 16).and_hms_opt(2, 45, 0).unwrap(), r2.end);
    }

    #[test]
    fn test_partition_by_day_within_day() {
        let range = DateRange::new(
            get_date(2025, 1, 15).and_hms_opt(19, 10, 0).unwrap(),
            get_date(2025, 1, 15).and_hms_opt(22, 07, 0).unwrap(),
        )
        .unwrap();

        let ranges = range.partition_by_day();
        assert_eq!(1, ranges.len());

        let r1 = &ranges[0];
        assert_eq!(
            get_date(2025, 1, 15).and_hms_opt(19, 10, 0).unwrap(),
            r1.start
        );
        assert_eq!(
            get_date(2025, 1, 15).and_hms_opt(22, 07, 0).unwrap(),
            r1.end
        );
    }

    #[test]
    fn test_partition_by_day_across_multiple_days() {
        let range = DateRange::new(
            get_date(2025, 1, 15).and_hms_opt(19, 10, 0).unwrap(),
            get_date(2025, 1, 17).and_hms_opt(04, 07, 0).unwrap(),
        )
        .unwrap();

        let ranges = range.partition_by_day();
        assert_eq!(3, ranges.len());

        let expected = vec![
            ((2025, 1, 15, 19, 10, 0), (2025, 1, 16, 0, 0, 0)),
            ((2025, 1, 16, 0, 0, 0), (2025, 1, 17, 0, 0, 0)),
            ((2025, 1, 17, 0, 0, 0), (2025, 1, 17, 4, 7, 0)),
        ];

        for (i, exp) in expected.into_iter().enumerate() {
            let range = &ranges[i];
            let (s, e) = exp;
            assert_eq!(
                get_date(s.0, s.1, s.2).and_hms_opt(s.3, s.4, s.5).unwrap(),
                range.start
            );
            assert_eq!(
                get_date(e.0, e.1, e.2).and_hms_opt(e.3, e.4, e.5).unwrap(),
                range.end
            );
        }
    }
}
