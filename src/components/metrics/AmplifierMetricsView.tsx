import { useState } from 'react';
import { Loader2, Info } from 'lucide-react';
import { usePRMetrics } from '@/hooks/usePRMetrics';
import { ProductivityOverview } from './ProductivityOverview';
import { SpeedSection } from './SpeedSection';
import { EaseSection } from './EaseSection';
import { QualitySection } from './QualitySection';

interface AmplifierMetricsViewProps {
  days?: number;
}

export function AmplifierMetricsView({ days = 30 }: AmplifierMetricsViewProps) {
  const { metrics, loading, error } = usePRMetrics({ days });
  const [selectedPeriod, setSelectedPeriod] = useState(days);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="flex items-center gap-2 text-gray-400">
          <Loader2 className="animate-spin" size={20} />
          Loading PR-based metrics...
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-6">
        <h3 className="text-red-800 font-semibold flex items-center gap-2 mb-2">
          <Info size={18} />
          Error loading metrics
        </h3>
        <p className="text-red-600 text-sm">{error}</p>
      </div>
    );
  }

  if (!metrics) {
    return (
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
        <p className="text-gray-600">No metrics available. Please sync your data first.</p>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      {/* Info Banner */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <div className="flex items-start gap-3">
          <Info className="text-blue-600 flex-shrink-0 mt-0.5" size={18} />
          <div className="flex-1">
            <h3 className="font-medium text-blue-900 mb-1">Amplifier-Style Metrics</h3>
            <p className="text-sm text-blue-800">
              These metrics use PR activity data to calculate productivity compared to industry benchmarks.
              All calculations are based on your synced PR data with no additional API calls.
            </p>
          </div>
        </div>
      </div>

      {/* Time Period Selector */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-800">
          Performance Analysis ({metrics.overview.period_days} days)
        </h2>
        <div className="flex gap-2">
          {[7, 30, 90].map((period) => (
            <button
              key={period}
              onClick={() => setSelectedPeriod(period)}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                selectedPeriod === period
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              {period}d
            </button>
          ))}
        </div>
      </div>

      {/* Productivity Overview */}
      <ProductivityOverview overview={metrics.overview} />

      {/* Speed Metrics */}
      <SpeedSection speed={metrics.speed} />

      {/* Ease Metrics */}
      <EaseSection ease={metrics.ease} />

      {/* Quality Metrics */}
      <QualitySection quality={metrics.quality} />

      {/* Methodology Footer */}
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-6 text-sm text-gray-600">
        <h4 className="font-semibold text-gray-800 mb-2">Methodology & Benchmarks</h4>
        <div className="space-y-1">
          <p>
            <strong>Industry benchmarks:</strong> Based on GitHub Octoverse 2023, Google DORA State of DevOps Research,
            and LinearB Engineering Benchmarks 2024.
          </p>
          <p>
            <strong>Productivity multiplier:</strong> Calculated as weighted average: 35% PR Velocity + 25% PR Speed +
            25% Repo Capacity + 15% Quality.
          </p>
          <p>
            <strong>Data source:</strong> All metrics calculated from synced PR data. No commit-level data required.
          </p>
        </div>
      </div>
    </div>
  );
}
