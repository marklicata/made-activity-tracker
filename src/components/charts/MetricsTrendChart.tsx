import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { format, parseISO } from 'date-fns';
import { TimeseriesDataPoint } from '@types/filters';

interface MetricsTrendChartProps {
  data: TimeseriesDataPoint[];
  metric: 'speed' | 'ease' | 'quality';
  title: string;
}

// Color schemes for each metric category
const METRIC_COLORS = {
  speed: {
    cycleTime: '#3b82f6', // blue-500
    leadTime: '#60a5fa', // blue-400
    deploymentFrequency: '#93c5fd', // blue-300
  },
  ease: {
    prApprovalTime: '#10b981', // green-500
    codeReviewLoad: '#34d399', // green-400
    prSize: '#6ee7b7', // green-300
  },
  quality: {
    changeFailureRate: '#8b5cf6', // purple-500
    bugFixRate: '#a78bfa', // purple-400
    testCoverage: '#c4b5fd', // purple-300
  },
};

// Metric labels
const METRIC_LABELS = {
  speed: {
    cycleTime: 'Cycle Time (days)',
    leadTime: 'Lead Time (days)',
    deploymentFrequency: 'Deploy Freq (per week)',
  },
  ease: {
    prApprovalTime: 'PR Approval Time (hours)',
    codeReviewLoad: 'Code Review Load (PRs/dev)',
    prSize: 'PR Size (LOC)',
  },
  quality: {
    changeFailureRate: 'Change Failure Rate (%)',
    bugFixRate: 'Bug Fix Rate (%)',
    testCoverage: 'Test Coverage (%)',
  },
};

export default function MetricsTrendChart({ data, metric, title }: MetricsTrendChartProps) {
  if (!data || data.length === 0) {
    return (
      <div className="bg-white p-6 rounded-lg border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">{title}</h3>
        <div className="flex items-center justify-center h-64 text-gray-500">
          No data available for the selected period
        </div>
      </div>
    );
  }

  const colors = METRIC_COLORS[metric];
  const labels = METRIC_LABELS[metric];

  // Format data for the chart
  const chartData = data.map((point) => ({
    date: point.date,
    ...point[metric],
  }));

  // Custom tooltip
  const CustomTooltip = ({ active, payload, label }: any) => {
    if (!active || !payload) return null;

    return (
      <div className="bg-white p-3 border border-gray-200 rounded-lg shadow-lg">
        <p className="font-semibold text-gray-900 mb-2">
          {format(parseISO(label), 'MMM d, yyyy')}
        </p>
        {payload.map((entry: any, index: number) => (
          <div key={index} className="flex items-center gap-2 text-sm">
            <div
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: entry.color }}
            />
            <span className="text-gray-600">{entry.name}:</span>
            <span className="font-medium text-gray-900">
              {typeof entry.value === 'number' ? entry.value.toFixed(2) : entry.value}
            </span>
          </div>
        ))}
      </div>
    );
  };

  // Format X-axis date labels
  const formatXAxis = (dateStr: string) => {
    try {
      return format(parseISO(dateStr), 'MMM d');
    } catch {
      return dateStr;
    }
  };

  return (
    <div className="bg-white p-6 rounded-lg border border-gray-200">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">{title}</h3>
      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={chartData} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
          <XAxis
            dataKey="date"
            tickFormatter={formatXAxis}
            stroke="#6b7280"
            style={{ fontSize: '12px' }}
          />
          <YAxis stroke="#6b7280" style={{ fontSize: '12px' }} />
          <Tooltip content={<CustomTooltip />} />
          <Legend
            wrapperStyle={{ fontSize: '12px' }}
            iconType="line"
          />

          {/* Render lines based on metric type */}
          {Object.entries(colors).map(([key, color]) => (
            <Line
              key={key}
              type="monotone"
              dataKey={key}
              name={labels[key as keyof typeof labels]}
              stroke={color}
              strokeWidth={2}
              dot={{ fill: color, r: 3 }}
              activeDot={{ r: 5 }}
            />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
