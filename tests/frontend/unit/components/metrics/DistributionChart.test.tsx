/**
 * Unit tests for DistributionChart component
 *
 * Tests for:
 * - Rendering bars with correct widths
 * - Showing labels and percentages
 * - Handling 0% data
 * - Handling 100% data
 * - Color coding
 * - Count display
 * - Multiple bars rendering
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { DistributionChart, DistributionBar } from '@/components/metrics/DistributionChart';

describe('DistributionChart', () => {
  const mockData = [
    { label: 'Feature', percentage: 50, count: 25, color: 'green' as const },
    { label: 'Bug Fix', percentage: 30, count: 15, color: 'red' as const },
    { label: 'Refactor', percentage: 20, count: 10, color: 'blue' as const },
  ];

  describe('Basic Rendering', () => {
    it('renders title correctly', () => {
      render(<DistributionChart title="PR Type Distribution" data={mockData} />);

      expect(screen.getByText('PR Type Distribution')).toBeInTheDocument();
    });

    it('renders all data items', () => {
      render(<DistributionChart title="PR Types" data={mockData} />);

      expect(screen.getByText('Feature')).toBeInTheDocument();
      expect(screen.getByText('Bug Fix')).toBeInTheDocument();
      expect(screen.getByText('Refactor')).toBeInTheDocument();
    });

    it('renders with empty data array', () => {
      render(<DistributionChart title="Empty Chart" data={[]} />);

      expect(screen.getByText('Empty Chart')).toBeInTheDocument();
    });

    it('renders with custom className', () => {
      const { container } = render(
        <DistributionChart title="Test" data={mockData} className="custom-class" />
      );

      const chart = container.firstChild as HTMLElement;
      expect(chart.className).toContain('custom-class');
    });
  });

  describe('Data Display', () => {
    it('displays percentages correctly', () => {
      render(<DistributionChart title="Test" data={mockData} />);

      expect(screen.getByText('50.0%')).toBeInTheDocument();
      expect(screen.getByText('30.0%')).toBeInTheDocument();
      expect(screen.getByText('20.0%')).toBeInTheDocument();
    });

    it('displays counts correctly', () => {
      render(<DistributionChart title="Test" data={mockData} />);

      expect(screen.getByText('(25)')).toBeInTheDocument();
      expect(screen.getByText('(15)')).toBeInTheDocument();
      expect(screen.getByText('(10)')).toBeInTheDocument();
    });

    it('formats decimal percentages to 1 decimal place', () => {
      const data = [{ label: 'Test', percentage: 33.333, count: 10 }];
      render(<DistributionChart title="Test" data={data} />);

      expect(screen.getByText('33.3%')).toBeInTheDocument();
    });

    it('handles zero percentage', () => {
      const data = [{ label: 'Zero', percentage: 0, count: 0 }];
      render(<DistributionChart title="Test" data={data} />);

      expect(screen.getByText('0.0%')).toBeInTheDocument();
      expect(screen.getByText('(0)')).toBeInTheDocument();
    });

    it('handles 100% percentage', () => {
      const data = [{ label: 'All', percentage: 100, count: 50 }];
      render(<DistributionChart title="Test" data={data} />);

      expect(screen.getByText('100.0%')).toBeInTheDocument();
    });
  });

  describe('Bar Widths', () => {
    it('applies correct width style based on percentage', () => {
      const data = [{ label: 'Test', percentage: 75, count: 30 }];
      const { container } = render(<DistributionChart title="Test" data={data} />);

      const bar = container.querySelector('.bg-blue-500') as HTMLElement;
      expect(bar?.style.width).toBe('75%');
    });

    it('caps width at 100% for percentages over 100', () => {
      const data = [{ label: 'Test', percentage: 150, count: 50 }];
      const { container } = render(<DistributionChart title="Test" data={data} />);

      const bar = container.querySelector('.bg-blue-500') as HTMLElement;
      expect(bar?.style.width).toBe('100%');
    });

    it('handles 0% width', () => {
      const data = [{ label: 'Zero', percentage: 0, count: 0 }];
      const { container } = render(<DistributionChart title="Test" data={data} />);

      const bar = container.querySelector('.bg-blue-500') as HTMLElement;
      expect(bar?.style.width).toBe('0%');
    });
  });

  describe('Color Coding', () => {
    it('applies green color class', () => {
      const data = [{ label: 'Success', percentage: 50, count: 10, color: 'green' as const }];
      const { container } = render(<DistributionChart title="Test" data={data} />);

      expect(container.querySelector('.bg-green-500')).toBeInTheDocument();
    });

    it('applies blue color class', () => {
      const data = [{ label: 'Info', percentage: 50, count: 10, color: 'blue' as const }];
      const { container } = render(<DistributionChart title="Test" data={data} />);

      expect(container.querySelector('.bg-blue-500')).toBeInTheDocument();
    });

    it('applies red color class', () => {
      const data = [{ label: 'Error', percentage: 50, count: 10, color: 'red' as const }];
      const { container } = render(<DistributionChart title="Test" data={data} />);

      expect(container.querySelector('.bg-red-500')).toBeInTheDocument();
    });

    it('applies yellow color class', () => {
      const data = [{ label: 'Warning', percentage: 50, count: 10, color: 'yellow' as const }];
      const { container } = render(<DistributionChart title="Test" data={data} />);

      expect(container.querySelector('.bg-yellow-500')).toBeInTheDocument();
    });

    it('applies purple color class', () => {
      const data = [{ label: 'Feature', percentage: 50, count: 10, color: 'purple' as const }];
      const { container } = render(<DistributionChart title="Test" data={data} />);

      expect(container.querySelector('.bg-purple-500')).toBeInTheDocument();
    });

    it('applies gray color class', () => {
      const data = [{ label: 'Neutral', percentage: 50, count: 10, color: 'gray' as const }];
      const { container } = render(<DistributionChart title="Test" data={data} />);

      expect(container.querySelector('.bg-gray-500')).toBeInTheDocument();
    });

    it('defaults to blue when color not specified', () => {
      const data = [{ label: 'Default', percentage: 50, count: 10 }];
      const { container } = render(<DistributionChart title="Test" data={data} />);

      expect(container.querySelector('.bg-blue-500')).toBeInTheDocument();
    });
  });
});

describe('DistributionBar', () => {
  const defaultProps = {
    label: 'Test Label',
    percentage: 45.5,
    count: 23,
    color: 'blue' as const,
  };

  describe('Basic Rendering', () => {
    it('renders label correctly', () => {
      render(<DistributionBar {...defaultProps} />);

      expect(screen.getByText('Test Label')).toBeInTheDocument();
    });

    it('renders percentage with 1 decimal place', () => {
      render(<DistributionBar {...defaultProps} />);

      expect(screen.getByText('45.5%')).toBeInTheDocument();
    });

    it('renders count in parentheses', () => {
      render(<DistributionBar {...defaultProps} />);

      expect(screen.getByText('(23)')).toBeInTheDocument();
    });
  });

  describe('Bar Width', () => {
    it('sets width style based on percentage', () => {
      const { container } = render(<DistributionBar {...defaultProps} percentage={75} />);

      const bar = container.querySelector('.bg-blue-500') as HTMLElement;
      expect(bar?.style.width).toBe('75%');
    });

    it('caps width at 100% for over 100% values', () => {
      const { container } = render(<DistributionBar {...defaultProps} percentage={125} />);

      const bar = container.querySelector('.bg-blue-500') as HTMLElement;
      expect(bar?.style.width).toBe('100%');
    });

    it('handles 0% correctly', () => {
      const { container } = render(<DistributionBar {...defaultProps} percentage={0} />);

      const bar = container.querySelector('.bg-blue-500') as HTMLElement;
      expect(bar?.style.width).toBe('0%');
    });
  });

  describe('Edge Cases', () => {
    it('handles very small percentages', () => {
      render(<DistributionBar {...defaultProps} percentage={0.1} />);

      expect(screen.getByText('0.1%')).toBeInTheDocument();
    });

    it('handles very large counts', () => {
      render(<DistributionBar {...defaultProps} count={999999} />);

      expect(screen.getByText('(999999)')).toBeInTheDocument();
    });

    it('handles negative percentages (edge case)', () => {
      const { container } = render(<DistributionBar {...defaultProps} percentage={-10} />);

      // Should still render but width will be 0 due to Math.min with 100
      expect(screen.getByText('-10.0%')).toBeInTheDocument();
    });

    it('handles decimal percentages correctly', () => {
      render(<DistributionBar {...defaultProps} percentage={33.333333} />);

      expect(screen.getByText('33.3%')).toBeInTheDocument();
    });
  });

  describe('Color Application', () => {
    it.each([
      ['green' as const, 'bg-green-500'],
      ['blue' as const, 'bg-blue-500'],
      ['yellow' as const, 'bg-yellow-500'],
      ['red' as const, 'bg-red-500'],
      ['purple' as const, 'bg-purple-500'],
      ['gray' as const, 'bg-gray-500'],
    ])('applies %s color class correctly', (color, expectedClass) => {
      const { container } = render(<DistributionBar {...defaultProps} color={color} />);

      expect(container.querySelector(`.${expectedClass}`)).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('maintains readable text contrast', () => {
      const { container } = render(<DistributionBar {...defaultProps} />);

      // Check that text elements have proper color classes
      expect(container.querySelector('.text-gray-700')).toBeInTheDocument();
      expect(container.querySelector('.text-gray-500')).toBeInTheDocument();
    });
  });
});
