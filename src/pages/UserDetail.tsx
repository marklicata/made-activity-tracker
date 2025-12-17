import { useParams, useNavigate } from 'react-router-dom';
import { ChevronLeft } from 'lucide-react';

export default function UserDetail() {
  const { username } = useParams<{ username: string }>();
  const navigate = useNavigate();

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Breadcrumb */}
      <div className="bg-white border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <button
            onClick={() => navigate('/team')}
            className="flex items-center text-gray-600 hover:text-gray-900 transition-colors"
          >
            <ChevronLeft className="w-4 h-4 mr-1" />
            <span className="text-sm">Back to Team View</span>
          </button>
          <div className="flex items-center text-sm text-gray-500 mt-2">
            <span>Team</span>
            <span className="mx-2">/</span>
            <span className="text-gray-900 font-medium">{username}</span>
          </div>
        </div>
      </div>

      {/* User Header */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-8">
        <div className="bg-white shadow sm:rounded-lg p-6">
          <div className="flex items-center gap-4">
            <div className="h-16 w-16 rounded-full bg-gray-200 flex items-center justify-center">
              <span className="text-2xl font-semibold text-gray-600">
                {username?.charAt(0).toUpperCase()}
              </span>
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900">@{username}</h1>
              <p className="text-sm text-gray-500">User details will load in Phase 2</p>
            </div>
          </div>
        </div>

        {/* Placeholder for future content */}
        <div className="mt-8 bg-white shadow sm:rounded-lg p-12 text-center">
          <p className="text-gray-500">
            User metrics, timeline, and analytics will be added in upcoming phases.
          </p>
        </div>
      </div>
    </div>
  );
}
