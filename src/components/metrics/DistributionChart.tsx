import clsx from 'clsx';

interface DistributionBarProps {
  label: string;
  percentage: number;
  count: number;
  color: 'green' | 'blue' | 'yellow' | 'red' | 'purple' | 'gray';
}

export function DistributionBar({ label, percentage, count, color }: DistributionBarProps) {
  const colorClasses = {
    green: 'bg-green-500',
    blue: 'bg-blue-500',
    yellow: 'bg-yellow-500',
    red: 'bg-red-500',
    purple: 'bg-purple-500',
    gray: 'bg-gray-500',
  };

  return (
    <div className="mb-3">
      <div className="flex justify-between text-sm mb-1.5">
        <span className="font-medium text-gray-700">{label}</span>
        <span className="text-gray-500">
          {percentage.toFixed(1)}% <span className="text-gray-400">({count})</span>
        </span>
      </div>
      <div className="w-full bg-gray-100 rounded-full h-2.5 overflow-hidden">
        <div
          className={clsx(colorClasses[color], 'h-2.5 rounded-full transition-all duration-500')}
          style={{ width: `${Math.min(percentage, 100)}%` }}
        />
      </div>
    </div>
  );
}

interface DistributionChartProps {
  title: string;
  data: Array<{
    label: string;
    percentage: number;
    count: number;
    color?: 'green' | 'blue' | 'yellow' | 'red' | 'purple' | 'gray';
  }>;
  className?: string;
}

export function DistributionChart({ title, data, className }: DistributionChartProps) {
  return (
    <div className={clsx('bg-white rounded-xl shadow-sm p-6', className)}>
      <h3 className="text-lg font-semibold text-gray-800 mb-4">{title}</h3>
      <div>
        {data.map((item, index) => (
          <DistributionBar
            key={item.label}
            label={item.label}
            percentage={item.percentage}
            count={item.count}
            color={item.color || 'blue'}
          />
        ))}
      </div>
    </div>
  );
}
