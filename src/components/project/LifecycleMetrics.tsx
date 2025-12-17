import { Clock, GitMerge, AlertTriangle, TrendingUp, ExternalLink } from 'lucide-react';
import clsx from 'clsx';

interface PullRequest {
  id: number;
  github_id: number;
  repo_id: number;
  number: number;
  title: string;
  body: string | null;
  state: string;
  author_id: number | null;
  created_at: string;
  updated_at: string;
  merged_at: string | null;
  closed_at: string | null;
  additions: number;
  deletions: number;
  changed_files: number;
  review_comments: number;
  labels: string[];
}

interface Issue {
  id: number;
  github_id: number;
  repo_id: number;
  number: number;
  title: string;
  body: string | null;
  state: string;
  author_id: number | null;
  assignee_id: number | null;
  milestone_id: number | null;
  created_at: string;
  updated_at: string;
  closed_at: string | null;
  labels: string[];
}

interface LifecycleMetrics {
  avg_time_to_merge: number;
  median_time_to_merge: number;
  p90_time_to_merge: number;
  avg_time_to_first_review: number;
  avg_review_cycles: number;
  open_prs_count: number;
  open_issues_count: number;
  bottleneck_prs: PullRequest[];
  bottleneck_issues: Issue[];
}

interface Repository {
  id: number;
  owner: string;
  name: string;
  github_id: number | null;
  enabled: boolean;
  last_synced_at: string | null;
}

interface LifecycleMetricsProps {
  data: LifecycleMetrics;
  repository: Repository;
}

export default function LifecycleMetrics({ data, repository }: LifecycleMetricsProps) {
  function formatHours(hours: number): string {
    if (hours < 1) return `${Math.round(hours * 60)} min`;
    if (hours < 24) return `${hours.toFixed(1)} hours`;
    const days = hours / 24;
    return `${days.toFixed(1)} days`;
  }

  function getDaysOpen(createdAt: string): number {
    const created = new Date(createdAt);
    const now = new Date();
    const diffMs = now.getTime() - created.getTime();
    return Math.floor(diffMs / 86400000);
  }

  function getGitHubPrUrl(prNumber: number): string {
    return `https://github.com/${repository.owner}/${repository.name}/pull/${prNumber}`;
  }

  function getGitHubIssueUrl(issueNumber: number): string {
    return `https://github.com/${repository.owner}/${repository.name}/issues/${issueNumber}`;
  }

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h2 className="text-lg font-semibold text-gray-900 mb-6 flex items-center gap-2">
        <TrendingUp className="w-5 h-5" />
        Lifecycle Metrics
      </h2>

      {/* Metric Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 mb-8">
        <MetricCard
          icon={GitMerge}
          label="Avg Time to Merge"
          value={formatHours(data.avg_time_to_merge)}
          subtitle={`Median: ${formatHours(data.median_time_to_merge)}`}
          color="blue"
        />
        <MetricCard
          icon={Clock}
          label="Avg Time to First Review"
          value={formatHours(data.avg_time_to_first_review)}
          subtitle="From PR creation"
          color="purple"
        />
        <MetricCard
          icon={TrendingUp}
          label="Avg Review Cycles"
          value={data.avg_review_cycles.toFixed(1)}
          subtitle="Per pull request"
          color="green"
        />
      </div>

      {/* P90 Note */}
      {data.p90_time_to_merge > 0 && (
        <div className="mb-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
          <p className="text-sm text-gray-700">
            <span className="font-medium">90th percentile:</span> {formatHours(data.p90_time_to_merge)} to merge
            <span className="text-gray-500 ml-2">(10% of PRs take longer than this)</span>
          </p>
        </div>
      )}

      {/* Bottlenecks Section */}
      {(data.bottleneck_prs.length > 0 || data.bottleneck_issues.length > 0) && (
        <div className="mt-8">
          <h3 className="text-md font-semibold text-gray-900 mb-4 flex items-center gap-2">
            <AlertTriangle className="w-4 h-4 text-orange-500" />
            Bottlenecks ({data.bottleneck_prs.length + data.bottleneck_issues.length})
          </h3>

          <div className="space-y-4">
            {/* Open PRs */}
            {data.bottleneck_prs.length > 0 && (
              <div>
                <h4 className="text-sm font-medium text-gray-700 mb-2">
                  Open Pull Requests ({data.open_prs_count})
                </h4>
                <div className="space-y-2">
                  {data.bottleneck_prs.map((pr) => (
                    <a
                      key={pr.id}
                      href={getGitHubPrUrl(pr.number)}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="block p-3 bg-orange-50 border border-orange-200 rounded-lg hover:bg-orange-100 transition-colors group"
                    >
                      <div className="flex items-start justify-between gap-2">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <span className="font-mono text-sm text-orange-600">#{pr.number}</span>
                            <span className="text-gray-900 truncate group-hover:underline">
                              {pr.title}
                            </span>
                          </div>
                          <div className="flex items-center gap-3 mt-1 text-sm text-gray-600">
                            <span>Open for {getDaysOpen(pr.created_at)} days</span>
                            {pr.review_comments > 0 && (
                              <span>{pr.review_comments} review comments</span>
                            )}
                          </div>
                        </div>
                        <ExternalLink className="w-4 h-4 text-gray-400 flex-shrink-0" />
                      </div>
                    </a>
                  ))}
                </div>
              </div>
            )}

            {/* Open Issues */}
            {data.bottleneck_issues.length > 0 && (
              <div>
                <h4 className="text-sm font-medium text-gray-700 mb-2">
                  Open Issues ({data.open_issues_count})
                </h4>
                <div className="space-y-2">
                  {data.bottleneck_issues.map((issue) => (
                    <a
                      key={issue.id}
                      href={getGitHubIssueUrl(issue.number)}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="block p-3 bg-yellow-50 border border-yellow-200 rounded-lg hover:bg-yellow-100 transition-colors group"
                    >
                      <div className="flex items-start justify-between gap-2">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <span className="font-mono text-sm text-yellow-600">#{issue.number}</span>
                            <span className="text-gray-900 truncate group-hover:underline">
                              {issue.title}
                            </span>
                          </div>
                          <div className="flex items-center gap-3 mt-1 text-sm text-gray-600">
                            <span>Open for {getDaysOpen(issue.created_at)} days</span>
                            {issue.labels.length > 0 && (
                              <div className="flex gap-1">
                                {issue.labels.slice(0, 3).map((label, idx) => (
                                  <span
                                    key={idx}
                                    className="px-2 py-0.5 bg-gray-200 text-gray-700 rounded text-xs"
                                  >
                                    {label}
                                  </span>
                                ))}
                              </div>
                            )}
                          </div>
                        </div>
                        <ExternalLink className="w-4 h-4 text-gray-400 flex-shrink-0" />
                      </div>
                    </a>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Empty state */}
      {data.bottleneck_prs.length === 0 && data.bottleneck_issues.length === 0 && (
        <div className="text-center py-8 text-gray-500">
          <AlertTriangle className="w-12 h-12 mx-auto mb-2 opacity-50" />
          <p>No bottlenecks found. Everything is moving smoothly!</p>
        </div>
      )}
    </div>
  );
}

interface MetricCardProps {
  icon: React.ElementType;
  label: string;
  value: string;
  subtitle?: string;
  color: 'blue' | 'purple' | 'green';
}

function MetricCard({ icon: Icon, label, value, subtitle, color }: MetricCardProps) {
  const colorClasses = {
    blue: 'bg-blue-50 text-blue-600',
    purple: 'bg-purple-50 text-purple-600',
    green: 'bg-green-50 text-green-600',
  };

  return (
    <div className="bg-gray-50 rounded-lg p-4 border border-gray-200">
      <div className="flex items-center justify-between mb-2">
        <p className="text-sm text-gray-600">{label}</p>
        <div className={clsx('p-2 rounded-lg', colorClasses[color])}>
          <Icon className="w-4 h-4" />
        </div>
      </div>
      <p className="text-2xl font-bold text-gray-900">{value}</p>
      {subtitle && <p className="text-xs text-gray-500 mt-1">{subtitle}</p>}
    </div>
  );
}
