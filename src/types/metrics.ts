// PR-Based Metrics Types (Amplifier-style)
// Generated from src-tauri/src/db/metrics_queries.rs

export interface DashboardMetrics {
  speed: SpeedMetrics;
  ease: EaseMetrics;
  quality: QualityMetrics;
  overview: OverviewMetrics;
}

export interface OverviewMetrics {
  productivity_multiplier: number;
  period_days: number;
  total_prs: number;
  active_developers: number;
}

// ============================================================================
// SPEED METRICS
// ============================================================================

export interface SpeedMetrics {
  prs_per_day: number;
  prs_per_day_per_dev: number;
  pr_turnaround_hours: number;
  loc_per_day: number;
  cycle_time_distribution: CycleTimeDistribution;
  benchmark_comparison: SpeedBenchmarks;
}

export interface CycleTimeDistribution {
  under_4h: number;
  under_4h_pct: number;
  h4_to_12: number;
  h4_to_12_pct: number;
  h12_to_24: number;
  h12_to_24_pct: number;
  over_24h: number;
  over_24h_pct: number;
}

export interface SpeedBenchmarks {
  prs_per_day_industry: number;      // 0.8
  prs_per_day_elite: number;         // 1.5
  pr_turnaround_industry: number;    // 89.0
  pr_turnaround_elite: number;       // 24.0
}

// ============================================================================
// EASE METRICS
// ============================================================================

export interface EaseMetrics {
  concurrent_repos: number;
  repos_per_dev: number;
  total_active_repos: number;
  active_repos: ActiveRepository[];
  repo_distribution: RepoDistribution;
  work_pattern: WorkPatternCell[];
  pr_switch_frequency: number;
  benchmark_comparison: EaseBenchmarks;
}

export interface ActiveRepository {
  repo_name: string;
  pr_count: number;
  total_loc: number;
  contributor_count: number;
  last_activity: string;
}

export interface RepoDistribution {
  org_repos: number;
  org_repos_pct: number;
  personal_repos: number;
  personal_repos_pct: number;
}

export interface WorkPatternCell {
  day_of_week: number;  // 0=Sunday, 1=Monday, etc.
  hour_of_day: number;  // 0-23
  activity_count: number;
}

export interface EaseBenchmarks {
  concurrent_repos_industry: number;  // 2.1
  concurrent_repos_elite: number;     // 3.5
}

// ============================================================================
// QUALITY METRICS
// ============================================================================

export interface QualityMetrics {
  pr_merge_rate: number;
  avg_files_per_pr: number;
  bug_pr_percentage: number;
  feature_pr_percentage: number;
  avg_review_cycle_hours: number;
  avg_review_comments: number;
  pr_type_distribution: PrTypeBreakdown[];
  files_per_pr_distribution: FilesPerPrDistribution;
  merge_rate_trend: MergeRateTrend[];
  benchmark_comparison: QualityBenchmarks;
}

export interface PrTypeBreakdown {
  pr_type: 'feature' | 'bug_fix' | 'refactor' | 'test' | 'docs' | 'other';
  count: number;
  percentage: number;
}

export interface FilesPerPrDistribution {
  range_1_3: number;
  range_1_3_pct: number;
  range_4_8: number;
  range_4_8_pct: number;
  range_9_15: number;
  range_9_15_pct: number;
  range_16_plus: number;
  range_16_plus_pct: number;
}

export interface MergeRateTrend {
  week: string;
  merge_rate: number;
  total_prs: number;
}

export interface QualityBenchmarks {
  merge_rate_industry: number;    // 68.0
  merge_rate_elite: number;       // 85.0
  bug_ratio_industry: number;     // 25.0
  bug_ratio_elite: number;        // 15.0
  files_per_pr_industry: number;  // 8.0
}

// ============================================================================
// HELPER TYPES
// ============================================================================

export type MetricTier = 'below_industry' | 'industry' | 'elite' | 'exceptional';

export interface MetricComparison {
  value: number;
  tier: MetricTier;
  vs_industry_pct: number;
  vs_elite_pct: number;
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Determine performance tier for a metric value
 */
export function getMetricTier(
  value: number,
  industryBenchmark: number,
  eliteBenchmark: number,
  higherIsBetter: boolean = true
): MetricTier {
  if (higherIsBetter) {
    if (value < industryBenchmark * 0.8) return 'below_industry';
    if (value < eliteBenchmark) return 'industry';
    if (value < eliteBenchmark * 1.5) return 'elite';
    return 'exceptional';
  } else {
    // For metrics where lower is better (e.g., PR turnaround time)
    if (value > industryBenchmark * 1.2) return 'below_industry';
    if (value > eliteBenchmark) return 'industry';
    if (value > eliteBenchmark * 0.5) return 'elite';
    return 'exceptional';
  }
}

/**
 * Calculate percentage difference vs benchmark
 */
export function getVsBenchmarkPct(value: number, benchmark: number, higherIsBetter: boolean = true): number {
  if (benchmark === 0) return 0;

  const diff = ((value - benchmark) / benchmark) * 100;
  return higherIsBetter ? diff : -diff;
}

/**
 * Format metric comparison
 */
export function formatMetricComparison(
  value: number,
  industryBenchmark: number,
  eliteBenchmark: number,
  higherIsBetter: boolean = true
): MetricComparison {
  return {
    value,
    tier: getMetricTier(value, industryBenchmark, eliteBenchmark, higherIsBetter),
    vs_industry_pct: getVsBenchmarkPct(value, industryBenchmark, higherIsBetter),
    vs_elite_pct: getVsBenchmarkPct(value, eliteBenchmark, higherIsBetter),
  };
}

/**
 * Get color for metric tier
 */
export function getTierColor(tier: MetricTier): string {
  switch (tier) {
    case 'below_industry': return '#ef4444'; // red
    case 'industry': return '#f59e0b'; // orange
    case 'elite': return '#3b82f6'; // blue
    case 'exceptional': return '#10b981'; // green
  }
}

/**
 * Get label for metric tier
 */
export function getTierLabel(tier: MetricTier): string {
  switch (tier) {
    case 'below_industry': return 'Below Industry';
    case 'industry': return 'Industry Average';
    case 'elite': return 'Elite Performer';
    case 'exceptional': return 'Exceptional';
  }
}

/**
 * Format hours to human readable string
 */
export function formatHours(hours: number): string {
  if (hours < 1) return `${Math.round(hours * 60)}m`;
  if (hours < 24) return `${hours.toFixed(1)}h`;
  const days = hours / 24;
  if (days < 7) return `${days.toFixed(1)}d`;
  return `${(days / 7).toFixed(1)}w`;
}

/**
 * Get day name from day of week number
 */
export function getDayName(dayOfWeek: number): string {
  const days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  return days[dayOfWeek] || '';
}

/**
 * Format hour to AM/PM
 */
export function formatHour(hour: number): string {
  if (hour === 0) return '12 AM';
  if (hour < 12) return `${hour} AM`;
  if (hour === 12) return '12 PM';
  return `${hour - 12} PM`;
}
