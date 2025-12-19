import clsx from 'clsx';
import { TrendingUp, TrendingDown, Minus } from 'lucide-react';
import type { MetricComparison } from '@/types/metrics';
import { getTierColor, getTierLabel } from '@/types/metrics';

interface BenchmarkMetricCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  comparison?: MetricComparison;
  icon: React.ElementType;
  color: 'speed' | 'ease' | 'quality';
  format?: (value: number) => string;
}

export function BenchmarkMetricCard({
  title,
  value,
  subtitle,
  comparison,
  icon: Icon,
  color,
}: BenchmarkMetricCardProps) {
  const colorClasses = {
    speed: 'border-blue-500 bg-blue-50',
    ease: 'border-green-500 bg-green-50',
    quality: 'border-purple-500 bg-purple-50',
  };

  const iconColorClasses = {
    speed: 'text-blue-600 bg-blue-100',
    ease: 'text-green-600 bg-green-100',
    quality: 'text-purple-600 bg-purple-100',
  };

  return (
    <div className={clsx('p-6 rounded-xl border-l-4 bg-white shadow-sm', colorClasses[color])}>
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <p className="text-sm font-medium text-gray-500">{title}</p>
          <p className="text-2xl font-bold mt-1 text-gray-900">{value}</p>
          {subtitle && <p className="text-xs text-gray-400 mt-1">{subtitle}</p>}

          {comparison && (
            <div className="mt-3">
              <div className="flex items-center gap-2 mb-1">
                <span
                  className="text-xs px-2 py-0.5 rounded font-medium"
                  style={{
                    backgroundColor: getTierColor(comparison.tier),
                    color: 'white',
                  }}
                >
                  {getTierLabel(comparison.tier)}
                </span>
              </div>
              <div className="flex items-center gap-3 text-xs text-gray-500">
                <div className="flex items-center gap-1">
                  {comparison.vs_industry_pct > 0 ? (
                    <TrendingUp size={12} className="text-green-600" />
                  ) : comparison.vs_industry_pct < 0 ? (
                    <TrendingDown size={12} className="text-red-600" />
                  ) : (
                    <Minus size={12} className="text-gray-400" />
                  )}
                  <span
                    className={clsx(
                      'font-medium',
                      comparison.vs_industry_pct > 0 && 'text-green-600',
                      comparison.vs_industry_pct < 0 && 'text-red-600',
                      comparison.vs_industry_pct === 0 && 'text-gray-500'
                    )}
                  >
                    {comparison.vs_industry_pct > 0 ? '+' : ''}
                    {comparison.vs_industry_pct.toFixed(0)}%
                  </span>
                  <span>vs industry</span>
                </div>
                <div className="flex items-center gap-1">
                  {comparison.vs_elite_pct > 0 ? (
                    <TrendingUp size={12} className="text-green-600" />
                  ) : comparison.vs_elite_pct < 0 ? (
                    <TrendingDown size={12} className="text-red-600" />
                  ) : (
                    <Minus size={12} className="text-gray-400" />
                  )}
                  <span
                    className={clsx(
                      'font-medium',
                      comparison.vs_elite_pct > 0 && 'text-green-600',
                      comparison.vs_elite_pct < 0 && 'text-red-600',
                      comparison.vs_elite_pct === 0 && 'text-gray-500'
                    )}
                  >
                    {comparison.vs_elite_pct > 0 ? '+' : ''}
                    {comparison.vs_elite_pct.toFixed(0)}%
                  </span>
                  <span>vs elite</span>
                </div>
              </div>
            </div>
          )}
        </div>
        <div className={clsx('p-2 rounded-lg flex-shrink-0', iconColorClasses[color])}>
          <Icon size={20} />
        </div>
      </div>
    </div>
  );
}
