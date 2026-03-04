use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Weekday};

/// Returns which quarter of the year the given date falls into
pub fn quarter_of(dt: NaiveDateTime) -> (i32, u32) {
    let quarter = (dt.month() - 1) / 3 + 1;
    (dt.year(), quarter)
}

pub fn start_of_this_week(dt: NaiveDateTime) -> NaiveDateTime {
    // Find Monday before (or on) the current date
    let weekday = dt.weekday();
    if let Weekday::Mon = weekday {
        dt.date().and_time(NaiveTime::MIN)
    } else {
        let diff_days = weekday.num_days_from_monday();
        (dt - Duration::days(diff_days as i64))
            .date()
            .and_time(NaiveTime::MIN)
    }
}

pub fn start_of_this_month(dt: NaiveDateTime) -> NaiveDateTime {
    let year = dt.year();
    let month = dt.month();
    NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap()
        .and_time(NaiveTime::MIN)
}

pub fn start_of_next_month(dt: NaiveDateTime) -> NaiveDateTime {
    let this_month = dt.month();
    let (month, year) = if this_month == 12 {
        (1, dt.year() + 1)
    } else {
        (this_month + 1, dt.year())
    };
    NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap()
        .and_time(NaiveTime::MIN)
}

pub fn start_of_this_quarter(dt: NaiveDateTime) -> NaiveDateTime {
    let (_, q) = quarter_of(dt);
    let month = match q {
        1 => 1,
        2 => 4,
        3 => 7,
        4 => 10,
        _ => panic!("Invalid quarter of the year: {q}"),
    };
    let year = dt.year();
    NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap()
        .and_time(NaiveTime::MIN)
}

pub fn start_of_next_quarter(dt: NaiveDateTime) -> NaiveDateTime {
    let (_, this_quarter) = quarter_of(dt);
    let (month, year) = match this_quarter {
        1 => (4, dt.year()),
        2 => (7, dt.year()),
        3 => (10, dt.year()),
        4 => (1, dt.year() + 1),
        _ => panic!("Invalid quarter of the year: {this_quarter}"),
    };
    NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap()
        .and_time(NaiveTime::MIN)
}

pub fn start_of_this_year(dt: NaiveDateTime) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(dt.year(), 1, 1)
        .unwrap()
        .and_time(NaiveTime::MIN)
}

pub fn start_of_next_year(dt: NaiveDateTime) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(dt.year() + 1, 1, 1)
        .unwrap()
        .and_time(NaiveTime::MIN)
}

pub fn secs_to_rounded_hours_mins(secs: i64) -> (i64, i64) {
    let rounded_minutes = (secs + 30) / 60;
    let hours = rounded_minutes / 60;
    let minutes = rounded_minutes % 60;
    (hours, minutes)
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

    #[test]
    fn test_quarter_of() {
        // Q1: months 1–3
        assert_eq!((2026, 1), quarter_of(dt(2026, 1, 15, 0, 0)));
        assert_eq!((2026, 1), quarter_of(dt(2026, 3, 31, 0, 0)));
        // Q2: months 4–6
        assert_eq!((2026, 2), quarter_of(dt(2026, 4, 1, 0, 0)));
        // Q3: months 7–9
        assert_eq!((2026, 3), quarter_of(dt(2026, 8, 10, 0, 0)));
        // Q4: months 10–12
        assert_eq!((2025, 4), quarter_of(dt(2025, 12, 31, 0, 0)));
        // Year is preserved
        assert_eq!((2020, 3), quarter_of(dt(2020, 7, 1, 0, 0)));
    }

    #[test]
    fn test_start_of_this_week() {
        // Already Monday — time is zeroed
        assert_eq!(dt(2026, 3, 2, 0, 0), start_of_this_week(dt(2026, 3, 2, 14, 30)));
        // Mid-week (Wednesday)
        assert_eq!(dt(2026, 3, 2, 0, 0), start_of_this_week(dt(2026, 3, 4, 9, 15)));
        // Sunday (last day of ISO week)
        assert_eq!(dt(2026, 3, 2, 0, 0), start_of_this_week(dt(2026, 3, 8, 23, 59)));
    }

    #[test]
    fn test_start_of_this_month() {
        // Mid-month
        assert_eq!(dt(2026, 3, 1, 0, 0), start_of_this_month(dt(2026, 3, 15, 10, 0)));
        // Already the 1st — no change
        assert_eq!(dt(2026, 3, 1, 0, 0), start_of_this_month(dt(2026, 3, 1, 5, 0)));
        // December
        assert_eq!(dt(2025, 12, 1, 0, 0), start_of_this_month(dt(2025, 12, 25, 8, 0)));
    }

    #[test]
    fn test_start_of_next_month() {
        // Normal case
        assert_eq!(dt(2026, 4, 1, 0, 0), start_of_next_month(dt(2026, 3, 15, 0, 0)));
        // December wraps to January of next year
        assert_eq!(dt(2026, 1, 1, 0, 0), start_of_next_month(dt(2025, 12, 10, 0, 0)));
    }

    #[test]
    fn test_start_of_this_quarter() {
        // Q1 → Jan 1
        assert_eq!(dt(2026, 1, 1, 0, 0), start_of_this_quarter(dt(2026, 2, 15, 0, 0)));
        // Q2 → Apr 1
        assert_eq!(dt(2026, 4, 1, 0, 0), start_of_this_quarter(dt(2026, 5, 1, 0, 0)));
        // Q3 → Jul 1
        assert_eq!(dt(2026, 7, 1, 0, 0), start_of_this_quarter(dt(2026, 9, 30, 0, 0)));
        // Q4 → Oct 1
        assert_eq!(dt(2025, 10, 1, 0, 0), start_of_this_quarter(dt(2025, 11, 1, 0, 0)));
    }

    #[test]
    fn test_start_of_next_quarter() {
        // Q1 → Q2
        assert_eq!(dt(2026, 4, 1, 0, 0), start_of_next_quarter(dt(2026, 2, 15, 0, 0)));
        // Q2 → Q3
        assert_eq!(dt(2026, 7, 1, 0, 0), start_of_next_quarter(dt(2026, 5, 1, 0, 0)));
        // Q3 → Q4
        assert_eq!(dt(2026, 10, 1, 0, 0), start_of_next_quarter(dt(2026, 8, 20, 0, 0)));
        // Q4 → Q1 of next year
        assert_eq!(dt(2026, 1, 1, 0, 0), start_of_next_quarter(dt(2025, 11, 1, 0, 0)));
    }

    #[test]
    fn test_start_of_this_year() {
        // Mid-year
        assert_eq!(dt(2026, 1, 1, 0, 0), start_of_this_year(dt(2026, 6, 15, 0, 0)));
        // Already Jan 1 — no change
        assert_eq!(dt(2026, 1, 1, 0, 0), start_of_this_year(dt(2026, 1, 1, 0, 0)));
    }

    #[test]
    fn test_start_of_next_year() {
        // Mid-year
        assert_eq!(dt(2027, 1, 1, 0, 0), start_of_next_year(dt(2026, 6, 15, 0, 0)));
        // Dec 31 still advances to Jan 1 of next year
        assert_eq!(dt(2026, 1, 1, 0, 0), start_of_next_year(dt(2025, 12, 31, 0, 0)));
    }

    #[test]
    fn test_secs_to_rounded_hours_mins() {
        // Exact values
        assert_eq!((1, 0), secs_to_rounded_hours_mins(3600));
        assert_eq!((1, 1), secs_to_rounded_hours_mins(3660));
        // 29s remainder rounds down
        assert_eq!((1, 0), secs_to_rounded_hours_mins(3629));
        // 30s remainder rounds up (boundary)
        assert_eq!((1, 1), secs_to_rounded_hours_mins(3630));
        // Zero
        assert_eq!((0, 0), secs_to_rounded_hours_mins(0));
        // Sub-minute: 29s rounds down, 30s rounds up
        assert_eq!((0, 0), secs_to_rounded_hours_mins(29));
        assert_eq!((0, 1), secs_to_rounded_hours_mins(30));
        // Large value: 7384s = 123m 4s → rounds to 123m = 2h 3m
        assert_eq!((2, 3), secs_to_rounded_hours_mins(7384));
    }
}
