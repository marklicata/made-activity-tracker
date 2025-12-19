import { Layers, BarChart3, GitBranch, Users } from 'lucide-react';
import type { EaseMetrics } from '@/types/metrics';
import { BenchmarkMetricCard } from './BenchmarkMetricCard';
import { formatMetricComparison } from '@/types/metrics';

interface EaseSectionProps {
  ease: EaseMetrics;
}

export function EaseSection({ ease }: EaseSectionProps) {
  return (
    <section>
      <h2 className="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
        <Layers className="text-green-500" size={20} />
        Ease
        <span className="text-sm font-normal text-gray-500 ml-2">
          Capacity for parallel work
        </span>
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <BenchmarkMetricCard
          title="Concurrent Repos"
          value={ease.concurrent_repos}
          subtitle="Projects in parallel"
          comparison={formatMetricComparison(
            ease.concurrent_repos,
            ease.benchmark_comparison.concurrent_repos_industry,
            ease.benchmark_comparison.concurrent_repos_elite,
            true
          )}
          icon={GitBranch}
          color="ease"
        />

        <BenchmarkMetricCard
          title="Repos per Developer"
          value={ease.repos_per_dev.toFixed(1)}
          subtitle="Multi-tasking capacity"
          icon={Users}
          color="ease"
        />

        <BenchmarkMetricCard
          title="Context Switch Rate"
          value={`${ease.pr_switch_frequency.toFixed(1)}%`}
          subtitle="PR repo changes"
          icon={BarChart3}
          color="ease"
        />

        <BenchmarkMetricCard
          title="Active Repos"
          value={ease.total_active_repos}
          subtitle="With activity this period"
          icon={GitBranch}
          color="ease"
        />
      </div>

      {/* Repository Distribution */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <div className="bg-white rounded-xl shadow-sm p-6">
          <h3 className="text-lg font-semibold text-gray-800 mb-4">Repository Distribution</h3>
          <div className="space-y-4">
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span className="font-medium text-gray-700">Organization Repos</span>
                <span className="text-gray-500">
                  {ease.repo_distribution.org_repos_pct.toFixed(1)}% ({ease.repo_distribution.org_repos})
                </span>
              </div>
              <div className="w-full bg-gray-100 rounded-full h-2.5">
                <div
                  className="bg-blue-500 h-2.5 rounded-full transition-all"
                  style={{ width: `${ease.repo_distribution.org_repos_pct}%` }}
                />
              </div>
            </div>
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span className="font-medium text-gray-700">Personal Repos</span>
                <span className="text-gray-500">
                  {ease.repo_distribution.personal_repos_pct.toFixed(1)}% ({ease.repo_distribution.personal_repos})
                </span>
              </div>
              <div className="w-full bg-gray-100 rounded-full h-2.5">
                <div
                  className="bg-green-500 h-2.5 rounded-full transition-all"
                  style={{ width: `${ease.repo_distribution.personal_repos_pct}%` }}
                />
              </div>
            </div>
          </div>
        </div>

        {/* Top Active Repositories */}
        <div className="bg-white rounded-xl shadow-sm p-6">
          <h3 className="text-lg font-semibold text-gray-800 mb-4">
            Top Active Repositories
          </h3>
          <div className="space-y-3 max-h-64 overflow-y-auto">
            {ease.active_repos.slice(0, 10).map((repo) => (
              <div
                key={repo.repo_name}
                className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 transition-colors"
              >
                <div className="flex-1 min-w-0">
                  <div className="font-mono text-sm text-gray-900 truncate">
                    {repo.repo_name}
                  </div>
                  <div className="text-xs text-gray-500 flex items-center gap-3 mt-1">
                    <span>{repo.pr_count} PRs</span>
                    <span>{repo.total_loc.toLocaleString()} LOC</span>
                    <span>{repo.contributor_count} contributors</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
