/**
 * Unit tests for AmplifierMetricsView component
 *
 * Tests for:
 * - Renders loading state
 * - Renders error state
 * - Renders metrics sections
 * - Time period selector works (7d/30d/90d)
 * - Fetches correct data for selected period
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AmplifierMetricsView } from '@/components/metrics/AmplifierMetricsView';
import type { DashboardMetrics } from '@/types/metrics';

// Mock the usePRMetrics hook
vi.mock('@/hooks/usePRMetrics', () => ({
  usePRMetrics: vi.fn(),
}));

import { usePRMetrics } from '@/hooks/usePRMetrics';

describe('AmplifierMetricsView', () => {
  const mockMetrics: DashboardMetrics = {
    overview: {
      productivity_multiplier: 2.5,
      period_days: 30,
      total_prs: 150,
      active_developers: 5,
    },
    speed: {
      prs_per_day: 5.0,
      prs_per_day_per_dev: 1.0,
      pr_turnaround_hours: 12.5,
      loc_per_day: 1500,
      cycle_time_distribution: {
        under_4h: 30,
        under_4h_pct: 20.0,
        h4_to_12: 60,
        h4_to_12_pct: 40.0,
        h12_to_24: 45,
        h12_to_24_pct: 30.0,
        over_24h: 15,
        over_24h_pct: 10.0,
      },
      benchmark_comparison: {
        prs_per_day_industry: 0.8,
        prs_per_day_elite: 1.5,
        pr_turnaround_industry: 89.0,
        pr_turnaround_elite: 24.0,
      },
    },
    ease: {
      concurrent_repos: 3,
      repos_per_dev: 1.5,
      total_active_repos: 8,
      active_repos: [],
      repo_distribution: {
        org_repos: 5,
        org_repos_pct: 62.5,
        personal_repos: 3,
        personal_repos_pct: 37.5,
      },
      work_pattern: [],
      pr_switch_frequency: 45.5,
      benchmark_comparison: {
        concurrent_repos_industry: 2.1,
        concurrent_repos_elite: 3.5,
      },
    },
    quality: {
      pr_merge_rate: 85.5,
      avg_files_per_pr: 6.2,
      bug_pr_percentage: 20.0,
      feature_pr_percentage: 60.0,
      avg_review_cycle_hours: 4.5,
      avg_review_comments: 3.2,
      pr_type_distribution: [
        { pr_type: 'feature', count: 60, percentage: 60.0 },
        { pr_type: 'bug_fix', count: 20, percentage: 20.0 },
      ],
      files_per_pr_distribution: {
        range_1_3: 40,
        range_1_3_pct: 40.0,
        range_4_8: 35,
        range_4_8_pct: 35.0,
        range_9_15: 15,
        range_9_15_pct: 15.0,
        range_16_plus: 10,
        range_16_plus_pct: 10.0,
      },
      merge_rate_trend: [],
      benchmark_comparison: {
        merge_rate_industry: 68.0,
        merge_rate_elite: 85.0,
        bug_ratio_industry: 25.0,
        bug_ratio_elite: 15.0,
        files_per_pr_industry: 8.0,
      },
    },
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Loading State', () => {
    it('renders loading spinner while fetching', () => {
      vi.mocked(usePRMetrics).mockReturnValue({
        metrics: null,
        loading: true,
        error: null,
        refresh: vi.fn(),
      });

      render(<AmplifierMetricsView />);

      expect(screen.getByText('Loading PR-based metrics...')).toBeInTheDocument();
    });

    it('shows loading spinner icon', () => {
      vi.mocked(usePRMetrics).mockReturnValue({
        metrics: null,
        loading: true,
        error: null,
        refresh: vi.fn(),
      });

      const { container } = render(<AmplifierMetricsView />);

      // Check for spinner animation class
      const spinner = container.querySelector('.animate-spin');
      expect(spinner).toBeInTheDocument();
    });
  });

  describe('Error State', () => {
    it('displays error message on fetch failure', () => {
      vi.mocked(usePRMetrics).mockReturnValue({
        metrics: null,
        loading: false,
        error: 'Failed to fetch metrics',
        refresh: vi.fn(),
      });

      render(<AmplifierMetricsView />);

      expect(screen.getByText('Error loading metrics')).toBeInTheDocument();
      expect(screen.getByText('Failed to fetch metrics')).toBeInTheDocument();
    });

    it('displays error with red styling', () => {
      vi.mocked(usePRMetrics).mockReturnValue({
        metrics: null,
        loading: false,
        error: 'Network error',
        refresh: vi.fn(),
      });

      const { container } = render(<AmplifierMetricsView />);

      const errorBox = container.querySelector('.bg-red-50');
      expect(errorBox).toBeInTheDocument();
    });
  });

  describe('No Metrics State', () => {
    it('displays "no metrics" message when data is null', () => {
      vi.mocked(usePRMetrics).mockReturnValue({
        metrics: null,
        loading: false,
        error: null,
        refresh: vi.fn(),
      });

      render(<AmplifierMetricsView />);

      expect(screen.getByText('No metrics available. Please sync your data first.')).toBeInTheDocument();
    });
  });

  describe('Metrics Sections Rendering', () => {
    beforeEach(() => {
      vi.mocked(usePRMetrics).mockReturnValue({
        metrics: mockMetrics,
        loading: false,
        error: null,
        refresh: vi.fn(),
      });
    });

    it('renders info banner with explanation', () => {
      render(<AmplifierMetricsView />);

      expect(screen.getByText('Amplifier-Style Metrics')).toBeInTheDocument();
      expect(screen.getByText(/These metrics use PR activity data/)).toBeInTheDocument();
    });

    it('renders performance analysis header with period', () => {
      render(<AmplifierMetricsView />);

      expect(screen.getByText('Performance Analysis (30 days)')).toBeInTheDocument();
    });

    it('renders productivity overview section', () => {
      render(<AmplifierMetricsView />);

      expect(screen.getByText('Team Productivity Multiplier')).toBeInTheDocument();
      expect(screen.getByText('2.5×')).toBeInTheDocument();
    });

    it('renders speed metrics section', () => {
      render(<AmplifierMetricsView />);

      expect(screen.getByText('Speed')).toBeInTheDocument();
      expect(screen.getByText('How fast work gets done')).toBeInTheDocument();
    });

    it('renders ease metrics section', () => {
      render(<AmplifierMetricsView />);

      expect(screen.getByText('Ease')).toBeInTheDocument();
      expect(screen.getByText('Capacity for parallel work')).toBeInTheDocument();
    });

    it('renders quality metrics section', () => {
      render(<AmplifierMetricsView />);

      expect(screen.getByText('Quality')).toBeInTheDocument();
      expect(screen.getByText('Standard of work output')).toBeInTheDocument();
    });

    it('renders methodology footer', () => {
      render(<AmplifierMetricsView />);

      expect(screen.getByText('Methodology & Benchmarks')).toBeInTheDocument();
    });
  });

  describe('Time Period Selector', () => {
    beforeEach(() => {
      vi.mocked(usePRMetrics).mockReturnValue({
        metrics: mockMetrics,
        loading: false,
        error: null,
        refresh: vi.fn(),
      });
    });

    it('renders all three period buttons (7d, 30d, 90d)', () => {
      render(<AmplifierMetricsView />);

      expect(screen.getByText('7d')).toBeInTheDocument();
      expect(screen.getByText('30d')).toBeInTheDocument();
      expect(screen.getByText('90d')).toBeInTheDocument();
    });

    it('highlights default period (30d)', () => {
      render(<AmplifierMetricsView days={30} />);

      const button30d = screen.getByText('30d').closest('button');
      expect(button30d?.className).toContain('bg-blue-600');
    });

    it('highlights selected period', () => {
      render(<AmplifierMetricsView days={7} />);

      const button7d = screen.getByText('7d').closest('button');
      expect(button7d?.className).toContain('bg-blue-600');
    });

    it('renders inactive buttons with gray background', () => {
      render(<AmplifierMetricsView days={30} />);

      const button7d = screen.getByText('7d').closest('button');
      expect(button7d?.className).toContain('bg-gray-100');
    });
  });

  describe('Days Prop', () => {
    it('defaults to 30 days when not specified', () => {
      vi.mocked(usePRMetrics).mockReturnValue({
        metrics: mockMetrics,
        loading: false,
        error: null,
        refresh: vi.fn(),
      });

      render(<AmplifierMetricsView />);

      expect(usePRMetrics).toHaveBeenCalledWith({ days: 30 });
    });

    it('uses custom days prop when provided', () => {
      vi.mocked(usePRMetrics).mockReturnValue({
        metrics: mockMetrics,
        loading: false,
        error: null,
        refresh: vi.fn(),
      });

      render(<AmplifierMetricsView days={90} />);

      expect(usePRMetrics).toHaveBeenCalledWith({ days: 90 });
    });
  });

  describe('Edge Cases', () => {
    it('handles metrics with zero values', () => {
      const zeroMetrics = {
        ...mockMetrics,
        overview: {
          ...mockMetrics.overview,
          productivity_multiplier: 0,
          total_prs: 0,
          active_developers: 0,
        },
      };

      vi.mocked(usePRMetrics).mockReturnValue({
        metrics: zeroMetrics,
        loading: false,
        error: null,
        refresh: vi.fn(),
      });

      render(<AmplifierMetricsView />);

      expect(screen.getByText('0.0×')).toBeInTheDocument();
    });

    it('handles very large metrics values', () => {
      const largeMetrics = {
        ...mockMetrics,
        overview: {
          ...mockMetrics.overview,
          productivity_multiplier: 10.5,
          total_prs: 10000,
        },
      };

      vi.mocked(usePRMetrics).mockReturnValue({
        metrics: largeMetrics,
        loading: false,
        error: null,
        refresh: vi.fn(),
      });

      render(<AmplifierMetricsView />);

      expect(screen.getByText('10.5×')).toBeInTheDocument();
    });
  });

  describe('Component Integration', () => {
    it('passes correct data to child components', () => {
      vi.mocked(usePRMetrics).mockReturnValue({
        metrics: mockMetrics,
        loading: false,
        error: null,
        refresh: vi.fn(),
      });

      render(<AmplifierMetricsView />);

      // Verify data flows to ProductivityOverview
      expect(screen.getByText('2.5×')).toBeInTheDocument();
      expect(screen.getByText('150')).toBeInTheDocument();
      expect(screen.getByText('5')).toBeInTheDocument();

      // Verify data flows to sections
      expect(screen.getByText('Speed')).toBeInTheDocument();
      expect(screen.getByText('Ease')).toBeInTheDocument();
      expect(screen.getByText('Quality')).toBeInTheDocument();
    });
  });
});
