/**
 * Unit tests for EaseSection component
 *
 * Tests for:
 * - Renders all 4 metric cards
 * - Renders repo distribution chart
 * - Renders top repos table
 * - Handles empty repos list
 * - Formats percentages correctly
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { EaseSection } from '@/components/metrics/EaseSection';
import type { EaseMetrics } from '@/types/metrics';

describe('EaseSection', () => {
  const createMockEaseMetrics = (): EaseMetrics => ({
    concurrent_repos: 3,
    repos_per_dev: 1.5,
    total_active_repos: 8,
    active_repos: [
      {
        repo_name: 'org/repo1',
        pr_count: 25,
        total_loc: 5000,
        contributor_count: 5,
        last_activity: '2024-01-15T10:00:00Z',
      },
      {
        repo_name: 'org/repo2',
        pr_count: 15,
        total_loc: 3000,
        contributor_count: 3,
        last_activity: '2024-01-14T10:00:00Z',
      },
    ],
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
  });

  describe('Section Header', () => {
    it('renders section title', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('Ease')).toBeInTheDocument();
    });

    it('renders section description', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('Capacity for parallel work')).toBeInTheDocument();
    });
  });

  describe('Metric Cards', () => {
    it('renders Concurrent Repos card', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('Concurrent Repos')).toBeInTheDocument();
      expect(screen.getByText('3')).toBeInTheDocument();
      expect(screen.getByText('Projects in parallel')).toBeInTheDocument();
    });

    it('renders Repos per Developer card', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('Repos per Developer')).toBeInTheDocument();
      expect(screen.getByText('1.5')).toBeInTheDocument();
      expect(screen.getByText('Multi-tasking capacity')).toBeInTheDocument();
    });

    it('renders Context Switch Rate card', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('Context Switch Rate')).toBeInTheDocument();
      expect(screen.getByText('45.5%')).toBeInTheDocument();
      expect(screen.getByText('PR repo changes')).toBeInTheDocument();
    });

    it('renders Active Repos card', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('Active Repos')).toBeInTheDocument();
      expect(screen.getByText('8')).toBeInTheDocument();
      expect(screen.getByText('With activity this period')).toBeInTheDocument();
    });
  });

  describe('Repository Distribution', () => {
    it('renders distribution section title', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('Repository Distribution')).toBeInTheDocument();
    });

    it('displays organization repos percentage and count', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('Organization Repos')).toBeInTheDocument();
      expect(screen.getByText('62.5% (5)')).toBeInTheDocument();
    });

    it('displays personal repos percentage and count', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('Personal Repos')).toBeInTheDocument();
      expect(screen.getByText('37.5% (3)')).toBeInTheDocument();
    });

    it('renders progress bars for distribution', () => {
      const ease = createMockEaseMetrics();
      const { container } = render(<EaseSection ease={ease} />);

      const bars = container.querySelectorAll('.bg-blue-500, .bg-green-500');
      expect(bars.length).toBeGreaterThan(0);
    });
  });

  describe('Top Active Repositories', () => {
    it('renders top repos section title', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('Top Active Repositories')).toBeInTheDocument();
    });

    it('displays repository names', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('org/repo1')).toBeInTheDocument();
      expect(screen.getByText('org/repo2')).toBeInTheDocument();
    });

    it('displays PR counts for each repo', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('25 PRs')).toBeInTheDocument();
      expect(screen.getByText('15 PRs')).toBeInTheDocument();
    });

    it('displays LOC with formatting', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('5,000 LOC')).toBeInTheDocument();
      expect(screen.getByText('3,000 LOC')).toBeInTheDocument();
    });

    it('displays contributor counts', () => {
      const ease = createMockEaseMetrics();
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('5 contributors')).toBeInTheDocument();
      expect(screen.getByText('3 contributors')).toBeInTheDocument();
    });

    it('limits display to top 10 repos', () => {
      const ease = {
        ...createMockEaseMetrics(),
        active_repos: Array.from({ length: 20 }, (_, i) => ({
          repo_name: `org/repo${i}`,
          pr_count: 10 - i,
          total_loc: 1000,
          contributor_count: 2,
          last_activity: '2024-01-01T00:00:00Z',
        })),
      };

      render(<EaseSection ease={ease} />);

      // Should only show first 10
      expect(screen.getByText('org/repo0')).toBeInTheDocument();
      expect(screen.getByText('org/repo9')).toBeInTheDocument();
      expect(screen.queryByText('org/repo10')).not.toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('handles zero concurrent repos', () => {
      const ease = { ...createMockEaseMetrics(), concurrent_repos: 0 };
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('0')).toBeInTheDocument();
    });

    it('handles zero switch frequency', () => {
      const ease = { ...createMockEaseMetrics(), pr_switch_frequency: 0 };
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('0.0%')).toBeInTheDocument();
    });

    it('handles empty active repos list', () => {
      const ease = { ...createMockEaseMetrics(), active_repos: [] };
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('Top Active Repositories')).toBeInTheDocument();
      // Section should render but be empty
    });

    it('handles 100% org repos', () => {
      const ease = {
        ...createMockEaseMetrics(),
        repo_distribution: {
          org_repos: 10,
          org_repos_pct: 100,
          personal_repos: 0,
          personal_repos_pct: 0,
        },
      };

      render(<EaseSection ease={ease} />);

      expect(screen.getByText('100.0% (10)')).toBeInTheDocument();
      expect(screen.getByText('0.0% (0)')).toBeInTheDocument();
    });

    it('handles large LOC numbers', () => {
      const ease = {
        ...createMockEaseMetrics(),
        active_repos: [
          {
            repo_name: 'org/large-repo',
            pr_count: 100,
            total_loc: 1234567,
            contributor_count: 50,
            last_activity: '2024-01-01T00:00:00Z',
          },
        ],
      };

      render(<EaseSection ease={ease} />);

      expect(screen.getByText('1,234,567 LOC')).toBeInTheDocument();
    });
  });

  describe('Formatting', () => {
    it('formats repos per dev to 1 decimal', () => {
      const ease = { ...createMockEaseMetrics(), repos_per_dev: 2.777 };
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('2.8')).toBeInTheDocument();
    });

    it('formats switch frequency to 1 decimal', () => {
      const ease = { ...createMockEaseMetrics(), pr_switch_frequency: 33.333 };
      render(<EaseSection ease={ease} />);

      expect(screen.getByText('33.3%')).toBeInTheDocument();
    });

    it('formats repo distribution percentages to 1 decimal', () => {
      const ease = {
        ...createMockEaseMetrics(),
        repo_distribution: {
          org_repos: 2,
          org_repos_pct: 66.666,
          personal_repos: 1,
          personal_repos_pct: 33.333,
        },
      };

      render(<EaseSection ease={ease} />);

      expect(screen.getByText('66.7% (2)')).toBeInTheDocument();
      expect(screen.getByText('33.3% (1)')).toBeInTheDocument();
    });
  });
});
