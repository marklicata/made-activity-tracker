import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { 
  TrendingUp, 
  Clock, 
  CheckCircle2, 
  AlertCircle,
  Users,
  GitPullRequest,
  Bug,
  Zap,
  RefreshCw,
  Database,
  Loader2,
  Settings,
} from 'lucide-react';
import { Link } from 'react-router-dom';
import clsx from 'clsx';

interface Metrics {
  speed: {
    avg_cycle_time_days: number;
    avg_pr_lead_time_hours: number;
    throughput_per_week: number;
    trend: number;
  };
  ease: {
    avg_pr_size_lines: number;
    avg_review_rounds: number;
    avg_time_to_first_review_hours: number;
    rework_rate: number;
  };
  quality: {
    bug_rate: number;
    reopen_rate: number;
    pr_rejection_rate: number;
    test_coverage_trend: number;
  };
}

interface SyncStats {
  issues: number;
  pull_requests: number;
  users: number;
  repositories: number;
}

interface SyncProgress {
  phase: string;
  current: number;
  total: number;
  message: string;
}

interface MetricCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  trend?: number;
  icon: React.ElementType;
  color: 'speed' | 'ease' | 'quality';
}

function MetricCard({ title, value, subtitle, trend, icon: Icon, color }: MetricCardProps) {
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
        <div>
          <p className="text-sm font-medium text-gray-500">{title}</p>
          <p className="text-2xl font-bold mt-1 text-gray-900">{value}</p>
          {subtitle && <p className="text-xs text-gray-400 mt-1">{subtitle}</p>}
        </div>
        <div className={clsx('p-2 rounded-lg', iconColorClasses[color])}>
          <Icon size={20} />
        </div>
      </div>
      {trend !== undefined && trend !== 0 && (
        <div className="mt-4 flex items-center gap-1">
          <TrendingUp size={14} className={trend >= 0 ? 'text-green-500' : 'text-red-500 rotate-180'} />
          <span className={clsx('text-sm font-medium', trend >= 0 ? 'text-green-600' : 'text-red-600')}>
            {Math.abs(trend)}%
          </span>
          <span className="text-xs text-gray-400">vs last period</span>
        </div>
      )}
    </div>
  );
}

export default function Dashboard() {
  const [metrics, setMetrics] = useState<Metrics | null>(null);
  const [stats, setStats] = useState<SyncStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [syncProgress, setSyncProgress] = useState<SyncProgress | null>(null);

  useEffect(() => {
    loadData();
    
    // Listen for sync progress events
    const unlisten = listen<SyncProgress>('sync-progress', (event) => {
      setSyncProgress(event.payload);
      if (event.payload.phase === 'complete') {
        setSyncing(false);
        loadData(); // Reload data after sync
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const loadData = async () => {
    setLoading(true);
    try {
      // Load stats
      const statsData = await invoke<SyncStats>('get_sync_stats');
      setStats(statsData);
      
      // Only load metrics if we have data
      if (statsData.issues > 0 || statsData.pull_requests > 0) {
        const metricsData = await invoke<Metrics>('get_dashboard_metrics');
        setMetrics(metricsData);
      }
    } catch (error) {
      console.error('Failed to load data:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSync = async () => {
    setSyncing(true);
    setSyncProgress({ phase: 'starting', current: 0, total: 0, message: 'Starting sync...' });
    
    try {
      await invoke('sync_github_data');
    } catch (error) {
      console.error('Sync failed:', error);
      setSyncing(false);
      setSyncProgress(null);
    }
  };

  // Empty state - no repositories configured
  const hasNoData = stats && stats.repositories === 0;
  const hasData = stats && (stats.issues > 0 || stats.pull_requests > 0);

  if (loading) {
    return (
      <div className="p-8 flex items-center justify-center h-full">
        <div className="flex items-center gap-2 text-gray-400">
          <Loader2 className="animate-spin" size={20} />
          Loading...
        </div>
      </div>
    );
  }

  // Empty state
  if (hasNoData) {
    return (
      <div className="p-8 flex items-center justify-center h-full">
        <div className="text-center max-w-md">
          <Database size={48} className="mx-auto text-gray-300 mb-4" />
          <h2 className="text-xl font-semibold text-gray-800 mb-2">No Repositories Configured</h2>
          <p className="text-gray-500 mb-6">
            Add some GitHub repositories to start tracking your team's activity and metrics.
          </p>
          <Link
            to="/settings"
            className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            <Settings size={18} />
            Configure Repositories
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="p-8">
      {/* Header */}
      <div className="mb-8 flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
          <p className="text-gray-500">Team productivity metrics across all repositories</p>
        </div>
        
        <div className="flex items-center gap-4">
          {/* Stats Summary */}
          {stats && (
            <div className="text-sm text-gray-500 text-right">
              <span className="font-medium text-gray-700">{stats.repositories}</span> repos · 
              <span className="font-medium text-gray-700"> {stats.issues}</span> issues · 
              <span className="font-medium text-gray-700"> {stats.pull_requests}</span> PRs
            </div>
          )}
          
          {/* Sync Button */}
          <button
            onClick={handleSync}
            disabled={syncing}
            className="flex items-center gap-2 px-4 py-2 bg-gray-900 text-white rounded-lg hover:bg-gray-800 disabled:opacity-50 transition-colors"
          >
            {syncing ? (
              <>
                <Loader2 size={18} className="animate-spin" />
                Syncing...
              </>
            ) : (
              <>
                <RefreshCw size={18} />
                Sync Now
              </>
            )}
          </button>
        </div>
      </div>

      {/* Sync Progress */}
      {syncProgress && syncing && (
        <div className="mb-8 p-4 bg-blue-50 border border-blue-200 rounded-xl">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium text-blue-800">{syncProgress.message}</span>
            {syncProgress.total > 0 && (
              <span className="text-sm text-blue-600">
                {syncProgress.current} / {syncProgress.total}
              </span>
            )}
          </div>
          {syncProgress.total > 0 && (
            <div className="w-full bg-blue-200 rounded-full h-2">
              <div
                className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                style={{ width: `${(syncProgress.current / syncProgress.total) * 100}%` }}
              />
            </div>
          )}
        </div>
      )}

      {/* No metrics yet - prompt to sync */}
      {!hasData && (
        <div className="mb-8 p-6 bg-amber-50 border border-amber-200 rounded-xl">
          <div className="flex items-start gap-3">
            <AlertCircle className="text-amber-600 flex-shrink-0" size={20} />
            <div>
              <h3 className="font-medium text-amber-800">No data synced yet</h3>
              <p className="text-sm text-amber-700 mt-1">
                Click "Sync Now" to fetch issues and pull requests from your configured repositories.
                This may take a few minutes for the initial sync.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Metrics Display */}
      {metrics && (
        <>
          {/* Speed Metrics */}
          <section className="mb-8">
            <h2 className="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
              <Zap className="text-blue-500" size={20} />
              Speed
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <MetricCard
                title="Avg Cycle Time"
                value={`${metrics.speed.avg_cycle_time_days} days`}
                subtitle="Issue open → closed (business days)"
                trend={metrics.speed.trend}
                icon={Clock}
                color="speed"
              />
              <MetricCard
                title="PR Lead Time"
                value={`${metrics.speed.avg_pr_lead_time_hours}h`}
                subtitle="PR open → merged"
                icon={GitPullRequest}
                color="speed"
              />
              <MetricCard
                title="Throughput"
                value={metrics.speed.throughput_per_week}
                subtitle="Items completed / week"
                icon={CheckCircle2}
                color="speed"
              />
            </div>
          </section>

          {/* Ease Metrics */}
          <section className="mb-8">
            <h2 className="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
              <Users className="text-green-500" size={20} />
              Ease
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <MetricCard
                title="Avg PR Size"
                value={`${metrics.ease.avg_pr_size_lines} lines`}
                subtitle="Smaller is easier to review"
                icon={GitPullRequest}
                color="ease"
              />
              <MetricCard
                title="Review Rounds"
                value={metrics.ease.avg_review_rounds.toFixed(1)}
                subtitle="Avg iterations per PR"
                icon={Users}
                color="ease"
              />
              <MetricCard
                title="Time to First Review"
                value={`${metrics.ease.avg_time_to_first_review_hours}h`}
                subtitle="PR open → first review"
                icon={Clock}
                color="ease"
              />
            </div>
          </section>

          {/* Quality Metrics */}
          <section className="mb-8">
            <h2 className="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
              <AlertCircle className="text-purple-500" size={20} />
              Quality
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <MetricCard
                title="Bug Rate"
                value={`${(metrics.quality.bug_rate * 100).toFixed(1)}%`}
                subtitle="Issues labeled as bugs"
                icon={Bug}
                color="quality"
              />
              <MetricCard
                title="Reopen Rate"
                value={`${(metrics.quality.reopen_rate * 100).toFixed(1)}%`}
                subtitle="Issues reopened after close"
                icon={AlertCircle}
                color="quality"
              />
              <MetricCard
                title="PR Rejection Rate"
                value={`${(metrics.quality.pr_rejection_rate * 100).toFixed(1)}%`}
                subtitle="PRs closed without merge"
                icon={GitPullRequest}
                color="quality"
              />
            </div>
          </section>
        </>
      )}
    </div>
  );
}
