//! Unit tests for PR-based Amplifier metrics
//!
//! Tests for metrics_queries.rs - the new PR-based metrics system
//! Priority 0 - CRITICAL (677 lines, 0% coverage)
//!
//! Tests cover:
//! - Speed metrics (PRs per day, turnaround time, cycle time distribution, LOC)
//! - Ease metrics (concurrent repos, work patterns, repo distribution)
//! - Quality metrics (merge rate, PR classification, files per PR)
//! - Overview metrics (productivity multiplier)
//! - Edge cases (empty data, division by zero, null handling)

use rusqlite::{Connection, Result};
use chrono::Utc;
use made_activity_tracker::db::metrics_queries::{
    get_dashboard_metrics, DashboardMetrics, SpeedMetrics, EaseMetrics, QualityMetrics,
};

// ============================================================================
// TEST DATABASE SETUP
// ============================================================================

/// Create an in-memory test database with full schema
fn setup_test_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;

    // Create complete schema needed for metrics
    conn.execute_batch(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            github_id INTEGER UNIQUE NOT NULL,
            login TEXT NOT NULL,
            name TEXT,
            avatar_url TEXT,
            is_bot INTEGER DEFAULT 0,
            tracked INTEGER DEFAULT 0,
            tracked_at TEXT
        );

        CREATE TABLE repositories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            owner TEXT NOT NULL,
            name TEXT NOT NULL,
            github_id INTEGER,
            enabled INTEGER DEFAULT 1,
            last_synced_at TEXT,
            UNIQUE(owner, name)
        );

        CREATE TABLE pull_requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            github_id INTEGER UNIQUE NOT NULL,
            repo_id INTEGER NOT NULL,
            number INTEGER NOT NULL,
            title TEXT NOT NULL,
            body TEXT,
            state TEXT NOT NULL,
            author_id INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            sync_updated_at TEXT,
            merged_at TEXT,
            closed_at TEXT,
            additions INTEGER DEFAULT 0,
            deletions INTEGER DEFAULT 0,
            changed_files INTEGER DEFAULT 0,
            review_comments INTEGER DEFAULT 0,
            labels TEXT DEFAULT '[]',
            FOREIGN KEY (repo_id) REFERENCES repositories(id),
            FOREIGN KEY (author_id) REFERENCES users(id)
        );

        CREATE INDEX idx_pr_created_at ON pull_requests(created_at);
        CREATE INDEX idx_pr_author_id ON pull_requests(author_id);
        CREATE INDEX idx_pr_repo_id ON pull_requests(repo_id);"
    )?;

    Ok(conn)
}

/// Helper to create a tracked user
fn create_tracked_user(conn: &Connection, github_id: i64, login: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO users (github_id, login, tracked) VALUES (?1, ?2, 1)",
        [&github_id.to_string(), login],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Helper to create a repository
fn create_repository(conn: &Connection, owner: &str, name: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO repositories (owner, name, github_id) VALUES (?1, ?2, ?3)",
        [owner, name, "12345"],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Helper to create a pull request
#[allow(clippy::too_many_arguments)]
fn create_pr(
    conn: &Connection,
    repo_id: i64,
    author_id: i64,
    number: i32,
    title: &str,
    created_at: &str,
    merged_at: Option<&str>,
    closed_at: Option<&str>,
    additions: i32,
    deletions: i32,
    changed_files: i32,
) -> Result<i64> {
    let state = if merged_at.is_some() || closed_at.is_some() {
        "closed"
    } else {
        "open"
    };

    conn.execute(
        "INSERT INTO pull_requests (
            github_id, repo_id, number, title, state, author_id,
            created_at, updated_at, merged_at, closed_at,
            additions, deletions, changed_files, labels
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            (repo_id * 1000 + number as i64), // Unique github_id
            repo_id,
            number,
            title,
            state,
            author_id,
            created_at,
            created_at,
            merged_at,
            closed_at,
            additions,
            deletions,
            changed_files,
            "[]"
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

// ============================================================================
// SPEED METRICS TESTS
// ============================================================================

#[cfg(test)]
mod speed_metrics_tests {
    use super::*;

    #[test]
    fn test_prs_per_day_calculation() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // Create 10 PRs over 5 days (should be 2 PRs per day)
        for i in 0..10 {
            let day = i / 2; // 2 PRs per day
            let created_at = format!("2024-01-{:02}T10:00:00Z", 1 + day);
            create_pr(
                &conn,
                repo_id,
                user_id,
                i as i32,
                "Test PR",
                &created_at,
                Some(&format!("2024-01-{:02}T14:00:00Z", 1 + day)),
                Some(&format!("2024-01-{:02}T14:00:00Z", 1 + day)),
                100,
                50,
                5,
            )
            .unwrap();
        }

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // Should be 2 PRs per day (10 PRs / 5 active days)
        assert_eq!(metrics.speed.prs_per_day, 2.0);
    }

    #[test]
    fn test_prs_per_day_per_dev_calculation() {
        let conn = setup_test_db().unwrap();
        let user1 = create_tracked_user(&conn, 1, "dev1").unwrap();
        let user2 = create_tracked_user(&conn, 2, "dev2").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // Dev1: 4 PRs on same day
        for i in 0..4 {
            create_pr(
                &conn,
                repo_id,
                user1,
                i,
                "PR from dev1",
                "2024-01-01T10:00:00Z",
                Some("2024-01-01T14:00:00Z"),
                Some("2024-01-01T14:00:00Z"),
                100,
                50,
                5,
            )
            .unwrap();
        }

        // Dev2: 6 PRs on same day
        for i in 0..6 {
            create_pr(
                &conn,
                repo_id,
                user2,
                100 + i,
                "PR from dev2",
                "2024-01-01T10:00:00Z",
                Some("2024-01-01T14:00:00Z"),
                Some("2024-01-01T14:00:00Z"),
                100,
                50,
                5,
            )
            .unwrap();
        }

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // 10 PRs / (2 devs * 1 day) = 5 PRs per dev per day
        assert_eq!(metrics.speed.prs_per_day_per_dev, 5.0);
    }

    #[test]
    fn test_pr_turnaround_hours() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // PR merged 4 hours after creation
        create_pr(
            &conn,
            repo_id,
            user_id,
            1,
            "Quick PR",
            "2024-01-01T10:00:00Z",
            Some("2024-01-01T14:00:00Z"),
            Some("2024-01-01T14:00:00Z"),
            50,
            20,
            3,
        )
        .unwrap();

        // PR merged 8 hours after creation
        create_pr(
            &conn,
            repo_id,
            user_id,
            2,
            "Slower PR",
            "2024-01-01T10:00:00Z",
            Some("2024-01-01T18:00:00Z"),
            Some("2024-01-01T18:00:00Z"),
            100,
            50,
            5,
        )
        .unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // Average: (4 + 8) / 2 = 6 hours
        assert_eq!(metrics.speed.pr_turnaround_hours, 6.0);
    }

    #[test]
    fn test_pr_turnaround_ignores_unmerged() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // Merged PR: 4 hours
        create_pr(
            &conn,
            repo_id,
            user_id,
            1,
            "Merged PR",
            "2024-01-01T10:00:00Z",
            Some("2024-01-01T14:00:00Z"),
            Some("2024-01-01T14:00:00Z"),
            100,
            50,
            5,
        )
        .unwrap();

        // Open PR (should be ignored)
        create_pr(
            &conn,
            repo_id,
            user_id,
            2,
            "Open PR",
            "2024-01-01T10:00:00Z",
            None,
            None,
            100,
            50,
            5,
        )
        .unwrap();

        // Closed without merge (should be ignored)
        create_pr(
            &conn,
            repo_id,
            user_id,
            3,
            "Rejected PR",
            "2024-01-01T10:00:00Z",
            None,
            Some("2024-01-01T20:00:00Z"),
            100,
            50,
            5,
        )
        .unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // Only merged PR should count
        assert_eq!(metrics.speed.pr_turnaround_hours, 4.0);
    }

    #[test]
    fn test_loc_per_day_calculation() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // PR with 500 LOC (300 additions + 200 deletions)
        create_pr(
            &conn,
            repo_id,
            user_id,
            1,
            "PR 1",
            "2024-01-01T10:00:00Z",
            Some("2024-01-01T14:00:00Z"),
            Some("2024-01-01T14:00:00Z"),
            300,
            200,
            10,
        )
        .unwrap();

        // PR with 100 LOC (50 additions + 50 deletions)
        create_pr(
            &conn,
            repo_id,
            user_id,
            2,
            "PR 2",
            "2024-01-02T10:00:00Z",
            Some("2024-01-02T14:00:00Z"),
            Some("2024-01-02T14:00:00Z"),
            50,
            50,
            5,
        )
        .unwrap();

        let metrics = get_dashboard_metrics(&conn, 7).unwrap();

        // Total: 600 LOC over 7 days = 85.71... LOC per day
        let expected = 600.0 / 7.0;
        assert!((metrics.speed.loc_per_day - expected).abs() < 0.1);
    }

    #[test]
    fn test_cycle_time_distribution() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // Under 4h: 2 PRs
        for i in 0..2 {
            create_pr(
                &conn,
                repo_id,
                user_id,
                i,
                "Fast PR",
                "2024-01-01T10:00:00Z",
                Some("2024-01-01T12:00:00Z"), // 2 hours
                Some("2024-01-01T12:00:00Z"),
                100,
                50,
                5,
            )
            .unwrap();
        }

        // 4-12h: 3 PRs
        for i in 10..13 {
            create_pr(
                &conn,
                repo_id,
                user_id,
                i,
                "Medium PR",
                "2024-01-02T10:00:00Z",
                Some("2024-01-02T18:00:00Z"), // 8 hours
                Some("2024-01-02T18:00:00Z"),
                100,
                50,
                5,
            )
            .unwrap();
        }

        // 12-24h: 1 PR
        create_pr(
            &conn,
            repo_id,
            user_id,
            20,
            "Slow PR",
            "2024-01-03T10:00:00Z",
            Some("2024-01-03T22:00:00Z"), // 12 hours
            Some("2024-01-03T22:00:00Z"),
            100,
            50,
            5,
        )
        .unwrap();

        // Over 24h: 4 PRs
        for i in 30..34 {
            create_pr(
                &conn,
                repo_id,
                user_id,
                i,
                "Very slow PR",
                "2024-01-04T10:00:00Z",
                Some("2024-01-06T10:00:00Z"), // 48 hours
                Some("2024-01-06T10:00:00Z"),
                100,
                50,
                5,
            )
            .unwrap();
        }

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();
        let dist = &metrics.speed.cycle_time_distribution;

        // Total: 10 PRs
        assert_eq!(dist.under_4h, 2);
        assert_eq!(dist.h4_to_12, 3);
        assert_eq!(dist.h12_to_24, 1);
        assert_eq!(dist.over_24h, 4);

        // Percentages
        assert_eq!(dist.under_4h_pct, 20.0);
        assert_eq!(dist.h4_to_12_pct, 30.0);
        assert_eq!(dist.h12_to_24_pct, 10.0);
        assert_eq!(dist.over_24h_pct, 40.0);
    }

    #[test]
    fn test_cycle_time_distribution_empty() {
        let conn = setup_test_db().unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();
        let dist = &metrics.speed.cycle_time_distribution;

        assert_eq!(dist.under_4h, 0);
        assert_eq!(dist.h4_to_12, 0);
        assert_eq!(dist.h12_to_24, 0);
        assert_eq!(dist.over_24h, 0);
        assert_eq!(dist.under_4h_pct, 0.0);
    }

    #[test]
    fn test_speed_benchmarks_are_set() {
        let conn = setup_test_db().unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();
        let bench = &metrics.speed.benchmark_comparison;

        // Verify industry benchmarks are present
        assert_eq!(bench.prs_per_day_industry, 0.8);
        assert_eq!(bench.prs_per_day_elite, 1.5);
        assert_eq!(bench.pr_turnaround_industry, 89.0);
        assert_eq!(bench.pr_turnaround_elite, 24.0);
    }

    #[test]
    fn test_only_tracked_users_counted() {
        let conn = setup_test_db().unwrap();
        let tracked_user = create_tracked_user(&conn, 1, "tracked").unwrap();

        // Create untracked user
        conn.execute(
            "INSERT INTO users (github_id, login, tracked) VALUES (2, 'untracked', 0)",
            [],
        )
        .unwrap();
        let untracked_user = conn.last_insert_rowid();

        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // PR from tracked user
        create_pr(
            &conn,
            repo_id,
            tracked_user,
            1,
            "Tracked PR",
            "2024-01-01T10:00:00Z",
            Some("2024-01-01T14:00:00Z"),
            Some("2024-01-01T14:00:00Z"),
            100,
            50,
            5,
        )
        .unwrap();

        // PR from untracked user (should be ignored)
        create_pr(
            &conn,
            repo_id,
            untracked_user,
            2,
            "Untracked PR",
            "2024-01-01T10:00:00Z",
            Some("2024-01-01T14:00:00Z"),
            Some("2024-01-01T14:00:00Z"),
            100,
            50,
            5,
        )
        .unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // Only 1 PR should be counted (from tracked user)
        assert_eq!(metrics.overview.total_prs, 1);
        assert_eq!(metrics.overview.active_developers, 1);
    }
}

// ============================================================================
// EDGE CASES TESTS
// ============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_empty_database() {
        let conn = setup_test_db().unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // All metrics should be zero or default
        assert_eq!(metrics.speed.prs_per_day, 0.0);
        assert_eq!(metrics.speed.prs_per_day_per_dev, 0.0);
        assert_eq!(metrics.speed.pr_turnaround_hours, 0.0);
        assert_eq!(metrics.speed.loc_per_day, 0.0);
        assert_eq!(metrics.overview.total_prs, 0);
        assert_eq!(metrics.overview.active_developers, 0);
    }

    #[test]
    fn test_single_pr() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        create_pr(
            &conn,
            repo_id,
            user_id,
            1,
            "Only PR",
            "2024-01-01T10:00:00Z",
            Some("2024-01-01T12:00:00Z"),
            Some("2024-01-01T12:00:00Z"),
            100,
            50,
            5,
        )
        .unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        assert_eq!(metrics.overview.total_prs, 1);
        assert_eq!(metrics.speed.prs_per_day, 1.0); // 1 PR on 1 active day
        assert_eq!(metrics.speed.pr_turnaround_hours, 2.0);
    }

    #[test]
    fn test_division_by_zero_protection() {
        let conn = setup_test_db().unwrap();

        // No PRs, no developers - should not panic
        let result = get_dashboard_metrics(&conn, 30);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert_eq!(metrics.speed.prs_per_day_per_dev, 0.0);
    }

    #[test]
    fn test_null_merged_at_ignored() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // Open PR with no merged_at
        create_pr(
            &conn,
            repo_id,
            user_id,
            1,
            "Open PR",
            "2024-01-01T10:00:00Z",
            None,
            None,
            100,
            50,
            5,
        )
        .unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // Should not include open PR in turnaround calculation
        assert_eq!(metrics.speed.pr_turnaround_hours, 0.0);
    }

    #[test]
    fn test_zero_lines_changed() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // PR with 0 additions and 0 deletions
        create_pr(
            &conn,
            repo_id,
            user_id,
            1,
            "Empty PR",
            "2024-01-01T10:00:00Z",
            Some("2024-01-01T12:00:00Z"),
            Some("2024-01-01T12:00:00Z"),
            0,
            0,
            1,
        )
        .unwrap();

        let metrics = get_dashboard_metrics(&conn, 7).unwrap();

        // LOC per day should be 0
        assert_eq!(metrics.speed.loc_per_day, 0.0);
    }

    #[test]
    fn test_very_large_pr() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // Massive PR: 50,000 additions
        create_pr(
            &conn,
            repo_id,
            user_id,
            1,
            "Huge PR",
            "2024-01-01T10:00:00Z",
            Some("2024-01-01T12:00:00Z"),
            Some("2024-01-01T12:00:00Z"),
            50000,
            10000,
            500,
        )
        .unwrap();

        let metrics = get_dashboard_metrics(&conn, 7).unwrap();

        // Total LOC: 60,000 over 7 days
        let expected = 60000.0 / 7.0;
        assert!((metrics.speed.loc_per_day - expected).abs() < 0.1);
    }

    #[test]
    fn test_negative_time_period() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // Create a PR in the past (beyond the query window)
        create_pr(
            &conn,
            repo_id,
            user_id,
            1,
            "Old PR",
            "2020-01-01T10:00:00Z",
            Some("2020-01-01T12:00:00Z"),
            Some("2020-01-01T12:00:00Z"),
            100,
            50,
            5,
        )
        .unwrap();

        // Query last 7 days (should not include the old PR)
        let metrics = get_dashboard_metrics(&conn, 7).unwrap();

        assert_eq!(metrics.overview.total_prs, 0);
    }

    #[test]
    fn test_future_dates_handled() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // PR with future date (shouldn't happen, but let's handle it)
        create_pr(
            &conn,
            repo_id,
            user_id,
            1,
            "Future PR",
            "2030-01-01T10:00:00Z",
            Some("2030-01-01T12:00:00Z"),
            Some("2030-01-01T12:00:00Z"),
            100,
            50,
            5,
        )
        .unwrap();

        // Should not panic
        let result = get_dashboard_metrics(&conn, 30);
        assert!(result.is_ok());
    }
}

// ============================================================================
// PRODUCTIVITY MULTIPLIER TESTS
// ============================================================================

#[cfg(test)]
mod productivity_multiplier_tests {
    use super::*;

    #[test]
    fn test_productivity_multiplier_formula() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "superdev").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // Create high-performing metrics
        // 2 PRs per day (industry: 0.8)
        for i in 0..14 {
            let day = i / 2;
            let created_at = format!("2024-01-{:02}T10:00:00Z", 1 + day);
            create_pr(
                &conn,
                repo_id,
                user_id,
                i,
                "High velocity PR",
                &created_at,
                Some(&format!("2024-01-{:02}T12:00:00Z", 1 + day)), // 2h turnaround
                Some(&format!("2024-01-{:02}T12:00:00Z", 1 + day)),
                150,
                50,
                5,
            )
            .unwrap();
        }

        let metrics = get_dashboard_metrics(&conn, 7).unwrap();

        // Productivity multiplier should be > 1.0 (performing above industry)
        assert!(metrics.overview.productivity_multiplier > 1.0);
    }

    #[test]
    fn test_productivity_multiplier_zero_division_safe() {
        let conn = setup_test_db().unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // With no data, multiplier calculation should not panic
        // It will have NaN or 0, but shouldn't crash
        assert!(metrics.overview.productivity_multiplier.is_finite() || metrics.overview.productivity_multiplier == 0.0);
    }

    #[test]
    fn test_overview_metrics_counts() {
        let conn = setup_test_db().unwrap();
        let user1 = create_tracked_user(&conn, 1, "dev1").unwrap();
        let user2 = create_tracked_user(&conn, 2, "dev2").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // Dev1: 5 PRs
        for i in 0..5 {
            create_pr(
                &conn,
                repo_id,
                user1,
                i,
                "PR",
                "2024-01-01T10:00:00Z",
                Some("2024-01-01T12:00:00Z"),
                Some("2024-01-01T12:00:00Z"),
                100,
                50,
                5,
            )
            .unwrap();
        }

        // Dev2: 3 PRs
        for i in 0..3 {
            create_pr(
                &conn,
                repo_id,
                user2,
                100 + i,
                "PR",
                "2024-01-01T10:00:00Z",
                Some("2024-01-01T12:00:00Z"),
                Some("2024-01-01T12:00:00Z"),
                100,
                50,
                5,
            )
            .unwrap();
        }

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        assert_eq!(metrics.overview.total_prs, 8);
        assert_eq!(metrics.overview.active_developers, 2);
        assert_eq!(metrics.overview.period_days, 30);
    }
}

// ============================================================================
// EASE METRICS TESTS
// ============================================================================

#[cfg(test)]
mod ease_metrics_tests {
    use super::*;

    #[test]
    fn test_concurrent_repos_calculation() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();

        // Create 3 different repositories
        let repo1 = create_repository(&conn, "org", "repo1").unwrap();
        let repo2 = create_repository(&conn, "org", "repo2").unwrap();
        let repo3 = create_repository(&conn, "org", "repo3").unwrap();

        // PRs across 3 repos
        create_pr(&conn, repo1, user_id, 1, "PR 1", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo2, user_id, 2, "PR 2", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo3, user_id, 3, "PR 3", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        assert_eq!(metrics.ease.concurrent_repos, 3);
        assert_eq!(metrics.ease.repos_per_dev, 3.0); // 3 repos / 1 dev
    }

    #[test]
    fn test_repos_per_dev_multiple_developers() {
        let conn = setup_test_db().unwrap();
        let user1 = create_tracked_user(&conn, 1, "dev1").unwrap();
        let user2 = create_tracked_user(&conn, 2, "dev2").unwrap();

        let repo1 = create_repository(&conn, "org", "repo1").unwrap();
        let repo2 = create_repository(&conn, "org", "repo2").unwrap();
        let repo3 = create_repository(&conn, "org", "repo3").unwrap();
        let repo4 = create_repository(&conn, "org", "repo4").unwrap();

        // Dev1 works on repo1 and repo2
        create_pr(&conn, repo1, user1, 1, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo2, user1, 2, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();

        // Dev2 works on repo3 and repo4
        create_pr(&conn, repo3, user2, 3, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo4, user2, 4, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // 4 repos across 2 devs = 2 repos per dev
        assert_eq!(metrics.ease.concurrent_repos, 4);
        assert_eq!(metrics.ease.repos_per_dev, 2.0);
    }

    #[test]
    fn test_active_repositories_list() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();

        let repo1 = create_repository(&conn, "org", "alpha").unwrap();
        let repo2 = create_repository(&conn, "org", "beta").unwrap();

        // Repo1: 5 PRs
        for i in 0..5 {
            create_pr(&conn, repo1, user_id, i, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        }

        // Repo2: 3 PRs
        for i in 0..3 {
            create_pr(&conn, repo2, user_id, 100 + i, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        }

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        assert_eq!(metrics.ease.active_repos.len(), 2);
        assert_eq!(metrics.ease.total_active_repos, 2);

        // Verify repos are ordered by PR count (desc)
        assert_eq!(metrics.ease.active_repos[0].repo_name, "org/alpha");
        assert_eq!(metrics.ease.active_repos[0].pr_count, 5);
        assert_eq!(metrics.ease.active_repos[1].repo_name, "org/beta");
        assert_eq!(metrics.ease.active_repos[1].pr_count, 3);
    }

    #[test]
    fn test_active_repositories_with_loc() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // PR with 500 LOC
        create_pr(&conn, repo_id, user_id, 1, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 300, 200, 5).unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        assert_eq!(metrics.ease.active_repos[0].total_loc, 500);
    }

    #[test]
    fn test_repo_distribution_org_vs_personal() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();

        // Microsoft org repos (2)
        let repo1 = create_repository(&conn, "microsoft", "repo1").unwrap();
        let repo2 = create_repository(&conn, "Microsoft", "repo2").unwrap(); // Case insensitive

        // Personal repos (3)
        let repo3 = create_repository(&conn, "johndoe", "repo3").unwrap();
        let repo4 = create_repository(&conn, "janedoe", "repo4").unwrap();
        let repo5 = create_repository(&conn, "alice", "repo5").unwrap();

        // Create PRs for each repo
        create_pr(&conn, repo1, user_id, 1, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo2, user_id, 2, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo3, user_id, 3, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo4, user_id, 4, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo5, user_id, 5, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // 2 org repos, 3 personal repos = 40% org, 60% personal
        assert_eq!(metrics.ease.repo_distribution.org_repos, 2);
        assert_eq!(metrics.ease.repo_distribution.personal_repos, 3);
        assert_eq!(metrics.ease.repo_distribution.org_repos_pct, 40.0);
        assert_eq!(metrics.ease.repo_distribution.personal_repos_pct, 60.0);
    }

    #[test]
    fn test_work_pattern_heatmap() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // PRs at different times
        create_pr(&conn, repo_id, user_id, 1, "Morning PR", "2024-01-01T09:00:00Z", Some("2024-01-01T10:00:00Z"), Some("2024-01-01T10:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "Morning PR", "2024-01-01T09:30:00Z", Some("2024-01-01T10:00:00Z"), Some("2024-01-01T10:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo_id, user_id, 3, "Afternoon PR", "2024-01-01T14:00:00Z", Some("2024-01-01T15:00:00Z"), Some("2024-01-01T15:00:00Z"), 100, 50, 5).unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // Should have work pattern data
        assert!(!metrics.ease.work_pattern.is_empty());

        // Find the 9am slot (should have 2 PRs)
        let morning_slot = metrics.ease.work_pattern.iter()
            .find(|cell| cell.hour_of_day == 9);
        assert!(morning_slot.is_some());
        assert_eq!(morning_slot.unwrap().activity_count, 2);

        // Find the 2pm slot (should have 1 PR)
        let afternoon_slot = metrics.ease.work_pattern.iter()
            .find(|cell| cell.hour_of_day == 14);
        assert!(afternoon_slot.is_some());
        assert_eq!(afternoon_slot.unwrap().activity_count, 1);
    }

    #[test]
    fn test_pr_switch_frequency() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();

        let repo1 = create_repository(&conn, "org", "repo1").unwrap();
        let repo2 = create_repository(&conn, "org", "repo2").unwrap();

        // Alternating between repos: repo1 -> repo2 -> repo1 -> repo2
        create_pr(&conn, repo1, user_id, 1, "PR", "2024-01-01T09:00:00Z", Some("2024-01-01T10:00:00Z"), Some("2024-01-01T10:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo2, user_id, 2, "PR", "2024-01-01T11:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo1, user_id, 3, "PR", "2024-01-01T13:00:00Z", Some("2024-01-01T14:00:00Z"), Some("2024-01-01T14:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo2, user_id, 4, "PR", "2024-01-01T15:00:00Z", Some("2024-01-01T16:00:00Z"), Some("2024-01-01T16:00:00Z"), 100, 50, 5).unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // 3 switches out of 3 transitions = 100%
        assert_eq!(metrics.ease.pr_switch_frequency, 100.0);
    }

    #[test]
    fn test_pr_switch_frequency_no_switching() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // All PRs in same repo
        for i in 0..5 {
            let time = format!("2024-01-01T{:02}:00:00Z", 9 + i);
            create_pr(&conn, repo_id, user_id, i, "PR", &time, Some("2024-01-01T10:00:00Z"), Some("2024-01-01T10:00:00Z"), 100, 50, 5).unwrap();
        }

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // 0 switches = 0%
        assert_eq!(metrics.ease.pr_switch_frequency, 0.0);
    }

    #[test]
    fn test_ease_benchmarks_are_set() {
        let conn = setup_test_db().unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        assert_eq!(metrics.ease.benchmark_comparison.concurrent_repos_industry, 2.1);
        assert_eq!(metrics.ease.benchmark_comparison.concurrent_repos_elite, 3.5);
    }
}

// ============================================================================
// QUALITY METRICS TESTS
// ============================================================================

#[cfg(test)]
mod quality_metrics_tests {
    use super::*;

    #[test]
    fn test_pr_merge_rate_all_merged() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // 5 merged PRs
        for i in 0..5 {
            create_pr(&conn, repo_id, user_id, i, "Merged PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        }

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // 100% merge rate
        assert_eq!(metrics.quality.pr_merge_rate, 100.0);
    }

    #[test]
    fn test_pr_merge_rate_partial() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // 3 merged PRs
        for i in 0..3 {
            create_pr(&conn, repo_id, user_id, i, "Merged", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        }

        // 2 closed without merge
        for i in 10..12 {
            create_pr(&conn, repo_id, user_id, i, "Rejected", "2024-01-01T10:00:00Z", None, Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        }

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // 3/5 = 60% merge rate
        assert_eq!(metrics.quality.pr_merge_rate, 60.0);
    }

    #[test]
    fn test_pr_merge_rate_ignores_open() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // 2 merged
        for i in 0..2 {
            create_pr(&conn, repo_id, user_id, i, "Merged", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        }

        // 5 open PRs (should not be counted)
        for i in 10..15 {
            create_pr(&conn, repo_id, user_id, i, "Open", "2024-01-01T10:00:00Z", None, None, 100, 50, 5).unwrap();
        }

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // Only closed PRs counted: 2/2 = 100%
        assert_eq!(metrics.quality.pr_merge_rate, 100.0);
    }

    #[test]
    fn test_avg_files_per_pr() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // PR with 3 files
        create_pr(&conn, repo_id, user_id, 1, "Small PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 3).unwrap();

        // PR with 9 files
        create_pr(&conn, repo_id, user_id, 2, "Medium PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 9).unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // Average: (3 + 9) / 2 = 6.0
        assert_eq!(metrics.quality.avg_files_per_pr, 6.0);
    }

    #[test]
    fn test_pr_type_classification_feature() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // Feature PRs
        create_pr(&conn, repo_id, user_id, 1, "feat: Add login", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "feature: Add logout", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo_id, user_id, 3, "Add user settings", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // Should have feature type in distribution
        let feature_type = metrics.quality.pr_type_distribution.iter()
            .find(|p| p.pr_type == "feature");
        assert!(feature_type.is_some());
        assert_eq!(feature_type.unwrap().count, 3);
        assert_eq!(feature_type.unwrap().percentage, 100.0);
        assert_eq!(metrics.quality.feature_pr_percentage, 100.0);
    }

    #[test]
    fn test_pr_type_classification_bug_fix() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // Bug fix PRs
        create_pr(&conn, repo_id, user_id, 1, "fix: Login error", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "bug: Fix crash", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        let bug_type = metrics.quality.pr_type_distribution.iter()
            .find(|p| p.pr_type == "bug_fix");
        assert!(bug_type.is_some());
        assert_eq!(bug_type.unwrap().count, 2);
        assert_eq!(metrics.quality.bug_pr_percentage, 100.0);
    }

    #[test]
    fn test_pr_type_classification_mixed() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // 2 features
        create_pr(&conn, repo_id, user_id, 1, "feat: New feature", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "feature: Another", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();

        // 3 bug fixes
        create_pr(&conn, repo_id, user_id, 3, "fix: Bug 1", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo_id, user_id, 4, "fix: Bug 2", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo_id, user_id, 5, "fix: Bug 3", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();

        // 1 refactor
        create_pr(&conn, repo_id, user_id, 6, "refactor: Clean code", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        // Total: 6 PRs
        // 2 features = 33.33%
        // 3 bug fixes = 50%
        // 1 refactor = 16.67%
        assert_eq!(metrics.quality.feature_pr_percentage, 33.333333333333336);
        assert_eq!(metrics.quality.bug_pr_percentage, 50.0);
    }

    #[test]
    fn test_files_per_pr_distribution() {
        let conn = setup_test_db().unwrap();
        let user_id = create_tracked_user(&conn, 1, "developer").unwrap();
        let repo_id = create_repository(&conn, "org", "repo").unwrap();

        // Range 1-3: 2 PRs
        create_pr(&conn, repo_id, user_id, 1, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 2).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 3).unwrap();

        // Range 4-8: 3 PRs
        create_pr(&conn, repo_id, user_id, 3, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 5).unwrap();
        create_pr(&conn, repo_id, user_id, 4, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 7).unwrap();
        create_pr(&conn, repo_id, user_id, 5, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 8).unwrap();

        // Range 9-15: 1 PR
        create_pr(&conn, repo_id, user_id, 6, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 12).unwrap();

        // Range 16+: 4 PRs
        create_pr(&conn, repo_id, user_id, 7, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 20).unwrap();
        create_pr(&conn, repo_id, user_id, 8, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 30).unwrap();
        create_pr(&conn, repo_id, user_id, 9, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 50).unwrap();
        create_pr(&conn, repo_id, user_id, 10, "PR", "2024-01-01T10:00:00Z", Some("2024-01-01T12:00:00Z"), Some("2024-01-01T12:00:00Z"), 100, 50, 100).unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();
        let dist = &metrics.quality.files_per_pr_distribution;

        // Total: 10 PRs
        assert_eq!(dist.range_1_3, 2);
        assert_eq!(dist.range_1_3_pct, 20.0);
        assert_eq!(dist.range_4_8, 3);
        assert_eq!(dist.range_4_8_pct, 30.0);
        assert_eq!(dist.range_9_15, 1);
        assert_eq!(dist.range_9_15_pct, 10.0);
        assert_eq!(dist.range_16_plus, 4);
        assert_eq!(dist.range_16_plus_pct, 40.0);
    }

    #[test]
    fn test_quality_benchmarks_are_set() {
        let conn = setup_test_db().unwrap();

        let metrics = get_dashboard_metrics(&conn, 30).unwrap();

        assert_eq!(metrics.quality.benchmark_comparison.merge_rate_industry, 68.0);
        assert_eq!(metrics.quality.benchmark_comparison.merge_rate_elite, 85.0);
        assert_eq!(metrics.quality.benchmark_comparison.bug_ratio_industry, 25.0);
        assert_eq!(metrics.quality.benchmark_comparison.bug_ratio_elite, 15.0);
        assert_eq!(metrics.quality.benchmark_comparison.files_per_pr_industry, 8.0);
    }
}
