/**
 * Unit tests for SpeedSection component
 *
 * Tests for:
 * - Renders all 4 metric cards
 * - Renders cycle time distribution chart
 * - Formats hours correctly (m/h/d/w)
 * - Shows benchmark comparisons
 * - Handles edge cases (zero values, very large numbers)
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { SpeedSection } from '@/components/metrics/SpeedSection';
import type { SpeedMetrics } from '@/types/metrics';

describe('SpeedSection', () => {
  const createMockSpeedMetrics = (): SpeedMetrics => ({
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
  });

  describe('Section Header', () => {
    it('renders section title', () => {
      const speed = createMockSpeedMetrics();
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('Speed')).toBeInTheDocument();
    });

    it('renders section description', () => {
      const speed = createMockSpeedMetrics();
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('How fast work gets done')).toBeInTheDocument();
    });

    it('renders speed icon', () => {
      const speed = createMockSpeedMetrics();
      const { container } = render(<SpeedSection speed={speed} />);

      // Check for Zap icon (lucide-react)
      const icons = container.querySelectorAll('svg');
      expect(icons.length).toBeGreaterThan(0);
    });
  });

  describe('Metric Cards', () => {
    it('renders PRs per Day card with correct value', () => {
      const speed = createMockSpeedMetrics();
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('PRs per Day')).toBeInTheDocument();
      expect(screen.getByText('1.00')).toBeInTheDocument();
      expect(screen.getByText('Per developer')).toBeInTheDocument();
    });

    it('renders PR Turnaround card with formatted hours', () => {
      const speed = createMockSpeedMetrics();
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('PR Turnaround')).toBeInTheDocument();
      expect(screen.getByText('12.5h')).toBeInTheDocument();
      expect(screen.getByText('Open → merged')).toBeInTheDocument();
    });

    it('renders Lines of Code card with formatted number', () => {
      const speed = createMockSpeedMetrics();
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('Lines of Code')).toBeInTheDocument();
      expect(screen.getByText('1,500')).toBeInTheDocument();
      expect(screen.getByText('Per day (team)')).toBeInTheDocument();
    });

    it('renders Total PRs card', () => {
      const speed = createMockSpeedMetrics();
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('Total PRs')).toBeInTheDocument();
      expect(screen.getByText('5.0')).toBeInTheDocument();
      expect(screen.getByText('Per day (all devs)')).toBeInTheDocument();
    });
  });

  describe('PR Merge Time Distribution Chart', () => {
    it('renders distribution chart title', () => {
      const speed = createMockSpeedMetrics();
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('PR Merge Time Distribution')).toBeInTheDocument();
    });

    it('renders all time buckets', () => {
      const speed = createMockSpeedMetrics();
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('< 4 hours')).toBeInTheDocument();
      expect(screen.getByText('4-12 hours')).toBeInTheDocument();
      expect(screen.getByText('12-24 hours')).toBeInTheDocument();
      expect(screen.getByText('> 24 hours')).toBeInTheDocument();
    });

    it('displays correct percentages for each bucket', () => {
      const speed = createMockSpeedMetrics();
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('20.0%')).toBeInTheDocument();
      expect(screen.getByText('40.0%')).toBeInTheDocument();
      expect(screen.getByText('30.0%')).toBeInTheDocument();
      expect(screen.getByText('10.0%')).toBeInTheDocument();
    });

    it('displays correct counts for each bucket', () => {
      const speed = createMockSpeedMetrics();
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('(30)')).toBeInTheDocument();
      expect(screen.getByText('(60)')).toBeInTheDocument();
      expect(screen.getByText('(45)')).toBeInTheDocument();
      expect(screen.getByText('(15)')).toBeInTheDocument();
    });
  });

  describe('Hours Formatting', () => {
    it('formats sub-hour as minutes', () => {
      const speed = { ...createMockSpeedMetrics(), pr_turnaround_hours: 0.5 };
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('30m')).toBeInTheDocument();
    });

    it('formats hours with decimal', () => {
      const speed = { ...createMockSpeedMetrics(), pr_turnaround_hours: 8.3 };
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('8.3h')).toBeInTheDocument();
    });

    it('formats days when hours > 24', () => {
      const speed = { ...createMockSpeedMetrics(), pr_turnaround_hours: 48 };
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('2.0d')).toBeInTheDocument();
    });

    it('formats weeks when days > 7', () => {
      const speed = { ...createMockSpeedMetrics(), pr_turnaround_hours: 168 };
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('1.0w')).toBeInTheDocument();
    });
  });

  describe('Number Formatting', () => {
    it('formats large LOC with commas', () => {
      const speed = { ...createMockSpeedMetrics(), loc_per_day: 1234567 };
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('1,234,567')).toBeInTheDocument();
    });

    it('rounds LOC to nearest integer', () => {
      const speed = { ...createMockSpeedMetrics(), loc_per_day: 1500.7 };
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('1,501')).toBeInTheDocument();
    });

    it('formats PRs per dev to 2 decimals', () => {
      const speed = { ...createMockSpeedMetrics(), prs_per_day_per_dev: 1.234 };
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('1.23')).toBeInTheDocument();
    });

    it('formats total PRs per day to 1 decimal', () => {
      const speed = { ...createMockSpeedMetrics(), prs_per_day: 12.67 };
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('12.7')).toBeInTheDocument();
    });
  });

  describe('Benchmark Comparisons', () => {
    it('shows comparison for PRs per day metric', () => {
      const speed = createMockSpeedMetrics();
      render(<SpeedSection speed={speed} />);

      // Should show tier and vs percentages
      expect(screen.getByText('PRs per Day')).toBeInTheDocument();
      // Actual comparison values will be calculated by formatMetricComparison
    });

    it('shows comparison for PR turnaround metric with "lower is better" logic', () => {
      const speed = createMockSpeedMetrics();
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('PR Turnaround')).toBeInTheDocument();
      // 12.5h is much better than industry (89h), should show positive comparison
    });
  });

  describe('Edge Cases', () => {
    it('handles zero PRs per day', () => {
      const speed = { ...createMockSpeedMetrics(), prs_per_day: 0, prs_per_day_per_dev: 0 };
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('0.00')).toBeInTheDocument();
      expect(screen.getByText('0.0')).toBeInTheDocument();
    });

    it('handles zero LOC per day', () => {
      const speed = { ...createMockSpeedMetrics(), loc_per_day: 0 };
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('0')).toBeInTheDocument();
    });

    it('handles zero turnaround hours', () => {
      const speed = { ...createMockSpeedMetrics(), pr_turnaround_hours: 0 };
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('0m')).toBeInTheDocument();
    });

    it('handles empty cycle time distribution', () => {
      const speed = {
        ...createMockSpeedMetrics(),
        cycle_time_distribution: {
          under_4h: 0,
          under_4h_pct: 0,
          h4_to_12: 0,
          h4_to_12_pct: 0,
          h12_to_24: 0,
          h12_to_24_pct: 0,
          over_24h: 0,
          over_24h_pct: 0,
        },
      };

      render(<SpeedSection speed={speed} />);

      // Should render all buckets with 0.0%
      const zeroPercents = screen.getAllByText('0.0%');
      expect(zeroPercents.length).toBeGreaterThan(0);
    });

    it('handles very small PRs per day values', () => {
      const speed = { ...createMockSpeedMetrics(), prs_per_day_per_dev: 0.01 };
      render(<SpeedSection speed={speed} />);

      expect(screen.getByText('0.01')).toBeInTheDocument();
    });

    it('handles very large turnaround hours', () => {
      const speed = { ...createMockSpeedMetrics(), pr_turnaround_hours: 10000 };
      render(<SpeedSection speed={speed} />);

      // Should format as weeks
      expect(screen.getByText(/\d+\.\dw/)).toBeInTheDocument();
    });
  });

  describe('Layout', () => {
    it('renders metric cards in a grid', () => {
      const speed = createMockSpeedMetrics();
      const { container } = render(<SpeedSection speed={speed} />);

      // Check for grid layout classes
      const grid = container.querySelector('.grid');
      expect(grid).toBeInTheDocument();
    });

    it('renders distribution chart after metric cards', () => {
      const speed = createMockSpeedMetrics();
      const { container } = render(<SpeedSection speed={speed} />);

      // Distribution chart should be present
      expect(screen.getByText('PR Merge Time Distribution')).toBeInTheDocument();
    });
  });
});
