import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import type { DashboardMetrics } from '../types/metrics';

interface UsePRMetricsOptions {
  days?: number;
  autoRefresh?: boolean;
  refreshInterval?: number; // in milliseconds
}

interface UsePRMetricsResult {
  metrics: DashboardMetrics | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

/**
 * Hook to fetch PR-based dashboard metrics
 *
 * @example
 * ```tsx
 * function Dashboard() {
 *   const { metrics, loading, error, refresh } = usePRMetrics({ days: 30 });
 *
 *   if (loading) return <div>Loading...</div>;
 *   if (error) return <div>Error: {error}</div>;
 *   if (!metrics) return null;
 *
 *   return (
 *     <div>
 *       <h1>Productivity Multiplier: {metrics.overview.productivity_multiplier.toFixed(1)}x</h1>
 *       <div>
 *         <h2>Speed</h2>
 *         <p>PRs per day: {metrics.speed.prs_per_day_per_dev.toFixed(2)}</p>
 *         <p>PR turnaround: {metrics.speed.pr_turnaround_hours.toFixed(1)}h</p>
 *       </div>
 *     </div>
 *   );
 * }
 * ```
 */
export function usePRMetrics(options: UsePRMetricsOptions = {}): UsePRMetricsResult {
  const { days = 30, autoRefresh = false, refreshInterval = 60000 } = options;

  const [metrics, setMetrics] = useState<DashboardMetrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchMetrics = async () => {
    try {
      setLoading(true);
      setError(null);

      const result = await invoke<DashboardMetrics>('get_pr_based_metrics', {
        days,
      });

      setMetrics(result);
    } catch (err) {
      console.error('Failed to fetch PR metrics:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchMetrics();

    if (autoRefresh && refreshInterval > 0) {
      const interval = setInterval(fetchMetrics, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [days, autoRefresh, refreshInterval]);

  return {
    metrics,
    loading,
    error,
    refresh: fetchMetrics,
  };
}

/**
 * Hook for multiple time periods comparison
 *
 * @example
 * ```tsx
 * function ComparisonView() {
 *   const current = usePRMetrics({ days: 30 });
 *   const previous = usePRMetrics({ days: 60 });
 *
 *   if (current.loading || previous.loading) return <div>Loading...</div>;
 *
 *   const currentMultiplier = current.metrics?.overview.productivity_multiplier || 0;
 *   const previousMultiplier = previous.metrics?.overview.productivity_multiplier || 0;
 *   const change = ((currentMultiplier - previousMultiplier) / previousMultiplier * 100).toFixed(1);
 *
 *   return <div>Productivity change: {change}%</div>;
 * }
 * ```
 */
export function usePRMetricsComparison(
  currentDays: number,
  previousDays: number
): {
  current: UsePRMetricsResult;
  previous: UsePRMetricsResult;
} {
  const current = usePRMetrics({ days: currentDays });
  const previous = usePRMetrics({ days: previousDays });

  return { current, previous };
}
