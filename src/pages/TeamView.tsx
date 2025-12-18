import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { User, UserSummary, CollaborationMatrix } from '@/types';
import { Loader2, Users as UsersIcon } from 'lucide-react';
import UserManager from '@components/team/UserManager';
import UserCard from '@components/team/UserCard';
import TeamSummary from '@components/team/TeamSummary';
import CollaborationGraph from '@components/team/CollaborationGraph';

export default function TeamView() {
  const [trackedUsers, setTrackedUsers] = useState<User[]>([]);
  const [userSummaries, setUserSummaries] = useState<UserSummary[]>([]);
  const [collaborationMatrix, setCollaborationMatrix] = useState<CollaborationMatrix | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Date range - default to last 30 days
  const [dateRange] = useState(() => {
    const endDate = new Date();
    const startDate = new Date();
    startDate.setDate(startDate.getDate() - 30);
    return {
      start: startDate.toISOString().split('T')[0],
      end: endDate.toISOString().split('T')[0],
    };
  });

  useEffect(() => {
    loadTrackedUsers();
  }, []);

  const loadTrackedUsers = async () => {
    setLoading(true);
    setError(null);

    try {
      const users = await invoke<User[]>('get_tracked_users');
      setTrackedUsers(users);

      if (users.length === 0) {
        setUserSummaries([]);
        setCollaborationMatrix(null);
        return;
      }

      // Load summaries for each user
      const summaries = await Promise.all(
        users.map(user =>
          invoke<UserSummary>('get_user_summary', {
            username: user.login,
            startDate: dateRange.start,
            endDate: dateRange.end,
          })
        )
      );
      setUserSummaries(summaries);

      // Load collaboration matrix if we have multiple users
      if (users.length > 1) {
        try {
          const matrix = await invoke<CollaborationMatrix>('get_team_collaboration_matrix', {
            usernames: users.map(u => u.login),
            startDate: dateRange.start,
            endDate: dateRange.end,
          });
          setCollaborationMatrix(matrix);
        } catch (matrixErr) {
          console.error('Failed to load collaboration matrix:', matrixErr);
          setCollaborationMatrix(null);
        }
      } else {
        setCollaborationMatrix(null);
      }
    } catch (err) {
      setError(err as string);
      console.error('Failed to load tracked users:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <h1 className="text-3xl font-bold text-gray-900">Team View</h1>
          <p className="mt-2 text-gray-600">Track specific users across all repositories</p>
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-8 space-y-8">
        {/* User Management */}
        <UserManager onUserAdded={loadTrackedUsers} onUserRemoved={loadTrackedUsers} />

        {loading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
            <span className="ml-3 text-lg text-gray-600">Loading tracked users...</span>
          </div>
        ) : error ? (
          <div className="bg-red-50 rounded-lg p-4">
            <p className="text-red-800">{error}</p>
          </div>
        ) : userSummaries.length === 0 ? (
          // Empty State
          <div className="bg-white shadow sm:rounded-lg p-12 text-center">
            <UsersIcon className="mx-auto h-12 w-12 text-gray-400" strokeWidth={1.5} />
            <h3 className="mt-2 text-sm font-medium text-gray-900">No tracked users</h3>
            <p className="mt-1 text-sm text-gray-500">
              Get started by adding a GitHub username above.
            </p>
          </div>
        ) : (
          <>
            {/* Team Summary */}
            <TeamSummary summaries={userSummaries} />

            {/* User Cards Grid */}
            <div>
              <h2 className="text-lg font-semibold text-gray-900 mb-4">
                Tracked Users ({userSummaries.length})
              </h2>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {userSummaries.map(summary => (
                  <UserCard
                    key={summary.user.id}
                    summary={summary}
                    onRemove={loadTrackedUsers}
                  />
                ))}
              </div>
            </div>

            {/* Collaboration Matrix */}
            {collaborationMatrix && userSummaries.length > 1 && (
              <CollaborationGraph matrix={collaborationMatrix} />
            )}
          </>
        )}
      </div>
    </div>
  );
}
