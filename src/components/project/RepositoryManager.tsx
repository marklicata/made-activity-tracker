import { useState } from 'react';
import { useConfigStore } from '@stores/configStore';
import { Plus } from 'lucide-react';

interface RepositoryManagerProps {
  onRepositoryAdded?: () => void;
}

export default function RepositoryManager({ onRepositoryAdded }: RepositoryManagerProps) {
  const { addRepository } = useConfigStore();
  const [repoInput, setRepoInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const handleAddRepository = async (e: React.FormEvent) => {
    e.preventDefault();
    const match = repoInput.match(/^([^/]+)\/([^/]+)$/);

    if (!match) {
      setError('Please enter a valid repository in format: owner/repository');
      return;
    }

    const [, owner, name] = match;
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      await addRepository(owner, name);
      setSuccess(`Added ${owner}/${name}`);
      setRepoInput('');
      onRepositoryAdded?.();

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
      <h2 className="text-lg font-medium text-gray-900 mb-4">Add Repository</h2>

      <form onSubmit={handleAddRepository} className="space-y-4">
        <div className="flex gap-4">
          <input
            type="text"
            value={repoInput}
            onChange={(e) => setRepoInput(e.target.value)}
            placeholder="owner/repository"
            className="flex-1 rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            disabled={loading}
          />
          <button
            type="submit"
            disabled={loading || !repoInput.trim()}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            <Plus size={16} />
            {loading ? 'Adding...' : 'Add Repository'}
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
        Add GitHub repositories to track. Format: owner/repository (e.g., facebook/react)
      </p>
    </div>
  );
}
