import { FocusMetrics } from '@/types';
import { Target, GitBranch } from 'lucide-react';

interface FocusAnalysisProps {
  metrics: FocusMetrics;
  username: string;
}

export default function FocusAnalysis({ metrics, username }: FocusAnalysisProps) {
  if (metrics.repos_touched === 0) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Focus Analysis</h2>
        <p className="text-gray-500">No repository activity found for this user.</p>
      </div>
    );
  }

  // Determine focus level based on concentration score (HHI)
  const getFocusLevel = () => {
    if (metrics.concentration_score >= 0.7) {
      return {
        label: 'Highly Focused',
        color: 'text-green-600 bg-green-50',
        description: 'Work is concentrated in very few repositories',
      };
    } else if (metrics.concentration_score >= 0.4) {
      return {
        label: 'Moderately Focused',
        color: 'text-yellow-600 bg-yellow-50',
        description: 'Work is spread across several repositories',
      };
    } else {
      return {
        label: 'Highly Distributed',
        color: 'text-blue-600 bg-blue-50',
        description: 'Work is spread across many repositories',
      };
    }
  };

  const focusLevel = getFocusLevel();

  // Get context switching assessment
  const getContextSwitchingLevel = () => {
    if (metrics.repos_touched === 1) return 'Minimal';
    if (metrics.repos_touched <= 3) return 'Low';
    if (metrics.repos_touched <= 5) return 'Moderate';
    return 'High';
  };

  const contextSwitching = getContextSwitchingLevel();

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="p-6">
        <div className="flex items-center gap-2 mb-4">
          <Target className="w-5 h-5 text-gray-700" />
          <h2 className="text-lg font-semibold text-gray-900">Focus Analysis</h2>
        </div>

        {/* Focus Score Card */}
        <div className={`p-4 rounded-lg mb-6 ${focusLevel.color}`}>
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-lg font-semibold">{focusLevel.label}</h3>
            <span className="text-2xl font-bold">
              {(metrics.concentration_score * 100).toFixed(0)}
            </span>
          </div>
          <p className="text-sm">{focusLevel.description}</p>
        </div>

        {/* Metrics Grid */}
        <div className="grid grid-cols-2 gap-4 mb-6">
          <div className="p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center gap-2 mb-2">
              <GitBranch className="w-4 h-4 text-gray-600" />
              <span className="text-sm text-gray-600">Repositories</span>
            </div>
            <p className="text-2xl font-bold text-gray-900">{metrics.repos_touched}</p>
            <p className="text-xs text-gray-500 mt-1">Active repositories</p>
          </div>

          <div className="p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center gap-2 mb-2">
              <Target className="w-4 h-4 text-gray-600" />
              <span className="text-sm text-gray-600">Top Repo</span>
            </div>
            <p className="text-2xl font-bold text-gray-900">
              {metrics.top_repo_percentage.toFixed(0)}%
            </p>
            <p className="text-xs text-gray-500 mt-1">Of total work</p>
          </div>
        </div>

        {/* Context Switching */}
        <div className="mb-6">
          <h3 className="text-sm font-medium text-gray-700 mb-2">Context Switching</h3>
          <div className="flex items-center gap-2">
            <div className="flex-1 bg-gray-200 rounded-full h-2">
              <div
                className={`h-2 rounded-full ${
                  contextSwitching === 'High'
                    ? 'bg-red-500'
                    : contextSwitching === 'Moderate'
                    ? 'bg-yellow-500'
                    : 'bg-green-500'
                }`}
                style={{
                  width: `${Math.min((metrics.repos_touched / 10) * 100, 100)}%`,
                }}
              />
            </div>
            <span className="text-sm font-medium text-gray-700 w-20">{contextSwitching}</span>
          </div>
          <p className="text-xs text-gray-500 mt-2">
            {contextSwitching === 'High'
              ? 'May be spread thin across many repositories'
              : contextSwitching === 'Moderate'
              ? 'Balanced workload across repositories'
              : 'Minimal context switching between repositories'}
          </p>
        </div>

        {/* Repository Distribution */}
        {metrics.repos_distribution.length > 0 && (
          <div>
            <h3 className="text-sm font-medium text-gray-700 mb-3">Top Repositories</h3>
            <div className="space-y-2">
              {metrics.repos_distribution.slice(0, 5).map(([repo, count]) => {
                const percentage = metrics.repos_distribution.length > 0
                  ? (count / metrics.repos_distribution.reduce((sum, [, c]) => sum + c, 0)) * 100
                  : 0;

                return (
                  <div key={repo}>
                    <div className="flex items-center justify-between text-sm mb-1">
                      <span className="text-gray-700 truncate max-w-[200px]" title={repo}>
                        {repo}
                      </span>
                      <span className="text-gray-600 font-medium">{count} contributions</span>
                    </div>
                    <div className="w-full bg-gray-200 rounded-full h-1.5">
                      <div
                        className="bg-blue-600 h-1.5 rounded-full"
                        style={{ width: `${percentage}%` }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
            {metrics.repos_distribution.length > 5 && (
              <p className="text-xs text-gray-500 mt-3 text-center">
                Showing top 5 of {metrics.repos_distribution.length} repositories
              </p>
            )}
          </div>
        )}

        {/* Insights */}
        <div className="mt-6 p-4 bg-blue-50 rounded-lg">
          <h3 className="text-sm font-semibold text-gray-900 mb-2">Insights</h3>
          <ul className="text-sm text-gray-700 space-y-1">
            {metrics.concentration_score >= 0.7 && (
              <li>• Highly focused developer with deep expertise in primary repository</li>
            )}
            {metrics.concentration_score < 0.4 && metrics.repos_touched > 5 && (
              <li>• Works across many repositories - consider if this is optimal</li>
            )}
            {metrics.top_repo_percentage > 80 && (
              <li>• Over 80% of work in one repository - strong domain ownership</li>
            )}
            {metrics.repos_touched === 1 && (
              <li>• Single repository focus - excellent for deep specialization</li>
            )}
          </ul>
        </div>
      </div>
    </div>
  );
}
