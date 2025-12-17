import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/tauri';
import { Package, Calendar, CheckCircle2, AlertCircle, Loader2, ExternalLink } from 'lucide-react';
import clsx from 'clsx';

interface Repository {
  id: number;
  owner: string;
  name: string;
  github_id: number | null;
  enabled: boolean;
  last_synced_at: string | null;
}

export default function Projects() {
  const navigate = useNavigate();
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadRepositories();
  }, []);

  async function loadRepositories() {
    try {
      setLoading(true);
      setError(null);
      const repos = await invoke<Repository[]>('get_all_repositories');
      setRepositories(repos);
    } catch (err) {
      console.error('Failed to load repositories:', err);
      setError(err instanceof Error ? err.message : 'Failed to load repositories');
    } finally {
      setLoading(false);
    }
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

  function handleProjectClick(repo: Repository) {
    navigate(`/projects/${repo.owner}/${repo.name}`);
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
        <span className="ml-3 text-lg text-gray-600">Loading projects...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-screen">
        <AlertCircle className="w-12 h-12 text-red-500 mb-4" />
        <p className="text-red-500 text-lg mb-2">{error}</p>
        <button
          onClick={loadRepositories}
          className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          Try Again
        </button>
      </div>
    );
  }

  if (repositories.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-screen">
        <Package className="w-16 h-16 text-gray-400 mb-4" />
        <h2 className="text-xl font-semibold text-gray-900 mb-2">No Projects Found</h2>
        <p className="text-gray-500 mb-4">Add repositories in Settings to get started</p>
        <button
          onClick={() => navigate('/settings')}
          className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          Go to Settings
        </button>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 p-8">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">Projects</h1>
          <p className="text-gray-600">
            Select a project to view detailed analytics and insights
          </p>
        </div>

        {/* Repository Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {repositories.map((repo) => (
            <button
              key={repo.id}
              onClick={() => handleProjectClick(repo)}
              className={clsx(
                'bg-white rounded-lg border-2 p-6 text-left transition-all',
                'hover:border-blue-500 hover:shadow-lg',
                repo.enabled
                  ? 'border-gray-200'
                  : 'border-gray-200 opacity-50 cursor-not-allowed'
              )}
              disabled={!repo.enabled}
            >
              {/* Repository Header */}
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-start gap-3 flex-1 min-w-0">
                  <div className="p-2 bg-blue-100 rounded-lg flex-shrink-0">
                    <Package className="w-6 h-6 text-blue-600" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <h3 className="font-semibold text-gray-900 truncate">
                      {repo.name}
                    </h3>
                    <p className="text-sm text-gray-500 truncate">
                      {repo.owner}
                    </p>
                  </div>
                </div>
                <ExternalLink className="w-4 h-4 text-gray-400 flex-shrink-0" />
              </div>

              {/* Status */}
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm">
                  {repo.enabled ? (
                    <>
                      <CheckCircle2 className="w-4 h-4 text-green-500" />
                      <span className="text-gray-600">Active</span>
                    </>
                  ) : (
                    <>
                      <AlertCircle className="w-4 h-4 text-gray-400" />
                      <span className="text-gray-600">Disabled</span>
                    </>
                  )}
                </div>

                <div className="flex items-center gap-2 text-sm text-gray-500">
                  <Calendar className="w-4 h-4" />
                  <span>{formatSyncTime(repo.last_synced_at)}</span>
                </div>
              </div>

              {/* View Details Link */}
              {repo.enabled && (
                <div className="mt-4 pt-4 border-t border-gray-200">
                  <span className="text-sm font-medium text-blue-600 group-hover:text-blue-700">
                    View Deep Dive →
                  </span>
                </div>
              )}
            </button>
          ))}
        </div>

        {/* Summary */}
        <div className="mt-8 p-4 bg-white rounded-lg border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">
                Tracking <span className="font-semibold text-gray-900">{repositories.length}</span> project
                {repositories.length !== 1 ? 's' : ''}
              </p>
              <p className="text-xs text-gray-500 mt-1">
                {repositories.filter((r) => r.enabled).length} active
              </p>
            </div>
            <button
              onClick={() => navigate('/settings')}
              className="text-sm text-blue-600 hover:text-blue-700 font-medium"
            >
              Manage Projects
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
