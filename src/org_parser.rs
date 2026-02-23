use crate::time_duration::DateTimeRange;
use chrono::NaiveDateTime;

/// Parses org-mode CLOCK entries from a line buffer and returns DateRange instances.
/// Each line is checked for CLOCK entries in the format:
/// CLOCK: [2025-01-23 Thu 12:28]--[2025-01-23 Thu 13:17] =>  0:49
/// If a clock entry spans multiple days, it creates separate DateRange instances for each day.
pub fn parse_org_clock_entries<'a>(lines: impl IntoIterator<Item = &'a str>) -> Vec<DateTimeRange> {
    let mut entries = Vec::new();

    for line in lines {
        entries.extend(parse_clock_entry(line));
    }

    entries
}

/// Parses a single CLOCK entry line and extracts the time range as a DateRange.
/// Returns empty Vec if the line is not a valid CLOCK entry.
fn parse_clock_entry(line: &str) -> Vec<DateTimeRange> {
    // Format: CLOCK: [2025-01-23 Thu 12:28]--[2025-01-23 Thu 13:17] =>  0:49
    if !line.contains("CLOCK:") {
        return Vec::new();
    }

    // Find both timestamps
    let first_start = match line.find('[') {
        Some(pos) => pos,
        None => return Vec::new(),
    };
    let first_end = match line[first_start..].find(']') {
        Some(pos) => pos,
        None => return Vec::new(),
    };
    let start_timestamp = &line[first_start + 1..first_start + first_end];

    // Find the second timestamp (after the --)
    let remaining = &line[first_start + first_end + 1..];
    let second_start = match remaining.find('[') {
        Some(pos) => pos,
        None => return Vec::new(),
    };
    let second_end = match remaining[second_start..].find(']') {
        Some(pos) => pos,
        None => return Vec::new(),
    };
    let end_timestamp = &remaining[second_start + 1..second_start + second_end];

    // Parse full timestamps "2025-01-23 Thu 12:28"
    let start_dt = match parse_org_timestamp(start_timestamp) {
        Some(dt) => dt,
        None => return Vec::new(),
    };
    let end_dt = match parse_org_timestamp(end_timestamp) {
        Some(dt) => dt,
        None => return Vec::new(),
    };

    // Create a single DateRange for the clock entry
    match DateTimeRange::new(start_dt, end_dt) {
        Ok(range) => range.partition_by_day(),
        Err(_) => Vec::new(),
    }
}

/// Parses an org-mode timestamp like "2025-01-23 Thu 12:28" into NaiveDateTime
fn parse_org_timestamp(timestamp: &str) -> Option<NaiveDateTime> {
    // Format: "2025-01-23 Thu 12:28"
    let parts: Vec<&str> = timestamp.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let date_str = parts[0];
    let time_str = parts[2];

    // Combine date and time: "2025-01-23 12:28"
    let datetime_str = format!("{} {}", date_str, time_str);
    NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M").ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn get_datetime(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> NaiveDateTime {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        date.and_hms_opt(hour, min, sec).unwrap()
    }

    #[test]
    fn test_parse_single_day_clock_entry() {
        let line = "   CLOCK: [2025-01-23 Thu 12:28]--[2025-01-23 Thu 13:17] =>  0:49";
        let result = parse_clock_entry(line);

        assert_eq!(1, result.len());
        assert_eq!(get_datetime(2025, 1, 23, 12, 28, 0), result[0].start);
        assert_eq!(get_datetime(2025, 1, 23, 13, 17, 0), result[0].end);

        // Check duration - should be 00:49 (49 minutes)
        let duration = result[0].duration();
        assert_eq!("00:49", duration.to_string());
    }

    #[test]
    fn test_parse_multi_day_clock_entry() {
        let line = "   CLOCK: [2025-01-23 Thu 23:30]--[2025-01-24 Fri 01:15] =>  1:45";
        let result = parse_clock_entry(line);

        assert_eq!(2, result.len());
        assert_eq!(get_datetime(2025, 1, 23, 23, 30, 0), result[0].start);
        assert_eq!(get_datetime(2025, 1, 24, 0, 0, 0), result[0].end);
        assert_eq!("00:30", result[0].duration().to_string());

        assert_eq!(get_datetime(2025, 1, 24, 0, 0, 0), result[1].start);
        assert_eq!(get_datetime(2025, 1, 24, 1, 15, 0), result[1].end);
        assert_eq!("01:15", result[1].duration().to_string());
    }

    #[test]
    fn test_parse_non_clock_line() {
        let line = "   - State \"DONE\"       from \"TODO\"       [2025-01-23 Thu 12:28]";
        let result = parse_clock_entry(line);

        assert_eq!(0, result.len());
    }

    #[test]
    fn test_parse_multiple_entries() {
        let lines = vec![
            ":LOGBOOK:",
            "CLOCK: [2025-11-28 Fri 09:32]--[2025-11-28 Fri 10:27] =>  0:55",
            "CLOCK: [2025-11-27 Thu 22:08]--[2025-11-27 Thu 23:57] =>  1:49",
            "CLOCK: [2025-11-26 Wed 23:25]--[2025-11-27 Thu 00:42] =>  1:17",
            ":END:",
        ];

        let results = parse_org_clock_entries(lines);

        assert_eq!(4, results.len());

        // First entry: 09:32 to 10:27 = 55 minutes
        assert_eq!(get_datetime(2025, 11, 28, 9, 32, 0), results[0].start);
        assert_eq!(get_datetime(2025, 11, 28, 10, 27, 0), results[0].end);
        assert_eq!("00:55", results[0].duration().to_string());

        // Second entry: 22:08 to 23:57 = 1 hour 49 minutes
        assert_eq!(get_datetime(2025, 11, 27, 22, 8, 0), results[1].start);
        assert_eq!(get_datetime(2025, 11, 27, 23, 57, 0), results[1].end);
        assert_eq!("01:49", results[1].duration().to_string());

        // Third entry in the org file spans midnight: 23:25 to 00:42
        // = 1 hour 17 minutes but since it's across a day, it gets
        // split

        // Third entry spans midnight: 23:25 to 23:59 = 35 minutes
        assert_eq!(get_datetime(2025, 11, 26, 23, 25, 0), results[2].start);
        assert_eq!(get_datetime(2025, 11, 27, 0, 0, 0), results[2].end);
        assert_eq!("00:35", results[2].duration().to_string());

        // Fourth entry spans 00:00 to 00:42 = 42 minutes
        assert_eq!(get_datetime(2025, 11, 27, 0, 0, 0), results[3].start);
        assert_eq!(get_datetime(2025, 11, 27, 0, 42, 0), results[3].end);
        assert_eq!("00:42", results[3].duration().to_string());
    }

    #[test]
    fn test_parse_empty_lines() {
        let lines: Vec<&str> = vec![];
        let results = parse_org_clock_entries(lines);

        assert_eq!(0, results.len());
    }
}
