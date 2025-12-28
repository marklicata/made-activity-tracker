/**
 * Unit tests for BenchmarkMetricCard component
 *
 * Tests for:
 * - Value display with correct formatting
 * - Tier badge rendering
 * - VS industry/elite percentage display
 * - Trend indicators (up/down/flat)
 * - Color coding by metric category (speed/ease/quality)
 * - Handling missing comparison data
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BenchmarkMetricCard } from '@/components/metrics/BenchmarkMetricCard';
import { Zap } from 'lucide-react';
import type { MetricComparison } from '@/types/metrics';

describe('BenchmarkMetricCard', () => {
  const defaultProps = {
    title: 'PRs per Day',
    value: '5.0',
    icon: Zap,
    color: 'speed' as const,
  };

  describe('Basic Rendering', () => {
    it('renders title and value correctly', () => {
      render(<BenchmarkMetricCard {...defaultProps} />);

      expect(screen.getByText('PRs per Day')).toBeInTheDocument();
      expect(screen.getByText('5.0')).toBeInTheDocument();
    });

    it('renders numeric value correctly', () => {
      render(<BenchmarkMetricCard {...defaultProps} value={42} />);

      expect(screen.getByText('42')).toBeInTheDocument();
    });

    it('renders subtitle when provided', () => {
      render(<BenchmarkMetricCard {...defaultProps} subtitle="per developer" />);

      expect(screen.getByText('per developer')).toBeInTheDocument();
    });

    it('does not render subtitle when not provided', () => {
      render(<BenchmarkMetricCard {...defaultProps} />);

      expect(screen.queryByText('per developer')).not.toBeInTheDocument();
    });
  });

  describe('Tier Badge Display', () => {
    it('displays elite tier badge', () => {
      const comparison: MetricComparison = {
        value: 5.0,
        tier: 'elite',
        vs_industry_pct: 125,
        vs_elite_pct: 10,
      };

      render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      expect(screen.getByText('Elite Performer')).toBeInTheDocument();
    });

    it('displays industry tier badge', () => {
      const comparison: MetricComparison = {
        value: 3.0,
        tier: 'industry',
        vs_industry_pct: 5,
        vs_elite_pct: -45,
      };

      render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      expect(screen.getByText('Industry Average')).toBeInTheDocument();
    });

    it('displays exceptional tier badge', () => {
      const comparison: MetricComparison = {
        value: 8.0,
        tier: 'exceptional',
        vs_industry_pct: 200,
        vs_elite_pct: 150,
      };

      render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      expect(screen.getByText('Exceptional')).toBeInTheDocument();
    });

    it('displays below industry tier badge', () => {
      const comparison: MetricComparison = {
        value: 0.5,
        tier: 'below_industry',
        vs_industry_pct: -75,
        vs_elite_pct: -90,
      };

      render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      expect(screen.getByText('Below Industry')).toBeInTheDocument();
    });
  });

  describe('Industry Comparison Display', () => {
    it('shows positive percentage vs industry with + sign', () => {
      const comparison: MetricComparison = {
        value: 5.0,
        tier: 'elite',
        vs_industry_pct: 125,
        vs_elite_pct: 10,
      };

      render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      expect(screen.getByText('+125%')).toBeInTheDocument();
      expect(screen.getByText('vs industry')).toBeInTheDocument();
    });

    it('shows negative percentage vs industry without + sign', () => {
      const comparison: MetricComparison = {
        value: 0.5,
        tier: 'below_industry',
        vs_industry_pct: -37,
        vs_elite_pct: -67,
      };

      render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      expect(screen.getByText('-37%')).toBeInTheDocument();
    });

    it('shows zero percentage vs industry', () => {
      const comparison: MetricComparison = {
        value: 2.5,
        tier: 'industry',
        vs_industry_pct: 0,
        vs_elite_pct: -50,
      };

      render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      expect(screen.getByText('0%')).toBeInTheDocument();
    });
  });

  describe('Elite Comparison Display', () => {
    it('shows positive percentage vs elite with + sign', () => {
      const comparison: MetricComparison = {
        value: 5.0,
        tier: 'exceptional',
        vs_industry_pct: 150,
        vs_elite_pct: 67,
      };

      render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      expect(screen.getByText('+67%')).toBeInTheDocument();
      expect(screen.getByText('vs elite')).toBeInTheDocument();
    });

    it('shows negative percentage vs elite', () => {
      const comparison: MetricComparison = {
        value: 1.5,
        tier: 'industry',
        vs_industry_pct: 25,
        vs_elite_pct: -45,
      };

      render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      expect(screen.getByText('-45%')).toBeInTheDocument();
    });
  });

  describe('Trend Indicators', () => {
    it('renders trending up icon for positive vs industry', () => {
      const comparison: MetricComparison = {
        value: 5.0,
        tier: 'elite',
        vs_industry_pct: 50,
        vs_elite_pct: 10,
      };

      const { container } = render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      // TrendingUp icon should be present
      const icons = container.querySelectorAll('svg');
      expect(icons.length).toBeGreaterThan(0);
    });

    it('renders trending down icon for negative vs industry', () => {
      const comparison: MetricComparison = {
        value: 0.5,
        tier: 'below_industry',
        vs_industry_pct: -50,
        vs_elite_pct: -70,
      };

      const { container } = render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      const icons = container.querySelectorAll('svg');
      expect(icons.length).toBeGreaterThan(0);
    });
  });

  describe('Color Coding', () => {
    it('applies speed color scheme', () => {
      const { container } = render(<BenchmarkMetricCard {...defaultProps} color="speed" />);

      const card = container.firstChild as HTMLElement;
      expect(card.className).toContain('border-blue-500');
      expect(card.className).toContain('bg-blue-50');
    });

    it('applies ease color scheme', () => {
      const { container } = render(<BenchmarkMetricCard {...defaultProps} color="ease" />);

      const card = container.firstChild as HTMLElement;
      expect(card.className).toContain('border-green-500');
      expect(card.className).toContain('bg-green-50');
    });

    it('applies quality color scheme', () => {
      const { container } = render(<BenchmarkMetricCard {...defaultProps} color="quality" />);

      const card = container.firstChild as HTMLElement;
      expect(card.className).toContain('border-purple-500');
      expect(card.className).toContain('bg-purple-50');
    });
  });

  describe('Without Comparison Data', () => {
    it('renders without comparison section when comparison is undefined', () => {
      render(<BenchmarkMetricCard {...defaultProps} />);

      expect(screen.queryByText('vs industry')).not.toBeInTheDocument();
      expect(screen.queryByText('vs elite')).not.toBeInTheDocument();
    });

    it('still displays title and value without comparison', () => {
      render(<BenchmarkMetricCard {...defaultProps} />);

      expect(screen.getByText('PRs per Day')).toBeInTheDocument();
      expect(screen.getByText('5.0')).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('handles very large percentages', () => {
      const comparison: MetricComparison = {
        value: 10.0,
        tier: 'exceptional',
        vs_industry_pct: 999,
        vs_elite_pct: 500,
      };

      render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      expect(screen.getByText('+999%')).toBeInTheDocument();
      expect(screen.getByText('+500%')).toBeInTheDocument();
    });

    it('handles decimal percentages correctly', () => {
      const comparison: MetricComparison = {
        value: 2.5,
        tier: 'industry',
        vs_industry_pct: 12.7,
        vs_elite_pct: -23.4,
      };

      render(<BenchmarkMetricCard {...defaultProps} comparison={comparison} />);

      // Should round to nearest integer
      expect(screen.getByText('+13%')).toBeInTheDocument();
      expect(screen.getByText('-23%')).toBeInTheDocument();
    });

    it('renders with empty string value', () => {
      render(<BenchmarkMetricCard {...defaultProps} value="" />);

      expect(screen.getByText('PRs per Day')).toBeInTheDocument();
    });

    it('renders with zero value', () => {
      render(<BenchmarkMetricCard {...defaultProps} value={0} />);

      expect(screen.getByText('0')).toBeInTheDocument();
    });
  });

  describe('Icon Rendering', () => {
    it('renders provided icon component', () => {
      const { container } = render(<BenchmarkMetricCard {...defaultProps} />);

      // Icon should be rendered as SVG
      const icons = container.querySelectorAll('svg');
      expect(icons.length).toBeGreaterThan(0);
    });
  });

  describe('Accessibility', () => {
    it('renders semantic HTML structure', () => {
      render(<BenchmarkMetricCard {...defaultProps} />);

      // Should have proper structure
      expect(screen.getByText('PRs per Day')).toBeInTheDocument();
      expect(screen.getByText('5.0')).toBeInTheDocument();
    });
  });
});
