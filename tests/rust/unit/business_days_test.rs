//! Unit tests for business days calculations
//!
//! These tests ensure correct handling of weekends, holidays, and date boundaries.

use chrono::NaiveDate;

// Import the business_days module from the main crate
// Tests will run against the actual implementation in src-tauri/src/metrics/business_days.rs

#[cfg(test)]
mod business_days_tests {
    use super::*;

    // Helper to call the business_days_between function from the main crate
    // This will be linked when running `cargo test` from src-tauri directory

    #[test]
    fn test_same_day() {
        // Start and end on same day = 0 business days
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-19",
            "2024-02-19"
        );
        assert_eq!(days, 0, "Same day should return 0 business days");
    }

    #[test]
    fn test_one_business_day() {
        // Monday to Tuesday = 1 business day
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-19",  // Monday
            "2024-02-20"   // Tuesday
        );
        assert_eq!(days, 1, "Monday to Tuesday should be 1 business day");
    }

    #[test]
    fn test_full_week() {
        // Monday to next Monday = 5 business days
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-19",  // Monday
            "2024-02-26"   // Next Monday
        );
        assert_eq!(days, 5, "Monday to next Monday should be 5 business days");
    }

    #[test]
    fn test_weekend_only() {
        // Saturday to Sunday = 0 business days
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-24",  // Saturday
            "2024-02-25"   // Sunday
        );
        assert_eq!(days, 0, "Weekend should have 0 business days");
    }

    #[test]
    fn test_friday_to_monday() {
        // Friday to Monday = 1 business day (just Friday)
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-23",  // Friday
            "2024-02-26"   // Monday
        );
        assert_eq!(days, 1, "Friday to Monday should be 1 business day (Friday only)");
    }

    #[test]
    fn test_year_boundary() {
        // Dec 29, 2023 (Friday) to Jan 2, 2024 (Tuesday)
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2023-12-29",  // Friday
            "2024-01-02"   // Tuesday
        );
        // Dec 29 (Fri), Jan 1 (Mon) = 2 business days
        assert_eq!(days, 2, "Year boundary should count business days correctly");
    }

    #[test]
    fn test_month_boundary() {
        // Jan 31 to Feb 1 (both weekdays in 2024)
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-01-31",  // Wednesday
            "2024-02-01"   // Thursday
        );
        assert_eq!(days, 1, "Month boundary should count correctly");
    }

    #[test]
    fn test_leap_year() {
        // Feb 28 to Mar 1 in leap year 2024
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-28",  // Wednesday
            "2024-03-01"   // Friday
        );
        // Feb 28 (Wed), Feb 29 (Thu) = 2 business days
        assert_eq!(days, 2, "Leap year should include Feb 29");
    }

    #[test]
    fn test_long_period() {
        // 90 days should be approximately 64 business days
        // Jan 1, 2024 (Mon) to Mar 31, 2024 (Sun) = 90 days
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-01-01",
            "2024-03-31"
        );
        // Approximately: 90 days - 26 weekend days = 64 business days
        assert!(days >= 62 && days <= 66, "90-day period should have ~64 business days, got {}", days);
    }

    #[test]
    fn test_negative_range() {
        // End before start = 0
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-20",
            "2024-02-19"
        );
        assert_eq!(days, 0, "Negative range should return 0");
    }

    #[test]
    fn test_weekday_sequence() {
        // Monday through Friday = 4 business days
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-19",  // Monday
            "2024-02-23"   // Friday
        );
        assert_eq!(days, 4, "Monday to Friday should be 4 business days");
    }

    #[test]
    fn test_weekend_start() {
        // Saturday to Tuesday = 2 business days (Mon, Tue)
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-24",  // Saturday
            "2024-02-27"   // Tuesday
        );
        assert_eq!(days, 2, "Saturday to Tuesday should be 2 business days");
    }

    #[test]
    fn test_weekend_end() {
        // Wednesday to Saturday = 3 business days (Wed, Thu, Fri)
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-21",  // Wednesday
            "2024-02-24"   // Saturday
        );
        assert_eq!(days, 3, "Wednesday to Saturday should be 3 business days");
    }

    #[test]
    fn test_span_includes_weekend() {
        // Thursday to Tuesday crossing one weekend
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-22",  // Thursday
            "2024-02-27"   // Tuesday
        );
        // Thu, Fri, Mon = 3 business days
        assert_eq!(days, 3, "Thu-Tue crossing weekend should be 3 business days");
    }

    #[test]
    fn test_iso8601_datetime_format() {
        // Test with full ISO 8601 datetime strings
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-19T09:00:00Z",
            "2024-02-23T17:00:00Z"
        );
        assert_eq!(days, 4, "ISO 8601 format should work correctly");
    }

    #[test]
    fn test_two_weeks() {
        // Two full weeks = 10 business days
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-19",  // Monday
            "2024-03-04"   // Monday two weeks later
        );
        assert_eq!(days, 10, "Two weeks should be 10 business days");
    }
}

#[cfg(test)]
mod business_hours_tests {
    use super::*;

    #[test]
    fn test_same_business_day_hours() {
        // 1 business day = 8 hours (9am to 5pm)
        let hours = made_activity_tracker::metrics::business_days::business_hours_between(
            "2024-02-19",
            "2024-02-20"
        );
        assert_eq!(hours, 8.0, "1 business day should be 8 hours");
    }

    #[test]
    fn test_weekend_hours() {
        // Time passing on weekend = 0 business hours
        let hours = made_activity_tracker::metrics::business_days::business_hours_between(
            "2024-02-24",  // Saturday
            "2024-02-25"   // Sunday
        );
        assert_eq!(hours, 0.0, "Weekend should have 0 business hours");
    }

    #[test]
    fn test_full_week_hours() {
        // 5 business days = 40 hours
        let hours = made_activity_tracker::metrics::business_days::business_hours_between(
            "2024-02-19",  // Monday
            "2024-02-26"   // Next Monday
        );
        assert_eq!(hours, 40.0, "Full week should be 40 hours");
    }

    #[test]
    fn test_cross_day_hours() {
        // Monday to Tuesday = 1 day = 8 hours
        let hours = made_activity_tracker::metrics::business_days::business_hours_between(
            "2024-02-19T14:00:00Z",  // Monday 2pm
            "2024-02-20T10:00:00Z"   // Tuesday 10am
        );
        assert_eq!(hours, 8.0, "Crossing day boundary should count full business day");
    }

    #[test]
    fn test_two_business_days_hours() {
        // 2 business days = 16 hours
        let hours = made_activity_tracker::metrics::business_days::business_hours_between(
            "2024-02-19",
            "2024-02-21"
        );
        assert_eq!(hours, 16.0, "2 business days should be 16 hours");
    }
}

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_invalid_date_format() {
        // Invalid date should return 0
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "invalid-date",
            "2024-02-20"
        );
        assert_eq!(days, 0, "Invalid start date should return 0");

        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-02-19",
            "invalid-date"
        );
        assert_eq!(days, 0, "Invalid end date should return 0");
    }

    #[test]
    fn test_empty_strings() {
        let days = made_activity_tracker::metrics::business_days::business_days_between("", "");
        assert_eq!(days, 0, "Empty strings should return 0");
    }

    #[test]
    fn test_very_long_period() {
        // One year: approximately 260 business days
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-01-01",
            "2025-01-01"
        );
        // 365 days - ~104 weekend days = ~261 business days (not accounting for holidays)
        assert!(days >= 250 && days <= 270, "One year should have ~260 business days, got {}", days);
    }

    #[test]
    fn test_dst_transition_spring() {
        // Test around DST transition (March 10, 2024 in US)
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-03-08",  // Friday before DST
            "2024-03-11"   // Monday after DST
        );
        assert_eq!(days, 1, "DST transition should not affect business day count");
    }

    #[test]
    fn test_dst_transition_fall() {
        // Test around DST transition (November 3, 2024 in US)
        let days = made_activity_tracker::metrics::business_days::business_days_between(
            "2024-11-01",  // Friday before DST
            "2024-11-04"   // Monday after DST
        );
        assert_eq!(days, 1, "DST transition should not affect business day count");
    }
}
