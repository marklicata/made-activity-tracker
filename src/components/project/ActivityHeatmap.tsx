import { Calendar, Clock, TrendingUp } from 'lucide-react';
import clsx from 'clsx';

interface ActivityHeatmapData {
  daily_counts: Record<string, number>;
  hourly_counts: Record<number, number>;
  weekday_counts: Record<string, number>;
}

interface ActivityHeatmapProps {
  data: ActivityHeatmapData;
}

export default function ActivityHeatmap({ data }: ActivityHeatmapProps) {
  const weekdays = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
  const hours = Array.from({ length: 24 }, (_, i) => i);

  // Get max values for scaling
  const maxHourly = Math.max(...Object.values(data.hourly_counts), 1);
  const maxWeekday = Math.max(...Object.values(data.weekday_counts), 1);

  function getIntensityColor(value: number, max: number): string {
    if (value === 0) return 'bg-gray-100';
    const intensity = value / max;
    if (intensity < 0.25) return 'bg-blue-200';
    if (intensity < 0.5) return 'bg-blue-400';
    if (intensity < 0.75) return 'bg-blue-600';
    return 'bg-blue-800';
  }

  function formatHour(hour: number): string {
    if (hour === 0) return '12 AM';
    if (hour === 12) return '12 PM';
    if (hour < 12) return `${hour} AM`;
    return `${hour - 12} PM`;
  }

  // Get daily activity for last 30 days (simplified calendar view)
  function getDailyActivity() {
    const days = [];
    const today = new Date();
    for (let i = 29; i >= 0; i--) {
      const date = new Date(today);
      date.setDate(date.getDate() - i);
      const dateStr = date.toISOString().split('T')[0];
      const count = data.daily_counts[dateStr] || 0;
      days.push({ date: dateStr, count, dayName: date.toLocaleDateString('en-US', { weekday: 'short' }) });
    }
    return days;
  }

  const dailyActivity = getDailyActivity();
  const maxDaily = Math.max(...dailyActivity.map((d) => d.count), 1);

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h2 className="text-lg font-semibold text-gray-900 mb-6 flex items-center gap-2">
        <TrendingUp className="w-5 h-5" />
        Activity Patterns
      </h2>

      <div className="space-y-8">
        {/* Daily Activity Calendar (Last 30 days) */}
        <div>
          <h3 className="text-sm font-medium text-gray-700 mb-3 flex items-center gap-2">
            <Calendar className="w-4 h-4" />
            Last 30 Days
          </h3>
          <div className="grid grid-cols-10 gap-1">
            {dailyActivity.map((day) => (
              <div
                key={day.date}
                className={clsx(
                  'h-10 rounded flex items-center justify-center text-xs font-medium transition-colors',
                  getIntensityColor(day.count, maxDaily),
                  day.count > 0 ? 'text-white' : 'text-gray-400'
                )}
                title={`${day.date}: ${day.count} events`}
              >
                {day.count > 0 && day.count}
              </div>
            ))}
          </div>
          <div className="flex items-center justify-between mt-2 text-xs text-gray-500">
            <span>Less activity</span>
            <div className="flex gap-1">
              <div className="w-3 h-3 rounded bg-gray-100" />
              <div className="w-3 h-3 rounded bg-blue-200" />
              <div className="w-3 h-3 rounded bg-blue-400" />
              <div className="w-3 h-3 rounded bg-blue-600" />
              <div className="w-3 h-3 rounded bg-blue-800" />
            </div>
            <span>More activity</span>
          </div>
        </div>

        {/* Hourly Activity */}
        <div>
          <h3 className="text-sm font-medium text-gray-700 mb-3 flex items-center gap-2">
            <Clock className="w-4 h-4" />
            Time of Day
          </h3>
          <div className="grid grid-cols-12 gap-1">
            {hours.map((hour) => {
              const count = data.hourly_counts[hour] || 0;
              return (
                <div key={hour} className="flex flex-col items-center gap-1">
                  <div
                    className={clsx(
                      'w-full h-16 rounded flex items-end justify-center text-xs font-medium transition-colors',
                      getIntensityColor(count, maxHourly),
                      count > 0 ? 'text-white pb-1' : 'text-gray-400'
                    )}
                    title={`${formatHour(hour)}: ${count} events`}
                  >
                    {count > 0 && count}
                  </div>
                  {hour % 3 === 0 && (
                    <span className="text-xs text-gray-500">{hour}</span>
                  )}
                </div>
              );
            })}
          </div>
        </div>

        {/* Weekday Activity */}
        <div>
          <h3 className="text-sm font-medium text-gray-700 mb-3">Day of Week</h3>
          <div className="grid grid-cols-7 gap-2">
            {weekdays.map((day) => {
              const count = data.weekday_counts[day] || 0;
              return (
                <div
                  key={day}
                  className={clsx(
                    'h-20 rounded-lg flex flex-col items-center justify-center transition-colors',
                    getIntensityColor(count, maxWeekday),
                    count > 0 ? 'text-white' : 'text-gray-400'
                  )}
                  title={`${day}: ${count} events`}
                >
                  <div className="text-xs font-medium mb-1">{day.slice(0, 3)}</div>
                  <div className="text-lg font-bold">{count}</div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}
