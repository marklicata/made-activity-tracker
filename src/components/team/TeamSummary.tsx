import { UserSummary } from '@/types';
import { GitPullRequest, MessageSquare, CheckCircle, Users } from 'lucide-react';

interface TeamSummaryProps {
  summaries: UserSummary[];
}

export default function TeamSummary({ summaries }: TeamSummaryProps) {
  if (summaries.length === 0) return null;

  const totalPRs = summaries.reduce((sum, s) => sum + s.total_prs_created, 0);
  const totalReviews = summaries.reduce((sum, s) => sum + s.total_prs_reviewed, 0);
  const totalIssues = summaries.reduce((sum, s) => sum + s.total_issues_opened, 0);
  const totalMerged = summaries.reduce((sum, s) => sum + s.total_prs_merged, 0);

  const activeCount = summaries.filter(s => s.activity_status === 'active').length;

  return (
    <div className="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg p-6 border border-blue-100">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold text-gray-900">Team Summary</h2>
        <span className="text-sm text-gray-600">
          {summaries.length} {summaries.length === 1 ? 'user' : 'users'} tracked
        </span>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="bg-white rounded-lg p-4 shadow-sm">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-blue-100 rounded-lg">
              <GitPullRequest size={20} className="text-blue-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-gray-900">{totalPRs}</p>
              <p className="text-xs text-gray-500">PRs Created</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg p-4 shadow-sm">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-green-100 rounded-lg">
              <CheckCircle size={20} className="text-green-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-gray-900">{totalMerged}</p>
              <p className="text-xs text-gray-500">PRs Merged</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg p-4 shadow-sm">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-purple-100 rounded-lg">
              <MessageSquare size={20} className="text-purple-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-gray-900">{totalReviews}</p>
              <p className="text-xs text-gray-500">Reviews</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg p-4 shadow-sm">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-amber-100 rounded-lg">
              <Users size={20} className="text-amber-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-gray-900">{activeCount}</p>
              <p className="text-xs text-gray-500">Active</p>
            </div>
          </div>
        </div>
      </div>

      {totalIssues > 0 && (
        <p className="mt-4 text-sm text-gray-600">
          {totalIssues} issues opened across all tracked users
        </p>
      )}
    </div>
  );
}
