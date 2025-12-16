use chrono::{DateTime, Datelike, Duration, NaiveDate, Weekday};

/// Calculate business days between two dates (excluding weekends)
pub fn business_days_between(start: &str, end: &str) -> i64 {
    let start_date = parse_date(start);
    let end_date = parse_date(end);
    
    match (start_date, end_date) {
        (Some(start), Some(end)) => count_business_days(start, end),
        _ => 0,
    }
}

/// Calculate business hours between two timestamps
/// Assumes 8-hour workday, 9 AM - 5 PM
pub fn business_hours_between(start: &str, end: &str) -> f64 {
    let days = business_days_between(start, end);
    days as f64 * 8.0
}

fn parse_date(date_str: &str) -> Option<NaiveDate> {
    // Try ISO 8601 format first
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return Some(dt.date_naive());
    }
    
    // Try simple date format
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Some(date);
    }
    
    None
}

fn count_business_days(start: NaiveDate, end: NaiveDate) -> i64 {
    if end <= start {
        return 0;
    }
    
    let mut count = 0;
    let mut current = start;
    
    while current < end {
        if is_business_day(current) {
            count += 1;
        }
        current += Duration::days(1);
    }
    
    count
}

fn is_business_day(date: NaiveDate) -> bool {
    matches!(
        date.weekday(),
        Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weekday_only() {
        // Monday to Friday = 4 business days
        let days = business_days_between("2024-02-19", "2024-02-23");
        assert_eq!(days, 4);
    }

    #[test]
    fn test_includes_weekend() {
        // Friday to Monday = 1 business day (Friday only)
        let days = business_days_between("2024-02-23", "2024-02-26");
        assert_eq!(days, 1);
    }

    #[test]
    fn test_full_week() {
        // Monday to next Monday = 5 business days
        let days = business_days_between("2024-02-19", "2024-02-26");
        assert_eq!(days, 5);
    }

    #[test]
    fn test_same_day() {
        let days = business_days_between("2024-02-19", "2024-02-19");
        assert_eq!(days, 0);
    }

    #[test]
    fn test_weekend_start() {
        // Saturday to Tuesday = 2 business days (Mon, Tue)
        let days = business_days_between("2024-02-24", "2024-02-27");
        assert_eq!(days, 2);
    }

    #[test]
    fn test_iso8601_format() {
        let days = business_days_between(
            "2024-02-19T10:00:00Z",
            "2024-02-23T18:00:00Z",
        );
        assert_eq!(days, 4);
    }

    #[test]
    fn test_year_boundary() {
        // Dec 29 (Fri) to Jan 2 (Tue) = 2 business days (Dec 29 and Jan 2)
        // Dec 30 = Sat, Dec 31 = Sun, Jan 1 = Mon (but we count until, not including end)
        let days = business_days_between("2023-12-29", "2024-01-02");
        assert_eq!(days, 2); // Dec 29 (Fri) and Jan 1 (Mon)
    }
}
