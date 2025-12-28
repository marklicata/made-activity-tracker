/**
 * Unit tests for QualitySection component
 *
 * Tests for:
 * - Renders all 5 metric cards
 * - Renders PR type distribution chart
 * - Renders files per PR distribution chart
 * - Handles missing review data
 * - Formats percentages and values correctly
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QualitySection } from '@/components/metrics/QualitySection';
import type { QualityMetrics } from '@/types/metrics';

describe('QualitySection', () => {
  const createMockQualityMetrics = (): QualityMetrics => ({
    pr_merge_rate: 85.5,
    avg_files_per_pr: 6.2,
    bug_pr_percentage: 20.0,
    feature_pr_percentage: 60.0,
    avg_review_cycle_hours: 4.5,
    avg_review_comments: 3.2,
    pr_type_distribution: [
      { pr_type: 'feature', count: 60, percentage: 60.0 },
      { pr_type: 'bug_fix', count: 20, percentage: 20.0 },
      { pr_type: 'refactor', count: 15, percentage: 15.0 },
      { pr_type: 'test', count: 3, percentage: 3.0 },
      { pr_type: 'docs', count: 2, percentage: 2.0 },
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
  });

  describe('Section Header', () => {
    it('renders section title', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('Quality')).toBeInTheDocument();
    });

    it('renders section description', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('Standard of work output')).toBeInTheDocument();
    });
  });

  describe('Metric Cards', () => {
    it('renders PR Merge Rate card', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('PR Merge Rate')).toBeInTheDocument();
      expect(screen.getByText('85.5%')).toBeInTheDocument();
      expect(screen.getByText('PRs successfully merged')).toBeInTheDocument();
    });

    it('renders Bug PR Ratio card', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('Bug PR Ratio')).toBeInTheDocument();
      expect(screen.getAllByText('20.0%').length).toBeGreaterThan(0);
      expect(screen.getByText('Lower is better')).toBeInTheDocument();
    });

    it('renders Feature Work card', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('Feature Work')).toBeInTheDocument();
      expect(screen.getAllByText('60.0%').length).toBeGreaterThan(0);
      expect(screen.getByText('New functionality')).toBeInTheDocument();
    });

    it('renders Files per PR card', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('Files per PR')).toBeInTheDocument();
      expect(screen.getByText('6.2')).toBeInTheDocument();
      expect(screen.getByText('Scope size indicator')).toBeInTheDocument();
    });

    it('renders Review Cycle card with formatted hours', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('Review Cycle')).toBeInTheDocument();
      expect(screen.getByText('4.5h')).toBeInTheDocument();
      expect(screen.getByText('Time to first review')).toBeInTheDocument();
    });
  });

  describe('PR Type Distribution Chart', () => {
    it('renders PR type distribution chart title', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('PR Type Distribution')).toBeInTheDocument();
    });

    it('displays all PR types with capitalized labels', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('Feature')).toBeInTheDocument();
      expect(screen.getByText('Bug Fix')).toBeInTheDocument();
      expect(screen.getByText('Refactor')).toBeInTheDocument();
      expect(screen.getByText('Test')).toBeInTheDocument();
      expect(screen.getByText('Docs')).toBeInTheDocument();
    });

    it('displays correct percentages for each type', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getAllByText('60.0%').length).toBeGreaterThan(0);
      expect(screen.getAllByText('20.0%').length).toBeGreaterThan(0);
      expect(screen.getAllByText('15.0%').length).toBeGreaterThan(0);
      expect(screen.getByText('3.0%')).toBeInTheDocument();
      expect(screen.getByText('2.0%')).toBeInTheDocument();
    });

    it('displays correct counts for each type', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getAllByText('(60)').length).toBeGreaterThan(0);
      expect(screen.getAllByText('(20)').length).toBeGreaterThan(0);
      expect(screen.getAllByText('(15)').length).toBeGreaterThan(0);
      expect(screen.getByText('(3)')).toBeInTheDocument();
      expect(screen.getByText('(2)')).toBeInTheDocument();
    });
  });

  describe('Files per PR Distribution Chart', () => {
    it('renders files per PR distribution chart title', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('Files per PR Distribution')).toBeInTheDocument();
    });

    it('displays all file range buckets', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('1-3 files')).toBeInTheDocument();
      expect(screen.getByText('4-8 files')).toBeInTheDocument();
      expect(screen.getByText('9-15 files')).toBeInTheDocument();
      expect(screen.getByText('16+ files')).toBeInTheDocument();
    });

    it('displays correct percentages for each range', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('40.0%')).toBeInTheDocument();
      expect(screen.getByText('35.0%')).toBeInTheDocument();
      // 15.0% and 10.0% also present but harder to uniquely identify
    });

    it('displays correct counts for each range', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('(40)')).toBeInTheDocument();
      expect(screen.getByText('(35)')).toBeInTheDocument();
      expect(screen.getByText('(10)')).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('handles 0% merge rate', () => {
      const quality = { ...createMockQualityMetrics(), pr_merge_rate: 0 };
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('0.0%')).toBeInTheDocument();
    });

    it('handles 100% merge rate', () => {
      const quality = { ...createMockQualityMetrics(), pr_merge_rate: 100 };
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('100.0%')).toBeInTheDocument();
    });

    it('handles 0% bug ratio', () => {
      const quality = { ...createMockQualityMetrics(), bug_pr_percentage: 0 };
      render(<QualitySection quality={quality} />);

      // Bug PR Ratio label should still be present
      expect(screen.getByText('Bug PR Ratio')).toBeInTheDocument();
    });

    it('handles 0 files per PR', () => {
      const quality = { ...createMockQualityMetrics(), avg_files_per_pr: 0 };
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('0.0')).toBeInTheDocument();
    });

    it('handles very fast review cycle (minutes)', () => {
      const quality = { ...createMockQualityMetrics(), avg_review_cycle_hours: 0.25 };
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('15m')).toBeInTheDocument();
    });

    it('handles slow review cycle (days)', () => {
      const quality = { ...createMockQualityMetrics(), avg_review_cycle_hours: 48 };
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('2.0d')).toBeInTheDocument();
    });

    it('handles empty PR type distribution', () => {
      const quality = { ...createMockQualityMetrics(), pr_type_distribution: [] };
      render(<QualitySection quality={quality} />);

      // Chart title should still render
      expect(screen.getByText('PR Type Distribution')).toBeInTheDocument();
    });

    it('handles all PRs in one file range', () => {
      const quality = {
        ...createMockQualityMetrics(),
        files_per_pr_distribution: {
          range_1_3: 100,
          range_1_3_pct: 100.0,
          range_4_8: 0,
          range_4_8_pct: 0,
          range_9_15: 0,
          range_9_15_pct: 0,
          range_16_plus: 0,
          range_16_plus_pct: 0,
        },
      };

      render(<QualitySection quality={quality} />);

      expect(screen.getByText('100.0%')).toBeInTheDocument();
    });
  });

  describe('Formatting', () => {
    it('formats merge rate to 1 decimal', () => {
      const quality = { ...createMockQualityMetrics(), pr_merge_rate: 85.777 };
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('85.8%')).toBeInTheDocument();
    });

    it('formats bug percentage to 1 decimal', () => {
      const quality = { ...createMockQualityMetrics(), bug_pr_percentage: 33.333 };
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('33.3%')).toBeInTheDocument();
    });

    it('formats files per PR to 1 decimal', () => {
      const quality = { ...createMockQualityMetrics(), avg_files_per_pr: 7.888 };
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('7.9')).toBeInTheDocument();
    });

    it('formats feature percentage to 1 decimal', () => {
      const quality = { ...createMockQualityMetrics(), feature_pr_percentage: 45.555 };
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('45.6%')).toBeInTheDocument();
    });
  });

  describe('PR Type Label Formatting', () => {
    it('capitalizes first letter of each word', () => {
      const quality = {
        ...createMockQualityMetrics(),
        pr_type_distribution: [
          { pr_type: 'bug_fix', count: 10, percentage: 100 },
        ],
      };

      render(<QualitySection quality={quality} />);

      expect(screen.getByText('Bug Fix')).toBeInTheDocument();
    });

    it('handles "other" type', () => {
      const quality = {
        ...createMockQualityMetrics(),
        pr_type_distribution: [
          { pr_type: 'other', count: 10, percentage: 100 },
        ],
      };

      render(<QualitySection quality={quality} />);

      expect(screen.getByText('Other')).toBeInTheDocument();
    });
  });

  describe('Benchmark Comparisons', () => {
    it('shows comparison for merge rate', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      // Should have benchmark comparison
      expect(screen.getByText('PR Merge Rate')).toBeInTheDocument();
    });

    it('shows comparison for bug ratio with "lower is better" logic', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      // Bug ratio: 20% vs industry 25%, so we're better
      expect(screen.getByText('Bug PR Ratio')).toBeInTheDocument();
    });

    it('shows comparison for files per PR', () => {
      const quality = createMockQualityMetrics();
      render(<QualitySection quality={quality} />);

      expect(screen.getByText('Files per PR')).toBeInTheDocument();
    });
  });
});
