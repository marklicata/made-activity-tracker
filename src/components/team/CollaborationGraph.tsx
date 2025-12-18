import { CollaborationMatrix } from '@/types';

interface CollaborationGraphProps {
  matrix: CollaborationMatrix;
}

export default function CollaborationGraph({ matrix }: CollaborationGraphProps) {
  if (matrix.users.length === 0) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Team Collaboration</h2>
        <p className="text-gray-500">No collaboration data available. Track more users to see interactions.</p>
      </div>
    );
  }

  // Calculate max value for color scaling
  const allValues: number[] = [];
  matrix.users.forEach(user => {
    const interactions = matrix.interactions[user.login];
    if (interactions) {
      Object.values(interactions).forEach(stats => {
        allValues.push(stats.reviews_given);
      });
    }
  });
  const maxReviews = Math.max(...allValues, 1);

  // Get color based on review count
  const getColorClass = (count: number): string => {
    if (count === 0) return 'bg-gray-50 text-gray-400';
    const intensity = count / maxReviews;
    if (intensity >= 0.7) return 'bg-blue-600 text-white font-semibold';
    if (intensity >= 0.4) return 'bg-blue-400 text-white';
    if (intensity >= 0.2) return 'bg-blue-200 text-gray-900';
    return 'bg-blue-100 text-gray-700';
  };

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-2">Team Collaboration</h2>
        <p className="text-sm text-gray-500 mb-4">
          Shows who reviews whose PRs. Rows are reviewers, columns are PR authors.
        </p>

        <div className="overflow-x-auto">
          <div className="inline-block min-w-full align-middle">
            <table className="min-w-full border-collapse">
              <thead>
                <tr>
                  <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-b border-r">
                    Reviewer →<br />Author ↓
                  </th>
                  {matrix.users.map(user => (
                    <th
                      key={user.id}
                      className="px-4 py-2 text-center text-xs font-medium text-gray-900 border-b"
                    >
                      <div className="flex flex-col items-center gap-1">
                        {user.avatar_url && (
                          <img
                            src={user.avatar_url}
                            alt={user.login}
                            className="w-6 h-6 rounded-full"
                          />
                        )}
                        <span className="truncate max-w-[80px]" title={user.login}>
                          {user.login}
                        </span>
                      </div>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {matrix.users.map(author => (
                  <tr key={author.id} className="hover:bg-gray-50">
                    <td className="px-4 py-2 text-sm font-medium text-gray-900 border-r bg-gray-50">
                      <div className="flex items-center gap-2">
                        {author.avatar_url && (
                          <img
                            src={author.avatar_url}
                            alt={author.login}
                            className="w-6 h-6 rounded-full"
                          />
                        )}
                        <span className="truncate max-w-[120px]" title={author.login}>
                          {author.login}
                        </span>
                      </div>
                    </td>
                    {matrix.users.map(reviewer => {
                      if (author.id === reviewer.id) {
                        return (
                          <td
                            key={reviewer.id}
                            className="px-4 py-2 text-center bg-gray-100"
                          >
                            <span className="text-gray-400">-</span>
                          </td>
                        );
                      }

                      const interactions = matrix.interactions[reviewer.login];
                      const stats = interactions?.[author.login];
                      const reviewCount = stats?.reviews_given || 0;

                      return (
                        <td
                          key={reviewer.id}
                          className="px-4 py-2 text-center"
                        >
                          <div
                            className={`inline-flex items-center justify-center w-12 h-12 rounded-lg transition-colors ${getColorClass(reviewCount)}`}
                            title={`${reviewer.login} reviewed ${reviewCount} of ${author.login}'s PRs`}
                          >
                            {reviewCount > 0 ? reviewCount : ''}
                          </div>
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* Legend */}
        <div className="mt-4 flex items-center justify-end gap-4 text-xs text-gray-500">
          <span>Review Count:</span>
          <div className="flex items-center gap-2">
            <div className="w-6 h-6 bg-gray-50 border border-gray-200 rounded"></div>
            <span>0</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-6 h-6 bg-blue-100 rounded"></div>
            <span>Low</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-6 h-6 bg-blue-400 rounded"></div>
            <span>Medium</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-6 h-6 bg-blue-600 rounded"></div>
            <span>High</span>
          </div>
        </div>

        {/* Insights */}
        {(() => {
          // Find most active reviewer
          let maxReviewsGiven = 0;
          let mostActiveReviewer = '';

          matrix.users.forEach(reviewer => {
            const interactions = matrix.interactions[reviewer.login];
            if (interactions) {
              const totalReviews = Object.values(interactions).reduce(
                (sum, stats) => sum + stats.reviews_given,
                0
              );
              if (totalReviews > maxReviewsGiven) {
                maxReviewsGiven = totalReviews;
                mostActiveReviewer = reviewer.login;
              }
            }
          });

          if (mostActiveReviewer && maxReviewsGiven > 0) {
            return (
              <div className="mt-4 p-3 bg-blue-50 rounded-lg">
                <p className="text-sm text-gray-700">
                  <span className="font-semibold">{mostActiveReviewer}</span> is the most active reviewer with{' '}
                  <span className="font-semibold">{maxReviewsGiven}</span> reviews given.
                </p>
              </div>
            );
          }
          return null;
        })()}
      </div>
    </div>
  );
}
