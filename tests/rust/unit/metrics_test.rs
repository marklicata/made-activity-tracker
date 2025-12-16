//! Unit tests for metrics calculations and filtering
//!
//! These tests verify the correctness of Speed, Ease, and Quality metrics,
//! as well as the filtering functionality.

#[cfg(test)]
mod filter_query_tests {
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // Create minimal schema for testing
        conn.execute_batch(
            "CREATE TABLE issues (
                id INTEGER PRIMARY KEY,
                github_id INTEGER UNIQUE NOT NULL,
                repo_id INTEGER NOT NULL,
                number INTEGER NOT NULL,
                title TEXT NOT NULL,
                state TEXT NOT NULL,
                author_id INTEGER,
                created_at TEXT NOT NULL,
                closed_at TEXT
            );

            CREATE TABLE pull_requests (
                id INTEGER PRIMARY KEY,
                github_id INTEGER UNIQUE NOT NULL,
                repo_id INTEGER NOT NULL,
                number INTEGER NOT NULL,
                title TEXT NOT NULL,
                state TEXT NOT NULL,
                author_id INTEGER,
                created_at TEXT NOT NULL,
                merged_at TEXT,
                closed_at TEXT,
                additions INTEGER DEFAULT 0,
                deletions INTEGER DEFAULT 0
            );

            CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                github_id INTEGER UNIQUE NOT NULL,
                login TEXT NOT NULL,
                is_bot INTEGER DEFAULT 0
            );

            CREATE TABLE squad_members (
                squad_id TEXT NOT NULL,
                user_id INTEGER NOT NULL
            );

            CREATE TABLE repositories (
                id INTEGER PRIMARY KEY,
                owner TEXT NOT NULL,
                name TEXT NOT NULL,
                enabled INTEGER DEFAULT 1
            );"
        ).unwrap();

        // Insert test data
        conn.execute_batch(
            "INSERT INTO repositories (id, owner, name) VALUES
                (1, 'test-org', 'repo-1'),
                (2, 'test-org', 'repo-2');

            INSERT INTO users (id, github_id, login, is_bot) VALUES
                (1, 101, 'alice', 0),
                (2, 102, 'bob', 0),
                (3, 103, 'bot', 1);

            INSERT INTO issues (id, github_id, repo_id, number, title, state, author_id, created_at, closed_at) VALUES
                (1, 1001, 1, 1, 'Issue 1', 'closed', 1, '2024-01-01T00:00:00Z', '2024-01-05T00:00:00Z'),
                (2, 1002, 1, 2, 'Issue 2', 'open', 2, '2024-01-10T00:00:00Z', NULL),
                (3, 1003, 2, 3, 'Issue 3', 'closed', 1, '2024-01-15T00:00:00Z', '2024-01-20T00:00:00Z'),
                (4, 1004, 1, 4, 'Bot Issue', 'closed', 3, '2024-01-01T00:00:00Z', '2024-01-02T00:00:00Z');

            INSERT INTO pull_requests (id, github_id, repo_id, number, title, state, author_id, created_at, merged_at, additions, deletions) VALUES
                (1, 2001, 1, 1, 'PR 1', 'closed', 1, '2024-01-01T00:00:00Z', '2024-01-03T00:00:00Z', 100, 50),
                (2, 2002, 2, 2, 'PR 2', 'closed', 2, '2024-01-10T00:00:00Z', '2024-01-12T00:00:00Z', 200, 100),
                (3, 2003, 1, 3, 'Bot PR', 'closed', 3, '2024-01-01T00:00:00Z', '2024-01-02T00:00:00Z', 50, 25);

            INSERT INTO squad_members (squad_id, user_id) VALUES
                ('frontend', 1),
                ('frontend', 2);"
        ).unwrap();

        conn
    }

    #[test]
    fn test_filter_by_date_range() {
        let conn = setup_test_db();

        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM issues
             WHERE created_at >= ?1 AND created_at <= ?2"
        ).unwrap();

        let count: i64 = stmt.query_row(
            ["2024-01-01T00:00:00Z", "2024-01-10T00:00:00Z"],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(count, 3, "Should find 3 issues in date range");
    }

    #[test]
    fn test_filter_by_repository() {
        let conn = setup_test_db();

        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM issues WHERE repo_id = ?1"
        ).unwrap();

        let count: i64 = stmt.query_row([1], |row| row.get(0)).unwrap();
        assert_eq!(count, 3, "Repo 1 should have 3 issues");

        let count: i64 = stmt.query_row([2], |row| row.get(0)).unwrap();
        assert_eq!(count, 1, "Repo 2 should have 1 issue");
    }

    #[test]
    fn test_filter_by_author() {
        let conn = setup_test_db();

        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM issues WHERE author_id = ?1"
        ).unwrap();

        let count: i64 = stmt.query_row([1], |row| row.get(0)).unwrap();
        assert_eq!(count, 2, "Alice should have 2 issues");
    }

    #[test]
    fn test_exclude_bots() {
        let conn = setup_test_db();

        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM issues i
             JOIN users u ON i.author_id = u.id
             WHERE u.is_bot = 0"
        ).unwrap();

        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 3, "Should exclude 1 bot issue, leaving 3");
    }

    #[test]
    fn test_filter_by_squad_members() {
        let conn = setup_test_db();

        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM issues i
             WHERE i.author_id IN (
                 SELECT user_id FROM squad_members WHERE squad_id = ?1
             )"
        ).unwrap();

        let count: i64 = stmt.query_row(["frontend"], |row| row.get(0)).unwrap();
        assert_eq!(count, 3, "Frontend squad members should have 3 issues");
    }

    #[test]
    fn test_combined_filters() {
        let conn = setup_test_db();

        // Date range + repository + exclude bots
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM issues i
             JOIN users u ON i.author_id = u.id
             WHERE i.created_at >= ?1
             AND i.created_at <= ?2
             AND i.repo_id = ?3
             AND u.is_bot = 0"
        ).unwrap();

        let count: i64 = stmt.query_row(
            ["2024-01-01T00:00:00Z", "2024-01-10T00:00:00Z", "1"],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(count, 2, "Combined filters should return 2 issues");
    }
}

#[cfg(test)]
mod metrics_calculation_tests {
    use made_activity_tracker::metrics::calculator::{calculate_dashboard_metrics};
    use made_activity_tracker::db::models::{Issue, PullRequest};

    fn make_issue(
        id: i64,
        created_at: &str,
        closed_at: Option<&str>,
        labels: Vec<&str>
    ) -> Issue {
        Issue {
            id,
            github_id: id * 1000,
            repo_id: 1,
            number: id as i32,
            title: format!("Test Issue {}", id),
            body: None,
            state: if closed_at.is_some() { "closed" } else { "open" }.to_string(),
            author_id: Some(1),
            assignee_id: None,
            milestone_id: None,
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
            closed_at: closed_at.map(|s| s.to_string()),
            labels: labels.into_iter().map(|s| s.to_string()).collect(),
            embedding_id: None,
        }
    }

    fn make_pr(
        id: i64,
        created_at: &str,
        merged_at: Option<&str>,
        additions: i32,
        deletions: i32,
    ) -> PullRequest {
        PullRequest {
            id,
            github_id: id * 2000,
            repo_id: 1,
            number: id as i32,
            title: format!("Test PR {}", id),
            body: None,
            state: if merged_at.is_some() { "closed" } else { "open" }.to_string(),
            author_id: Some(1),
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
            merged_at: merged_at.map(|s| s.to_string()),
            closed_at: merged_at.map(|s| s.to_string()),
            additions,
            deletions,
            changed_files: 1,
            review_comments: 4,
            labels: vec![],
            embedding_id: None,
        }
    }

    #[test]
    fn test_bug_rate_calculation() {
        let issues = vec![
            make_issue(1, "2024-01-01T00:00:00Z", Some("2024-01-05T00:00:00Z"), vec!["bug"]),
            make_issue(2, "2024-01-02T00:00:00Z", Some("2024-01-06T00:00:00Z"), vec!["feature"]),
            make_issue(3, "2024-01-03T00:00:00Z", Some("2024-01-07T00:00:00Z"), vec!["bug", "urgent"]),
            make_issue(4, "2024-01-04T00:00:00Z", None, vec!["enhancement"]),
        ];

        let metrics = calculate_dashboard_metrics(
            &issues,
            &[],
            &["bug".to_string()],
            90
        );

        assert_eq!(metrics.quality.bug_rate, 0.5, "Bug rate should be 50% (2 out of 4)");
    }

    #[test]
    fn test_pr_rejection_rate() {
        let prs = vec![
            make_pr(1, "2024-01-01T00:00:00Z", Some("2024-01-05T00:00:00Z"), 100, 50), // merged
            make_pr(2, "2024-01-02T00:00:00Z", Some("2024-01-06T00:00:00Z"), 200, 100), // merged
            make_pr(3, "2024-01-03T00:00:00Z", None, 150, 75), // closed without merge
        ];

        // Manually set closed_at for rejected PR
        let mut prs = prs;
        prs[2].closed_at = Some("2024-01-10T00:00:00Z".to_string());

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 90);

        let expected = 0.333;
        let diff = (metrics.quality.pr_rejection_rate - expected).abs();
        assert!(diff < 0.01, "PR rejection rate should be ~33% (1 out of 3)");
    }

    #[test]
    fn test_avg_pr_size() {
        let prs = vec![
            make_pr(1, "2024-01-01T00:00:00Z", Some("2024-01-05T00:00:00Z"), 100, 50),  // 150 lines
            make_pr(2, "2024-01-02T00:00:00Z", Some("2024-01-06T00:00:00Z"), 200, 100), // 300 lines
            make_pr(3, "2024-01-03T00:00:00Z", Some("2024-01-07T00:00:00Z"), 50, 25),   // 75 lines
        ];

        let metrics = calculate_dashboard_metrics(&[], &prs, &[], 90);

        // Average: (150 + 300 + 75) / 3 = 175
        assert_eq!(metrics.ease.avg_pr_size_lines, 175.0, "Average PR size should be 175 lines");
    }

    #[test]
    fn test_throughput_calculation() {
        let issues = vec![
            make_issue(1, "2024-01-01T00:00:00Z", Some("2024-01-05T00:00:00Z"), vec![]),
            make_issue(2, "2024-01-02T00:00:00Z", Some("2024-01-06T00:00:00Z"), vec![]),
        ];

        let prs = vec![
            make_pr(1, "2024-01-01T00:00:00Z", Some("2024-01-05T00:00:00Z"), 100, 50),
        ];

        let metrics = calculate_dashboard_metrics(&issues, &prs, &[], 7); // 1 week

        // 2 closed issues + 1 merged PR = 3 items in 1 week
        assert_eq!(metrics.speed.throughput_per_week, 3.0, "Throughput should be 3 items/week");
    }

    #[test]
    fn test_empty_data() {
        let metrics = calculate_dashboard_metrics(&[], &[], &[], 90);

        assert_eq!(metrics.speed.avg_cycle_time_days, 0.0);
        assert_eq!(metrics.speed.avg_pr_lead_time_hours, 0.0);
        assert_eq!(metrics.speed.throughput_per_week, 0.0);
        assert_eq!(metrics.ease.avg_pr_size_lines, 0.0);
        assert_eq!(metrics.quality.bug_rate, 0.0);
    }
}

#[cfg(test)]
mod filter_serialization_tests {
    use made_activity_tracker::metrics::filter_params::{MetricsFilters, DateRange};
    use serde_json;

    #[test]
    fn test_metrics_filters_serialization() {
        let filters = MetricsFilters {
            date_range: Some(DateRange {
                start: "2024-01-01T00:00:00Z".to_string(),
                end: "2024-01-31T00:00:00Z".to_string(),
            }),
            repository_ids: Some(vec![1, 2, 3]),
            squad_id: Some("frontend".to_string()),
            user_id: None,
        };

        let json = serde_json::to_string(&filters).unwrap();
        let deserialized: MetricsFilters = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.repository_ids, Some(vec![1, 2, 3]));
        assert_eq!(deserialized.squad_id, Some("frontend".to_string()));
        assert_eq!(deserialized.user_id, None);
    }

    #[test]
    fn test_camel_case_serialization() {
        let filters = MetricsFilters {
            date_range: Some(DateRange {
                start: "2024-01-01T00:00:00Z".to_string(),
                end: "2024-01-31T00:00:00Z".to_string(),
            }),
            repository_ids: Some(vec![1]),
            squad_id: None,
            user_id: Some(42),
        };

        let json = serde_json::to_string(&filters).unwrap();

        // Should use camelCase for TypeScript interop
        assert!(json.contains("dateRange"));
        assert!(json.contains("repositoryIds"));
        assert!(json.contains("userId"));
    }

    #[test]
    fn test_optional_fields() {
        let filters = MetricsFilters {
            date_range: None,
            repository_ids: None,
            squad_id: None,
            user_id: None,
        };

        let json = serde_json::to_string(&filters).unwrap();
        let deserialized: MetricsFilters = serde_json::from_str(&json).unwrap();

        assert!(deserialized.date_range.is_none());
        assert!(deserialized.repository_ids.is_none());
        assert!(deserialized.squad_id.is_none());
        assert!(deserialized.user_id.is_none());
    }
}
