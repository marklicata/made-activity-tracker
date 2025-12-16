use super::business_days::business_days_between;
use crate::db::models::{Issue, PullRequest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub speed: SpeedMetrics,
    pub ease: EaseMetrics,
    pub quality: QualityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedMetrics {
    pub avg_cycle_time_days: f64,
    pub avg_pr_lead_time_hours: f64,
    pub throughput_per_week: f64,
    pub trend: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EaseMetrics {
    pub avg_pr_size_lines: f64,
    pub avg_review_rounds: f64,
    pub avg_time_to_first_review_hours: f64,
    pub rework_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub bug_rate: f64,
    pub reopen_rate: f64,
    pub pr_rejection_rate: f64,
    pub test_coverage_trend: f64,
}

/// Calculate all dashboard metrics from issues and PRs
pub fn calculate_dashboard_metrics(
    issues: &[Issue],
    prs: &[PullRequest],
    bug_labels: &[String],
    days_in_period: i64,
) -> DashboardMetrics {
    DashboardMetrics {
        speed: calculate_speed_metrics(issues, prs, days_in_period),
        ease: calculate_ease_metrics(prs),
        quality: calculate_quality_metrics(issues, prs, bug_labels),
    }
}

fn calculate_speed_metrics(
    issues: &[Issue],
    prs: &[PullRequest],
    days_in_period: i64,
) -> SpeedMetrics {
    // Cycle time: time from issue creation to close (business days)
    let closed_issues: Vec<_> = issues.iter().filter(|i| i.closed_at.is_some()).collect();
    
    let avg_cycle_time = if !closed_issues.is_empty() {
        let total_days: i64 = closed_issues
            .iter()
            .map(|i| business_days_between(&i.created_at, i.closed_at.as_ref().unwrap()))
            .sum();
        total_days as f64 / closed_issues.len() as f64
    } else {
        0.0
    };

    // PR lead time: time from PR open to merge (in hours)
    let merged_prs: Vec<_> = prs.iter().filter(|p| p.merged_at.is_some()).collect();
    
    let avg_pr_lead_time = if !merged_prs.is_empty() {
        let total_hours: f64 = merged_prs
            .iter()
            .map(|p| {
                let days = business_days_between(&p.created_at, p.merged_at.as_ref().unwrap());
                days as f64 * 8.0 // Convert to hours (8-hour day)
            })
            .sum();
        total_hours / merged_prs.len() as f64
    } else {
        0.0
    };

    // Throughput: items completed per week
    let weeks = (days_in_period as f64 / 7.0).max(1.0);
    let completed = closed_issues.len() + merged_prs.len();
    let throughput = completed as f64 / weeks;

    SpeedMetrics {
        avg_cycle_time_days: round_to_decimals(avg_cycle_time, 1),
        avg_pr_lead_time_hours: round_to_decimals(avg_pr_lead_time, 1),
        throughput_per_week: round_to_decimals(throughput, 1),
        trend: 0.0, // TODO: Calculate vs previous period
    }
}

fn calculate_ease_metrics(prs: &[PullRequest]) -> EaseMetrics {
    // Average PR size
    let avg_size = if !prs.is_empty() {
        let total_lines: i32 = prs.iter().map(|p| p.additions + p.deletions).sum();
        total_lines as f64 / prs.len() as f64
    } else {
        0.0
    };

    // Average review comments (proxy for review rounds)
    let avg_reviews = if !prs.is_empty() {
        let total_comments: i32 = prs.iter().map(|p| p.review_comments).sum();
        // Rough estimate: every 2 comments = 1 round
        (total_comments as f64 / prs.len() as f64 / 2.0).max(1.0)
    } else {
        1.0
    };

    // TODO: Calculate time to first review (requires review timestamps)
    
    // Rework rate: PRs with > 100 lines of changes after initial review
    // This is a simplified proxy
    let large_prs = prs
        .iter()
        .filter(|p| (p.additions + p.deletions) > 500)
        .count();
    let rework_rate = if !prs.is_empty() {
        large_prs as f64 / prs.len() as f64
    } else {
        0.0
    };

    EaseMetrics {
        avg_pr_size_lines: round_to_decimals(avg_size, 0),
        avg_review_rounds: round_to_decimals(avg_reviews, 1),
        avg_time_to_first_review_hours: 4.0, // Placeholder
        rework_rate: round_to_decimals(rework_rate, 2),
    }
}

fn calculate_quality_metrics(
    issues: &[Issue],
    prs: &[PullRequest],
    bug_labels: &[String],
) -> QualityMetrics {
    // Bug rate: percentage of issues that are bugs
    let bug_count = issues
        .iter()
        .filter(|i| i.labels.iter().any(|l| bug_labels.contains(l)))
        .count();
    let bug_rate = if !issues.is_empty() {
        bug_count as f64 / issues.len() as f64
    } else {
        0.0
    };

    // Reopen rate: issues that were reopened (simplified: closed then state changed)
    // TODO: Implement proper reopen tracking via issue events
    let reopen_rate = 0.0;

    // PR rejection rate: PRs closed without merge
    let closed_not_merged = prs
        .iter()
        .filter(|p| p.closed_at.is_some() && p.merged_at.is_none())
        .count();
    let pr_rejection_rate = if !prs.is_empty() {
        closed_not_merged as f64 / prs.len() as f64
    } else {
        0.0
    };

    QualityMetrics {
        bug_rate: round_to_decimals(bug_rate, 3),
        reopen_rate: round_to_decimals(reopen_rate, 3),
        pr_rejection_rate: round_to_decimals(pr_rejection_rate, 3),
        test_coverage_trend: 0.0, // TODO: Integrate with coverage tools
    }
}

fn round_to_decimals(value: f64, decimals: u32) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (value * multiplier).round() / multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_issue(created_at: &str, closed_at: Option<&str>, labels: Vec<&str>) -> Issue {
        Issue {
            id: 1,
            github_id: 1,
            repo_id: 1,
            number: 1,
            title: "Test".to_string(),
            body: None,
            state: if closed_at.is_some() { "closed" } else { "open" }.to_string(),
            author_id: None,
            assignee_id: None,
            milestone_id: None,
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
            closed_at: closed_at.map(|s| s.to_string()),
            labels: labels.into_iter().map(|s| s.to_string()).collect(),
            embedding_id: None,
        }
    }

    fn make_pr(created_at: &str, merged_at: Option<&str>, additions: i32, deletions: i32) -> PullRequest {
        PullRequest {
            id: 1,
            github_id: 1,
            repo_id: 1,
            number: 1,
            title: "Test".to_string(),
            body: None,
            state: "closed".to_string(),
            author_id: None,
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
            merged_at: merged_at.map(|s| s.to_string()),
            closed_at: merged_at.map(|s| s.to_string()),
            additions,
            deletions,
            changed_files: 1,
            review_comments: 2,
            labels: vec![],
            embedding_id: None,
        }
    }

    #[test]
    fn test_bug_rate_calculation() {
        let issues = vec![
            make_issue("2024-02-01", Some("2024-02-05"), vec!["bug"]),
            make_issue("2024-02-01", Some("2024-02-05"), vec!["feature"]),
            make_issue("2024-02-01", Some("2024-02-05"), vec!["bug", "urgent"]),
            make_issue("2024-02-01", None, vec!["enhancement"]),
        ];
        
        let metrics = calculate_quality_metrics(&issues, &[], &["bug".to_string()]);
        assert_eq!(metrics.bug_rate, 0.5); // 2 out of 4
    }

    #[test]
    fn test_pr_rejection_rate() {
        let prs = vec![
            make_pr("2024-02-01", Some("2024-02-05"), 100, 50), // merged
            make_pr("2024-02-01", Some("2024-02-05"), 100, 50), // merged
            make_pr("2024-02-01", None, 100, 50), // closed without merge
        ];
        
        // Manually set closed_at for the rejected PR
        let mut prs = prs;
        prs[2].closed_at = Some("2024-02-05".to_string());
        
        let metrics = calculate_quality_metrics(&[], &prs, &[]);
        assert!((metrics.pr_rejection_rate - 0.333).abs() < 0.01);
    }
}
