import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/tauri';
import { UserSummary } from '@/types';
import { formatDistanceToNow } from 'date-fns';
import { Trash2 } from 'lucide-react';

interface UserCardProps {
  summary: UserSummary;
  onRemove: () => void;
}

export default function UserCard({ summary, onRemove }: UserCardProps) {
  const navigate = useNavigate();
  const { user, activity_status, last_activity } = summary;

  const statusColors = {
    active: 'bg-green-100 text-green-800',
    quiet: 'bg-yellow-100 text-yellow-800',
    idle: 'bg-gray-100 text-gray-800',
  };

  const statusDots = {
    active: 'bg-green-500',
    quiet: 'bg-yellow-500',
    idle: 'bg-gray-500',
  };

  const handleRemove = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!confirm(`Remove ${user.login} from tracked users?`)) return;

    try {
      await invoke('remove_tracked_user', { username: user.login });
      onRemove();
    } catch (err) {
      alert(`Failed to remove user: ${err}`);
    }
  };

  const lastActivityText = last_activity
    ? formatDistanceToNow(new Date(last_activity), { addSuffix: true })
    : 'No activity';

  return (
    <div
      className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 hover:shadow-md transition-shadow cursor-pointer"
      onClick={() => navigate(`/team/${user.login}`)}
    >
      {/* User Header */}
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          {user.avatar_url ? (
            <img
              src={user.avatar_url}
              alt={user.login}
              className="w-12 h-12 rounded-full"
            />
          ) : (
            <div className="w-12 h-12 rounded-full bg-gray-200 flex items-center justify-center">
              <span className="text-xl font-semibold text-gray-600">
                {user.login.charAt(0).toUpperCase()}
              </span>
            </div>
          )}
          <div>
            <h3 className="font-medium text-gray-900">
              {user.name || user.login}
            </h3>
            <p className="text-sm text-gray-500">@{user.login}</p>
          </div>
        </div>
        <button
          onClick={handleRemove}
          className="text-gray-400 hover:text-red-600 transition-colors"
          title="Remove from tracked users"
        >
          <Trash2 size={16} />
        </button>
      </div>

      {/* Activity Status */}
      <div className="flex items-center gap-2 mb-4">
        <span className={`w-2 h-2 rounded-full ${statusDots[activity_status]}`} />
        <span className={`text-xs font-medium px-2 py-1 rounded-full ${statusColors[activity_status]}`}>
          {activity_status.charAt(0).toUpperCase() + activity_status.slice(1)}
        </span>
        <span className="text-xs text-gray-500">Last: {lastActivityText}</span>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-2 gap-3 mb-4">
        <div>
          <p className="text-2xl font-bold text-gray-900">{summary.total_prs_created}</p>
          <p className="text-xs text-gray-500">PRs Created</p>
        </div>
        <div>
          <p className="text-2xl font-bold text-gray-900">{summary.total_prs_reviewed}</p>
          <p className="text-xs text-gray-500">PRs Reviewed</p>
        </div>
        <div>
          <p className="text-2xl font-bold text-gray-900">{summary.total_issues_opened}</p>
          <p className="text-xs text-gray-500">Issues</p>
        </div>
        <div>
          <p className="text-2xl font-bold text-gray-900">{summary.repositories_touched}</p>
          <p className="text-xs text-gray-500">Repos</p>
        </div>
      </div>

      {/* Code Stats */}
      <div className="pt-3 border-t border-gray-100">
        <p className="text-xs text-gray-500">
          <span className="text-green-600 font-medium">+{summary.lines_added.toLocaleString()}</span>
          {' / '}
          <span className="text-red-600 font-medium">-{summary.lines_deleted.toLocaleString()}</span>
          {' lines'}
        </p>
      </div>
    </div>
  );
}
