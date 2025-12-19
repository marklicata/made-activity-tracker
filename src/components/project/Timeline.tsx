import { GitPullRequest, GitMerge, AlertCircle, CheckCircle2, MessageSquare, File } from 'lucide-react';
import clsx from 'clsx';

interface User {
  id: number;
  github_id: number;
  login: string;
  name: string | null;
  avatar_url: string | null;
  is_bot: boolean;
}

interface TimelineEvent {
  id: string;
  event_type: string;
  timestamp: string;
  author: User;
  title: string;
  description: string | null;
  url: string | null;
  metadata: any;
}

interface Repository {
  id: number;
  owner: string;
  name: string;
  github_id: number | null;
  enabled: boolean;
  last_synced_at: string | null;
}

interface TimelineProps {
  events: TimelineEvent[];
  repository: Repository;
}

export default function Timeline({ events, repository }: TimelineProps) {
  function getEventIcon(eventType: string) {
    switch (eventType) {
      case 'pr_opened':
        return <GitPullRequest className="w-4 h-4" />;
      case 'pr_merged':
        return <GitMerge className="w-4 h-4" />;
      case 'issue_opened':
        return <AlertCircle className="w-4 h-4" />;
      case 'issue_closed':
        return <CheckCircle2 className="w-4 h-4" />;
      case 'review':
        return <MessageSquare className="w-4 h-4" />;
      default:
        return <File className="w-4 h-4" />;
    }
  }

  function getEventColor(eventType: string): string {
    switch (eventType) {
      case 'pr_opened':
        return 'bg-blue-100 text-blue-600 border-blue-200';
      case 'pr_merged':
        return 'bg-purple-100 text-purple-600 border-purple-200';
      case 'issue_opened':
        return 'bg-orange-100 text-orange-600 border-orange-200';
      case 'issue_closed':
        return 'bg-green-100 text-green-600 border-green-200';
      case 'review':
        return 'bg-indigo-100 text-indigo-600 border-indigo-200';
      default:
        return 'bg-gray-100 text-gray-600 border-gray-200';
    }
  }

  function getEventLabel(eventType: string): string {
    switch (eventType) {
      case 'pr_opened':
        return 'opened PR';
      case 'pr_merged':
        return 'merged PR';
      case 'issue_opened':
        return 'opened issue';
      case 'issue_closed':
        return 'closed issue';
      case 'review':
        return 'reviewed';
      default:
        return eventType;
    }
  }

  function formatTimestamp(timestamp: string) {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins} min ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 30) return `${diffDays}d ago`;

    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }

  function getGitHubUrl(event: TimelineEvent): string {
    const baseUrl = `https://github.com/${repository.owner}/${repository.name}`;
    if (event.event_type.startsWith('pr_')) {
      return `${baseUrl}/pull/${event.metadata.pr_number}`;
    } else if (event.event_type.startsWith('issue_')) {
      return `${baseUrl}/issues/${event.metadata.issue_number}`;
    }
    return baseUrl;
  }

  if (events.length === 0) {
    return (
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-8">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Timeline</h2>
        <div className="text-center text-gray-500">
          <File className="w-12 h-12 mx-auto mb-2 opacity-50" />
          <p>No activity in the selected time range</p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold text-gray-900">Timeline</h2>
        <span className="text-sm text-gray-500">{events.length} events</span>
      </div>

      <div className="space-y-4">
        {events.map((event) => (
          <div key={event.id} className="flex gap-4 group">
            {/* Icon */}
            <div className={clsx('flex-shrink-0 p-2 rounded-lg border', getEventColor(event.event_type))}>
              {getEventIcon(event.event_type)}
            </div>

            {/* Content */}
            <div className="flex-1 min-w-0">
              <div className="flex items-start justify-between gap-2">
                <div className="flex items-center gap-2 flex-wrap">
                  <img
                    src={event.author.avatar_url || `https://github.com/${event.author.login}.png`}
                    alt={event.author.login}
                    className="w-5 h-5 rounded-full"
                  />
                  <span className="font-medium text-gray-900">{event.author.login}</span>
                  <span className="text-gray-500">{getEventLabel(event.event_type)}</span>
                  {event.event_type.startsWith('pr_') && event.metadata.pr_number && (
                    <span className="text-blue-600 font-mono text-sm">
                      #{event.metadata.pr_number}
                    </span>
                  )}
                  {event.event_type.startsWith('issue_') && event.metadata.issue_number && (
                    <span className="text-orange-600 font-mono text-sm">
                      #{event.metadata.issue_number}
                    </span>
                  )}
                </div>
                <span className="text-sm text-gray-400 whitespace-nowrap">
                  {formatTimestamp(event.timestamp)}
                </span>
              </div>

              <a
                href={getGitHubUrl(event)}
                target="_blank"
                rel="noopener noreferrer"
                className="block mt-1 text-gray-900 hover:text-blue-600 transition-colors group-hover:underline"
              >
                {event.title}
              </a>

              {event.event_type === 'pr_merged' && event.metadata.additions !== undefined && (
                <div className="flex items-center gap-3 mt-2 text-sm text-gray-500">
                  <span>
                    <span className="text-green-600">+{event.metadata.additions}</span>
                    {' / '}
                    <span className="text-red-600">-{event.metadata.deletions}</span>
                  </span>
                  <span>{event.metadata.changed_files} files</span>
                </div>
              )}

              {event.event_type === 'review' && event.metadata.review_state && (
                <div className="mt-2">
                  <span
                    className={clsx(
                      'inline-flex items-center px-2 py-1 rounded text-xs font-medium',
                      event.metadata.review_state === 'APPROVED'
                        ? 'bg-green-100 text-green-700'
                        : event.metadata.review_state === 'CHANGES_REQUESTED'
                        ? 'bg-red-100 text-red-700'
                        : 'bg-gray-100 text-gray-700'
                    )}
                  >
                    {event.metadata.review_state}
                  </span>
                </div>
              )}
            </div>
          </div>
        ))}
      </div>

      {events.length >= 1000 && (
        <div className="mt-6 text-center text-sm text-gray-500">
          Showing first 1000 events. Adjust date range to see more recent activity.
        </div>
      )}
    </div>
  );
}
