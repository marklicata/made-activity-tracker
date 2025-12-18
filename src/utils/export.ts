import { UserSummary } from '@/types';

/**
 * Export user summaries to CSV format
 */
export function exportTeamToCSV(summaries: UserSummary[], dateRange: { start: string; end: string }): void {
  // Build CSV header
  const headers = [
    'Username',
    'Name',
    'Activity Status',
    'Total Commits',
    'PRs Created',
    'PRs Merged',
    'PRs Reviewed',
    'Issues Opened',
    'Issues Closed',
    'Lines Added',
    'Lines Deleted',
    'Repositories',
    'First Activity',
    'Last Activity',
  ];

  // Build CSV rows
  const rows = summaries.map(summary => [
    summary.user.login,
    summary.user.name || '',
    summary.activity_status,
    summary.total_commits,
    summary.total_prs_created,
    summary.total_prs_merged,
    summary.total_prs_reviewed,
    summary.total_issues_opened,
    summary.total_issues_closed,
    summary.lines_added,
    summary.lines_deleted,
    summary.repositories_touched,
    summary.first_activity || '',
    summary.last_activity || '',
  ]);

  // Combine headers and rows
  const csvContent = [
    `# Team Activity Report`,
    `# Date Range: ${dateRange.start} to ${dateRange.end}`,
    `# Generated: ${new Date().toISOString()}`,
    '',
    headers.join(','),
    ...rows.map(row => row.map(cell => `"${cell}"`).join(',')),
  ].join('\n');

  // Create download link
  const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
  const link = document.createElement('a');
  const url = URL.createObjectURL(blob);

  link.setAttribute('href', url);
  link.setAttribute('download', `team-activity-${dateRange.start}-to-${dateRange.end}.csv`);
  link.style.visibility = 'hidden';

  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}

/**
 * Export single user summary to CSV format
 */
export function exportUserToCSV(
  summary: UserSummary,
  dateRange: { start: string; end: string }
): void {
  const csvContent = [
    `# User Activity Report: ${summary.user.login}`,
    `# Date Range: ${dateRange.start} to ${dateRange.end}`,
    `# Generated: ${new Date().toISOString()}`,
    '',
    'Metric,Value',
    `Username,"${summary.user.login}"`,
    `Name,"${summary.user.name || ''}"`,
    `Activity Status,"${summary.activity_status}"`,
    `Total Commits,${summary.total_commits}`,
    `PRs Created,${summary.total_prs_created}`,
    `PRs Merged,${summary.total_prs_merged}`,
    `PRs Reviewed,${summary.total_prs_reviewed}`,
    `Issues Opened,${summary.total_issues_opened}`,
    `Issues Closed,${summary.total_issues_closed}`,
    `Lines Added,${summary.lines_added}`,
    `Lines Deleted,${summary.lines_deleted}`,
    `Repositories Touched,${summary.repositories_touched}`,
    `First Activity,"${summary.first_activity || ''}"`,
    `Last Activity,"${summary.last_activity || ''}"`,
  ].join('\n');

  const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
  const link = document.createElement('a');
  const url = URL.createObjectURL(blob);

  link.setAttribute('href', url);
  link.setAttribute('download', `user-${summary.user.login}-${dateRange.start}-to-${dateRange.end}.csv`);
  link.style.visibility = 'hidden';

  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}
