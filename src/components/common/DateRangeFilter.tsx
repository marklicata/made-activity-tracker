import { useState, useEffect } from 'react';
import { Calendar } from 'lucide-react';

interface DateRangeFilterProps {
  startDate: string;
  endDate: string;
  onDateRangeChange: (startDate: string, endDate: string) => void;
}

export default function DateRangeFilter({
  startDate,
  endDate,
  onDateRangeChange,
}: DateRangeFilterProps) {
  const [localStartDate, setLocalStartDate] = useState(startDate);
  const [localEndDate, setLocalEndDate] = useState(endDate);
  const [preset, setPreset] = useState<string>('custom');

  useEffect(() => {
    setLocalStartDate(startDate);
    setLocalEndDate(endDate);
  }, [startDate, endDate]);

  const applyPreset = (days: number) => {
    const end = new Date();
    const start = new Date();
    start.setDate(start.getDate() - days);

    const startStr = start.toISOString().split('T')[0];
    const endStr = end.toISOString().split('T')[0];

    setLocalStartDate(startStr);
    setLocalEndDate(endStr);
    onDateRangeChange(startStr, endStr);
  };

  const handlePresetClick = (presetName: string, days: number) => {
    setPreset(presetName);
    applyPreset(days);
  };

  const handleApplyCustom = () => {
    setPreset('custom');
    onDateRangeChange(localStartDate, localEndDate);
  };

  return (
    <div className="bg-white rounded-lg shadow p-4">
      <div className="flex items-center gap-2 mb-3">
        <Calendar className="w-4 h-4 text-gray-600" />
        <h3 className="text-sm font-semibold text-gray-900">Date Range</h3>
      </div>

      {/* Quick Presets */}
      <div className="flex flex-wrap gap-2 mb-4">
        <button
          onClick={() => handlePresetClick('7d', 7)}
          className={`px-3 py-1 text-xs rounded-md transition-colors ${
            preset === '7d'
              ? 'bg-blue-600 text-white'
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
          }`}
        >
          Last 7 days
        </button>
        <button
          onClick={() => handlePresetClick('30d', 30)}
          className={`px-3 py-1 text-xs rounded-md transition-colors ${
            preset === '30d'
              ? 'bg-blue-600 text-white'
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
          }`}
        >
          Last 30 days
        </button>
        <button
          onClick={() => handlePresetClick('90d', 90)}
          className={`px-3 py-1 text-xs rounded-md transition-colors ${
            preset === '90d'
              ? 'bg-blue-600 text-white'
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
          }`}
        >
          Last 90 days
        </button>
        <button
          onClick={() => handlePresetClick('180d', 180)}
          className={`px-3 py-1 text-xs rounded-md transition-colors ${
            preset === '180d'
              ? 'bg-blue-600 text-white'
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
          }`}
        >
          Last 6 months
        </button>
      </div>

      {/* Custom Date Range */}
      <div className="space-y-2">
        <div>
          <label className="block text-xs text-gray-600 mb-1">Start Date</label>
          <input
            type="date"
            value={localStartDate}
            onChange={(e) => setLocalStartDate(e.target.value)}
            className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label className="block text-xs text-gray-600 mb-1">End Date</label>
          <input
            type="date"
            value={localEndDate}
            onChange={(e) => setLocalEndDate(e.target.value)}
            className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <button
          onClick={handleApplyCustom}
          className="w-full px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 transition-colors"
        >
          Apply Custom Range
        </button>
      </div>
    </div>
  );
}
