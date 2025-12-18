import { useEffect, useState } from 'react';
import { useParams, useNavigate, useSearchParams } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/tauri';
import { ChevronLeft, Loader2 } from 'lucide-react';
import ProjectHeader from '@components/project/ProjectHeader';
import Timeline from '@components/project/Timeline';
import ContributorTable from '@components/project/ContributorTable';
import ActivityHeatmap from '@components/project/ActivityHeatmap';
import LifecycleMetrics from '@components/project/LifecycleMetrics';

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

interface TimelineEvent {
  id: string;
  event_type: string;
  timestamp: string;
  author: {
    id: number;
    github_id: number;
    login: string;
    name: string | null;
    avatar_url: string | null;
    is_bot: boolean;
  };
  title: string;
  description: string | null;
  url: string | null;
  metadata: any;
}

interface ContributorStats {
  user: {
    id: number;
    github_id: number;
    login: string;
    name: string | null;
    avatar_url: string | null;
    is_bot: boolean;
  };
  total_commits: number;
  total_prs: number;
  total_prs_reviewed: number;
  total_issues: number;
  lines_added: number;
  lines_deleted: number;
  files_changed: number;
  first_contribution: string;
  last_contribution: string;
  activity_trend: string;
}

interface ActivityHeatmapData {
  daily_counts: Record<string, number>;
  hourly_counts: Record<number, number>;
  weekday_counts: Record<string, number>;
}

interface LifecycleMetrics {
  avg_time_to_merge: number;
  median_time_to_merge: number;
  p90_time_to_merge: number;
  avg_time_to_first_review: number;
  avg_review_cycles: number;
  open_prs_count: number;
  open_issues_count: number;
  bottleneck_prs: any[];
  bottleneck_issues: any[];
}

export default function ProjectDeepDive() {
  const { owner, repo } = useParams<{ owner: string; repo: string }>();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [repository, setRepository] = useState<Repository | null>(null);
  const [summary, setSummary] = useState<ProjectSummary | null>(null);
  const [timeline, setTimeline] = useState<TimelineEvent[]>([]);
  const [contributors, setContributors] = useState<ContributorStats[]>([]);
  const [heatmap, setHeatmap] = useState<ActivityHeatmapData | null>(null);
  const [lifecycle, setLifecycle] = useState<LifecycleMetrics | null>(null);

  // Date range filter from URL or default to last 30 days
  const [dateRange, setDateRange] = useState<{ start: string; end: string }>(() => {
    const start = searchParams.get('start');
    const end = searchParams.get('end');
    if (start && end) {
      return { start, end };
    }
    // Default to last 30 days
    const endDate = new Date();
    const startDate = new Date();
    startDate.setDate(startDate.getDate() - 30);
    return {
      start: startDate.toISOString().split('T')[0],
      end: endDate.toISOString().split('T')[0],
    };
  });

  // Selected user filter
  const [selectedUserId, setSelectedUserId] = useState<number | null>(null);

  useEffect(() => {
    loadProjectData();
  }, [owner, repo, dateRange]);

  async function loadProjectData() {
    if (!owner || !repo) return;

    try {
      setLoading(true);
      setError(null);

      // Get repository details
      const allRepos = await invoke<Repository[]>('get_all_repositories');
      const foundRepo = allRepos.find(
        (r) => r.owner.toLowerCase() === owner.toLowerCase() && r.name.toLowerCase() === repo.toLowerCase()
      );

      if (!foundRepo) {
        setError(`Repository ${owner}/${repo} not found`);
        setLoading(false);
        return;
      }

      setRepository(foundRepo);

      // Fetch all data in parallel
      const [summaryData, timelineData, contributorsData, heatmapData, lifecycleData] = await Promise.all([
        invoke<ProjectSummary>('get_project_summary', {
          repoId: foundRepo.id,
          startDate: dateRange.start,
          endDate: dateRange.end,
        }),
        invoke<TimelineEvent[]>('get_project_timeline', {
          repoId: foundRepo.id,
          startDate: dateRange.start,
          endDate: dateRange.end,
          eventTypes: null,
          userId: selectedUserId,
          limit: 1000,
        }),
        invoke<ContributorStats[]>('get_project_contributors', {
          repoId: foundRepo.id,
          startDate: dateRange.start,
          endDate: dateRange.end,
        }),
        invoke<ActivityHeatmapData>('get_project_activity_heatmap', {
          repoId: foundRepo.id,
          startDate: dateRange.start,
          endDate: dateRange.end,
        }),
        invoke<LifecycleMetrics>('get_project_lifecycle_metrics', {
          repoId: foundRepo.id,
          startDate: dateRange.start,
          endDate: dateRange.end,
        }),
      ]);

      setSummary(summaryData);
      setTimeline(timelineData);
      setContributors(contributorsData);
      setHeatmap(heatmapData);
      setLifecycle(lifecycleData);
    } catch (err) {
      console.error('Failed to load project data:', err);
      setError(err instanceof Error ? err.message : 'Failed to load project data');
    } finally {
      setLoading(false);
    }
  }

  function handleDateRangeChange(start: string, end: string) {
    setDateRange({ start, end });
    setSearchParams({ start, end });
  }

  function handleUserFilterChange(userId: number | null) {
    setSelectedUserId(userId);
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
        <span className="ml-3 text-lg text-gray-600">Loading project data...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-screen">
        <p className="text-red-500 text-lg">{error}</p>
        <button
          onClick={() => navigate('/')}
          className="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          Back to Dashboard
        </button>
      </div>
    );
  }

  if (!repository || !summary) {
    return null;
  }

  return (
    <div className="min-h-screen bg-gray-50 pb-12">
      {/* Breadcrumb */}
      <div className="bg-white border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <button
            onClick={() => navigate('/')}
            className="flex items-center text-gray-600 hover:text-gray-900 transition-colors"
          >
            <ChevronLeft className="w-4 h-4 mr-1" />
            <span className="text-sm">Back to Dashboard</span>
          </button>
          <div className="flex items-center text-sm text-gray-500 mt-2">
            <span>Dashboard</span>
            <span className="mx-2">/</span>
            <span className="text-gray-900 font-medium">
              {owner}/{repo}
            </span>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-8">
        <ProjectHeader
          repository={repository}
          summary={summary}
          dateRange={dateRange}
          onDateRangeChange={handleDateRangeChange}
          onSyncComplete={loadProjectData}
        />

        {/* Activity Heatmap */}
        {heatmap && (
          <div className="mt-8">
            <ActivityHeatmap data={heatmap} />
          </div>
        )}

        {/* Lifecycle Metrics */}
        {lifecycle && (
          <div className="mt-8">
            <LifecycleMetrics data={lifecycle} repository={repository} />
          </div>
        )}

        {/* Contributors Table */}
        <div className="mt-8">
          <ContributorTable
            contributors={contributors}
            onUserSelect={handleUserFilterChange}
            selectedUserId={selectedUserId}
          />
        </div>

        {/* Timeline */}
        <div className="mt-8">
          <Timeline events={timeline} repository={repository} />
        </div>
      </div>
    </div>
  );
}
