/* -----------------------------------------------------------------------------
 * util/date.rs
 * Week bounds, date formatting, and UTC timestamp generation.
 * -------------------------------------------------------------------------- */

/* --- Week Bounds ---------------------------------------------------------- */

pub fn get_week_bounds(offset_weeks: i64) -> (String, String) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO)
        .as_secs() as i64;

    // 1970-01-01 is a Thursday; +4 shifts so that Monday ≡ 0, Tuesday ≡ 1, etc.
    let days_since_epoch = now / 86400;
    let day_of_week = ((days_since_epoch + 4) % 7) as i64;

    let days_to_monday = (7 - day_of_week + 4) % 7;
    let monday_epoch = days_since_epoch - days_to_monday + (offset_weeks * 7);
    let from_date = epoch_to_ymd(monday_epoch);
    let to_date = epoch_to_ymd(monday_epoch + 4);

    (from_date, to_date)
}

/* --- Epoch to YMD --------------------------------------------------------- */

fn epoch_to_ymd(epoch_days: i64) -> String {
    let mut year: i64 = 1970;
    let mut remaining = epoch_days;

    loop {
        let is_leap_year = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if is_leap_year { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let mut month: i64 = 1;
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let days_in_months = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    for days in days_in_months.iter() {
        if remaining < *days as i64 {
            break;
        }
        remaining -= *days as i64;
        month += 1;
    }

    let day = remaining + 1;
    format!("{:04}-{:02}-{:02}", year, month, day)
}

/* --- Week Label ----------------------------------------------------------- */

pub fn format_week_label(offset_weeks: i64) -> String {
    let (from, to) = get_week_bounds(offset_weeks);
    let from_parts: Vec<&str> = from.split('-').collect();
    let to_parts: Vec<&str> = to.split('-').collect();

    let from_day: u32 = from_parts.last().unwrap_or(&"1").parse().unwrap_or(1);
    let to_day: u32 = to_parts.last().unwrap_or(&"1").parse().unwrap_or(1);
    let to_month: u32 = to_parts.get(1).unwrap_or(&"01").parse().unwrap_or(1);
    let year: u32 = from_parts.get(0).unwrap_or(&"2025").parse().unwrap_or(2025);

    let month_name = match to_month {
        1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
        5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
        9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
        _ => "?"
    };

    format!("({}-{} {} {})", from_day, to_day, month_name, year)
}

/* --- Timestamp ------------------------------------------------------------ */

pub fn utc_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO);

    let secs = now.as_secs();
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    let date_str = epoch_to_ymd(days as i64);
    let parts: Vec<&str> = date_str.split('-').collect();
    let year = parts.get(0).unwrap_or(&"1970").parse().unwrap_or(1970);
    let month = parts.get(1).unwrap_or(&"01").parse().unwrap_or(1);
    let day = parts.get(2).unwrap_or(&"01").parse().unwrap_or(1);

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hours, minutes, seconds)
}
