// Common type definitions

export interface Repository {
  owner: string;
  name: string;
  enabled: boolean;
}

export interface Squad {
  id: string;
  name: string;
  members: string[];
  color: string;
}

export interface Config {
  repositories: Repository[];
  squads: Squad[];
  excluded_bots: string[];
  excluded_labels: string[];
  bug_labels: string[];
  history_days: number;
}

export interface GitHubUser {
  id: number;
  login: string;
  name: string | null;
  avatar_url: string;
}

export interface SyncStats {
  issues: number;
  pull_requests: number;
  users: number;
  repositories: number;
}

export interface Metrics {
  speed: {
    avg_cycle_time_days: number;
    avg_pr_lead_time_hours: number;
    throughput_per_week: number;
    trend: number;
  };
  ease: {
    avg_pr_size_lines: number;
    avg_review_rounds: number;
    avg_time_to_first_review_hours: number;
    rework_rate: number;
  };
  quality: {
    bug_rate: number;
    reopen_rate: number;
    pr_rejection_rate: number;
    test_coverage_trend: number;
  };
}

export interface User {
  id: number;
  github_id: number;
  login: string;
  name: string | null;
  avatar_url: string | null;
  is_bot: boolean;
}

export interface UserSummary {
  user: User;
  total_commits: number;
  total_prs_created: number;
  total_prs_merged: number;
  total_prs_reviewed: number;
  total_issues_opened: number;
  total_issues_closed: number;
  lines_added: number;
  lines_deleted: number;
  repositories_touched: number;
  first_activity: string | null;
  last_activity: string | null;
  activity_status: 'active' | 'quiet' | 'idle';
}

export interface RepositoryContribution {
  repo_id: number;
  owner: string;
  name: string;
  pr_count: number;
  issue_count: number;
  review_count: number;
  total_contributions: number;
  percentage_of_user_work: number;
}

export interface InteractionStats {
  reviews_given: number;
  reviews_received: number;
}

export interface CollaborationMatrix {
  users: User[];
  interactions: Record<string, Record<string, InteractionStats>>;
}
