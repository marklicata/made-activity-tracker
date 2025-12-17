import { Package, Users, GitPullRequest, AlertCircle, CheckCircle2, RefreshCw } from 'lucide-react';
import clsx from 'clsx';

interface Repository {
  id: number;
  owner: string;
  name: string;
  github_id: number | null;
  enabled: boolean;
  last_synced_at: string | null;
}

interface ProjectSummary {
  total_contributors: number;
  total_commits: number;
  total_prs: number;
  total_issues: number;
  last_synced_at: string | null;
}

interface ProjectHeaderProps {
  repository: Repository;
  summary: ProjectSummary;
  dateRange: { start: string; end: string };
  onDateRangeChange: (start: string, end: string) => void;
}

export default function ProjectHeader({ repository, summary, dateRange, onDateRangeChange }: ProjectHeaderProps) {
  const presets = [
    { label: 'Last 7 days', days: 7 },
    { label: 'Last 30 days', days: 30 },
    { label: 'Last 90 days', days: 90 },
    { label: 'Last 6 months', days: 180 },
    { label: 'Last year', days: 365 },
  ];

  function handlePresetClick(days: number) {
    const end = new Date();
    const start = new Date();
    start.setDate(start.getDate() - days);
    onDateRangeChange(start.toISOString().split('T')[0], end.toISOString().split('T')[0]);
  }

  function formatDate(dateString: string) {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }

  function formatSyncTime(timestamp: string | null) {
    if (!timestamp) return 'Never synced';
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins} min ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    return `${diffDays}d ago`;
  }

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      {/* Repository info */}
      <div className="flex items-start justify-between mb-6">
        <div>
          <div className="flex items-center gap-2">
            <Package className="w-6 h-6 text-gray-600" />
            <h1 className="text-2xl font-bold text-gray-900">
              {repository.owner}/{repository.name}
            </h1>
          </div>
          <div className="flex items-center gap-2 mt-2">
            {repository.enabled ? (
              <CheckCircle2 className="w-4 h-4 text-green-500" />
            ) : (
              <AlertCircle className="w-4 h-4 text-gray-400" />
            )}
            <span className="text-sm text-gray-500">
              Last synced: {formatSyncTime(summary.last_synced_at)}
            </span>
          </div>
        </div>
        <button
          className="flex items-center gap-2 px-4 py-2 text-sm text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
          onClick={() => {
            // TODO: Trigger sync
          }}
        >
          <RefreshCw className="w-4 h-4" />
          Sync Now
        </button>
      </div>

      {/* Date Range Filter */}
      <div className="mb-6">
        <label className="text-sm font-medium text-gray-700 mb-2 block">Date Range</label>
        <div className="flex flex-wrap gap-2">
          {presets.map((preset) => (
            <button
              key={preset.label}
              onClick={() => handlePresetClick(preset.days)}
              className={clsx(
                'px-4 py-2 text-sm rounded-lg transition-colors',
                'hover:bg-blue-50 border',
                dateRange.start === new Date(Date.now() - preset.days * 86400000).toISOString().split('T')[0]
                  ? 'bg-blue-100 border-blue-500 text-blue-700'
                  : 'bg-white border-gray-300 text-gray-700'
              )}
            >
              {preset.label}
            </button>
          ))}
        </div>
        <div className="mt-2 text-sm text-gray-500">
          Showing data from {formatDate(dateRange.start)} to {formatDate(dateRange.end)}
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <SummaryCard
          icon={Users}
          label="Contributors"
          value={summary.total_contributors}
          color="blue"
        />
        <SummaryCard
          icon={GitPullRequest}
          label="Pull Requests"
          value={summary.total_prs}
          color="purple"
        />
        <SummaryCard
          icon={AlertCircle}
          label="Issues"
          value={summary.total_issues}
          color="orange"
        />
        <SummaryCard
          icon={CheckCircle2}
          label="Commits"
          value={summary.total_commits}
          color="green"
        />
      </div>
    </div>
  );
}

interface SummaryCardProps {
  icon: React.ElementType;
  label: string;
  value: number;
  color: 'blue' | 'purple' | 'orange' | 'green';
}

function SummaryCard({ icon: Icon, label, value, color }: SummaryCardProps) {
  const colorClasses = {
    blue: 'bg-blue-50 text-blue-600',
    purple: 'bg-purple-50 text-purple-600',
    orange: 'bg-orange-50 text-orange-600',
    green: 'bg-green-50 text-green-600',
  };

  return (
    <div className="bg-gray-50 rounded-lg p-4 border border-gray-200">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm text-gray-500 mb-1">{label}</p>
          <p className="text-2xl font-bold text-gray-900">{value}</p>
        </div>
        <div className={clsx('p-2 rounded-lg', colorClasses[color])}>
          <Icon className="w-5 h-5" />
        </div>
      </div>
    </div>
  );
}
