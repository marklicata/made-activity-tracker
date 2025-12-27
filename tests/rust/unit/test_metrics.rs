//! Unit tests for metrics calculations
//!
//! Tests for:
//! - Speed metrics (cycle time, PR lead time, throughput)
//! - Ease metrics (PR size, review rounds, rework rate)
//! - Quality metrics (bug rate, reopen rate, rejection rate)

use made_activity_tracker::db::models::{Issue, PullRequest};
use made_activity_tracker::metrics::calculator::{calculate_dashboard_metrics, DashboardMetrics};

// Helper functions to create test data
fn make_issue(created_at: &str, closed_at: Option<&str>, labels: Vec<&str>) -> Issue {
    Issue {
        id: 1,
        github_id: 1,
        repo_id: 1,
        number: 1,
        title: "Test Issue".to_string(),
        body: Some("Test body".to_string()),
        state: if closed_at.is_some() { "closed" } else { "open" }.to_string(),
        author_id: Some(1),
        assignee_id: None,
        milestone_id: None,
        created_at: created_at.to_string(),
        updated_at: created_at.to_string(),
        sync_updated_at: Some(created_at.to_string()),
        closed_at: closed_at.map(|s| s.to_string()),
        labels: labels.into_iter().map(|s| s.to_string()).collect(),
    }
}

fn make_pr(
    created_at: &str,
    merged_at: Option<&str>,
    closed_at: Option<&str>,
    additions: i32,
    deletions: i32,
    review_comments: i32,
) -> PullRequest {
    PullRequest {
        id: 1,
        github_id: 1,
        repo_id: 1,
        number: 1,
        title: "Test PR".to_string(),
        body: Some("Test body".to_string()),
        state: if merged_at.is_some() || closed_at.is_some() { "closed" } else { "open" }.to_string(),
        author_id: Some(1),
        created_at: created_at.to_string(),
        updated_at: created_at.to_string(),
        sync_updated_at: Some(created_at.to_string()),
        merged_at: merged_at.map(|s| s.to_string()),
        closed_at: closed_at.map(|s| s.to_string()),
        additions,
        deletions,
        changed_files: 1,
        review_comments,
        labels: vec![],
    }
}

#[cfg(test)]
mod speed_metrics_tests {
    use super::*;

    #[test]
    fn test_cycle_time_calculation() {
        // Test basic cycle time calculation
        let issues = vec![
            make_issue("2024-02-19", Some("2024-02-23"), vec![]),  // Monday to Friday = 4 business days
            make_issue("2024-02-20", Some("2024-02-22"), vec![]),  // Tuesday to Thursday = 2 business days
        ];

        let metrics = calculate_dashboard_metrics(&issues, &[], &[], 30);

        // Average: (4 + 2) / 2 = 3.0 business days
        assert_eq!(metrics.speed.avg_cycle_time_days, 3.0);
    }

    #[test]
    fn test_cycle_time_with_weekends() {
        // Test that weekends are excluded from cycle time
        let issues = vec![
            make_issue("2024-02-23", Some("2024-02-26"), vec![]),  // Friday to Monday = 1 business day
        ];

        let metrics = calculate_dashboard_metrics(&issues, &[], &[], 30);

        assert_eq!(metrics.speed.avg_cycle_time_days, 1.0);
    }

    #[test]
    fn test_pr_lead_time_calculation() {
        // Test PR lead time (in hours)
        let prs = vec![
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 100, 50, 2),  // 1 day = 8 hours
            make_pr("2024-02-19", Some("2024-02-22"), Some("2024-02-22"), 100, 50, 2),  // 3 days = 24 hours
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 30);

        // Average: (8 + 24) / 2 = 16.0 hours
        assert_eq!(metrics.speed.avg_pr_lead_time_hours, 16.0);
    }

    #[test]
    fn test_throughput_calculation() {
        // Test throughput: items completed per week
        let issues = vec![
            make_issue("2024-02-01", Some("2024-02-05"), vec![]),
            make_issue("2024-02-01", Some("2024-02-06"), vec![]),
        ];

        let prs = vec![
            make_pr("2024-02-01", Some("2024-02-05"), Some("2024-02-05"), 100, 50, 2),
            make_pr("2024-02-01", Some("2024-02-06"), Some("2024-02-06"), 100, 50, 2),
            make_pr("2024-02-01", Some("2024-02-07"), Some("2024-02-07"), 100, 50, 2),
        ];

        let metrics = calculate_dashboard_metrics(&issues, &prs, &[], 14); // 2 weeks

        // 5 items in 2 weeks = 2.5 per week
        assert_eq!(metrics.speed.throughput_per_week, 2.5);
    }

    #[test]
    fn test_empty_data_returns_zeros() {
        let metrics = calculate_dashboard_metrics(&[], &[], &[], 30);

        assert_eq!(metrics.speed.avg_cycle_time_days, 0.0);
        assert_eq!(metrics.speed.avg_pr_lead_time_hours, 0.0);
        assert_eq!(metrics.speed.throughput_per_week, 0.0);
    }

    #[test]
    fn test_only_open_issues_ignored() {
        // Open issues should not contribute to cycle time
        let issues = vec![
            make_issue("2024-02-19", None, vec![]),  // Open issue
            make_issue("2024-02-19", Some("2024-02-23"), vec![]),  // Closed issue
        ];

        let metrics = calculate_dashboard_metrics(&issues, &[], &[], 30);

        // Only the closed issue should count
        assert_eq!(metrics.speed.avg_cycle_time_days, 4.0);
    }

    #[test]
    fn test_only_merged_prs_counted() {
        // Only merged PRs should contribute to lead time
        let prs = vec![
            make_pr("2024-02-19", None, None, 100, 50, 2),  // Open PR
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 100, 50, 2),  // Merged PR
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 30);

        assert_eq!(metrics.speed.avg_pr_lead_time_hours, 8.0);
    }
}

#[cfg(test)]
mod ease_metrics_tests {
    use super::*;

    #[test]
    fn test_avg_pr_size_calculation() {
        let prs = vec![
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 100, 50, 2),  // 150 lines
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 200, 100, 2), // 300 lines
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 50, 50, 2),   // 100 lines
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 30);

        // Average: (150 + 300 + 100) / 3 = 183.33, rounded to 183
        assert_eq!(metrics.ease.avg_pr_size_lines, 183.0);
    }

    #[test]
    fn test_review_rounds_calculation() {
        let prs = vec![
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 100, 50, 4),  // 4 comments = 2 rounds
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 100, 50, 6),  // 6 comments = 3 rounds
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 100, 50, 2),  // 2 comments = 1 round
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 30);

        // Average: (2 + 3 + 1) / 3 = 2.0 rounds
        assert_eq!(metrics.ease.avg_review_rounds, 2.0);
    }

    #[test]
    fn test_rework_rate_calculation() {
        let prs = vec![
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 600, 50, 2),  // > 500 lines (rework)
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 100, 50, 2),  // < 500 lines
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 300, 50, 2),  // < 500 lines
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 400, 200, 2), // > 500 lines (rework)
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 30);

        // Rework rate: 2 / 4 = 0.50
        assert_eq!(metrics.ease.rework_rate, 0.50);
    }

    #[test]
    fn test_time_to_first_review() {
        let prs = vec![
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 100, 50, 2),
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 30);

        // Currently placeholder value
        assert_eq!(metrics.ease.avg_time_to_first_review_hours, 4.0);
    }

    #[test]
    fn test_empty_pr_list() {
        let metrics = calculate_dashboard_metrics(&[], &[], &[], 30);

        assert_eq!(metrics.ease.avg_pr_size_lines, 0.0);
        assert_eq!(metrics.ease.avg_review_rounds, 1.0);  // Default minimum
        assert_eq!(metrics.ease.rework_rate, 0.0);
    }
}

#[cfg(test)]
mod quality_metrics_tests {
    use super::*;

    #[test]
    fn test_bug_rate_with_various_labels() {
        let issues = vec![
            make_issue("2024-02-01", Some("2024-02-05"), vec!["bug"]),
            make_issue("2024-02-01", Some("2024-02-05"), vec!["feature"]),
            make_issue("2024-02-01", Some("2024-02-05"), vec!["bug", "critical"]),
            make_issue("2024-02-01", Some("2024-02-05"), vec!["enhancement"]),
            make_issue("2024-02-01", Some("2024-02-05"), vec!["documentation"]),
        ];

        let bug_labels = vec!["bug".to_string()];
        let metrics = calculate_dashboard_metrics(&issues, &[], &bug_labels, 30);

        // Bug rate: 2 bugs out of 5 issues = 0.400
        assert_eq!(metrics.quality.bug_rate, 0.400);
    }

    #[test]
    fn test_bug_rate_with_multiple_bug_labels() {
        let issues = vec![
            make_issue("2024-02-01", Some("2024-02-05"), vec!["bug"]),
            make_issue("2024-02-01", Some("2024-02-05"), vec!["defect"]),
            make_issue("2024-02-01", Some("2024-02-05"), vec!["feature"]),
        ];

        let bug_labels = vec!["bug".to_string(), "defect".to_string()];
        let metrics = calculate_dashboard_metrics(&issues, &[], &bug_labels, 30);

        // Bug rate: 2 bugs out of 3 issues = 0.667
        assert!((metrics.quality.bug_rate - 0.667).abs() < 0.001);
    }

    #[test]
    fn test_reopen_rate_calculation() {
        // Currently not implemented, should return 0.0
        let issues = vec![
            make_issue("2024-02-01", Some("2024-02-05"), vec![]),
        ];

        let metrics = calculate_dashboard_metrics(&issues, &[], &[], 30);

        assert_eq!(metrics.quality.reopen_rate, 0.0);
    }

    #[test]
    fn test_pr_rejection_rate() {
        let prs = vec![
            make_pr("2024-02-01", Some("2024-02-05"), Some("2024-02-05"), 100, 50, 2), // Merged
            make_pr("2024-02-01", Some("2024-02-05"), Some("2024-02-05"), 100, 50, 2), // Merged
            make_pr("2024-02-01", None, Some("2024-02-05"), 100, 50, 2),               // Closed without merge
            make_pr("2024-02-01", None, Some("2024-02-06"), 100, 50, 2),               // Closed without merge
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 30);

        // Rejection rate: 2 / 4 = 0.500
        assert_eq!(metrics.quality.pr_rejection_rate, 0.500);
    }

    #[test]
    fn test_quality_metrics_with_no_bugs() {
        let issues = vec![
            make_issue("2024-02-01", Some("2024-02-05"), vec!["feature"]),
            make_issue("2024-02-01", Some("2024-02-05"), vec!["enhancement"]),
        ];

        let bug_labels = vec!["bug".to_string()];
        let metrics = calculate_dashboard_metrics(&issues, &[], &bug_labels, 30);

        assert_eq!(metrics.quality.bug_rate, 0.0);
    }

    #[test]
    fn test_all_prs_merged() {
        let prs = vec![
            make_pr("2024-02-01", Some("2024-02-05"), Some("2024-02-05"), 100, 50, 2),
            make_pr("2024-02-01", Some("2024-02-05"), Some("2024-02-05"), 100, 50, 2),
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 30);

        assert_eq!(metrics.quality.pr_rejection_rate, 0.0);
    }

    #[test]
    fn test_empty_data_quality_metrics() {
        let metrics = calculate_dashboard_metrics(&[], &[], &[], 30);

        assert_eq!(metrics.quality.bug_rate, 0.0);
        assert_eq!(metrics.quality.reopen_rate, 0.0);
        assert_eq!(metrics.quality.pr_rejection_rate, 0.0);
        assert_eq!(metrics.quality.test_coverage_trend, 0.0);
    }
}

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_missing_timestamps() {
        // PRs without merged_at should not affect lead time
        let prs = vec![
            make_pr("2024-02-19", None, None, 100, 50, 2),
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 30);

        assert_eq!(metrics.speed.avg_pr_lead_time_hours, 0.0);
    }

    #[test]
    fn test_future_dates() {
        // Future dates should still calculate (though unusual)
        let issues = vec![
            make_issue("2024-02-19", Some("2025-02-19"), vec![]),
        ];

        let metrics = calculate_dashboard_metrics(&issues, &[], &[], 30);

        // Should not crash, should have some positive value
        assert!(metrics.speed.avg_cycle_time_days > 0.0);
    }

    #[test]
    fn test_zero_additions_and_deletions() {
        let prs = vec![
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 0, 0, 2),
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 30);

        assert_eq!(metrics.ease.avg_pr_size_lines, 0.0);
    }

    #[test]
    fn test_negative_time_period() {
        // Closed before created (data corruption scenario)
        let issues = vec![
            make_issue("2024-02-20", Some("2024-02-19"), vec![]),
        ];

        let metrics = calculate_dashboard_metrics(&issues, &[], &[], 30);

        // Should handle gracefully (business_days_between returns 0 for negative ranges)
        assert_eq!(metrics.speed.avg_cycle_time_days, 0.0);
    }

    #[test]
    fn test_single_day_period() {
        let issues = vec![
            make_issue("2024-02-01", Some("2024-02-05"), vec![]),
        ];

        let metrics = calculate_dashboard_metrics(&issues, &[], &[], 1);  // 1 day period

        // Throughput calculation should handle small periods
        assert!(metrics.speed.throughput_per_week >= 0.0);
    }

    #[test]
    fn test_very_large_pr() {
        let prs = vec![
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 10000, 5000, 2),
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 30);

        assert_eq!(metrics.ease.avg_pr_size_lines, 15000.0);
        assert_eq!(metrics.ease.rework_rate, 1.0);  // Large PR counts as rework
    }

    #[test]
    fn test_zero_review_comments() {
        let prs = vec![
            make_pr("2024-02-19", Some("2024-02-20"), Some("2024-02-20"), 100, 50, 0),
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 30);

        // Should default to minimum 1.0 round
        assert_eq!(metrics.ease.avg_review_rounds, 1.0);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_dashboard_metrics() {
        // Test a realistic scenario with mixed data
        let issues = vec![
            make_issue("2024-02-01", Some("2024-02-05"), vec!["bug"]),
            make_issue("2024-02-01", Some("2024-02-06"), vec!["feature"]),
            make_issue("2024-02-01", None, vec!["enhancement"]),  // Still open
        ];

        let prs = vec![
            make_pr("2024-02-01", Some("2024-02-05"), Some("2024-02-05"), 100, 50, 4),
            make_pr("2024-02-01", Some("2024-02-06"), Some("2024-02-06"), 200, 100, 2),
            make_pr("2024-02-01", None, Some("2024-02-07"), 300, 50, 6),  // Rejected
        ];

        let bug_labels = vec!["bug".to_string()];
        let metrics = calculate_dashboard_metrics(&issues, &prs, &bug_labels, 30);

        // Verify all metric categories are calculated
        assert!(metrics.speed.avg_cycle_time_days > 0.0);
        assert!(metrics.speed.avg_pr_lead_time_hours > 0.0);
        assert!(metrics.speed.throughput_per_week > 0.0);

        assert!(metrics.ease.avg_pr_size_lines > 0.0);
        assert!(metrics.ease.avg_review_rounds >= 1.0);

        assert!(metrics.quality.bug_rate > 0.0);
        assert!(metrics.quality.pr_rejection_rate > 0.0);
    }

    #[test]
    fn test_dashboard_metrics_structure() {
        let metrics = calculate_dashboard_metrics(&[], &[], &[], 30);

        // Verify structure exists
        assert!(metrics.speed.trend == 0.0);  // Currently not implemented
        assert!(metrics.ease.avg_time_to_first_review_hours == 4.0);  // Placeholder
        assert!(metrics.quality.test_coverage_trend == 0.0);  // Not implemented
    }
}
