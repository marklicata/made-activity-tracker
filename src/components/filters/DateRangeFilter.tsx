import { useState, useRef, useEffect } from 'react';
import { Calendar, ChevronDown } from 'lucide-react';
import clsx from 'clsx';
import { useDashboardFilterStore } from '@stores/dashboardFilterStore';
import { formatDateRange, DATE_RANGE_PRESETS } from '@types/filters';

interface DateRangeOption {
  label: string;
  days: number;
}

const DATE_OPTIONS: DateRangeOption[] = [
  { label: 'Last 7 days', days: DATE_RANGE_PRESETS.LAST_7_DAYS },
  { label: 'Last 30 days', days: DATE_RANGE_PRESETS.LAST_30_DAYS },
  { label: 'Last 90 days', days: DATE_RANGE_PRESETS.LAST_90_DAYS },
  { label: 'Last 6 months', days: DATE_RANGE_PRESETS.LAST_180_DAYS },
  { label: 'Last year', days: DATE_RANGE_PRESETS.LAST_365_DAYS },
];

export default function DateRangeFilter() {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const { filters, setDateRangePreset } = useDashboardFilterStore();

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen]);

  const handleSelectOption = (days: number) => {
    setDateRangePreset(days);
    setIsOpen(false);
  };

  const displayText = filters.dateRange
    ? formatDateRange(filters.dateRange)
    : 'Select date range';

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={clsx(
          'flex items-center gap-2 px-4 py-2 rounded-lg border text-sm font-medium transition-colors',
          isOpen
            ? 'bg-primary-50 border-primary-300 text-primary-700'
            : 'bg-white border-gray-300 text-gray-700 hover:bg-gray-50'
        )}
      >
        <Calendar size={16} />
        <span>{displayText}</span>
        <ChevronDown
          size={16}
          className={clsx('transition-transform', isOpen && 'rotate-180')}
        />
      </button>

      {isOpen && (
        <div className="absolute top-full left-0 mt-2 w-64 bg-white border border-gray-200 rounded-lg shadow-lg z-50">
          <div className="p-2">
            {DATE_OPTIONS.map((option) => (
              <button
                key={option.days}
                onClick={() => handleSelectOption(option.days)}
                className="w-full text-left px-3 py-2 rounded-lg text-sm text-gray-700 hover:bg-gray-100 transition-colors"
              >
                {option.label}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
