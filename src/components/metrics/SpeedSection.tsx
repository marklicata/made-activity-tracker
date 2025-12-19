import { Zap, GitPullRequest, Code, Clock } from 'lucide-react';
import type { SpeedMetrics } from '@/types/metrics';
import { BenchmarkMetricCard } from './BenchmarkMetricCard';
import { DistributionChart } from './DistributionChart';
import { formatMetricComparison, formatHours } from '@/types/metrics';

interface SpeedSectionProps {
  speed: SpeedMetrics;
}

export function SpeedSection({ speed }: SpeedSectionProps) {
  // Format cycle time distribution for chart
  const cycleTimeData = [
    {
      label: '< 4 hours',
      percentage: speed.cycle_time_distribution.under_4h_pct,
      count: speed.cycle_time_distribution.under_4h,
      color: 'green' as const,
    },
    {
      label: '4-12 hours',
      percentage: speed.cycle_time_distribution.h4_to_12_pct,
      count: speed.cycle_time_distribution.h4_to_12,
      color: 'blue' as const,
    },
    {
      label: '12-24 hours',
      percentage: speed.cycle_time_distribution.h12_to_24_pct,
      count: speed.cycle_time_distribution.h12_to_24,
      color: 'yellow' as const,
    },
    {
      label: '> 24 hours',
      percentage: speed.cycle_time_distribution.over_24h_pct,
      count: speed.cycle_time_distribution.over_24h,
      color: 'red' as const,
    },
  ];

  return (
    <section>
      <h2 className="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
        <Zap className="text-blue-500" size={20} />
        Speed
        <span className="text-sm font-normal text-gray-500 ml-2">
          How fast work gets done
        </span>
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <BenchmarkMetricCard
          title="PRs per Day"
          value={speed.prs_per_day_per_dev.toFixed(2)}
          subtitle="Per developer"
          comparison={formatMetricComparison(
            speed.prs_per_day_per_dev,
            speed.benchmark_comparison.prs_per_day_industry,
            speed.benchmark_comparison.prs_per_day_elite,
            true
          )}
          icon={GitPullRequest}
          color="speed"
        />

        <BenchmarkMetricCard
          title="PR Turnaround"
          value={formatHours(speed.pr_turnaround_hours)}
          subtitle="Open → merged"
          comparison={formatMetricComparison(
            speed.pr_turnaround_hours,
            speed.benchmark_comparison.pr_turnaround_industry,
            speed.benchmark_comparison.pr_turnaround_elite,
            false // lower is better
          )}
          icon={Clock}
          color="speed"
        />

        <BenchmarkMetricCard
          title="Lines of Code"
          value={Math.round(speed.loc_per_day).toLocaleString()}
          subtitle="Per day (team)"
          icon={Code}
          color="speed"
        />

        <BenchmarkMetricCard
          title="Total PRs"
          value={speed.prs_per_day.toFixed(1)}
          subtitle="Per day (all devs)"
          icon={GitPullRequest}
          color="speed"
        />
      </div>

      <DistributionChart title="PR Merge Time Distribution" data={cycleTimeData} />
    </section>
  );
}
