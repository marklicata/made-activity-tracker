import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { User } from '@/types';

interface UserManagerProps {
  onUserAdded: () => void;
  onUserRemoved: () => void;
}

export default function UserManager({ onUserAdded, onUserRemoved }: UserManagerProps) {
  const [username, setUsername] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const handleAddUser = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!username.trim()) return;

    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      await invoke<User>('add_tracked_user', { username: username.trim() });
      setSuccess(`Added ${username} to tracked users`);
      setUsername('');
      onUserAdded();

      // Clear success message after 3 seconds
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="bg-white shadow sm:rounded-lg p-6">
      <h2 className="text-lg font-medium text-gray-900 mb-4">Add Users to Track</h2>

      <form onSubmit={handleAddUser} className="space-y-4">
        <div className="flex gap-4">
          <input
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder="Enter GitHub username..."
            className="flex-1 rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            disabled={loading}
          />
          <button
            type="submit"
            disabled={loading || !username.trim()}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? 'Adding...' : 'Add User'}
          </button>
        </div>

        {error && (
          <div className="rounded-md bg-red-50 p-4">
            <p className="text-sm text-red-800">{error}</p>
          </div>
        )}

        {success && (
          <div className="rounded-md bg-green-50 p-4">
            <p className="text-sm text-green-800">{success}</p>
          </div>
        )}
      </form>

      <p className="mt-4 text-sm text-gray-500">
        Add GitHub usernames to track their activity across all repositories. Users must exist in your synced data.
      </p>
    </div>
  );
}
