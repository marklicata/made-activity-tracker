import { TrendingUp, Zap, Users as UsersIcon, CheckCircle } from 'lucide-react';
import clsx from 'clsx';
import type { OverviewMetrics } from '@/types/metrics';
import { formatMetricComparison, getTierColor, getTierLabel } from '@/types/metrics';

interface ProductivityOverviewProps {
  overview: OverviewMetrics;
}

export function ProductivityOverview({ overview }: ProductivityOverviewProps) {
  const multiplier = overview.productivity_multiplier ?? 0;

  // Determine tier for visual styling
  let tier: 'below' | 'industry' | 'elite' | 'exceptional';
  let tierColor: string;

  if (multiplier < 0.8) {
    tier = 'below';
    tierColor = 'from-red-500 to-orange-500';
  } else if (multiplier < 1.5) {
    tier = 'industry';
    tierColor = 'from-yellow-500 to-orange-500';
  } else if (multiplier < 3) {
    tier = 'elite';
    tierColor = 'from-blue-500 to-indigo-500';
  } else {
    tier = 'exceptional';
    tierColor = 'from-purple-500 to-pink-500';
  }

  const getTierText = () => {
    if (multiplier < 0.8) return 'Below Industry Average';
    if (multiplier < 1.5) return 'Industry Average Performance';
    if (multiplier < 3) return 'Elite Tier Performance';
    return 'Exceptional Performance';
  };

  return (
    <div className={clsx('rounded-xl p-8 text-white shadow-lg bg-gradient-to-br', tierColor)}>
      <div className="flex items-start justify-between mb-6">
        <div>
          <h2 className="text-lg font-medium opacity-90 mb-1">Team Productivity Multiplier</h2>
          <p className="text-sm opacity-75">
            Compared to industry benchmarks
          </p>
        </div>
        <div className="bg-white/20 p-3 rounded-lg backdrop-blur-sm">
          <TrendingUp size={24} />
        </div>
      </div>

      <div className="flex items-baseline gap-4 mb-6">
        <div className="text-6xl font-bold">
          {multiplier.toFixed(1)}×
        </div>
        <div className="flex flex-col">
          <span className="text-xl font-semibold">{getTierText()}</span>
          <span className="text-sm opacity-75">
            {multiplier > 1 ? `${((multiplier - 1) * 100).toFixed(0)}% above baseline` : `${((1 - multiplier) * 100).toFixed(0)}% below baseline`}
          </span>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4 pt-6 border-t border-white/20">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <Zap size={16} className="opacity-75" />
            <span className="text-xs font-medium opacity-75">Period</span>
          </div>
          <div className="text-xl font-semibold">{overview.period_days} days</div>
        </div>
        <div>
          <div className="flex items-center gap-2 mb-1">
            <CheckCircle size={16} className="opacity-75" />
            <span className="text-xs font-medium opacity-75">Total PRs</span>
          </div>
          <div className="text-xl font-semibold">{overview.total_prs}</div>
        </div>
        <div>
          <div className="flex items-center gap-2 mb-1">
            <UsersIcon size={16} className="opacity-75" />
            <span className="text-xs font-medium opacity-75">Active Devs</span>
          </div>
          <div className="text-xl font-semibold">{overview.active_developers}</div>
        </div>
      </div>

      <div className="mt-6 p-4 bg-white/10 rounded-lg backdrop-blur-sm">
        <div className="text-xs font-medium mb-2 opacity-90">Formula Breakdown:</div>
        <div className="text-xs opacity-75 leading-relaxed">
          35% PR Velocity + 25% PR Speed + 25% Repo Capacity + 15% Quality
        </div>
      </div>
    </div>
  );
}
