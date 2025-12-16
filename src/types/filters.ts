/**
 * Filter types for dashboard metrics
 * These match the backend Rust types in src-tauri/src/metrics/filter_params.rs
 */

export interface DateRange {
  start: string; // ISO 8601 format (YYYY-MM-DDTHH:MM:SSZ)
  end: string; // ISO 8601 format (YYYY-MM-DDTHH:MM:SSZ)
}

export interface MetricsFilters {
  dateRange?: DateRange;
  repositoryIds?: number[];
  squadId?: string;
  userId?: number;
}

/**
 * Timeseries data point for chart visualizations
 */
export interface TimeseriesDataPoint {
  date: string; // YYYY-MM-DD format
  speed: SpeedMetrics;
  ease: EaseMetrics;
  quality: QualityMetrics;
}

// Metric subcategory types (matching backend calculator.rs)
export interface SpeedMetrics {
  cycleTime: number;
  cycleTimeTrend: number;
  leadTime: number;
  leadTimeTrend: number;
  deploymentFrequency: number;
  deploymentFrequencyTrend: number;
}

export interface EaseMetrics {
  prApprovalTime: number;
  prApprovalTimeTrend: number;
  codeReviewLoad: number;
  codeReviewLoadTrend: number;
  prSize: number;
  prSizeTrend: number;
}

export interface QualityMetrics {
  changeFailureRate: number;
  changeFailureRateTrend: number;
  bugFixRate: number;
  bugFixRateTrend: number;
  testCoverage: number;
  testCoverageTrend: number;
}

/**
 * Date range presets for quick selection
 */
export const DATE_RANGE_PRESETS = {
  LAST_7_DAYS: 7,
  LAST_30_DAYS: 30,
  LAST_90_DAYS: 90,
  LAST_180_DAYS: 180,
  LAST_365_DAYS: 365,
} as const;

/**
 * Helper to create a date range from days back
 */
export function createDateRangeFromDays(days: number): DateRange {
  const end = new Date();
  const start = new Date();
  start.setDate(start.getDate() - days);

  return {
    start: start.toISOString(),
    end: end.toISOString(),
  };
}

/**
 * Helper to format date range for display
 */
export function formatDateRange(range: DateRange): string {
  const start = new Date(range.start);
  const end = new Date(range.end);

  const formatDate = (date: Date) => {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };

  return `${formatDate(start)} - ${formatDate(end)}`;
}
