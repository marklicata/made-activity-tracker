import React from 'react';
import { usePRMetrics } from '../hooks/usePRMetrics';
import {
  formatMetricComparison,
  getTierColor,
  getTierLabel,
  formatHours,
} from '../types/metrics';

/**
 * Example dashboard component showing how to use PR-based metrics
 *
 * This is a reference implementation. Customize the UI to match your design system.
 */
export function PRBasedDashboard() {
  const { metrics, loading, error, refresh } = usePRMetrics({ days: 30 });

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-lg">Loading metrics...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <h3 className="text-red-800 font-semibold">Error loading metrics</h3>
        <p className="text-red-600">{error}</p>
        <button
          onClick={refresh}
          className="mt-2 px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
        >
          Retry
        </button>
      </div>
    );
  }

  if (!metrics) return null;

  const { speed, ease, quality, overview } = metrics;

  return (
    <div className="space-y-6 p-6">
      {/* Overview Section */}
      <div className="bg-gradient-to-r from-purple-500 to-blue-500 rounded-lg p-6 text-white">
        <h1 className="text-3xl font-bold mb-2">Productivity Dashboard</h1>
        <div className="flex items-baseline gap-2">
          <span className="text-5xl font-bold">
            {overview.productivity_multiplier.toFixed(1)}×
          </span>
          <span className="text-xl">Productivity Multiplier</span>
        </div>
        <div className="mt-4 flex gap-6 text-sm">
          <div>
            <span className="opacity-80">Period:</span>{' '}
            <span className="font-semibold">{overview.period_days} days</span>
          </div>
          <div>
            <span className="opacity-80">Total PRs:</span>{' '}
            <span className="font-semibold">{overview.total_prs}</span>
          </div>
          <div>
            <span className="opacity-80">Active Developers:</span>{' '}
            <span className="font-semibold">{overview.active_developers}</span>
          </div>
        </div>
      </div>

      {/* Main Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* SPEED */}
        <MetricCard title="Speed" color="blue">
          <MetricValue
            label="PRs per Day (per dev)"
            value={speed.prs_per_day_per_dev}
            comparison={formatMetricComparison(
              speed.prs_per_day_per_dev,
              speed.benchmark_comparison.prs_per_day_industry,
              speed.benchmark_comparison.prs_per_day_elite,
              true
            )}
            format={(v) => v.toFixed(2)}
          />
          <MetricValue
            label="PR Turnaround"
            value={speed.pr_turnaround_hours}
            comparison={formatMetricComparison(
              speed.pr_turnaround_hours,
              speed.benchmark_comparison.pr_turnaround_industry,
              speed.benchmark_comparison.pr_turnaround_elite,
              false // lower is better
            )}
            format={formatHours}
          />
          <MetricValue
            label="Lines of Code per Day"
            value={speed.loc_per_day}
            format={(v) => Math.round(v).toLocaleString()}
          />
        </MetricCard>

        {/* EASE */}
        <MetricCard title="Ease" color="green">
          <MetricValue
            label="Concurrent Repos"
            value={ease.concurrent_repos}
            comparison={formatMetricComparison(
              ease.concurrent_repos,
              ease.benchmark_comparison.concurrent_repos_industry,
              ease.benchmark_comparison.concurrent_repos_elite,
              true
            )}
            format={(v) => Math.round(v).toString()}
          />
          <MetricValue
            label="Repos per Developer"
            value={ease.repos_per_dev}
            format={(v) => v.toFixed(1)}
          />
          <MetricValue
            label="Context Switch Rate"
            value={ease.pr_switch_frequency}
            format={(v) => `${v.toFixed(1)}%`}
          />
        </MetricCard>

        {/* QUALITY */}
        <MetricCard title="Quality" color="purple">
          <MetricValue
            label="PR Merge Rate"
            value={quality.pr_merge_rate}
            comparison={formatMetricComparison(
              quality.pr_merge_rate,
              quality.benchmark_comparison.merge_rate_industry,
              quality.benchmark_comparison.merge_rate_elite,
              true
            )}
            format={(v) => `${v.toFixed(1)}%`}
          />
          <MetricValue
            label="Bug PR Ratio"
            value={quality.bug_pr_percentage}
            comparison={formatMetricComparison(
              quality.bug_pr_percentage,
              quality.benchmark_comparison.bug_ratio_industry,
              quality.benchmark_comparison.bug_ratio_elite,
              false // lower is better
            )}
            format={(v) => `${v.toFixed(1)}%`}
          />
          <MetricValue
            label="Files per PR"
            value={quality.avg_files_per_pr}
            format={(v) => v.toFixed(1)}
          />
        </MetricCard>
      </div>

      {/* PR Cycle Time Distribution */}
      <div className="bg-white rounded-lg shadow p-6">
        <h3 className="text-lg font-semibold mb-4">PR Cycle Time Distribution</h3>
        <div className="space-y-2">
          <DistributionBar
            label="< 4 hours"
            percentage={speed.cycle_time_distribution.under_4h_pct}
            count={speed.cycle_time_distribution.under_4h}
            color="green"
          />
          <DistributionBar
            label="4-12 hours"
            percentage={speed.cycle_time_distribution.h4_to_12_pct}
            count={speed.cycle_time_distribution.h4_to_12}
            color="blue"
          />
          <DistributionBar
            label="12-24 hours"
            percentage={speed.cycle_time_distribution.h12_to_24_pct}
            count={speed.cycle_time_distribution.h12_to_24}
            color="yellow"
          />
          <DistributionBar
            label="> 24 hours"
            percentage={speed.cycle_time_distribution.over_24h_pct}
            count={speed.cycle_time_distribution.over_24h}
            color="red"
          />
        </div>
      </div>

      {/* Active Repositories */}
      <div className="bg-white rounded-lg shadow p-6">
        <h3 className="text-lg font-semibold mb-4">
          Active Repositories (Top {Math.min(10, ease.active_repos.length)})
        </h3>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="text-left border-b">
                <th className="pb-2">Repository</th>
                <th className="pb-2 text-right">PRs</th>
                <th className="pb-2 text-right">LOC</th>
                <th className="pb-2 text-right">Contributors</th>
              </tr>
            </thead>
            <tbody>
              {ease.active_repos.slice(0, 10).map((repo) => (
                <tr key={repo.repo_name} className="border-b last:border-0">
                  <td className="py-2 font-mono text-sm">{repo.repo_name}</td>
                  <td className="py-2 text-right">{repo.pr_count}</td>
                  <td className="py-2 text-right">{repo.total_loc.toLocaleString()}</td>
                  <td className="py-2 text-right">{repo.contributor_count}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* PR Type Distribution */}
      <div className="bg-white rounded-lg shadow p-6">
        <h3 className="text-lg font-semibold mb-4">PR Type Distribution</h3>
        <div className="space-y-2">
          {quality.pr_type_distribution.map((type) => (
            <DistributionBar
              key={type.pr_type}
              label={type.pr_type.replace('_', ' ')}
              percentage={type.percentage}
              count={type.count}
              color={getPRTypeColor(type.pr_type)}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

// Helper Components

function MetricCard({
  title,
  color,
  children,
}: {
  title: string;
  color: 'blue' | 'green' | 'purple';
  children: React.ReactNode;
}) {
  const colorClasses = {
    blue: 'border-blue-200 bg-blue-50',
    green: 'border-green-200 bg-green-50',
    purple: 'border-purple-200 bg-purple-50',
  };

  return (
    <div className={`rounded-lg border-2 ${colorClasses[color]} p-6`}>
      <h2 className="text-xl font-bold mb-4">{title}</h2>
      <div className="space-y-4">{children}</div>
    </div>
  );
}

function MetricValue({
  label,
  value,
  comparison,
  format = (v) => v.toString(),
}: {
  label: string;
  value: number;
  comparison?: {
    tier: string;
    vs_industry_pct: number;
    vs_elite_pct: number;
  };
  format?: (value: number) => string;
}) {
  return (
    <div>
      <div className="text-sm text-gray-600 mb-1">{label}</div>
      <div className="flex items-baseline gap-2">
        <span className="text-2xl font-bold">{format(value)}</span>
        {comparison && (
          <span
            className="text-xs px-2 py-1 rounded"
            style={{
              backgroundColor: getTierColor(comparison.tier as any),
              color: 'white',
            }}
          >
            {getTierLabel(comparison.tier as any)}
          </span>
        )}
      </div>
      {comparison && (
        <div className="text-xs text-gray-500 mt-1">
          {comparison.vs_industry_pct > 0 ? '+' : ''}
          {comparison.vs_industry_pct.toFixed(0)}% vs industry
        </div>
      )}
    </div>
  );
}

function DistributionBar({
  label,
  percentage,
  count,
  color,
}: {
  label: string;
  percentage: number;
  count: number;
  color: 'green' | 'blue' | 'yellow' | 'red' | 'purple' | 'gray';
}) {
  const colorClasses = {
    green: 'bg-green-500',
    blue: 'bg-blue-500',
    yellow: 'bg-yellow-500',
    red: 'bg-red-500',
    purple: 'bg-purple-500',
    gray: 'bg-gray-500',
  };

  return (
    <div>
      <div className="flex justify-between text-sm mb-1">
        <span>{label}</span>
        <span className="text-gray-600">
          {percentage.toFixed(1)}% ({count})
        </span>
      </div>
      <div className="w-full bg-gray-200 rounded-full h-2">
        <div
          className={`${colorClasses[color]} h-2 rounded-full transition-all`}
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  );
}

function getPRTypeColor(
  type: string
): 'green' | 'blue' | 'yellow' | 'red' | 'purple' | 'gray' {
  switch (type) {
    case 'feature':
      return 'green';
    case 'bug_fix':
      return 'red';
    case 'refactor':
      return 'blue';
    case 'test':
      return 'purple';
    case 'docs':
      return 'yellow';
    default:
      return 'gray';
  }
}

export default PRBasedDashboard;
