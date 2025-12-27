import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/tauri';
import { ChevronLeft, Loader2, GitPullRequest, MessageSquare, AlertCircle, FolderGit2, Download } from 'lucide-react';
import { UserSummary, RepositoryContribution, ActivityDataPoint, FocusMetrics } from '@/types';
import Timeline from '@components/project/Timeline';
import RepositoryDistribution from '@components/team/RepositoryDistribution';
import UserActivityTrend from '@components/team/UserActivityTrend';
import FocusAnalysis from '@components/team/FocusAnalysis';
import DateRangeFilter from '@components/common/DateRangeFilter';
import { exportUserToCSV } from '@/utils/export';

interface TimelineEvent {
  id: string;
  event_type: string;
  timestamp: string;
  author: any;
  title: string;
  description: string | null;
  url: string | null;
  metadata: any;
}

export default function UserDetail() {
  const { username } = useParams<{ username: string }>();
  const navigate = useNavigate();

  const [summary, setSummary] = useState<UserSummary | null>(null);
  const [timeline, setTimeline] = useState<TimelineEvent[]>([]);
  const [contributions, setContributions] = useState<RepositoryContribution[]>([]);
  const [activityTrend, setActivityTrend] = useState<ActivityDataPoint[]>([]);
  const [focusMetrics, setFocusMetrics] = useState<FocusMetrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Date range - default to last 30 days
  const [dateRange, setDateRange] = useState(() => {
    const endDate = new Date();
    const startDate = new Date();
    startDate.setDate(startDate.getDate() - 30);
    return {
      start: startDate.toISOString().split('T')[0],
      end: endDate.toISOString().split('T')[0],
    };
  });

  useEffect(() => {
    if (username) {
      loadUserData();
    }
  }, [username]);

  // Reload data when date range changes
  useEffect(() => {
    if (username && !loading) {
      loadUserData();
    }
  }, [dateRange]);

  const handleDateRangeChange = (start: string, end: string) => {
    setDateRange({ start, end });
  };

  const handleExport = () => {
    if (summary) {
      exportUserToCSV(summary, dateRange);
    }
  };

  const loadUserData = async () => {
    if (!username) return;

    setLoading(true);
    setError(null);

    try {
      // Load user summary first - this is required
      const summaryData = await invoke<UserSummary>('get_user_summary', {
        username,
        startDate: dateRange.start,
        endDate: dateRange.end,
      });
      setSummary(summaryData);

      // Load other data independently - failures won't crash the page
      invoke<TimelineEvent[]>('get_user_activity_timeline', {
        username,
        startDate: dateRange.start,
        endDate: dateRange.end,
        limit: 100,
      })
        .then(setTimeline)
        .catch(err => {
          console.error('Failed to load timeline:', err);
          setTimeline([]);
        });

      invoke<RepositoryContribution[]>('get_user_repository_distribution', {
        username,
        startDate: dateRange.start,
        endDate: dateRange.end,
      })
        .then(setContributions)
        .catch(err => {
          console.error('Failed to load contributions:', err);
          setContributions([]);
        });

      invoke<ActivityDataPoint[]>('get_user_activity_trend', {
        username,
        startDate: dateRange.start,
        endDate: dateRange.end,
        granularity: 'week',
      })
        .then(setActivityTrend)
        .catch(err => {
          console.error('Failed to load activity trend:', err);
          setActivityTrend([]);
        });

      invoke<FocusMetrics>('get_user_focus_metrics', {
        username,
        startDate: dateRange.start,
        endDate: dateRange.end,
      })
        .then(setFocusMetrics)
        .catch(err => {
          console.error('Failed to load focus metrics:', err);
          setFocusMetrics(null);
        });
    } catch (err) {
      setError(err as string);
      console.error('Failed to load user data:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
        <span className="ml-3 text-lg text-gray-600">Loading user data...</span>
      </div>
    );
  }

  if (error || !summary) {
    return (
      <div className="flex flex-col items-center justify-center h-screen">
        <p className="text-red-500 text-lg">{error || 'Failed to load user data'}</p>
        <button
          onClick={() => navigate('/team')}
          className="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          Back to Team View
        </button>
      </div>
    );
  }

  const statusColors = {
    active: 'bg-green-100 text-green-800',
    quiet: 'bg-yellow-100 text-yellow-800',
    idle: 'bg-gray-100 text-gray-800',
  };

  const statusDots = {
    active: 'bg-green-500',
    quiet: 'bg-yellow-500',
    idle: 'bg-gray-500',
  };

  return (
    <div className="min-h-screen bg-gray-50 pb-12">
      {/* Breadcrumb */}
      <div className="bg-white border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <button
            onClick={() => navigate('/team')}
            className="flex items-center text-gray-600 hover:text-gray-900 transition-colors"
          >
            <ChevronLeft className="w-4 h-4 mr-1" />
            <span className="text-sm">Back to Team View</span>
          </button>
          <div className="flex items-center text-sm text-gray-500 mt-2">
            <span>Team</span>
            <span className="mx-2">/</span>
            <span className="text-gray-900 font-medium">{username}</span>
          </div>
        </div>
      </div>

      {/* User Header */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-8">
        {/* Date Range Filter and Export */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
          <div className="lg:col-span-2">
            <button
              onClick={handleExport}
              className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-green-600 rounded-md hover:bg-green-700 transition-colors"
            >
              <Download className="w-4 h-4" />
              Export to CSV
            </button>
          </div>
          <div>
            <DateRangeFilter
              startDate={dateRange.start}
              endDate={dateRange.end}
              onDateRangeChange={handleDateRangeChange}
            />
          </div>
        </div>

        <div className="bg-white shadow rounded-lg p-6">
          <div className="flex items-start justify-between">
            <div className="flex items-center gap-4">
              {summary.user.avatar_url ? (
                <img
                  src={summary.user.avatar_url}
                  alt={summary.user.login}
                  className="w-16 h-16 rounded-full"
                />
              ) : (
                <div className="h-16 w-16 rounded-full bg-gray-200 flex items-center justify-center">
                  <span className="text-2xl font-semibold text-gray-600">
                    {username?.charAt(0).toUpperCase()}
                  </span>
                </div>
              )}
              <div>
                <h1 className="text-2xl font-bold text-gray-900">
                  {summary.user.name || `@${username}`}
                </h1>
                {summary.user.name && (
                  <p className="text-sm text-gray-500">@{username}</p>
                )}
                <div className="flex items-center gap-2 mt-2">
                  <span className={`w-2 h-2 rounded-full ${statusDots[summary.activity_status]}`} />
                  <span className={`text-xs font-medium px-2 py-1 rounded-full ${statusColors[summary.activity_status]}`}>
                    {summary.activity_status.charAt(0).toUpperCase() + summary.activity_status.slice(1)}
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* Stats Grid */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
            <div className="text-center p-4 bg-blue-50 rounded-lg">
              <GitPullRequest className="w-6 h-6 mx-auto mb-2 text-blue-600" />
              <p className="text-2xl font-bold text-gray-900">{summary.total_prs_created}</p>
              <p className="text-xs text-gray-500">PRs Created</p>
            </div>
            <div className="text-center p-4 bg-purple-50 rounded-lg">
              <MessageSquare className="w-6 h-6 mx-auto mb-2 text-purple-600" />
              <p className="text-2xl font-bold text-gray-900">{summary.total_prs_reviewed}</p>
              <p className="text-xs text-gray-500">PRs Reviewed</p>
            </div>
            <div className="text-center p-4 bg-amber-50 rounded-lg">
              <AlertCircle className="w-6 h-6 mx-auto mb-2 text-amber-600" />
              <p className="text-2xl font-bold text-gray-900">{summary.total_issues_opened}</p>
              <p className="text-xs text-gray-500">Issues</p>
            </div>
            <div className="text-center p-4 bg-green-50 rounded-lg">
              <FolderGit2 className="w-6 h-6 mx-auto mb-2 text-green-600" />
              <p className="text-2xl font-bold text-gray-900">{summary.repositories_touched}</p>
              <p className="text-xs text-gray-500">Repos</p>
            </div>
          </div>

          {/* Code Stats */}
          <div className="mt-4 pt-4 border-t border-gray-200">
            <p className="text-sm text-gray-600">
              <span className="text-green-600 font-medium">+{summary.lines_added.toLocaleString()}</span>
              {' / '}
              <span className="text-red-600 font-medium">-{summary.lines_deleted.toLocaleString()}</span>
              {' lines changed'}
            </p>
          </div>
        </div>

        {/* Repository Distribution */}
        {contributions.length > 0 && (
          <div className="mt-8">
            <RepositoryDistribution contributions={contributions} />
          </div>
        )}

        {/* Focus Analysis */}
        {focusMetrics && (
          <div className="mt-8">
            <FocusAnalysis metrics={focusMetrics} username={username || ''} />
          </div>
        )}

        {/* Activity Trend */}
        {activityTrend.length > 0 && (
          <div className="mt-8">
            <UserActivityTrend data={activityTrend} username={username || ''} />
          </div>
        )}

        {/* Activity Timeline */}
        {timeline.length > 0 && (
          <div className="mt-8">
            <Timeline events={timeline} repository={null} />
          </div>
        )}
      </div>
    </div>
  );
}
