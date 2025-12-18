import { ActivityDataPoint } from '@/types';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

interface UserActivityTrendProps {
  data: ActivityDataPoint[];
  username: string;
}

export default function UserActivityTrend({ data, username }: UserActivityTrendProps) {
  if (data.length === 0) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Activity Trend</h2>
        <p className="text-gray-500">No activity data available for the selected time period.</p>
      </div>
    );
  }

  // Format timestamp for display
  const formatTimestamp = (timestamp: string) => {
    if (timestamp.includes('W')) {
      // Week format: 2024-W12
      return timestamp;
    } else if (timestamp.length === 7) {
      // Month format: 2024-03
      return timestamp;
    } else {
      // Day format: 2024-03-15
      const date = new Date(timestamp);
      return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    }
  };

  // Calculate trend direction
  const calculateTrend = () => {
    if (data.length < 2) return null;

    const firstHalf = data.slice(0, Math.floor(data.length / 2));
    const secondHalf = data.slice(Math.floor(data.length / 2));

    const firstAvg = firstHalf.reduce((sum, d) => sum + d.total_activity, 0) / firstHalf.length;
    const secondAvg = secondHalf.reduce((sum, d) => sum + d.total_activity, 0) / secondHalf.length;

    const change = ((secondAvg - firstAvg) / firstAvg) * 100;

    if (Math.abs(change) < 10) return { label: 'Stable', color: 'text-gray-600', icon: '→' };
    if (change > 0) return { label: 'Increasing', color: 'text-green-600', icon: '↑' };
    return { label: 'Decreasing', color: 'text-red-600', icon: '↓' };
  };

  const trend = calculateTrend();

  // Calculate stats
  const totalActivity = data.reduce((sum, d) => sum + d.total_activity, 0);
  const avgActivity = (totalActivity / data.length).toFixed(1);
  const maxActivity = Math.max(...data.map(d => d.total_activity));

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900">Activity Trend</h2>
          {trend && (
            <div className={`flex items-center gap-1 text-sm font-medium ${trend.color}`}>
              <span>{trend.icon}</span>
              <span>{trend.label}</span>
            </div>
          )}
        </div>

        {/* Stats */}
        <div className="grid grid-cols-3 gap-4 mb-6">
          <div className="text-center p-3 bg-gray-50 rounded-lg">
            <p className="text-2xl font-bold text-gray-900">{totalActivity}</p>
            <p className="text-xs text-gray-500">Total Activity</p>
          </div>
          <div className="text-center p-3 bg-gray-50 rounded-lg">
            <p className="text-2xl font-bold text-gray-900">{avgActivity}</p>
            <p className="text-xs text-gray-500">Avg per Period</p>
          </div>
          <div className="text-center p-3 bg-gray-50 rounded-lg">
            <p className="text-2xl font-bold text-gray-900">{maxActivity}</p>
            <p className="text-xs text-gray-500">Peak Activity</p>
          </div>
        </div>

        {/* Chart */}
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={data} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis
              dataKey="timestamp"
              tickFormatter={formatTimestamp}
              angle={-45}
              textAnchor="end"
              height={80}
            />
            <YAxis />
            <Tooltip
              labelFormatter={formatTimestamp}
              formatter={(value: number, name: string) => {
                const nameMap: Record<string, string> = {
                  pr_count: 'PRs',
                  review_count: 'Reviews',
                  issue_count: 'Issues',
                  total_activity: 'Total',
                };
                return [value, nameMap[name] || name];
              }}
            />
            <Legend
              formatter={(value: string) => {
                const nameMap: Record<string, string> = {
                  pr_count: 'PRs',
                  review_count: 'Reviews',
                  issue_count: 'Issues',
                  total_activity: 'Total Activity',
                };
                return nameMap[value] || value;
              }}
            />
            <Line type="monotone" dataKey="pr_count" stroke="#3b82f6" strokeWidth={2} name="pr_count" />
            <Line type="monotone" dataKey="review_count" stroke="#8b5cf6" strokeWidth={2} name="review_count" />
            <Line type="monotone" dataKey="issue_count" stroke="#f59e0b" strokeWidth={2} name="issue_count" />
            <Line
              type="monotone"
              dataKey="total_activity"
              stroke="#10b981"
              strokeWidth={3}
              strokeDasharray="5 5"
              name="total_activity"
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
