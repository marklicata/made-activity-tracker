import { CheckCircle, FileText, Bug, Sparkles, Clock } from 'lucide-react';
import type { QualityMetrics } from '@/types/metrics';
import { BenchmarkMetricCard } from './BenchmarkMetricCard';
import { DistributionChart } from './DistributionChart';
import { formatMetricComparison, formatHours } from '@/types/metrics';

interface QualitySectionProps {
  quality: QualityMetrics;
}

export function QualitySection({ quality }: QualitySectionProps) {
  // Format PR type distribution for chart
  const prTypeColors: Record<string, 'green' | 'red' | 'blue' | 'purple' | 'yellow' | 'gray'> = {
    feature: 'green',
    bug_fix: 'red',
    refactor: 'blue',
    test: 'purple',
    docs: 'yellow',
    other: 'gray',
  };

  const prTypeData = quality.pr_type_distribution.map((type) => ({
    label: type.pr_type.replace('_', ' ').replace(/\b\w/g, (l) => l.toUpperCase()),
    percentage: type.percentage,
    count: type.count,
    color: prTypeColors[type.pr_type] || 'gray',
  }));

  // Format files per PR distribution
  const filesData = [
    {
      label: '1-3 files',
      percentage: quality.files_per_pr_distribution.range_1_3_pct,
      count: quality.files_per_pr_distribution.range_1_3,
      color: 'green' as const,
    },
    {
      label: '4-8 files',
      percentage: quality.files_per_pr_distribution.range_4_8_pct,
      count: quality.files_per_pr_distribution.range_4_8,
      color: 'blue' as const,
    },
    {
      label: '9-15 files',
      percentage: quality.files_per_pr_distribution.range_9_15_pct,
      count: quality.files_per_pr_distribution.range_9_15,
      color: 'yellow' as const,
    },
    {
      label: '16+ files',
      percentage: quality.files_per_pr_distribution.range_16_plus_pct,
      count: quality.files_per_pr_distribution.range_16_plus,
      color: 'red' as const,
    },
  ];

  return (
    <section>
      <h2 className="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
        <Sparkles className="text-purple-500" size={20} />
        Quality
        <span className="text-sm font-normal text-gray-500 ml-2">
          Standard of work output
        </span>
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4 mb-6">
        <BenchmarkMetricCard
          title="PR Merge Rate"
          value={`${quality.pr_merge_rate.toFixed(1)}%`}
          subtitle="PRs successfully merged"
          comparison={formatMetricComparison(
            quality.pr_merge_rate,
            quality.benchmark_comparison.merge_rate_industry,
            quality.benchmark_comparison.merge_rate_elite,
            true
          )}
          icon={CheckCircle}
          color="quality"
        />

        <BenchmarkMetricCard
          title="Bug PR Ratio"
          value={`${quality.bug_pr_percentage.toFixed(1)}%`}
          subtitle="Lower is better"
          comparison={formatMetricComparison(
            quality.bug_pr_percentage,
            quality.benchmark_comparison.bug_ratio_industry,
            quality.benchmark_comparison.bug_ratio_elite,
            false // lower is better
          )}
          icon={Bug}
          color="quality"
        />

        <BenchmarkMetricCard
          title="Feature Work"
          value={`${quality.feature_pr_percentage.toFixed(1)}%`}
          subtitle="New functionality"
          icon={Sparkles}
          color="quality"
        />

        <BenchmarkMetricCard
          title="Files per PR"
          value={quality.avg_files_per_pr.toFixed(1)}
          subtitle="Scope size indicator"
          comparison={formatMetricComparison(
            quality.avg_files_per_pr,
            quality.benchmark_comparison.files_per_pr_industry,
            quality.benchmark_comparison.files_per_pr_industry,
            false // staying close to industry is good
          )}
          icon={FileText}
          color="quality"
        />

        <BenchmarkMetricCard
          title="Review Cycle"
          value={formatHours(quality.avg_review_cycle_hours)}
          subtitle="Time to first review"
          icon={Clock}
          color="quality"
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <DistributionChart title="PR Type Distribution" data={prTypeData} />
        <DistributionChart title="Files per PR Distribution" data={filesData} />
      </div>
    </section>
  );
}
