import { useState, useRef, useEffect } from 'react';
import { FolderGit2, ChevronDown, Check } from 'lucide-react';
import clsx from 'clsx';
import { invoke } from '@tauri-apps/api/tauri';
import { useDashboardFilterStore } from '@stores/dashboardFilterStore';

interface Repository {
  id: number;
  owner: string;
  name: string;
  github_id?: number;
  enabled: boolean;
  last_synced_at?: string;
}

export default function RepositoryFilter() {
  const [isOpen, setIsOpen] = useState(false);
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [loading, setLoading] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const { filters, setRepositories: setSelectedRepos } = useDashboardFilterStore();

  // Load repositories on mount
  useEffect(() => {
    loadRepositories();
  }, []);

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

  const loadRepositories = async () => {
    try {
      setLoading(true);
      const repos = await invoke<Repository[]>('get_all_repositories');
      setRepositories(repos);
    } catch (error) {
      console.error('Failed to load repositories:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleToggleRepository = (repoId: number) => {
    const currentIds = filters.repositoryIds || [];
    const newIds = currentIds.includes(repoId)
      ? currentIds.filter((id) => id !== repoId)
      : [...currentIds, repoId];

    setSelectedRepos(newIds.length > 0 ? newIds : null);
  };

  const handleSelectAll = () => {
    setSelectedRepos(repositories.map((r) => r.id));
  };

  const handleClearAll = () => {
    setSelectedRepos(null);
  };

  const selectedCount = filters.repositoryIds?.length || 0;
  const displayText =
    selectedCount === 0
      ? 'All repositories'
      : selectedCount === 1
      ? '1 repository'
      : `${selectedCount} repositories`;

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={clsx(
          'flex items-center gap-2 px-4 py-2 rounded-lg border text-sm font-medium transition-colors',
          isOpen || selectedCount > 0
            ? 'bg-primary-50 border-primary-300 text-primary-700'
            : 'bg-white border-gray-300 text-gray-700 hover:bg-gray-50'
        )}
      >
        <FolderGit2 size={16} />
        <span>{displayText}</span>
        {selectedCount > 0 && (
          <span className="px-2 py-0.5 bg-primary-600 text-white text-xs rounded-full">
            {selectedCount}
          </span>
        )}
        <ChevronDown
          size={16}
          className={clsx('transition-transform', isOpen && 'rotate-180')}
        />
      </button>

      {isOpen && (
        <div className="absolute top-full left-0 mt-2 w-72 bg-white border border-gray-200 rounded-lg shadow-lg z-50">
          {/* Header with actions */}
          <div className="p-2 border-b border-gray-200 flex items-center justify-between">
            <span className="text-xs font-medium text-gray-500">
              {repositories.length} repositories
            </span>
            <div className="flex gap-1">
              <button
                onClick={handleSelectAll}
                className="px-2 py-1 text-xs text-primary-600 hover:bg-primary-50 rounded transition-colors"
              >
                All
              </button>
              <button
                onClick={handleClearAll}
                className="px-2 py-1 text-xs text-gray-600 hover:bg-gray-100 rounded transition-colors"
              >
                Clear
              </button>
            </div>
          </div>

          {/* Repository list */}
          <div className="max-h-80 overflow-y-auto">
            {loading ? (
              <div className="p-4 text-sm text-gray-500 text-center">
                Loading repositories...
              </div>
            ) : repositories.length === 0 ? (
              <div className="p-4 text-sm text-gray-500 text-center">
                No repositories found
              </div>
            ) : (
              <div className="p-2">
                {repositories.map((repo) => {
                  const isSelected = filters.repositoryIds?.includes(repo.id) || false;
                  return (
                    <button
                      key={repo.id}
                      onClick={() => handleToggleRepository(repo.id)}
                      className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm hover:bg-gray-100 transition-colors"
                    >
                      <div
                        className={clsx(
                          'flex items-center justify-center w-4 h-4 border-2 rounded transition-colors',
                          isSelected
                            ? 'bg-primary-600 border-primary-600'
                            : 'border-gray-300'
                        )}
                      >
                        {isSelected && <Check size={12} className="text-white" />}
                      </div>
                      <div className="flex-1 text-left">
                        <div className="text-gray-900 font-medium">
                          {repo.owner}/{repo.name}
                        </div>
                        {!repo.enabled && (
                          <span className="text-xs text-gray-400">(disabled)</span>
                        )}
                      </div>
                    </button>
                  );
                })}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
