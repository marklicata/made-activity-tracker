/**
 * Unit tests for ProductivityOverview component
 *
 * Tests for:
 * - Rendering with different tier levels
 * - Color coding by tier (red/yellow/blue/purple)
 * - Period, PRs, and developers display
 * - Decimal formatting
 * - Formula breakdown display
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ProductivityOverview } from '@/components/metrics/ProductivityOverview';
import type { OverviewMetrics } from '@/types/metrics';

describe('ProductivityOverview', () => {
  const createMockOverview = (multiplier: number): OverviewMetrics => ({
    productivity_multiplier: multiplier,
    period_days: 30,
    total_prs: 150,
    active_developers: 5,
  });

  describe('Tier Classification', () => {
    it('renders "Below Industry Average" tier for multiplier < 0.8', () => {
      const overview = createMockOverview(0.5);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('Below Industry Average')).toBeInTheDocument();
      expect(screen.getByText('0.5×')).toBeInTheDocument();
    });

    it('renders "Industry Average Performance" tier for multiplier 0.8-1.5', () => {
      const overview = createMockOverview(1.2);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('Industry Average Performance')).toBeInTheDocument();
      expect(screen.getByText('1.2×')).toBeInTheDocument();
    });

    it('renders "Elite Tier Performance" tier for multiplier 1.5-3.0', () => {
      const overview = createMockOverview(2.5);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('Elite Tier Performance')).toBeInTheDocument();
      expect(screen.getByText('2.5×')).toBeInTheDocument();
    });

    it('renders "Exceptional Performance" tier for multiplier >= 3.0', () => {
      const overview = createMockOverview(4.2);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('Exceptional Performance')).toBeInTheDocument();
      expect(screen.getByText('4.2×')).toBeInTheDocument();
    });
  });

  describe('Baseline Comparison', () => {
    it('shows percentage above baseline for multiplier > 1', () => {
      const overview = createMockOverview(2.5);
      render(<ProductivityOverview overview={overview} />);

      // 2.5 - 1 = 1.5 = 150% above baseline
      expect(screen.getByText('150% above baseline')).toBeInTheDocument();
    });

    it('shows percentage below baseline for multiplier < 1', () => {
      const overview = createMockOverview(0.6);
      render(<ProductivityOverview overview={overview} />);

      // 1 - 0.6 = 0.4 = 40% below baseline
      expect(screen.getByText('40% below baseline')).toBeInTheDocument();
    });

    it('handles multiplier exactly at 1.0', () => {
      const overview = createMockOverview(1.0);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('0% above baseline')).toBeInTheDocument();
    });
  });

  describe('Overview Stats Display', () => {
    it('displays period days correctly', () => {
      const overview = createMockOverview(2.0);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('30 days')).toBeInTheDocument();
      expect(screen.getByText('Period')).toBeInTheDocument();
    });

    it('displays total PRs correctly', () => {
      const overview = createMockOverview(2.0);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('150')).toBeInTheDocument();
      expect(screen.getByText('Total PRs')).toBeInTheDocument();
    });

    it('displays active developers correctly', () => {
      const overview = createMockOverview(2.0);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('5')).toBeInTheDocument();
      expect(screen.getByText('Active Devs')).toBeInTheDocument();
    });

    it('handles different period lengths', () => {
      const overview = { ...createMockOverview(2.0), period_days: 7 };
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('7 days')).toBeInTheDocument();
    });

    it('handles large PR counts', () => {
      const overview = { ...createMockOverview(2.0), total_prs: 1250 };
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('1250')).toBeInTheDocument();
    });
  });

  describe('Decimal Formatting', () => {
    it('formats multiplier to 1 decimal place', () => {
      const overview = createMockOverview(2.567);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('2.6×')).toBeInTheDocument();
    });

    it('handles very small multipliers', () => {
      const overview = createMockOverview(0.123);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('0.1×')).toBeInTheDocument();
    });

    it('handles zero multiplier', () => {
      const overview = createMockOverview(0);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('0.0×')).toBeInTheDocument();
    });
  });

  describe('Formula Breakdown', () => {
    it('displays formula breakdown section', () => {
      const overview = createMockOverview(2.0);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('Formula Breakdown:')).toBeInTheDocument();
      expect(screen.getByText('35% PR Velocity + 25% PR Speed + 25% Repo Capacity + 15% Quality')).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('handles null/undefined multiplier gracefully', () => {
      const overview = { ...createMockOverview(2.0), productivity_multiplier: null as any };
      render(<ProductivityOverview overview={overview} />);

      // Should default to 0
      expect(screen.getByText('0.0×')).toBeInTheDocument();
    });

    it('handles negative multiplier (edge case)', () => {
      const overview = createMockOverview(-1.5);
      render(<ProductivityOverview overview={overview} />);

      // Should render but show below industry
      expect(screen.getByText('-1.5×')).toBeInTheDocument();
      expect(screen.getByText('Below Industry Average')).toBeInTheDocument();
    });

    it('handles very large multipliers', () => {
      const overview = createMockOverview(10.7);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('10.7×')).toBeInTheDocument();
      expect(screen.getByText('Exceptional Performance')).toBeInTheDocument();
      expect(screen.getByText('970% above baseline')).toBeInTheDocument();
    });
  });

  describe('Visual Elements', () => {
    it('renders trend icon', () => {
      const overview = createMockOverview(2.0);
      const { container } = render(<ProductivityOverview overview={overview} />);

      // Check for icon presence (lucide-react icons render as SVG)
      const icons = container.querySelectorAll('svg');
      expect(icons.length).toBeGreaterThan(0);
    });

    it('renders header text', () => {
      const overview = createMockOverview(2.0);
      render(<ProductivityOverview overview={overview} />);

      expect(screen.getByText('Team Productivity Multiplier')).toBeInTheDocument();
      expect(screen.getByText('Compared to industry benchmarks')).toBeInTheDocument();
    });
  });
});
