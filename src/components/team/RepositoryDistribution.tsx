import { useNavigate } from 'react-router-dom';
import { RepositoryContribution } from '@/types';
import { GitPullRequest, MessageCircle, AlertCircle, ExternalLink } from 'lucide-react';

interface RepositoryDistributionProps {
  contributions: RepositoryContribution[];
}

export default function RepositoryDistribution({ contributions }: RepositoryDistributionProps) {
  const navigate = useNavigate();

  if (contributions.length === 0) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Repository Distribution</h2>
        <p className="text-gray-500">No repository contributions found in this date range.</p>
      </div>
    );
  }

  // Calculate focus score (higher = more focused on fewer repos)
  const focusScore = contributions.length > 0
    ? contributions[0].percentage_of_user_work / 100
    : 0;

  const focusLabel = focusScore >= 0.6
    ? 'Highly focused'
    : focusScore >= 0.4
    ? 'Moderately focused'
    : 'Distributed';

  const focusColor = focusScore >= 0.6
    ? 'text-green-600'
    : focusScore >= 0.4
    ? 'text-yellow-600'
    : 'text-blue-600';

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900">Repository Distribution</h2>
          <div className="text-sm">
            <span className="text-gray-500">Focus: </span>
            <span className={`font-medium ${focusColor}`}>{focusLabel}</span>
          </div>
        </div>

        <div className="space-y-4">
          {contributions.map((contrib) => (
            <div key={contrib.repo_id} className="border-l-4 border-blue-500 pl-4">
              <div className="flex items-start justify-between mb-2">
                <div className="flex-1">
                  <button
                    onClick={() => navigate(`/projects/${contrib.owner}/${contrib.name}`)}
                    className="flex items-center gap-2 text-gray-900 hover:text-blue-600 transition-colors group"
                  >
                    <h3 className="font-medium">
                      {contrib.owner}/{contrib.name}
                    </h3>
                    <ExternalLink size={14} className="opacity-0 group-hover:opacity-100 transition-opacity" />
                  </button>
                  <p className="text-sm text-gray-500 mt-1">
                    {contrib.percentage_of_user_work.toFixed(1)}% of user's work
                  </p>
                </div>
                <div className="text-right">
                  <p className="text-2xl font-bold text-gray-900">{contrib.total_contributions}</p>
                  <p className="text-xs text-gray-500">contributions</p>
                </div>
              </div>

              {/* Progress Bar */}
              <div className="w-full bg-gray-200 rounded-full h-2 mb-3">
                <div
                  className="bg-blue-600 h-2 rounded-full"
                  style={{ width: `${Math.min(contrib.percentage_of_user_work, 100)}%` }}
                />
              </div>

              {/* Breakdown */}
              <div className="flex items-center gap-4 text-sm">
                {contrib.pr_count > 0 && (
                  <div className="flex items-center gap-1 text-gray-600">
                    <GitPullRequest size={14} />
                    <span>{contrib.pr_count} PRs</span>
                  </div>
                )}
                {contrib.review_count > 0 && (
                  <div className="flex items-center gap-1 text-gray-600">
                    <MessageCircle size={14} />
                    <span>{contrib.review_count} reviews</span>
                  </div>
                )}
                {contrib.issue_count > 0 && (
                  <div className="flex items-center gap-1 text-gray-600">
                    <AlertCircle size={14} />
                    <span>{contrib.issue_count} issues</span>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>

        {contributions.length > 5 && (
          <p className="mt-4 text-sm text-gray-500 text-center">
            Showing top {contributions.length} repositories
          </p>
        )}
      </div>
    </div>
  );
}
