use std::collections::HashMap;

use chrono::{Datelike, Days, Duration, NaiveDate};

use crate::time_duration::DateTimeRange;

/// Maps total seconds for a day to a display character.
///
/// Thresholds: 0 = `.`, <1h = `░`, 1–3h = `▒`, 3–6h = `▓`, ≥6h = `█`
fn intensity_char(seconds: i64) -> char {
    match seconds {
        0 => '.',
        s if s < 3600 => '░',
        s if s < 3 * 3600 => '▒',
        s if s < 6 * 3600 => '▓',
        _ => '█',
    }
}

/// Returns the Monday of the ISO week that contains `date`.
fn week_monday(date: NaiveDate) -> NaiveDate {
    date - Days::new(date.weekday().num_days_from_monday() as u64)
}

/// Prints a GitHub-style calendar heatmap to stdout.
///
/// Weeks run left-to-right (x-axis); days of the week run top-to-bottom
/// (y-axis, Mon–Sun). Intensity is determined by total clocked seconds per
/// day. Days outside `range` are shown as spaces.
pub fn print_heatmap(entries: &[DateTimeRange], range: &DateTimeRange) {
    // Aggregate seconds per day for entries that fall within range.
    let mut day_totals: HashMap<NaiveDate, i64> = HashMap::new();
    for entry in entries {
        if entry.is_between(range) {
            *day_totals.entry(entry.start.date()).or_insert(0) += entry.duration().seconds();
        }
    }

    let start_date = range.start.date();
    // range.end is exclusive; subtract 1ns to find the last inclusive date.
    let end_date = (range.end - Duration::nanoseconds(1)).date();

    let first_week = week_monday(start_date);
    let last_week = week_monday(end_date);

    let mut weeks: Vec<NaiveDate> = Vec::new();
    let mut w = first_week;
    loop {
        weeks.push(w);
        if w >= last_week {
            break;
        }
        w = w + Days::new(7);
    }

    let n_weeks = weeks.len();
    // Each cell is 2 chars wide (unicode char + space).
    // Day label prefix is 4 chars wide ("Mon ").
    let cell_w = 2;
    let label_w = 4;

    // Month label row: place each 3-char month name at the column where it
    // first appears, overwriting spaces.
    let row_len = label_w + n_weeks * cell_w;
    let mut month_row: Vec<char> = vec![' '; row_len];
    let mut last_month = 0u32;
    for (i, week) in weeks.iter().enumerate() {
        if week.month() != last_month {
            last_month = week.month();
            let label = week.format("%b").to_string();
            let pos = label_w + i * cell_w;
            for (j, c) in label.chars().enumerate() {
                if pos + j < row_len {
                    month_row[pos + j] = c;
                }
            }
        }
    }
    println!("{}", month_row.iter().collect::<String>());

    // One row per day of the week (Mon=0 .. Sun=6).
    let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    for (day_idx, day_name) in day_names.iter().enumerate() {
        print!("{day_name:<label_w$}");
        for week in &weeks {
            let date = *week + Days::new(day_idx as u64);
            let ch = if date >= start_date && date <= end_date {
                intensity_char(*day_totals.get(&date).unwrap_or(&0))
            } else {
                ' '
            };
            print!("{ch} ");
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_week_monday() {
        // Wednesday 2026-03-04 → Monday 2026-03-02
        assert_eq!(week_monday(date(2026, 3, 4)), date(2026, 3, 2));
        // Monday is its own week start
        assert_eq!(week_monday(date(2026, 3, 2)), date(2026, 3, 2));
        // Sunday 2026-03-08 → Monday 2026-03-02
        assert_eq!(week_monday(date(2026, 3, 8)), date(2026, 3, 2));
    }

    #[test]
    fn test_intensity_char() {
        assert_eq!(intensity_char(0), '.');
        assert_eq!(intensity_char(1800), '░'); // 30 min
        assert_eq!(intensity_char(3600), '▒'); // 1h
        assert_eq!(intensity_char(7200), '▒'); // 2h
        assert_eq!(intensity_char(3 * 3600), '▓'); // 3h
        assert_eq!(intensity_char(6 * 3600), '█'); // 6h
    }
}
