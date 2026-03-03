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
        dt - Duration::days(diff_days as i64)
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
