import { useState } from 'react';
import { Users, ArrowUpDown, Download, X } from 'lucide-react';
import clsx from 'clsx';

interface User {
  id: number;
  github_id: number;
  login: string;
  name: string | null;
  avatar_url: string | null;
  is_bot: boolean;
}

interface ContributorStats {
  user: User;
  total_commits: number;
  total_prs: number;
  total_prs_reviewed: number;
  total_issues: number;
  lines_added: number;
  lines_deleted: number;
  files_changed: number;
  first_contribution: string;
  last_contribution: string;
  activity_trend: string;
}

interface ContributorTableProps {
  contributors: ContributorStats[];
  onUserSelect: (userId: number | null) => void;
  selectedUserId: number | null;
}

type SortField = 'name' | 'prs' | 'reviews' | 'issues' | 'lines';
type SortDirection = 'asc' | 'desc';

export default function ContributorTable({ contributors, onUserSelect, selectedUserId }: ContributorTableProps) {
  const [sortField, setSortField] = useState<SortField>('prs');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');

  function handleSort(field: SortField) {
    if (sortField === field) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDirection('desc');
    }
  }

  function getSortedContributors() {
    const sorted = [...contributors].sort((a, b) => {
      let aVal, bVal;
      switch (sortField) {
        case 'name':
          aVal = a.user.login.toLowerCase();
          bVal = b.user.login.toLowerCase();
          break;
        case 'prs':
          aVal = a.total_prs;
          bVal = b.total_prs;
          break;
        case 'reviews':
          aVal = a.total_prs_reviewed;
          bVal = b.total_prs_reviewed;
          break;
        case 'issues':
          aVal = a.total_issues;
          bVal = b.total_issues;
          break;
        case 'lines':
          aVal = a.lines_added + a.lines_deleted;
          bVal = b.lines_added + b.lines_deleted;
          break;
        default:
          aVal = 0;
          bVal = 0;
      }

      if (sortDirection === 'asc') {
        return aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
      } else {
        return aVal > bVal ? -1 : aVal < bVal ? 1 : 0;
      }
    });
    return sorted;
  }

  function exportToCSV() {
    const headers = ['Name', 'Login', 'PRs', 'Reviews', 'Issues', 'Lines Added', 'Lines Deleted', 'Files Changed'];
    const rows = contributors.map((c) => [
      c.user.name || c.user.login,
      c.user.login,
      c.total_prs,
      c.total_prs_reviewed,
      c.total_issues,
      c.lines_added,
      c.lines_deleted,
      c.files_changed,
    ]);

    const csv = [headers, ...rows].map((row) => row.join(',')).join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'contributors.csv';
    a.click();
    URL.revokeObjectURL(url);
  }

  const sortedContributors = getSortedContributors();
  const selectedContributor = selectedUserId
    ? contributors.find((c) => c.user.id === selectedUserId)
    : null;

  if (contributors.length === 0) {
    return (
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-8">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Contributors</h2>
        <div className="text-center text-gray-500">
          <Users className="w-12 h-12 mx-auto mb-2 opacity-50" />
          <p>No contributors found in the selected time range</p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-lg font-semibold text-gray-900">Contributors</h2>
          {selectedContributor && (
            <div className="flex items-center gap-2 mt-2">
              <span className="text-sm text-gray-500">
                Filtered to: {selectedContributor.user.login}
              </span>
              <button
                onClick={() => onUserSelect(null)}
                className="text-gray-400 hover:text-gray-600"
                title="Clear filter"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          )}
        </div>
        <button
          onClick={exportToCSV}
          className="flex items-center gap-2 px-4 py-2 text-sm text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
        >
          <Download className="w-4 h-4" />
          Export CSV
        </button>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b border-gray-200">
              <th className="text-left py-3 px-4">
                <button
                  onClick={() => handleSort('name')}
                  className="flex items-center gap-1 text-sm font-medium text-gray-700 hover:text-gray-900"
                >
                  Contributor
                  <ArrowUpDown className="w-3 h-3" />
                </button>
              </th>
              <th className="text-right py-3 px-4">
                <button
                  onClick={() => handleSort('prs')}
                  className="flex items-center gap-1 justify-end text-sm font-medium text-gray-700 hover:text-gray-900 ml-auto"
                >
                  PRs
                  <ArrowUpDown className="w-3 h-3" />
                </button>
              </th>
              <th className="text-right py-3 px-4">
                <button
                  onClick={() => handleSort('reviews')}
                  className="flex items-center gap-1 justify-end text-sm font-medium text-gray-700 hover:text-gray-900 ml-auto"
                >
                  Reviews
                  <ArrowUpDown className="w-3 h-3" />
                </button>
              </th>
              <th className="text-right py-3 px-4">
                <button
                  onClick={() => handleSort('issues')}
                  className="flex items-center gap-1 justify-end text-sm font-medium text-gray-700 hover:text-gray-900 ml-auto"
                >
                  Issues
                  <ArrowUpDown className="w-3 h-3" />
                </button>
              </th>
              <th className="text-right py-3 px-4">
                <button
                  onClick={() => handleSort('lines')}
                  className="flex items-center gap-1 justify-end text-sm font-medium text-gray-700 hover:text-gray-900 ml-auto"
                >
                  Lines
                  <ArrowUpDown className="w-3 h-3" />
                </button>
              </th>
            </tr>
          </thead>
          <tbody>
            {sortedContributors.map((contributor) => (
              <tr
                key={contributor.user.id}
                onClick={() => onUserSelect(contributor.user.id === selectedUserId ? null : contributor.user.id)}
                className={clsx(
                  'border-b border-gray-100 hover:bg-gray-50 cursor-pointer transition-colors',
                  contributor.user.id === selectedUserId && 'bg-blue-50'
                )}
              >
                <td className="py-3 px-4">
                  <div className="flex items-center gap-3">
                    <img
                      src={
                        contributor.user.avatar_url || `https://github.com/${contributor.user.login}.png`
                      }
                      alt={contributor.user.login}
                      className="w-8 h-8 rounded-full"
                    />
                    <div>
                      <div className="font-medium text-gray-900">
                        {contributor.user.name || contributor.user.login}
                      </div>
                      <div className="text-sm text-gray-500">@{contributor.user.login}</div>
                    </div>
                  </div>
                </td>
                <td className="py-3 px-4 text-right">
                  <span className="font-medium text-gray-900">{contributor.total_prs}</span>
                </td>
                <td className="py-3 px-4 text-right">
                  <span className="font-medium text-gray-900">{contributor.total_prs_reviewed}</span>
                </td>
                <td className="py-3 px-4 text-right">
                  <span className="font-medium text-gray-900">{contributor.total_issues}</span>
                </td>
                <td className="py-3 px-4 text-right">
                  <div className="text-sm">
                    <span className="text-green-600">+{contributor.lines_added}</span>
                    {' / '}
                    <span className="text-red-600">-{contributor.lines_deleted}</span>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="mt-4 text-sm text-gray-500">
        Showing {contributors.length} contributor{contributors.length !== 1 ? 's' : ''}
      </div>
    </div>
  );
}
